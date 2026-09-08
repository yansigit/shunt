//! Read-only subscription credentials; never shared with the API-key product.
use std::path::{Path, PathBuf};

use super::{auth_error, shared, Credential};
use crate::{
    adapters::AdapterError,
    config::{AuthMode, ProviderConfig, ProviderKind},
};

pub(crate) const TOKEN_ENV: &str = "SHUNT_COMMAND_CODE_TOKEN";
pub(crate) const ENDPOINT: &str = "https://api.commandcode.ai/alpha/generate";

/// Byte bound for the CLI auth file read (14-05 CCS-01): a credential file
/// far larger than any real {apiKey, userId} document is rejected instead of
/// being read into memory unboundedly.
const CLI_AUTH_FILE_MAX_BYTES: usize = 16 * 1024;
const CLI_CREDENTIAL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

pub(crate) fn validate_provider(provider: &ProviderConfig) -> Result<(), &'static str> {
    if provider.kind != ProviderKind::CommandCode || provider.auth != AuthMode::CommandCodeOauth {
        return Err("command_code requires command_code_oauth exclusively");
    }
    let url =
        reqwest::Url::parse(&provider.base_url).map_err(|_| "invalid subscription destination")?;
    if url.scheme() != "https"
        || url.host_str() != Some("api.commandcode.ai")
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !matches!(url.path(), "" | "/" | "/alpha/generate")
    {
        return Err(
            "subscription credentials require canonical HTTPS api.commandcode.ai/alpha/generate",
        );
    }
    Ok(())
}

#[cfg(test)]
pub(crate) static LOOKUPS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
#[cfg(test)]
pub(crate) static FILE_READS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

pub(crate) async fn resolve(provider: &ProviderConfig) -> Result<Credential, AdapterError> {
    validate_provider(provider).map_err(auth_error)?;
    #[cfg(test)]
    LOOKUPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let env_source = std::env::var(TOKEN_ENV);
    let cli_path = cli_auth_path();
    resolve_sources(env_source, cli_path.as_deref()).await
}

/// Fixed, read-only Command Code CLI credential path
/// (~/.commandcode/auth.json). Production only - tests inject a temporary
/// path through resolve_sources and must never resolve the developer's real
/// CLI home.
fn cli_auth_path() -> Option<PathBuf> {
    shared::home_dir().map(|home| home.join(".commandcode").join("auth.json"))
}

/// Resolve a Command Code credential from the explicit env source, falling
/// back to the read-only CLI file only when no explicit source is present.
/// Both inputs are injected so tests never touch process env or the real CLI
/// home (14-05 test isolation contract).
pub(crate) async fn resolve_sources(
    env_source: Result<String, std::env::VarError>,
    cli_path: Option<&Path>,
) -> Result<Credential, AdapterError> {
    match env_source {
        // Fail closed on EVERY present env value (empty included): an
        // explicit source must never be swapped for the CLI file, and a
        // present-but-invalid value must not silently change identity.
        Ok(token) => explicit_credential(Ok(token)),
        Err(std::env::VarError::NotUnicode(value)) => {
            explicit_credential(Err(std::env::VarError::NotUnicode(value)))
        }
        // Only true absence may fall through to the read-only CLI file.
        Err(std::env::VarError::NotPresent) => cli_credential_fallback(cli_path).await,
    }
}

/// Bounded, read-only read of the CLI auth file with a hard deadline. Never
/// writes, refreshes, copies, or probes the credential upstream (no whoami):
/// the file itself is the credential source (D-03).
async fn cli_credential_fallback(cli_path: Option<&Path>) -> Result<Credential, AdapterError> {
    #[cfg(test)]
    FILE_READS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let Some(path) = cli_path else {
        return Err(auth_error("Command Code CLI auth file is unavailable"));
    };
    let path = path.to_path_buf();
    let bytes = super::with_credential_timeout(
        CLI_CREDENTIAL_TIMEOUT,
        async {
            tokio::task::spawn_blocking(move || read_cli_auth_file_bounded(&path))
                .await
                .map_err(|_| auth_error("Command Code CLI auth file is missing or unreadable"))?
        },
        "Command Code credential resolution timed out",
    )
    .await?;
    cli_file_credential(&bytes)
}

fn cli_file_credential(bytes: &[u8]) -> Result<Credential, AdapterError> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct CliAuthFile {
        api_key: String,
        // Optional userId: only validated as a string when present (schema
        // {apiKey: nonempty, userId?}), never used or logged. The underscore
        // name plus explicit rename keeps the schema check without a
        // dead-field warning.
        #[serde(rename = "userId", default)]
        _user_id: Option<String>,
    }

    let parsed: CliAuthFile = serde_json::from_slice(bytes)
        .map_err(|_| auth_error("Command Code CLI auth file is malformed"))?;
    let token = parsed.api_key;
    if token.is_empty() || token.len() > 16384 || !token.bytes().all(|b| b.is_ascii_graphic()) {
        return Err(auth_error("Command Code CLI auth file apiKey is invalid"));
    }
    Ok(Credential::CommandCodeOauth {
        access_token: token,
    })
}

/// Open a credential file read-only without ever blocking on a FIFO: on Unix
/// the open carries O_NONBLOCK (so an unopened FIFO cannot stall the open
/// syscall) and a following fstat (via File::metadata) rejects every
/// non-regular file before any read is attempted.
#[cfg(unix)]
fn open_cli_auth_file(path: &Path) -> Result<std::fs::File, AdapterError> {
    use std::os::unix::ffi::OsStrExt;
    let cpath = std::ffi::CString::new(path.as_os_str().as_bytes())
        .map_err(|_| auth_error("Command Code CLI auth file is missing or unreadable"))?;
    // SAFETY: open(2) only reads the NUL-terminated path buffer. The returned
    // descriptor is immediately owned by a File, whose Drop closes it on
    // every path below.
    let fd = unsafe {
        libc::open(
            cpath.as_ptr(),
            libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(auth_error(
            "Command Code CLI auth file is missing or unreadable",
        ));
    }
    Ok(unsafe { std::os::unix::io::FromRawFd::from_raw_fd(fd) })
}

#[cfg(not(unix))]
fn open_cli_auth_file(path: &Path) -> Result<std::fs::File, AdapterError> {
    std::fs::File::open(path)
        .map_err(|_| auth_error("Command Code CLI auth file is missing or unreadable"))
}

fn read_cli_auth_file_bounded(path: &Path) -> Result<Vec<u8>, AdapterError> {
    let file = open_cli_auth_file(path)?;
    let metadata = file
        .metadata()
        .map_err(|_| auth_error("Command Code CLI auth file is missing or unreadable"))?;
    if !metadata.is_file() {
        return Err(auth_error(
            "Command Code CLI auth file is missing or unreadable",
        ));
    }
    use std::io::Read;
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 4096];
    let mut file = file;
    loop {
        match file.read(&mut chunk) {
            Ok(0) => return Ok(bytes),
            Ok(n) => {
                if bytes.len() + n > CLI_AUTH_FILE_MAX_BYTES {
                    return Err(auth_error("Command Code CLI auth file is too large"));
                }
                bytes.extend_from_slice(&chunk[..n]);
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => {
                return Err(auth_error(
                    "Command Code CLI auth file is missing or unreadable",
                ));
            }
        }
    }
}
fn explicit_credential(
    source: Result<String, std::env::VarError>,
) -> Result<Credential, AdapterError> {
    let token =
        source.map_err(|_| auth_error("Command Code subscription token is absent or invalid"))?;
    if token.is_empty() || token.len() > 16384 || !token.bytes().all(|b| b.is_ascii_graphic()) {
        return Err(auth_error("Command Code subscription token is invalid"));
    }
    Ok(Credential::CommandCodeOauth {
        access_token: token,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    mod matrix;

    static FILE_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    fn lifetime_dir(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "shunt-command-code-lifetime-{label}-{}",
            uuid::Uuid::new_v4()
        ))
    }

    /// Write a CLI auth fixture and capture the file-state triple the
    /// scenarios must preserve: bytes (SHA-256), mtime, and directory
    /// inventory count.
    fn capture_file_state(path: &std::path::Path) -> (String, std::time::SystemTime, usize) {
        let bytes = std::fs::read(path).unwrap();
        let mtime = std::fs::metadata(path).unwrap().modified().unwrap();
        let inventory = std::fs::read_dir(path.parent().unwrap()).unwrap().count();
        let sha = format!("{:x}", Sha256::digest(&bytes));
        (sha, mtime, inventory)
    }

    fn assert_file_unchanged(
        path: &std::path::Path,
        state: &(String, std::time::SystemTime, usize),
    ) {
        let (sha, mtime, inventory) = capture_file_state(path);
        assert_eq!(sha, state.0, "file bytes changed");
        assert_eq!(mtime, state.1, "file mtime changed");
        assert_eq!(inventory, state.2, "directory inventory changed");
    }

    fn assert_auth_error(error: &AdapterError) {
        assert_eq!(
            error.response.status(),
            axum::http::StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn command_code_lifetime_file_valid_file_authenticates_read_only() {
        let _lock = FILE_TEST_LOCK.lock().await;
        let dir = lifetime_dir("valid");
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("auth.json");
        for bytes in [
            br#"{"apiKey":"synthetic-file-key","userId":"fixture-user"}"#.as_slice(),
            br#"{"apiKey":"synthetic-file-key-no-user"}"#.as_slice(),
        ] {
            std::fs::write(&path, bytes).unwrap();
            let state = capture_file_state(&path);
            let credential = resolve_sources(Err(std::env::VarError::NotPresent), Some(&path))
                .await
                .unwrap();
            let expected = match std::str::from_utf8(bytes).unwrap() {
                contents if contents.contains("no-user") => "synthetic-file-key-no-user",
                _ => "synthetic-file-key",
            };
            assert_eq!(
                credential,
                Credential::CommandCodeOauth {
                    access_token: expected.to_string(),
                }
            );
            assert!(!format!("{credential:?}").contains("synthetic"));
            assert_file_unchanged(&path, &state);
        }
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn command_code_lifetime_file_matrix_failures_yield_auth_errors() {
        let _lock = FILE_TEST_LOCK.lock().await;
        let dir = lifetime_dir("matrix");
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("auth.json");
        // Missing file.
        let error = resolve_sources(Err(std::env::VarError::NotPresent), Some(&path))
            .await
            .unwrap_err();
        assert_auth_error(&error);
        // Malformed JSON, non-string apiKey, non-string userId, empty apiKey.
        for bytes in [
            br#"{not json"#.as_slice(),
            br#"{"apiKey":123}"#.as_slice(),
            br#"{"apiKey":"ok","userId":5}"#.as_slice(),
            br#"{"apiKey":""}"#.as_slice(),
        ] {
            std::fs::write(&path, bytes).unwrap();
            let error = resolve_sources(Err(std::env::VarError::NotPresent), Some(&path))
                .await
                .unwrap_err();
            assert_auth_error(&error);
        }
        // Non-regular "file" (directory at the auth path) must be rejected,
        // covering the unreadable case without depending on root privileges.
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        let error = resolve_sources(Err(std::env::VarError::NotPresent), Some(&path))
            .await
            .unwrap_err();
        assert_auth_error(&error);
        // Oversized file (valid apiKey, oversized userId padding) exceeds the
        // byte bound and is rejected before schema validation matters.
        std::fs::remove_dir(&path).unwrap();
        let oversized = format!(
            "{{\"apiKey\":\"ok\",\"userId\":\"{}\"}}",
            "x".repeat(20_000)
        );
        std::fs::write(&path, oversized).unwrap();
        let error = resolve_sources(Err(std::env::VarError::NotPresent), Some(&path))
            .await
            .unwrap_err();
        assert_auth_error(&error);
        // Header-unsafe apiKey in the file.
        std::fs::write(&path, br#"{"apiKey":"bad\r\nheader"}"#).unwrap();
        let error = resolve_sources(Err(std::env::VarError::NotPresent), Some(&path))
            .await
            .unwrap_err();
        assert_auth_error(&error);
        // No CLI path at all (no HOME) is a normal authentication error.
        let error = resolve_sources(Err(std::env::VarError::NotPresent), None)
            .await
            .unwrap_err();
        assert_auth_error(&error);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn command_code_lifetime_file_precedence_env_wins_and_invalid_fails_closed() {
        let _lock = FILE_TEST_LOCK.lock().await;
        let dir = lifetime_dir("precedence");
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("auth.json");
        let bytes = br#"{"apiKey":"synthetic-file-key","userId":"fixture-user"}"#;
        std::fs::write(&path, bytes).unwrap();
        let state = capture_file_state(&path);
        // A valid explicit env wins without ever consulting the file.
        let reads_before = FILE_READS.load(std::sync::atomic::Ordering::SeqCst);
        let credential = resolve_sources(Ok("synthetic-env-key".to_string()), Some(&path))
            .await
            .unwrap();
        assert_eq!(
            credential,
            Credential::CommandCodeOauth {
                access_token: "synthetic-env-key".to_string(),
            }
        );
        assert_eq!(
            FILE_READS.load(std::sync::atomic::Ordering::SeqCst),
            reads_before,
            "file must not be read while a valid explicit env is present"
        );
        assert_file_unchanged(&path, &state);
        // A present-but-invalid explicit env fails closed: no file read.
        for source in [
            Ok(String::new()),
            Ok("bad token".to_string()),
            Ok("bad\r\nheader".to_string()),
            Ok("x".repeat(16385)),
            Err(std::env::VarError::NotUnicode(std::ffi::OsString::from(
                "invalid",
            ))),
        ] {
            let reads_before = FILE_READS.load(std::sync::atomic::Ordering::SeqCst);
            let error = resolve_sources(source, Some(&path)).await.unwrap_err();
            assert_auth_error(&error);
            assert_eq!(
                FILE_READS.load(std::sync::atomic::Ordering::SeqCst),
                reads_before,
                "invalid explicit env must never read the CLI file"
            );
            assert_file_unchanged(&path, &state);
        }
        // Only a truly absent env value falls back to the CLI file.
        let reads_before = FILE_READS.load(std::sync::atomic::Ordering::SeqCst);
        let credential = resolve_sources(Err(std::env::VarError::NotPresent), Some(&path))
            .await
            .unwrap();
        assert_eq!(
            credential,
            Credential::CommandCodeOauth {
                access_token: "synthetic-file-key".to_string(),
            }
        );
        assert!(
            FILE_READS.load(std::sync::atomic::Ordering::SeqCst) > reads_before,
            "absent env must fall back to the CLI file"
        );
        assert_file_unchanged(&path, &state);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn command_code_lifetime_file_fifo_is_rejected_without_blocking() {
        let _lock = FILE_TEST_LOCK.lock().await;
        let dir = lifetime_dir("fifo");
        std::fs::create_dir(&dir).unwrap();
        let fifo = dir.join("auth.json");
        let cpath = std::ffi::CString::new(fifo.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(cpath.as_ptr(), 0o600) }, 0);
        let inventory = std::fs::read_dir(&dir).unwrap().count();
        let started = std::time::Instant::now();
        let error = resolve_sources(Err(std::env::VarError::NotPresent), Some(&fifo))
            .await
            .unwrap_err();
        assert_auth_error(&error);
        // A blocked FIFO open would burn the full 5-second deadline; a safe
        // non-regular rejection returns well before it.
        assert!(
            started.elapsed() < std::time::Duration::from_secs(4),
            "FIFO lookup must not block to the deadline"
        );
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), inventory);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn command_code_tracer_env_matrix_inputs_are_read_only() {
        // Test the env stage by injection: absent input never consults a real
        // CLI path, even when the production resolver gains file fallback.
        let dir =
            std::env::temp_dir().join(format!("shunt-command-code-auth-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("auth.json");
        let bytes = br#"{"apiKey":"synthetic-file-key","userId":"fixture"}"#;
        std::fs::write(&path, bytes).unwrap();
        let before = std::fs::metadata(&path).unwrap().modified().unwrap();
        let sha = Sha256::digest(bytes);
        for input in [
            Err(std::env::VarError::NotPresent),
            Err(std::env::VarError::NotUnicode(std::ffi::OsString::from(
                "invalid",
            ))),
            Ok(String::new()),
            Ok("bad token".into()),
            Ok("bad\r\nheader".into()),
            Ok("x".repeat(16385)),
        ] {
            let error = explicit_credential(input).unwrap_err();
            assert_eq!(
                error.response.status(),
                axum::http::StatusCode::UNAUTHORIZED
            );
            assert_eq!(Sha256::digest(std::fs::read(&path).unwrap()), sha);
            assert_eq!(
                std::fs::metadata(&path).unwrap().modified().unwrap(),
                before
            );
            assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        }
        assert!(explicit_credential(Ok("x".repeat(16384))).is_ok());
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    #[tokio::test]
    async fn command_code_tracer_env_matrix_immutable_snapshots() {
        let (a, b) = tokio::join!(
            async { explicit_credential(Ok("synthetic-a".into())).unwrap() },
            async { explicit_credential(Ok("synthetic-b".into())).unwrap() }
        );
        assert_eq!(
            a,
            Credential::CommandCodeOauth {
                access_token: "synthetic-a".into()
            }
        );
        assert_eq!(
            b,
            Credential::CommandCodeOauth {
                access_token: "synthetic-b".into()
            }
        );
        assert!(!format!("{a:?}{b:?}").contains("synthetic-"));
    }
}
