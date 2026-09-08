//! Read-only subscription credentials; never shared with the API-key product.
use super::{auth_error, Credential};
use crate::{
    adapters::AdapterError,
    config::{AuthMode, ProviderConfig, ProviderKind},
};

pub(crate) const TOKEN_ENV: &str = "SHUNT_COMMAND_CODE_TOKEN";
pub(crate) const ENDPOINT: &str = "https://api.commandcode.ai/alpha/generate";

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

pub(crate) async fn resolve(provider: &ProviderConfig) -> Result<Credential, AdapterError> {
    validate_provider(provider).map_err(auth_error)?;
    #[cfg(test)]
    LOOKUPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    // Env-only vertical slice. Bounded read-only CLI fallback is added in 14-05.
    super::with_credential_timeout(
        std::time::Duration::from_secs(5),
        async { explicit_credential(std::env::var(TOKEN_ENV)) },
        "Command Code credential resolution timed out",
    )
    .await
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
