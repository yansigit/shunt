//! Crate-local OpenCode Go evidence-gate tests.
//!
//! These tests drive the real router (injected credential resolver + HTTP
//! client, real loopback upstream fixture) and prove that an unadmitted
//! OpenCode Go selection fails before the credential seam or any upstream
//! socket, while a generic control still routes end to end. The Go provider
//! keeps its canonical base URL; the test HTTP client pins DNS for every name
//! to the loopback fixture, so even a regression that let a Go request through
//! could never touch the real provider network.

use std::{
    collections::BTreeMap,
    fs,
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{Request, StatusCode, Uri},
    response::IntoResponse,
    Json, Router,
};
use bytes::Bytes;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::{
    adapters::responses::inbound::InboundOperation,
    auth::{Credential, CredentialFuture, CredentialResolver},
    codex_endpoint,
    config::{
        ApiKeyHeader, AuthMap, AuthMode, CodexEndpointConfig, Config, ModelConfig, ProviderConfig,
        ProviderKind, RetryConfig, UpstreamAuth, UpstreamConfig,
    },
    routing::Route,
    server::build_router_with_test_dependencies,
};

// ---------------------------------------------------------------------------
// Config acceptance (D-02): the exact preset is admitted; lookalike
// destination/auth/env combinations for this kind are rejected at boot.
// ---------------------------------------------------------------------------

#[test]
fn opencode_go_config_acceptance() {
    let path = std::env::temp_dir().join(format!(
        "shunt-opencode-go-acceptance-{}-{}.toml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before epoch")
            .as_nanos()
    ));
    let synthetic = r#"
[server]
default_provider = "go"

[[upstreams]]
name = "go"
provider = "opencode-go"
"#;
    fs::write(&path, synthetic).expect("write synthetic config");
    let result = Config::load(Some(&path));
    let _ = fs::remove_file(&path);
    assert!(result.is_ok(), "{result:?}");
    let config = result.expect("preset acceptance");
    let provider = config
        .provider("go")
        .expect("preset expands into a provider");
    assert_eq!(provider.kind, ProviderKind::OpenCodeGo);
    assert_eq!(provider.base_url, "https://opencode.ai/zen/go/v1");
    assert_eq!(provider.auth, AuthMode::ApiKey);
    assert_eq!(
        provider.api_key_env.as_deref(),
        Some("SHUNT_OPENCODE_GO_API_KEY")
    );
}

#[test]
fn opencode_go_config_rejects_noncanonical_identity() {
    let base = r#"
[server]
default_provider = "go"

[providers.go]
kind = "opencode_go"
base_url = "https://opencode.ai/zen/go/v1"
auth = "api_key"
api_key_env = "SHUNT_OPENCODE_GO_API_KEY"
"#;
    let cases: [(&str, String); 3] = [
        (
            "wrong destination",
            base.replace(
                "https://opencode.ai/zen/go/v1",
                "https://evil.example.com/zen/go/v1",
            ),
        ),
        (
            "wrong auth",
            base.replace("auth = \"api_key\"", "auth = \"passthrough\""),
        ),
        (
            "wrong env",
            base.replace(
                "SHUNT_OPENCODE_GO_API_KEY",
                "SHUNT_OPENCODE_GO_LOOKALIKE_KEY",
            ),
        ),
    ];
    for (label, synthetic) in cases {
        let path = std::env::temp_dir().join(format!(
            "shunt-opencode-go-identity-{}-{}-{}.toml",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock before epoch")
                .as_nanos(),
            label.replace(" ", "-")
        ));
        fs::write(&path, synthetic).expect("write synthetic config");
        let result = Config::load(Some(&path));
        let _ = fs::remove_file(&path);
        let error = result.expect_err("{label} must fail config validation");
        assert!(
            error.to_string().contains("OpenCode Go"),
            "{label}: unexpected error {error}"
        );
    }
}

// ---------------------------------------------------------------------------
// Shared admission gate unit behavior (empty allowlist, fallback filtering).
// ---------------------------------------------------------------------------

fn route(provider: &str) -> Route {
    Route {
        provider: provider.into(),
        adapter: crate::routing::AdapterKind::OpenAiChat,
        model: "alias".into(),
        upstream_model: "model".into(),
        effort: None,
        service_tier: None,
    }
}

#[test]
fn opencode_go_admission_is_empty_and_preserves_generic_fallbacks() {
    use crate::proxy::capability::enforce_opencode_go_admission;

    let mut config = Config::default();
    config.providers.insert(
        "go".into(),
        ProviderConfig {
            kind: ProviderKind::OpenCodeGo,
            base_url: "https://opencode.ai/zen/go/v1".into(),
            auth: AuthMode::ApiKey,
            api_key_env: Some("SHUNT_OPENCODE_GO_API_KEY".into()),
            ..config.providers["openai"].clone()
        },
    );
    let mut routes = vec![route("openai"), route("go")];
    assert!(enforce_opencode_go_admission(&config, &mut routes).is_ok());
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].provider, "openai");

    let mut primary = vec![route("go")];
    assert!(enforce_opencode_go_admission(&config, &mut primary).is_err());
}

// ---------------------------------------------------------------------------
// Real-router boundary evidence (D-03): injected credential seam counters plus
// a real loopback upstream fixture behind a DNS-pinned client.
// ---------------------------------------------------------------------------

/// Credential seam counters, separated by provider kind so a Go lookup is
/// observable even when generic lookups legitimately happen.
#[derive(Default)]
struct SeamCounters {
    go_resolves: AtomicUsize,
    generic_resolves: AtomicUsize,
}

struct CountingResolver {
    counters: Arc<SeamCounters>,
}

impl CredentialResolver for CountingResolver {
    fn resolve<'a>(
        &'a self,
        config: &'a Config,
        route: &'a Route,
        _client: &'a reqwest::Client,
    ) -> CredentialFuture<'a> {
        let counters = Arc::clone(&self.counters);
        let is_go = config
            .provider(&route.provider)
            .is_some_and(|provider| provider.kind == ProviderKind::OpenCodeGo);
        Box::pin(async move {
            if is_go {
                counters.go_resolves.fetch_add(1, Ordering::SeqCst);
            } else {
                counters.generic_resolves.fetch_add(1, Ordering::SeqCst);
            }
            Ok(Credential::ApiKey {
                value: "synthetic-boundary-key".to_string(),
                header: ApiKeyHeader::Bearer,
            })
        })
    }
}

/// Loopback upstream fixture: counts requests per provider path so any leaked
/// Go egress (even without a credential) is observable.
#[derive(Default)]
struct FixtureCounts {
    generic_requests: AtomicUsize,
    go_requests: AtomicUsize,
}

async fn fixture_upstream(
    State(counts): State<Arc<FixtureCounts>>,
    uri: Uri,
    body: Body,
) -> axum::response::Response {
    let bytes = to_bytes(body, usize::MAX).await.expect("fixture body");
    let _payload: Value = serde_json::from_slice(&bytes).expect("fixture receives a JSON body");
    // The canonical Go base URL is https://opencode.ai/zen/go/v1, so leaked
    // Go egress arrives on a /zen/go/... path, not a /go/ prefix.
    if uri.path().contains("/zen/go/") {
        counts.go_requests.fetch_add(1, Ordering::SeqCst);
    } else {
        counts.generic_requests.fetch_add(1, Ordering::SeqCst);
    }
    Json(json!({
        "id": "chatcmpl-fixture",
        "object": "chat.completion",
        "created": 1,
        "model": "gpt-boundary",
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": "fixture reply"},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 5, "completion_tokens": 2, "total_tokens": 7}
    }))
    .into_response()
}

struct Fixture {
    base_url: String,
    address: SocketAddr,
    counts: Arc<FixtureCounts>,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn start_fixture() -> Fixture {
    let counts = Arc::new(FixtureCounts::default());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind loopback fixture");
    let address = listener.local_addr().expect("fixture address");
    let app = Router::new()
        .fallback(fixture_upstream)
        .with_state(Arc::clone(&counts));
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("fixture server");
    });
    Fixture {
        base_url: format!("http://{address}"),
        address,
        counts,
        task,
    }
}

/// Test client whose DNS resolves every name to the loopback fixture. The Go
/// provider keeps its canonical https://opencode.ai/zen/go/v1 base URL, so a
/// leaked Go request can only ever reach this fixture's port — never the real
/// provider network.
struct FixtureDns {
    address: SocketAddr,
}

impl Resolve for FixtureDns {
    fn resolve(&self, _name: Name) -> Resolving {
        let address = self.address;
        Box::pin(async move {
            let addrs: Addrs = Box::new(std::iter::once(address));
            Ok(addrs)
        })
    }
}

fn fixture_client(address: SocketAddr) -> reqwest::Client {
    reqwest::Client::builder()
        .dns_resolver(Arc::new(FixtureDns { address }))
        .build()
        .expect("build fixture-pinned client")
}

fn upstream_config(
    name: &str,
    kind: ProviderKind,
    base_url: String,
    api_key_env: &str,
) -> UpstreamConfig {
    UpstreamConfig {
        name: name.into(),
        provider: None,
        kind: Some(kind),
        base_url: Some(base_url),
        auth: Some(UpstreamAuth::Map(AuthMap::ApiKey {
            env: Some(api_key_env.into()),
            header: ApiKeyHeader::Bearer,
        })),
        effort: None,
        service_tier: None,
        count_tokens: Default::default(),
        websocket: false,
        tool_search: None,
        request_compression: true,
        retry: RetryConfig {
            max_retries: 0,
            ..Default::default()
        },
        workspace_roots: Vec::new(),
        sandbox: true,
    }
}

/// A bound-then-closed loopback port: connecting to it fails immediately, the
/// way a dead upstream would.
async fn dead_port() -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind throwaway port");
    let address = listener.local_addr().expect("throwaway address");
    drop(listener);
    address
}

/// Boundary config built through the production ordered-upstreams path: a
/// generic OpenAI-Chat control on the loopback fixture, a dead generic
/// upstream (for the fallback-drop scenario), and the canonical Go provider.
/// Declaration order fixes upstream_order; model maps pick each chain.
async fn boundary_config(fixture: &Fixture, model_map: &[(&str, &str)]) -> Config {
    let dead = dead_port().await;
    let mut config = Config {
        upstreams: vec![
            upstream_config(
                "generic",
                ProviderKind::OpenAiChat,
                format!("{}/generic", fixture.base_url),
                "SHUNT_OPENCODE_GO_BOUNDARY_GENERIC_KEY",
            ),
            upstream_config(
                "generic_dead",
                ProviderKind::OpenAiChat,
                format!("http://{dead}/generic-dead"),
                "SHUNT_OPENCODE_GO_BOUNDARY_DEAD_KEY",
            ),
            upstream_config(
                "go",
                ProviderKind::OpenCodeGo,
                "https://opencode.ai/zen/go/v1".into(),
                "SHUNT_OPENCODE_GO_API_KEY",
            ),
            // The inbound Codex endpoint only admits chatgpt_oauth providers at
            // validation time; the loopback exemption in the chatgpt host guard is
            // what lets the native-endpoint tests run against the fixture.
            UpstreamConfig {
                name: "codex".into(),
                provider: None,
                kind: Some(ProviderKind::Responses),
                base_url: Some(format!("{}/codex", fixture.base_url)),
                auth: Some(UpstreamAuth::Map(AuthMap::ChatgptOauth {
                    account: None,
                    accounts: None,
                })),
                effort: None,
                service_tier: None,
                count_tokens: Default::default(),
                websocket: false,
                tool_search: None,
                request_compression: true,
                retry: RetryConfig {
                    max_retries: 0,
                    ..Default::default()
                },
                workspace_roots: Vec::new(),
                sandbox: true,
            },
        ],
        ..Default::default()
    };
    config.server.default_provider = "generic".into();
    config.models = vec![ModelConfig {
        id: "boundary-model".into(),
        display_name: None,
        upstream_model: Some(
            model_map
                .iter()
                .map(|(provider, model)| ((*provider).to_string(), (*model).to_string()))
                .collect::<BTreeMap<_, _>>(),
        ),
    }];
    config
}

fn boundary_router(config: Config, fixture: &Fixture) -> (Router, Arc<SeamCounters>) {
    let counters = Arc::new(SeamCounters::default());
    let router = build_router_with_test_dependencies(
        config,
        fixture_client(fixture.address),
        Arc::new(CountingResolver {
            counters: Arc::clone(&counters),
        }),
    )
    .expect("boundary router builds")
    .0;
    (router, counters)
}

fn post_json(path: &'static str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .expect("build request")
}

fn messages_request(model: &str) -> Request<Body> {
    post_json(
        "/v1/messages",
        json!({
            "model": model,
            "max_tokens": 64,
            "stream": false,
            "messages": [{"role": "user", "content": "boundary fixture"}]
        }),
    )
}

fn count_tokens_request(model: &str) -> Request<Body> {
    post_json(
        "/v1/messages/count_tokens",
        json!({
            "model": model,
            "messages": [{"role": "user", "content": "boundary fixture"}]
        }),
    )
}

async fn response_text(response: axum::response::Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    String::from_utf8_lossy(&bytes).into_owned()
}

fn assert_zero_go(counters: &SeamCounters, fixture: &Fixture, label: &str) {
    assert_eq!(
        counters.go_resolves.load(Ordering::SeqCst),
        0,
        "{label}: Go credential lookups must stay zero"
    );
    assert_eq!(
        fixture.counts.go_requests.load(Ordering::SeqCst),
        0,
        "{label}: Go upstream requests must stay zero"
    );
}

#[tokio::test]
async fn opencode_go_router_boundaries_go_primary_fails_before_seams() {
    let fixture = start_fixture().await;
    let (router, counters) = boundary_router(
        boundary_config(&fixture, &[("go", "go-boundary")]).await,
        &fixture,
    );
    let response = router
        .oneshot(messages_request("boundary-model"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let text = response_text(response).await;
    assert!(text.contains("not admitted"), "{text}");
    assert_zero_go(&counters, &fixture, "Go primary");
    assert_eq!(
        counters.generic_resolves.load(Ordering::SeqCst),
        0,
        "a Go primary must not fall through to generic seams"
    );
    assert_eq!(
        fixture.counts.generic_requests.load(Ordering::SeqCst),
        0,
        "a Go primary must not dispatch upstream"
    );
}

#[tokio::test]
async fn opencode_go_router_boundaries_generic_primary_serves_and_go_fallback_drops() {
    let fixture = start_fixture().await;
    let mut config = boundary_config(&fixture, &[("generic", "gpt-boundary")]).await;
    config.models[0].id = "control-model".into();
    // The boundary model chains a dead generic leg ahead of Go: only the
    // admission gate stands between a live request and Go dispatch once the
    // dead leg fails with a connection error.
    config.models.push(ModelConfig {
        id: "boundary-model".into(),
        display_name: None,
        upstream_model: Some(
            [
                ("generic_dead".to_string(), "gpt-boundary".to_string()),
                ("go".to_string(), "go-boundary".to_string()),
            ]
            .into_iter()
            .collect(),
        ),
    });
    let (router, counters) = boundary_router(config, &fixture);
    // Positive control: the healthy generic leg serves a real end-to-end 200.
    let response = router
        .clone()
        .oneshot(messages_request("control-model"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let text = response_text(response).await;
    assert!(text.contains("fixture reply"), "{text}");
    assert!(
        counters.generic_resolves.load(Ordering::SeqCst) >= 1,
        "the generic positive control must reach the credential seam"
    );
    assert_eq!(
        fixture.counts.generic_requests.load(Ordering::SeqCst),
        1,
        "the generic positive control must reach the loopback upstream",
    );
    assert_zero_go(&counters, &fixture, "healthy generic primary");
    // The dead generic leg fails before headers, so the failover loop advances
    // — and the only later candidate is Go, which the admission gate must have
    // dropped from the chain. A 502 back to the client with zero Go seam
    // traffic proves the drop happened on a live request, and this test fails
    // loudly (nonzero Go seam counters) if the gate is ever removed.
    let response = router
        .clone()
        .oneshot(messages_request("boundary-model"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_zero_go(&counters, &fixture, "dead generic with Go fallback");
}

#[tokio::test]
async fn opencode_go_router_boundaries_count_tokens_rejects_go_and_serves_generic_control() {
    let fixture = start_fixture().await;

    let (go_router, go_counters) = boundary_router(
        boundary_config(&fixture, &[("go", "go-boundary")]).await,
        &fixture,
    );
    let response = go_router
        .oneshot(count_tokens_request("boundary-model"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let text = response_text(response).await;
    assert!(text.contains("not admitted"), "{text}");
    assert_zero_go(&go_counters, &fixture, "Go count_tokens");
    assert_eq!(
        go_counters.generic_resolves.load(Ordering::SeqCst),
        0,
        "Go count_tokens must not fall through to generic seams"
    );

    let (generic_router, generic_counters) = boundary_router(
        boundary_config(&fixture, &[("generic", "gpt-boundary")]).await,
        &fixture,
    );
    let response = generic_router
        .oneshot(count_tokens_request("boundary-model"))
        .await
        .unwrap();
    // OpenAiChat count_tokens is forced to estimate mode by design (no
    // Chat-specific tiktoken counter), so the honest router answer is the
    // 501 not_supported fallback — served without touching any seam.
    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    let text = response_text(response).await;
    assert!(text.contains("not available"), "{text}");
    assert_eq!(
        generic_counters.generic_resolves.load(Ordering::SeqCst),
        0,
        "estimate-mode count_tokens must not touch the credential seam"
    );
    assert_eq!(
        fixture.counts.generic_requests.load(Ordering::SeqCst),
        0,
        "estimate-mode count_tokens must not dispatch upstream"
    );
    assert_zero_go(&generic_counters, &fixture, "generic count_tokens control");
}

/// Native inbound Codex endpoint: a model exactly mapped to the Go provider is
/// rejected by resolve_native_inbound before any credential or client seam.
#[tokio::test]
async fn opencode_go_router_boundaries_native_inbound_exact_go_rejected() {
    let fixture = start_fixture().await;
    let mut config = boundary_config(&fixture, &[("go", "go-boundary")]).await;
    config.server.codex_endpoint = Some(CodexEndpointConfig {
        provider: "codex".into(),
        collaboration: false,
    });
    let (router, counters) = boundary_router(config, &fixture);
    let response = router
        .oneshot(post_json(
            "/v1/responses",
            json!({"model": "boundary-model", "input": "boundary fixture", "stream": false}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let text = response_text(response).await;
    assert!(text.contains("unsupported provider"), "{text}");
    assert_zero_go(&counters, &fixture, "native exact Go mapping");
    assert_eq!(
        counters.generic_resolves.load(Ordering::SeqCst),
        0,
        "native Go rejection must not touch generic seams"
    );
}

/// Defense-in-depth at forward_turn: a pinned native Go route (the endpoint
/// provider pinned at Go behind the already-built router state) is rejected by
/// the shared admission gate before the resolver or client. Validated configs
/// can never pin the Codex endpoint at a non-chatgpt_oauth provider, so this
/// exercises the belt-and-braces gate that must keep failing closed if that
/// invariant ever changes.
#[tokio::test]
async fn opencode_go_router_boundaries_native_pinned_go_rejected_before_seams() {
    let fixture = start_fixture().await;
    let counters = Arc::new(SeamCounters::default());
    let mut config = boundary_config(&fixture, &[("generic", "gpt-boundary")]).await;
    config.server.codex_endpoint = Some(CodexEndpointConfig {
        provider: "codex".into(),
        collaboration: false,
    });
    let (_, _, mut state) = build_router_with_test_dependencies(
        config,
        fixture_client(fixture.address),
        Arc::new(CountingResolver {
            counters: Arc::clone(&counters),
        }),
    )
    .expect("native router builds");
    // Crate-local only: pin the (never-validated) endpoint at the Go provider
    // behind the live state and call the real forward_turn boundary.
    let mut pinned_config = (*state.config).clone();
    pinned_config.server.codex_endpoint = Some(CodexEndpointConfig {
        provider: "go".into(),
        collaboration: false,
    });
    state.config = Arc::new(pinned_config);
    let error = codex_endpoint::forward_turn(
        state,
        None,
        None,
        Default::default(),
        Bytes::from(json!({"input": "boundary fixture"}).to_string()),
        Instant::now(),
        InboundOperation::Responses,
    )
    .await
    .expect_err("pinned native Go must be rejected");
    assert_eq!(error.response.status(), StatusCode::BAD_REQUEST);
    assert!(error.message.contains("not admitted"), "{}", error.message);
    assert_zero_go(&counters, &fixture, "pinned native Go");
    assert_eq!(
        counters.generic_resolves.load(Ordering::SeqCst),
        0,
        "pinned native Go must not touch generic seams"
    );
}
