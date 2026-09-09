//! `shunt check` runs the routed-Antigravity credential guard, not just
//! `Config::load(...).validate()` (issue #382).
//!
//! The guard is what makes the `kind = "antigravity"` rename non-silent: a
//! migrated config that routes to the native upstream with nobody having run
//! `shunt login antigravity` must be refused. It used to live only in the
//! run/serve path, so `check` reported `config ok` for a config `run` dies on
//! — exactly backwards for the CI and deploy scripts that gate a rollout on
//! `check`.
//!
//! Every case below drives the real binary, so the credential path is set on
//! the *child* process. Writing `SHUNT_ANTIGRAVITY_AUTH_FILE` into this
//! process would instead race — and leak into — every other test sharing it.

use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "shunt-check-cli-{tag}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).expect("create temp directory");
        Self(path)
    }

    /// Write `config` as `shunt.toml` and return the path to hand `--config`.
    fn config(&self, config: &str) -> PathBuf {
        let path = self.0.join("shunt.toml");
        std::fs::write(&path, config).expect("write config");
        path
    }

    /// The credential path the child probes. Nothing is written here unless a
    /// test calls [`TempDir::credential`], so by default it does not exist.
    fn credential_path(&self) -> PathBuf {
        self.0.join("antigravity-auth.json")
    }

    fn credential(&self) -> PathBuf {
        let path = self.credential_path();
        std::fs::write(&path, "{}").expect("write credential");
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn check(config: &Path, credential: &Path) -> Output {
    // `Path` is passed straight through: `Command::arg` takes `AsRef<OsStr>`,
    // so converting to `&str` first would only add a panic on a non-UTF-8
    // temp path without buying anything.
    Command::new(env!("CARGO_BIN_EXE_shunt"))
        .arg("check")
        .arg("--config")
        .arg(config)
        .env("SHUNT_ANTIGRAVITY_AUTH_FILE", credential)
        .output()
        .expect("shunt binary should run")
}

fn stdout(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> &str {
    std::str::from_utf8(&output.stderr).expect("stderr should be UTF-8")
}

#[test]
fn check_refuses_a_routed_antigravity_provider_without_a_credential() {
    let dir = TempDir::new("routed-no-credential");
    let config = dir.config("[server]\ndefault_provider = \"antigravity\"\n");

    let output = check(&config, &dir.credential_path());

    assert!(
        !output.status.success(),
        "a routed antigravity provider with no credential must fail the check; \
         stdout: {}",
        stdout(&output)
    );
    assert!(
        !stdout(&output).contains("config ok"),
        "the refused config must not also be reported ok: {}",
        stdout(&output)
    );
    // The message has to be actionable on its own: which provider, what is
    // missing, and the command that fixes it.
    let message = stderr(&output);
    for expected in [
        "provider `antigravity`",
        "no credential",
        "shunt login antigravity",
    ] {
        assert!(
            message.contains(expected),
            "the failure must name {expected:?}: {message}"
        );
    }
}

#[test]
fn check_accepts_a_routed_antigravity_provider_with_a_credential() {
    let dir = TempDir::new("routed-with-credential");
    let config = dir.config("[server]\ndefault_provider = \"antigravity\"\n");

    let output = check(&config, &dir.credential());

    assert!(
        output.status.success(),
        "a credential at the probed path must satisfy the guard; stderr: {}",
        stderr(&output)
    );
    assert!(stdout(&output).contains("config ok"), "{}", stdout(&output));
}

#[test]
fn check_accepts_an_empty_credential_because_the_probe_is_presence_only() {
    // Pins a deliberate limit rather than a bug. The guard asks whether the
    // credential file exists, never what is in it, so an empty (or stale, or
    // malformed) file passes `check` and fails later on the request path.
    //
    // This test exists because the docs make that claim in prose
    // (README.md, site guides/providers.mdx). Without it the claim is
    // unfalsifiable, and a future change that started parsing the credential
    // here would silently tighten what `shunt check` accepts relative to
    // `shunt run` — reopening the divergence issue #382 closed.
    let dir = TempDir::new("empty-credential");
    let config = dir.config("[server]\ndefault_provider = \"antigravity\"\n");
    let credential = dir.credential_path();
    std::fs::write(&credential, "").expect("write empty credential");

    let output = check(&config, &credential);

    assert!(
        output.status.success(),
        "presence-only means a zero-byte credential satisfies the guard; stderr: {}",
        stderr(&output)
    );
    assert!(stdout(&output).contains("config ok"), "{}", stdout(&output));
}

#[test]
fn check_accepts_the_built_in_antigravity_provider_when_nothing_routes_to_it() {
    // Every default config seeds a built-in `antigravity` provider, so a guard
    // keyed on the provider merely *existing* would fail this — that is, every
    // config anyone has ever written — rather than on the config being able to
    // send a request to it. Keying on routing is the whole reason the run-path
    // guard is shaped the way it is; `check` inherits that, not a copy of it.
    let dir = TempDir::new("seeded-not-routed");
    let config = dir.config("[server]\ndefault_provider = \"anthropic\"\n");

    let output = check(&config, &dir.credential_path());

    assert!(
        output.status.success(),
        "a config that never routes to antigravity must pass with no credential; \
         stderr: {}",
        stderr(&output)
    );
    assert!(stdout(&output).contains("config ok"), "{}", stdout(&output));
}

#[test]
fn check_accepts_routing_to_the_antigravity_cli_transport_without_a_credential() {
    // `antigravity-cli` is the deprecated `agy` subprocess transport. It
    // authenticates through the local CLI, not through the shunt-owned
    // credential file, so the native upstream's guard must not fire on it —
    // the migration path #372 offers is precisely "route to `antigravity-cli`
    // to stay put", and a guard that refused it would close that path.
    let dir = TempDir::new("cli-transport");
    let config = dir.config("[server]\ndefault_provider = \"antigravity-cli\"\n");

    let output = check(&config, &dir.credential_path());

    assert!(
        output.status.success(),
        "antigravity_cli routing must be unaffected by the native guard; stderr: {}",
        stderr(&output)
    );
    assert!(stdout(&output).contains("config ok"), "{}", stdout(&output));
}

/// A real process-level rejection smoke, not evidence of live Go support.
/// Crate-local router tests separately prove zero credential seam calls.
#[tokio::test]
async fn opencode_go_cli_negative() {
    use std::{net::TcpListener, process::Stdio, time::Duration};

    struct Gateway(std::process::Child);
    impl Drop for Gateway {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    let dir = TempDir::new("opencode-go-negative");
    let home = dir.0.clone();
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve gateway address");
    let address = listener.local_addr().unwrap();
    assert_ne!(address.port(), 10100);
    // Keep the proxy port reserved but never forward anything. The Chat client
    // honors these proxy variables, containing an accidental dispatch to loopback.
    let proxy = TcpListener::bind("127.0.0.1:0").expect("reserve outbound sentinel");
    proxy.set_nonblocking(true).unwrap();
    let proxy_url = format!("http://{}", proxy.local_addr().unwrap());
    let config = dir.config(&format!(
        r#"[server]
bind = "{address}"
default_provider = "go"

[[upstreams]]
name = "go"
provider = "opencode-go"
base_url = "https://opencode.ai/zen/go/v1"

[[routes]]
model = "go-smoke"
provider = "go"
upstream_model = "glm-5.3-flash"
"#
    ));
    drop(listener);
    let mut gateway = Gateway(
        Command::new(env!("CARGO_BIN_EXE_shunt"))
            .args(["run", "--config"])
            .arg(&config)
            .current_dir(&dir.0)
            .env_clear()
            .env("OPENCODEX_HOME", &dir.0)
            .env("OPENCODEX_PORT", address.port().to_string())
            .env("SHUNT_OPENCODE_GO_API_KEY", "synthetic-go-smoke-key")
            .env("HTTP_PROXY", &proxy_url)
            .env("HTTPS_PROXY", &proxy_url)
            .env("ALL_PROXY", &proxy_url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start owned gateway"),
    );
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    let base = format!("http://{address}");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        assert!(
            gateway.0.try_wait().unwrap().is_none(),
            "gateway exited before readiness"
        );
        if client
            .head(&base)
            .send()
            .await
            .is_ok_and(|r| r.status().is_success())
        {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "gateway readiness timed out"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    // Unary and streaming clients must both hit admission, not a startup error.
    for stream in [false, true] {
        let response = client
            .post(format!("{base}/v1/messages"))
            .json(&serde_json::json!({
                "model": "go-smoke", "max_tokens": 8, "stream": stream,
                "messages": [{"role": "user", "content": "synthetic smoke"}]
            }))
            .send()
            .await
            .expect("actual routed request");
        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["type"], "error");
        assert_eq!(body["error"]["type"], "invalid_request_error");
        assert_eq!(
            body["error"]["message"],
            "OpenCode Go selection is not admitted by exact evidence"
        );
    }
    assert!(
        matches!(proxy.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock),
        "unexpected outbound proxy connection"
    );
    drop(gateway);
    let files: Vec<_> = std::fs::read_dir(&home)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(
        files,
        [std::ffi::OsString::from("shunt.toml")],
        "unexpected persisted state"
    );
    drop(dir);
    assert!(!home.exists(), "temporary home must be removed");
}
