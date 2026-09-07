//! Synthetic router and OAuth fixtures; production URLs are never overridden.
use crate::auth::{Credential, CredentialFuture, CredentialResolver};
use crate::config::Config;
use serde_json::{json, Value};
use std::{net::SocketAddr, time::Duration};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

struct ReplayResolver {
    store: crate::auth::antigravity::auth::AntigravityAuthStore,
    resolves: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}
impl CredentialResolver for ReplayResolver {
    fn resolve<'a>(
        &'a self,
        _: &'a Config,
        _: &'a crate::routing::Route,
        _: &'a reqwest::Client,
    ) -> CredentialFuture<'a> {
        self.resolves
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Box::pin(async move { self.store.get_valid().await.map(as_credential) })
    }
    fn refresh<'a>(
        &'a self,
        _: &'a Config,
        _: &'a crate::routing::Route,
        _: &'a reqwest::Client,
        expected: &'a Credential,
    ) -> CredentialFuture<'a> {
        Box::pin(async move {
            let Credential::AntigravityOauth {
                account_fingerprint,
                project_id,
                ..
            } = expected
            else {
                unreachable!()
            };
            self.store
                .force_refresh_in_memory(account_fingerprint, project_id)
                .await
                .map(as_credential)
        })
    }
}
fn as_credential(c: crate::auth::antigravity::auth::AntigravityCred) -> Credential {
    Credential::AntigravityOauth {
        access_token: c.access_token,
        project_id: c.project_id,
        account_fingerprint: c.account_fingerprint,
    }
}

// ---------------------------------------------------------------------------
// Plan 11-05: bounded pre-commit 401 refresh/replay (ANT-08, D-16/D-17).
//
// Every scenario drives the real Axum gateway against loopback upstreams. The
// Credentials and upstreams are synthetic; OAuth URLs are injected privately.
// ---------------------------------------------------------------------------

/// A running gateway plus everything a 401 scenario needs cleaned up.
pub(super) struct ReplayGateway {
    addr: SocketAddr,
    handle: tokio::task::JoinHandle<()>,
    dir: std::path::PathBuf,
    pub(super) credential_path: std::path::PathBuf,
    /// Exact bytes of the synthetic credential file, captured before the
    /// request for the unchanged-file assertion.
    pub(super) credential_bytes: Vec<u8>,
    resolves: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl ReplayGateway {
    /// Start the gateway with a valid stored credential, a loopback
    /// catalog+inference backend, and a loopback token endpoint. A stored_email
    /// of None models the legacy refresh-token-keyed store.
    pub(super) async fn start(
        backend: &MockServer,
        token_server: &MockServer,
        access_token: &str,
        stored_email: Option<&str>,
        extra_config: &str,
    ) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "shunt-antigravity-401-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let credential_path = dir.join("antigravity-auth.json");
        let expiry = (std::time::SystemTime::now() + Duration::from_secs(3600))
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let mut stored = json!({
            "access_token": access_token,
            "refresh_token": "stored-refresh",
            "expiry_date": expiry,
            "project_id": "proj-401",
        });
        if let Some(email) = stored_email {
            stored["email"] = json!(email);
        }
        let credential_bytes = serde_json::to_vec(&stored).unwrap();
        std::fs::write(&credential_path, &credential_bytes).unwrap();

        let config_path = dir.join("shunt.toml");
        std::fs::write(
            &config_path,
            format!(
                "[server]\ndefault_provider = \"antigravity\"\n\n[providers.antigravity]\nauth = \"antigravity_oauth\"\nbase_url = \"{}\"\n{extra_config}",
                backend.uri()
            ),
        )
        .unwrap();
        let mut config = Config::load(Some(&config_path)).unwrap();
        config.server.bind = "127.0.0.1:0".to_string();
        let listener = tokio::net::TcpListener::bind(config.server.bind_addr().unwrap())
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();
        let resolves = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let resolver = std::sync::Arc::new(ReplayResolver {
            resolves: resolves.clone(),
            store: crate::auth::antigravity::auth::AntigravityAuthStore::with_urls(
                credential_path.clone(),
                reqwest::Client::new(),
                format!("{}/token", token_server.uri()),
                backend.uri(),
                backend.uri(),
            ),
        });
        let (app, _, _) =
            super::super::build_router_with_dependencies(config, reqwest::Client::new(), resolver)
                .unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        Self {
            addr,
            handle,
            dir,
            credential_path,
            credential_bytes,
            resolves,
        }
    }

    pub(super) async fn post_messages(&self, body: Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("http://{}/v1/messages", self.addr))
            .json(&body)
            .send()
            .await
            .unwrap()
    }

    pub(super) fn assert_credential_file_unchanged(&self) {
        assert_eq!(self.resolves.load(std::sync::atomic::Ordering::SeqCst), 1);
        let bytes = std::fs::read(&self.credential_path).unwrap();
        assert_eq!(
            bytes, self.credential_bytes,
            "the 401 replay seam must never write the credential file"
        );
    }
}

impl Drop for ReplayGateway {
    fn drop(&mut self) {
        self.handle.abort();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The one catalog the fixtures admit, mounted on the shared backend.
pub(super) async fn mount_catalog(backend: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/v1internal:fetchAvailableModels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "models": {"gemini-3.8-flash-tiered": {"model": "MODEL_PLACEHOLDER"}}
        })))
        .mount(backend)
        .await;
}

/// Mount the two bearer-keyed inference mocks: the pre-refresh bearer answers
/// with pre_refresh_status (typically 401) and the refreshed bearer with
/// refreshed_status. The expect() counts are the exact-attempt assertions.
pub(super) async fn mount_bearer_keyed_inference(
    backend: &MockServer,
    pre_refresh_status: u16,
    pre_refresh_expect: u64,
    refreshed_status: u16,
    refreshed_expect: u64,
) {
    let success_body = "data: {\"response\":{\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"OK\"}]},\"finishReason\":\"STOP\"}]}}\n\ndata: [DONE]\n\n";
    let body_for = |status: u16| {
        if status == 200 {
            ResponseTemplate::new(status).set_body_string(success_body)
        } else {
            ResponseTemplate::new(status).set_body_json(json!({
                "error": {"code": 401, "message": "Invalid credentials", "status": "UNAUTHENTICATED"}
            }))
        }
    };
    Mock::given(method("POST"))
        .and(path("/v1internal:streamGenerateContent"))
        .and(wiremock::matchers::header(
            "authorization",
            "Bearer pre-401-token",
        ))
        .respond_with(body_for(pre_refresh_status))
        .expect(pre_refresh_expect)
        .mount(backend)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1internal:streamGenerateContent"))
        .and(wiremock::matchers::header(
            "authorization",
            "Bearer refreshed-token",
        ))
        .respond_with(body_for(refreshed_status))
        .expect(refreshed_expect)
        .mount(backend)
        .await;
}

/// Mock token endpoint answering with a fresh (non-rotating) access token.
pub(super) async fn mount_token_success(token_server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "refreshed-token",
            "expires_in": 3600
        })))
        .mount(token_server)
        .await;
}

pub(super) fn client_body() -> Value {
    json!({
        "model": "gemini-3.8-flash-tiered",
        "max_tokens": 16,
        "messages": [{"role": "user", "content": "Reply with OK."}]
    })
}
