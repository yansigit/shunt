use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

use crate::{
    accounts::AccountPool,
    admin::{self, AdminAuth, AdminStores},
    auth::{inbound::InboundAuth, Credential, CredentialResolver, DefaultCredentialResolver},
    codex_analytics, codex_endpoint,
    concurrency::{limit_requests, ConcurrencyLimit},
    config::{Config, ConfigError},
    discovery,
    gateway::{self, GatewayAuth, GatewayStores},
    http_tuning::{enforce_http_tuning, HttpTuningLayer},
    oauth_usage, protocol, proxy,
    reload::{RuntimeState, SharedState},
    routes,
    upstream_status::StatusStore,
    usage,
};

#[derive(Clone)]
pub struct AppState {
    /// Per-request config snapshot (see [`AppState::refreshed`]).
    pub config: Arc<Config>,
    pub http_client: reqwest::Client,
    pub accounts: Arc<AccountPool>,
    /// Observation-only store of the most recently polled upstream provider
    /// status (`[server.status]`). Process-lifetime like `accounts`, and
    /// likewise never consulted by routing, failover, or pool/cooldown
    /// decisions — see [`crate::status_poll`].
    pub status: Arc<StatusStore>,
    /// Inbound client-token auth snapshot for this request (None ⇒ open).
    pub inbound_auth: Option<Arc<InboundAuth>>,
    /// Admin-surface auth snapshot for this request (None ⇒ admin disabled).
    /// Re-snapshotted per request so token/header edits hot-apply.
    pub admin_auth: Option<Arc<AdminAuth>>,
    /// Process-lifetime admin session/pending/rate-limit stores. Like
    /// [`AppState::accounts`], created once and kept across reloads so an
    /// operator's browser session is not dropped by an unrelated config edit.
    pub admin_stores: Arc<AdminStores>,
    /// Gateway-login JWT/approval snapshot for this request (None ⇒ disabled).
    pub gateway_auth: Option<Arc<GatewayAuth>>,
    /// Process-lifetime device grants, IdP states/cache, refresh tokens, and limits.
    pub gateway_stores: Arc<GatewayStores>,
    /// Whether the listener this process actually bound at startup is
    /// loopback. Fixed at boot like `server.bind` itself (see
    /// `reload::warn_on_restart_only_changes`): a reload can rewrite
    /// `config.server.bind`, but the socket already accepting connections
    /// does not move until a restart, so auth gates that key off "is this
    /// bind loopback" must key off this boot-time value, never
    /// `state.config.server.bind_addr()`.
    pub boot_is_loopback: bool,
    credential_resolver: Arc<dyn CredentialResolver>,
    #[cfg(test)]
    default_resolver_calls: Option<Arc<std::sync::atomic::AtomicUsize>>,
    /// The live, hot-swappable runtime state a reload updates. Private so the
    /// only way in is a snapshot method that keeps `config`/`inbound_auth`/
    /// `admin_auth` consistent with it.
    shared: SharedState,
}

struct RequestDependencies {
    http_client: reqwest::Client,
    credential_resolver: Arc<dyn CredentialResolver>,
    #[cfg(test)]
    default_resolver_calls: Option<Arc<std::sync::atomic::AtomicUsize>>,
}

impl RequestDependencies {
    fn production(http_client: reqwest::Client) -> Self {
        let resolver = DefaultCredentialResolver::default();
        Self {
            http_client,
            #[cfg(test)]
            default_resolver_calls: Some(Arc::clone(&resolver.calls)),
            credential_resolver: Arc::new(resolver),
        }
    }
}

impl AppState {
    /// Build state from a config, owning a fresh shared store. Used by tests and
    /// by callers that do not thread an external [`SharedState`].
    pub fn new(config: Config, http_client: reqwest::Client) -> Result<Self, ConfigError> {
        let boot_is_loopback = boot_is_loopback(&config);
        let spend_state_path = spend_state_path(&config);
        let rate_limits = config.server.rate_limits.clone();
        let runtime = RuntimeState::from_config(config)?;
        let shared: SharedState = Arc::new(arc_swap::ArcSwap::from_pointee(runtime));
        Ok(Self::from_shared(
            shared,
            http_client,
            Arc::new(AccountPool::new()),
            Arc::new(StatusStore::new()),
            Arc::new(AdminStores::new()),
            Arc::new(GatewayStores::new(&rate_limits, spend_state_path)),
            boot_is_loopback,
        ))
    }

    /// Snapshot the current runtime state from an existing shared store.
    pub fn from_shared(
        shared: SharedState,
        http_client: reqwest::Client,
        accounts: Arc<AccountPool>,
        status: Arc<StatusStore>,
        admin_stores: Arc<AdminStores>,
        gateway_stores: Arc<GatewayStores>,
        boot_is_loopback: bool,
    ) -> Self {
        Self::from_shared_with_dependencies(
            shared,
            RequestDependencies::production(http_client),
            accounts,
            status,
            admin_stores,
            gateway_stores,
            boot_is_loopback,
        )
    }

    fn from_shared_with_dependencies(
        shared: SharedState,
        dependencies: RequestDependencies,
        accounts: Arc<AccountPool>,
        status: Arc<StatusStore>,
        admin_stores: Arc<AdminStores>,
        gateway_stores: Arc<GatewayStores>,
        boot_is_loopback: bool,
    ) -> Self {
        let current = shared.load();
        Self {
            config: current.config.clone(),
            inbound_auth: current.inbound_auth.clone(),
            admin_auth: current.admin_auth.clone(),
            gateway_auth: current.gateway_auth.clone(),
            http_client: dependencies.http_client,
            accounts,
            status,
            admin_stores,
            gateway_stores,
            boot_is_loopback,
            #[cfg(test)]
            default_resolver_calls: dependencies.default_resolver_calls,
            credential_resolver: dependencies.credential_resolver,
            shared,
        }
    }

    pub(crate) async fn resolve_route_credential(
        &self,
        route: &crate::routing::Route,
    ) -> Result<Credential, crate::adapters::AdapterError> {
        self.credential_resolver
            .resolve(&self.config, route, &self.http_client)
            .await
    }

    /// Re-snapshot the live shared state into a new `AppState`, so a request
    /// entry picks up the latest reloaded config while holding one stable
    /// snapshot for the whole request. Cheap: clones `Arc`s and the client.
    pub(crate) fn refreshed(&self) -> Self {
        Self::from_shared_with_dependencies(
            self.shared.clone(),
            RequestDependencies {
                http_client: self.http_client.clone(),
                credential_resolver: self.credential_resolver.clone(),
                #[cfg(test)]
                default_resolver_calls: self.default_resolver_calls.clone(),
            },
            self.accounts.clone(),
            self.status.clone(),
            self.admin_stores.clone(),
            self.gateway_stores.clone(),
            self.boot_is_loopback,
        )
    }
}

fn spend_state_path(config: &Config) -> Option<std::path::PathBuf> {
    config
        .server
        .spend
        .as_ref()?
        .state_path()
        .map(ToOwned::to_owned)
}

/// Whether `config.server.bind` (the value in force at process startup,
/// before any reload can rewrite it) resolves to a loopback address. Computed
/// once, from the pre-boot config, and then carried unchanged across reloads
/// by [`AppState`]/[`AppState::refreshed`] — see `boot_is_loopback`'s field
/// docs for why a reloaded config must never be consulted here.
fn boot_is_loopback(config: &Config) -> bool {
    config
        .server
        .bind_addr()
        .is_ok_and(|addr| addr.ip().is_loopback())
}

/// Build the router and return it alongside the [`SharedState`] it reads and a
/// clone of the request [`AppState`], so the caller can spawn reload watchers
/// that hot-swap the same store and background tasks (the usage poller) that
/// share the same [`AccountPool`] the request handlers use.
pub fn build_router(config: Config) -> Result<(Router, SharedState, AppState), ConfigError> {
    let resolver = DefaultCredentialResolver::default();
    #[cfg(test)]
    let resolver_calls = Arc::clone(&resolver.calls);
    let result = build_router_with_dependencies(config, reqwest::Client::new(), Arc::new(resolver));
    #[cfg(test)]
    return attach_default_resolver_calls(result, resolver_calls);
    #[cfg(not(test))]
    result
}

#[cfg(test)]
fn attach_default_resolver_calls(
    mut result: Result<(Router, SharedState, AppState), ConfigError>,
    calls: Arc<std::sync::atomic::AtomicUsize>,
) -> Result<(Router, SharedState, AppState), ConfigError> {
    if let Ok((_, _, state)) = result.as_mut() {
        state.default_resolver_calls = Some(calls);
    }
    result
}

fn build_router_with_dependencies(
    config: Config,
    http_client: reqwest::Client,
    credential_resolver: Arc<dyn CredentialResolver>,
) -> Result<(Router, SharedState, AppState), ConfigError> {
    // Validate before deriving boot-fixed layers and stores. `Config::load` already
    // validates, but callers may construct `Config` programmatically.
    let runtime = RuntimeState::from_config(config)?;
    let config = runtime.config.as_ref();
    // The inbound concurrency gate is fixed at boot because its semaphore is
    // installed as a router layer. Reloaded values take effect after restart.
    let max_concurrent_requests = config.server.max_concurrent_requests;
    let http_tuning = HttpTuningLayer::new(
        config.server.access_control.clone(),
        config.server.limits.clone(),
    );
    let http_tuning_enabled = config.server.access_control.enabled()
        || config.server.limits.max_request_header_bytes.is_some()
        || config.server.limits.max_url_length.is_some();
    // Whether the admin surface exists is decided once here, from the initial
    // config: a reload cannot add or drop routes (it only re-resolves tokens).
    let admin_enabled = config.server.admin.is_some();
    // Gateway-login routes are likewise fixed at boot; signing/user edits are
    // re-resolved through `gateway_auth`, while toggling requires a restart.
    let gateway_enabled = config.server.gateway.is_some();
    // Spend-limit routes are registered from `[server.spend]` alone: they
    // authenticate with the `[server.admin]` credential, so they do not need —
    // and must not require — the gateway login surface.
    let spend_enabled = config.server.spend.is_some();
    // The inbound Responses (Codex) routes are likewise registered once from the
    // initial config; a reload can only change the target provider, not add or
    // drop the routes.
    let codex_endpoint_enabled = config.server.codex_endpoint.is_some();
    // The client-facing usage endpoint (`GET /usage`) is likewise registered once
    // from the initial config; a reload only re-resolves the client tokens it
    // authenticates against, it cannot add or drop the route.
    let usage_enabled = config.server.usage.is_some();
    // The Claude Code CLI's own native usage-bar synthesizer (`GET
    // /api/oauth/usage`, M-A) is likewise registered once from the initial
    // config; a reload only re-resolves the auth it gates against on a
    // non-loopback bind, it cannot add or drop the route.
    let oauth_usage_enabled = config.server.oauth_usage.is_some();
    // Like the `_enabled` flags above, the bind's loopback-ness is a boot-time
    // capability: `oauth_usage::get` gates auth on it, and a later reload
    // rewriting `server.bind` must not move that gate without a restart (see
    // `AppState::boot_is_loopback`).
    let boot_is_loopback = boot_is_loopback(config);
    let spend_state_path = spend_state_path(config);
    let rate_limits = config.server.rate_limits.clone();
    let shared: SharedState = Arc::new(arc_swap::ArcSwap::from_pointee(runtime));
    let state = AppState::from_shared_with_dependencies(
        shared.clone(),
        RequestDependencies {
            http_client,
            credential_resolver,
            #[cfg(test)]
            default_resolver_calls: None,
        },
        Arc::new(AccountPool::new()),
        Arc::new(StatusStore::new()),
        Arc::new(AdminStores::new()),
        Arc::new(GatewayStores::new(&rate_limits, spend_state_path)),
        boot_is_loopback,
    );

    // `/` and `/health` stay unauthenticated and outside the inbound concurrency
    // gate: healthcheck tools rarely carry tokens, and shedding liveness probes
    // under load could make a load balancer evict an instance that is still
    // serving work. They expose no config, credentials, or upstream details —
    // only version, status, and the already-public endpoint list. Discovery
    // handlers enforce their own endpoint-specific auth policy against the same
    // refreshed state.
    let liveness_router = Router::new()
        .route("/", get(root_index))
        .route("/health", get(health));
    let model_discovery = if codex_endpoint_enabled {
        get(discovery::get_negotiated)
    } else {
        get(discovery::get)
    };
    let mut router = Router::new()
        .route("/protocol", get(protocol::get))
        .route("/v1/models", model_discovery)
        .route("/routes", get(routes::get))
        .route("/v1/messages", post(proxy::post))
        .route("/v1/messages/count_tokens", post(proxy::post));

    // Opt-in admin surface (M9): registered only when `[server.admin]` is set,
    // so the default HTTP surface is unchanged. Its handlers authenticate every
    // request against the separate `[server.admin]` credential.
    if admin_enabled {
        router = router.merge(admin::admin_router());
    }

    // Opt-in Claude apps gateway surface (M-A/M-B): registered only when
    // `[server.gateway]` was present at boot. OAuth handlers remain unauthenticated;
    // minted JWTs authenticate managed settings and injected-credential inference.
    // Static client tokens are a separate alternative, and `/v1/models` keeps its
    // existing endpoint-specific authentication behavior.
    if gateway_enabled {
        router = router.merge(gateway::gateway_router());
    }

    // Opt-in spend-limit admin API: registered only when `[server.spend]` is
    // set, independently of the gateway surface. Validation guarantees
    // `[server.admin]` is present, which is what its handlers authenticate
    // against.
    if spend_enabled {
        router = router.merge(gateway::spend::spend_router());
    }

    // Opt-in inbound Responses (Codex) endpoint: registered only when
    // `[server.codex_endpoint]` is set, so the default HTTP surface is unchanged.
    // The Codex CLI appends `/responses` and its analytics path to the configured
    // base URL. Responses use the passthrough handler; analytics is accepted and
    // discarded locally after recording sanitized counters. Both are gated by
    // `[server.auth]` like the other injected-credential routes.
    if codex_endpoint_enabled {
        for path in discovery::CODEX_PATHS {
            router = router.route(path, get(discovery::get_codex));
        }
        // Register from the same constants `concurrency::is_codex_path`
        // classifies against, so a route cannot be added here without also
        // getting the OpenAI-shaped gateway errors its clients expect.
        for path in codex_endpoint::PATHS {
            router = router.route(
                path,
                get(codex_endpoint::websocket::get).post(codex_endpoint::post),
            );
        }
        router = router.route(codex_endpoint::COMPACT_PATH, post(codex_endpoint::compact));
        for path in codex_analytics::PATHS {
            router = router.route(path, post(codex_analytics::post));
        }
    }

    // Opt-in client-facing usage endpoint (`GET /usage`): registered only when
    // `[server.usage]` is set, so the default HTTP surface is unchanged. The
    // handler authenticates every request against `[server.auth]` client tokens
    // (validation guarantees that table is present) and returns a sanitized,
    // aggregated pool-quota view — never per-account identity or capacity.
    if usage_enabled {
        router = router.route("/usage", get(usage::get));
    }

    // Opt-in Claude Code CLI native usage-bar synthesizer (`GET
    // /api/oauth/usage`, M-A): registered only when `[server.oauth_usage]` is
    // set, so the default HTTP surface is unchanged. Unlike `/usage`, its auth
    // is bind-topology-gated rather than client-token-gated — see
    // `oauth_usage::get` and `docs/m14-oauth-usage-endpoint.md`.
    if oauth_usage_enabled {
        router = router.route("/api/oauth/usage", get(oauth_usage::get));
    }

    // Shed above the boot-time limit instead of queueing, and keep each permit
    // attached to the response body so a live SSE stream remains in flight. A
    // zero value preserves the previous unlimited behavior without a layer.
    if max_concurrent_requests > 0 {
        router = router.layer(middleware::from_fn_with_state(
            ConcurrencyLimit::new(max_concurrent_requests, codex_endpoint_enabled),
            limit_requests,
        ));
    }
    router = router.merge(liveness_router);
    // Access control must wrap liveness too: allow rules exempt `/` and
    // `/health`, but deny rules still apply to them. Header and URL limits share
    // this boot-fixed layer. The body limit is read in each handler so it can
    // hot-apply on reload.
    if http_tuning_enabled {
        router = router.layer(middleware::from_fn_with_state(
            http_tuning,
            enforce_http_tuning,
        ));
    }

    // Clone the state into the router; the returned clone shares the same
    // `AccountPool`/`SharedState` Arcs, so a background poller populating quota
    // writes to the very pool the handlers read.
    Ok((router.with_state(state.clone()), shared, state))
}

/// Human-facing landing page; axum also serves HEAD `/` from this handler,
/// which keeps the pre-existing liveness probe working.
async fn root_index() -> String {
    format!(
        "shunt v{} — Anthropic Messages proxy. Endpoints: /v1/models, /routes, /v1/messages, /v1/messages/count_tokens, /protocol, /health\n",
        env!("CARGO_PKG_VERSION")
    )
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

/// Machine-facing liveness endpoint: the process is up and config loaded
/// (the router cannot exist otherwise). Deliberately does not check upstream
/// connectivity — that is decided per request and would only cause flapping.
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[cfg(test)]
mod tests {
    use std::{
        convert::Infallible,
        net::{Ipv4Addr, SocketAddr},
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc, Mutex,
        },
        time::Duration,
    };

    use axum::{
        body::{to_bytes, Body, HttpBody},
        extract::{OriginalUri, State},
        http::{HeaderMap, Request, StatusCode},
        response::{IntoResponse, Response},
        Router,
    };
    use futures_util::{stream, StreamExt};
    use reqwest::dns::{Addrs, Name, Resolve, Resolving};
    use serde_json::{json, Value};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        sync::mpsc,
        task::JoinHandle,
    };
    use tower::ServiceExt;

    use crate::{
        auth::{Credential, CredentialFuture, CredentialResolver},
        config::{
            AccountConfig, AuthMode, Config, InboundAuthConfig, OauthUsageConfig, RetryConfig,
            RouteConfig, UsageEndpointConfig,
        },
        routing::Route,
    };

    use super::{build_router, build_router_with_dependencies};

    const SYNTHETIC_BEARER: &str = "synthetic-test-bearer-marker";
    const SYNTHETIC_PROJECT: &str = "synthetic-test-project-marker";

    struct SyntheticGoogleOauthResolver {
        calls: Arc<AtomicUsize>,
    }

    impl CredentialResolver for SyntheticGoogleOauthResolver {
        fn resolve<'a>(
            &'a self,
            _config: &'a Config,
            _route: &'a Route,
            _client: &'a reqwest::Client,
        ) -> CredentialFuture<'a> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Box::pin(async {
                Ok(Credential::GoogleOauth {
                    access_token: SYNTHETIC_BEARER.to_string(),
                    project_id: SYNTHETIC_PROJECT.to_string(),
                })
            })
        }
    }

    struct RetryLoopbackResolver {
        calls: Arc<AtomicUsize>,
        success: SocketAddr,
    }

    impl Resolve for RetryLoopbackResolver {
        fn resolve(&self, name: Name) -> Resolving {
            assert_eq!(name.as_str(), "localhost");
            let address = if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
                SocketAddr::from((Ipv4Addr::LOCALHOST, 1))
            } else {
                self.success
            };
            let addresses: Addrs = Box::new(std::iter::once(address));
            Box::pin(async move { Ok(addresses) })
        }
    }

    #[derive(Debug)]
    struct CapturedGoogleRequest {
        path_and_query: String,
        authorization: Option<String>,
        api_key: Option<String>,
        body: Value,
    }

    struct GoogleUpstreamState {
        requests: Mutex<Vec<CapturedGoogleRequest>>,
        hold_stream: bool,
        dropped: Arc<tokio::sync::Notify>,
        catalog_model: Option<String>,
    }

    async fn google_upstream(
        State(state): State<Arc<GoogleUpstreamState>>,
        OriginalUri(uri): OriginalUri,
        headers: HeaderMap,
        body: Body,
    ) -> Response {
        let bytes = to_bytes(body, usize::MAX).await.unwrap();
        state.requests.lock().unwrap().push(CapturedGoogleRequest {
            path_and_query: uri
                .path_and_query()
                .map(ToString::to_string)
                .unwrap_or_default(),
            authorization: headers
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .map(str::to_string),
            api_key: headers
                .get("x-goog-api-key")
                .and_then(|value| value.to_str().ok())
                .map(str::to_string),
            body: serde_json::from_slice(&bytes).unwrap(),
        });

        if uri.path().contains("fetchAvailableModels") {
            let model = state.catalog_model.as_deref().unwrap_or("gemini-2.5-pro");
            return axum::Json(json!({"models": {model: {"model": "fixture"}}})).into_response();
        }

        if uri.path().contains("streamGenerateContent") {
            if state.hold_stream {
                let (sender, receiver) = mpsc::channel::<Result<bytes::Bytes, Infallible>>(1);
                sender
                    .send(Ok(bytes::Bytes::from_static(
                        b"data: {\"response\":{\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"early\"}]}}]}}\n\n",
                    )))
                    .await
                    .unwrap();
                let dropped = state.dropped.clone();
                tokio::spawn(async move {
                    sender.closed().await;
                    dropped.notify_one();
                });
                return Body::from_stream(stream::unfold(receiver, |mut receiver| async move {
                    receiver.recv().await.map(|item| (item, receiver))
                }))
                .into_response();
            }
            return (
                [("content-type", "text/event-stream")],
                concat!(
                    "data: {\"response\":{\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"stream-ok\"}]},\"finishReason\":\"STOP\"}]}}\n\n",
                    "data: [DONE]\n\n"
                ),
            )
                .into_response();
        }

        axum::Json(json!({
            "response": {
                "candidates": [{
                    "content": {"parts": [{"text": "unary-ok"}]},
                    "finishReason": "STOP"
                }]
            }
        }))
        .into_response()
    }

    struct TestUpstream {
        base_url: String,
        state: Arc<GoogleUpstreamState>,
        task: JoinHandle<()>,
    }

    impl Drop for TestUpstream {
        fn drop(&mut self) {
            self.task.abort();
        }
    }

    async fn start_google_upstream(hold_stream: bool) -> TestUpstream {
        let state = Arc::new(GoogleUpstreamState {
            requests: Mutex::new(Vec::new()),
            hold_stream,
            dropped: Arc::new(tokio::sync::Notify::new()),
            catalog_model: None,
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = Router::new()
            .fallback(google_upstream)
            .with_state(state.clone());
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        TestUpstream {
            base_url: format!("http://{address}"),
            state,
            task,
        }
    }

    async fn start_native_upstream(hold_stream: bool, model: &str) -> TestUpstream {
        let state = Arc::new(GoogleUpstreamState {
            requests: Mutex::new(Vec::new()),
            hold_stream,
            dropped: Arc::new(tokio::sync::Notify::new()),
            catalog_model: Some(model.to_string()),
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = Router::new()
            .fallback(google_upstream)
            .with_state(state.clone());
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        TestUpstream {
            base_url: format!("http://{address}"),
            state,
            task,
        }
    }

    struct NativeResolver {
        calls: Arc<AtomicUsize>,
        access_token: &'static str,
        project_id: &'static str,
        account_fingerprint: &'static str,
    }

    impl CredentialResolver for NativeResolver {
        fn resolve<'a>(
            &'a self,
            _config: &'a Config,
            _route: &'a Route,
            _client: &'a reqwest::Client,
        ) -> CredentialFuture<'a> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                Ok(Credential::AntigravityOauth {
                    access_token: self.access_token.to_string(),
                    project_id: self.project_id.to_string(),
                    account_fingerprint: self.account_fingerprint.to_string(),
                })
            })
        }
    }

    #[derive(Clone)]
    struct MutableNativeTuple {
        access_token: String,
        project_id: String,
        account_fingerprint: String,
    }

    struct SwappingNativeResolver {
        tuple: Arc<Mutex<MutableNativeTuple>>,
        calls: Arc<AtomicUsize>,
    }

    impl CredentialResolver for SwappingNativeResolver {
        fn resolve<'a>(
            &'a self,
            _: &'a Config,
            _: &'a Route,
            _: &'a reqwest::Client,
        ) -> CredentialFuture<'a> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let tuple = self.tuple.lock().unwrap().clone();
            Box::pin(async move {
                Ok(Credential::AntigravityOauth {
                    access_token: tuple.access_token,
                    project_id: tuple.project_id,
                    account_fingerprint: tuple.account_fingerprint,
                })
            })
        }
    }

    fn antigravity_config(base_url: String, _project: &str) -> Config {
        let mut config = Config::default();
        let provider = config.providers.get_mut("antigravity").unwrap();
        provider.base_url = base_url;
        provider.auth = AuthMode::AntigravityOauth;
        config.server.default_provider = "antigravity".to_string();
        config.routes = vec![RouteConfig {
            model: "gemini-3.8-flash-medium".to_string(),
            provider: "antigravity".to_string(),
            upstream_model: Some("gemini-3.8-flash-medium".to_string()),
            effort: None,
            service_tier: None,
        }];
        config
    }

    fn google_oauth_config(base_url: String, max_retries: u32, capacity: usize) -> Config {
        let mut config = Config::default();
        let provider = config.providers.get_mut("gemini").unwrap();
        provider.base_url = base_url;
        provider.auth = AuthMode::GoogleOauth;
        provider.retry = RetryConfig {
            max_retries,
            initial_backoff_ms: 1,
            max_backoff_ms: 2,
            ..RetryConfig::default()
        };
        config.server.default_provider = "gemini".to_string();
        config.server.max_concurrent_requests = capacity;
        config.routes = vec![RouteConfig {
            model: "claude-via-google-oauth".to_string(),
            provider: "gemini".to_string(),
            upstream_model: Some("gemini-2.5-pro".to_string()),
            effort: None,
            service_tier: None,
        }];
        config
    }

    fn google_request(stream: bool) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/v1/messages")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "model": "claude-via-google-oauth",
                    "max_tokens": 64,
                    "stream": stream,
                    "messages": [{"role": "user", "content": "synthetic fixture"}]
                })
                .to_string(),
            ))
            .unwrap()
    }

    fn oauth_router(config: Config, calls: Arc<AtomicUsize>) -> Router {
        oauth_router_with_client(config, calls, reqwest::Client::new())
    }

    fn oauth_router_with_client(
        config: Config,
        calls: Arc<AtomicUsize>,
        http_client: reqwest::Client,
    ) -> Router {
        build_router_with_dependencies(
            config,
            http_client,
            Arc::new(SyntheticGoogleOauthResolver { calls }),
        )
        .unwrap()
        .0
    }

    fn native_router(
        config: Config,
        calls: Arc<AtomicUsize>,
        token: &'static str,
        project: &'static str,
        account: &'static str,
        http_client: reqwest::Client,
    ) -> Router {
        build_router_with_dependencies(
            config,
            http_client,
            Arc::new(NativeResolver {
                calls,
                access_token: token,
                project_id: project,
                account_fingerprint: account,
            }),
        )
        .unwrap()
        .0
    }

    fn assert_google_request(request: &CapturedGoogleRequest, streaming: bool) {
        let method = if streaming {
            "streamGenerateContent?alt=sse"
        } else {
            "generateContent"
        };
        assert_eq!(request.path_and_query, format!("/v1internal:{method}"));
        assert_eq!(
            request.authorization.as_deref(),
            Some("Bearer synthetic-test-bearer-marker")
        );
        assert_eq!(request.api_key, None);
        assert_eq!(request.body["model"], "gemini-2.5-pro");
        assert_eq!(request.body["project"], SYNTHETIC_PROJECT);
        assert_eq!(
            request.body.pointer("/request/contents/0/parts/0/text"),
            Some(&json!("synthetic fixture"))
        );
    }

    async fn read_raw_google_request(socket: &mut tokio::net::TcpStream) -> CapturedGoogleRequest {
        let mut raw = Vec::new();
        let header_end = loop {
            let mut chunk = [0_u8; 4096];
            let read = socket.read(&mut chunk).await.unwrap();
            assert!(read > 0, "request closed before headers");
            raw.extend_from_slice(&chunk[..read]);
            if let Some(index) = raw.windows(4).position(|window| window == b"\r\n\r\n") {
                break index + 4;
            }
        };
        let headers = String::from_utf8(raw[..header_end].to_vec()).unwrap();
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().unwrap())
            })
            .unwrap();
        while raw.len() < header_end + content_length {
            let mut chunk = [0_u8; 4096];
            let read = socket.read(&mut chunk).await.unwrap();
            assert!(read > 0, "request closed before body");
            raw.extend_from_slice(&chunk[..read]);
        }
        let path_and_query = headers
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap()
            .to_string();
        let header = |wanted: &str| {
            headers.lines().find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case(wanted)
                    .then(|| value.trim().to_string())
            })
        };
        CapturedGoogleRequest {
            path_and_query,
            authorization: header("authorization"),
            api_key: header("x-goog-api-key"),
            body: serde_json::from_slice(&raw[header_end..header_end + content_length]).unwrap(),
        }
    }

    #[tokio::test]
    async fn gemini_google_oauth_code_assist_lifetime_unary_and_streaming() {
        for streaming in [false, true] {
            let upstream = start_google_upstream(false).await;
            let calls = Arc::new(AtomicUsize::new(0));
            let router = oauth_router(
                google_oauth_config(upstream.base_url.clone(), 1, 0),
                calls.clone(),
            );
            let response = router.oneshot(google_request(streaming)).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let downstream = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let downstream = String::from_utf8(downstream.to_vec()).unwrap();
            assert!(!downstream.contains(SYNTHETIC_BEARER));
            assert!(!downstream.contains(SYNTHETIC_PROJECT));
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            let requests = upstream.state.requests.lock().unwrap();
            assert_eq!(requests.len(), 1);
            assert_google_request(&requests[0], streaming);
        }
    }

    #[tokio::test]
    async fn gemini_google_oauth_code_assist_lifetime_connect_retry_reuses_identity_and_body() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let upstream = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let request = read_raw_google_request(&mut socket).await;
            let body = r#"{"response":{"candidates":[{"content":{"parts":[{"text":"retry-ok"}]},"finishReason":"STOP"}]}}"#;
            socket
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
            request
        });

        let calls = Arc::new(AtomicUsize::new(0));
        let dns_calls = Arc::new(AtomicUsize::new(0));
        let client = reqwest::Client::builder()
            .no_proxy()
            .dns_resolver(Arc::new(RetryLoopbackResolver {
                calls: dns_calls.clone(),
                success: address,
            }))
            .build()
            .unwrap();
        let router = oauth_router_with_client(
            google_oauth_config("http://localhost".to_string(), 1, 0),
            calls.clone(),
            client,
        );
        let response = router.oneshot(google_request(false)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let downstream = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(String::from_utf8_lossy(&downstream).contains("retry-ok"));
        let request = tokio::time::timeout(std::time::Duration::from_secs(1), upstream)
            .await
            .expect("retry reaches second connection")
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(dns_calls.load(Ordering::SeqCst), 2);
        assert_google_request(&request, false);
    }

    #[tokio::test]
    async fn gemini_google_oauth_code_assist_lifetime_post_send_close_does_not_retry() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let upstream = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let request = read_raw_google_request(&mut socket).await;
            drop(socket);
            let retried =
                tokio::time::timeout(std::time::Duration::from_millis(100), listener.accept())
                    .await
                    .is_ok();
            (request, retried)
        });

        let calls = Arc::new(AtomicUsize::new(0));
        let router = oauth_router(
            google_oauth_config(format!("http://{address}"), 1, 0),
            calls.clone(),
        );
        let response = router.oneshot(google_request(false)).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        let (request, retried) = tokio::time::timeout(std::time::Duration::from_secs(1), upstream)
            .await
            .expect("upstream observes the no-retry window")
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(!retried, "a post-send close must not redispatch the POST");
        assert_google_request(&request, false);
    }

    #[tokio::test]
    async fn gemini_google_oauth_code_assist_lifetime_cancellation_releases_capacity() {
        let upstream = start_google_upstream(true).await;
        let calls = Arc::new(AtomicUsize::new(0));
        let router = oauth_router(
            google_oauth_config(upstream.base_url.clone(), 2, 1),
            calls.clone(),
        );
        let first = router.clone().oneshot(google_request(true)).await.unwrap();
        assert_eq!(first.status(), StatusCode::OK);
        let mut first_body = first.into_body().into_data_stream();
        let chunk = tokio::time::timeout(std::time::Duration::from_secs(1), first_body.next())
            .await
            .expect("first translated chunk")
            .unwrap()
            .unwrap();
        assert!(String::from_utf8_lossy(&chunk).contains("early"));

        let saturated = router.clone().oneshot(google_request(true)).await.unwrap();
        assert_eq!(saturated.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        drop(first_body);
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            upstream.state.dropped.notified(),
        )
        .await
        .expect("dropping downstream body cancels upstream body");

        let reacquired = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            router.oneshot(google_request(true)),
        )
        .await
        .expect("capacity reacquired")
        .unwrap();
        assert_eq!(reacquired.status(), StatusCode::OK);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let requests = upstream.state.requests.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert_google_request(&requests[0], true);
        assert_google_request(&requests[1], true);
    }

    fn native_request(streaming: bool) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/v1/messages")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "model": "gemini-3.8-flash-medium", "max_tokens": 16,
                    "stream": streaming,
                    "messages": [{"role": "user", "content": "native fixture"}]
                })
                .to_string(),
            ))
            .unwrap()
    }

    #[tokio::test]
    async fn antigravity_native_tool_signature_real_router_roundtrip_and_rejection() {
        use wiremock::{
            matchers::{method, path},
            Mock, MockServer, ResponseTemplate,
        };

        for streaming in [false, true] {
            let project = if streaming {
                "scope-fixture-stream"
            } else {
                "scope-fixture-unary"
            };
            let upstream = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/v1internal:fetchAvailableModels"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "models":{"gemini-3.8-flash-medium":{}}
                })))
                .expect(1)
                .mount(&upstream)
                .await;
            let transcript = format!(
                "data: {}\n\ndata: [DONE]\n\n",
                json!({"response":{
                    "candidates":[{"content":{"parts":[{
                        "functionCall":{"name":"lookup","args":{}},
                        "thoughtSignature":"upstream-original-signature"
                    }]},"finishReason":"STOP"}]
                }})
            );
            Mock::given(method("POST"))
                .and(path("/v1internal:streamGenerateContent"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .insert_header("content-type", "text/event-stream")
                        .set_body_string(transcript),
                )
                .expect(2)
                .mount(&upstream)
                .await;
            let calls = Arc::new(AtomicUsize::new(0));
            let router = native_router(
                antigravity_config(upstream.uri(), project),
                calls,
                "token-a",
                project,
                "account-a",
                reqwest::Client::new(),
            );
            let make_request = |body: &Value| {
                Request::builder()
                    .method("POST")
                    .uri("/v1/messages")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap()
            };
            let mut request = json!({"model":"gemini-3.8-flash-medium", "max_tokens":64,
                "stream":streaming, "messages":[{"role":"user","content":"opening"}]});
            let response = router
                .clone()
                .oneshot(make_request(&request))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let tool = if streaming {
                String::from_utf8(bytes.to_vec())
                    .unwrap()
                    .lines()
                    .filter_map(|line| line.strip_prefix("data: "))
                    .filter_map(|data| serde_json::from_str::<Value>(data).ok())
                    .find_map(|event| {
                        (event["content_block"]["type"] == "tool_use")
                            .then(|| event["content_block"].clone())
                    })
                    .unwrap()
            } else {
                serde_json::from_slice::<Value>(&bytes).unwrap()["content"][0].clone()
            };
            assert!(tool["id"]
                .as_str()
                .unwrap()
                .starts_with("call_antigravity_v2_"));
            request["messages"].as_array_mut().unwrap().extend([
                json!({"role":"assistant","content":[tool.clone()]}),
                json!({"role":"user","content":[{"type":"tool_result","tool_use_id":tool["id"],"content":"done"}]})
            ]);
            let response = router
                .clone()
                .oneshot(make_request(&request))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let _ = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let captured = upstream.received_requests().await.unwrap();
            let inference: Vec<Value> = captured
                .iter()
                .filter(|request| request.url.path() == "/v1internal:streamGenerateContent")
                .map(|request| request.body_json().unwrap())
                .collect();
            assert_eq!(inference.len(), 2);
            assert_eq!(
                inference[0]["request"]["sessionId"],
                inference[1]["request"]["sessionId"]
            );
            assert_eq!(
                inference[1]["request"]["contents"][1]["parts"][0]["thoughtSignature"],
                "upstream-original-signature"
            );
            assert_eq!(
                inference[1]["request"]["contents"][2]["parts"][0]["functionResponse"]["name"],
                "lookup"
            );
            for mutation in ["account", "opening", "arguments", "orphan", "duplicate"] {
                let mut invalid = request.clone();
                let target = if mutation == "account" {
                    native_router(
                        antigravity_config(upstream.uri(), project),
                        Arc::new(AtomicUsize::new(0)),
                        "token-b",
                        project,
                        "account-b",
                        reqwest::Client::new(),
                    )
                } else {
                    router.clone()
                };
                match mutation {
                    "opening" => invalid["messages"][0]["content"] = json!("another opening"),
                    "arguments" => {
                        invalid["messages"][1]["content"][0]["input"] = json!({"changed":true})
                    }
                    "orphan" => {
                        invalid["messages"][2]["content"][0]["tool_use_id"] = json!("unknown")
                    }
                    "duplicate" => {
                        let result = invalid["messages"][2]["content"][0].clone();
                        invalid["messages"][2]["content"]
                            .as_array_mut()
                            .unwrap()
                            .push(result);
                    }
                    _ => {}
                }
                let response = target.oneshot(make_request(&invalid)).await.unwrap();
                assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{mutation}");
                let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
                let error = String::from_utf8_lossy(&bytes);
                assert!(!error.contains(tool["id"].as_str().unwrap()));
                assert_eq!(
                    upstream.received_requests().await.unwrap().len(),
                    captured.len(),
                    "{mutation} must fail before any upstream request"
                );
            }
            upstream.verify().await;
        }
    }

    #[tokio::test]
    async fn antigravity_native_affinity_injected_resolver_once_and_tuple_distinct() {
        let upstream = start_native_upstream(false, "gemini-3.8-flash-medium").await;
        let calls = Arc::new(AtomicUsize::new(0));
        let router = native_router(
            antigravity_config(upstream.base_url.clone(), "project-a"),
            calls.clone(),
            "token-a",
            "project-a",
            "account-a",
            reqwest::Client::new(),
        );
        for _ in 0..2 {
            let response = router.clone().oneshot(native_request(false)).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let _ = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        }
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "one resolution per request"
        );
        {
            let requests = upstream.state.requests.lock().unwrap();
            let inference: Vec<_> = requests
                .iter()
                .filter(|r| r.path_and_query.contains("streamGenerateContent"))
                .collect();
            assert_eq!(inference.len(), 2);
            for request in inference {
                assert_eq!(request.authorization.as_deref(), Some("Bearer token-a"));
                assert_eq!(request.body["project"], "project-a");
                assert_eq!(request.body["model"], "gemini-3.8-flash-medium");
            }
        }

        // One account may legitimately have multiple projects; project remains
        // part of the immutable tuple and must not collapse to the first one.
        let upstream_project_b = start_native_upstream(false, "gemini-3.8-flash-medium").await;
        let calls_project_b = Arc::new(AtomicUsize::new(0));
        let router_project_b = native_router(
            antigravity_config(upstream_project_b.base_url.clone(), "project-b"),
            calls_project_b.clone(),
            "token-a",
            "project-b",
            "account-a",
            reqwest::Client::new(),
        );
        let response = router_project_b
            .oneshot(native_request(false))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let _ = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(calls_project_b.load(Ordering::SeqCst), 1);
        {
            let project_b_requests = upstream_project_b.state.requests.lock().unwrap();
            let request = project_b_requests
                .iter()
                .find(|r| r.path_and_query.contains("streamGenerateContent"))
                .unwrap();
            assert_eq!(request.authorization.as_deref(), Some("Bearer token-a"));
            assert_eq!(request.body["project"], "project-b");
        }

        // Same project with a different account is a separate request-local tuple.
        let upstream_b = start_native_upstream(false, "gemini-3.8-flash-medium").await;
        let calls_b = Arc::new(AtomicUsize::new(0));
        let router_b = native_router(
            antigravity_config(upstream_b.base_url.clone(), "project-a"),
            calls_b.clone(),
            "token-b",
            "project-a",
            "account-b",
            reqwest::Client::new(),
        );
        let response = router_b.oneshot(native_request(false)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let _ = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(calls_b.load(Ordering::SeqCst), 1);
        let requests_b = upstream_b.state.requests.lock().unwrap();
        let request = requests_b
            .iter()
            .find(|r| r.path_and_query.contains("streamGenerateContent"))
            .unwrap();
        assert_eq!(request.authorization.as_deref(), Some("Bearer token-b"));
        assert_eq!(request.body["project"], "project-a");
    }

    #[tokio::test]
    async fn antigravity_native_affinity_cancellation_drops_request_body_promptly() {
        let upstream = start_native_upstream(true, "gemini-3.8-flash-medium").await;
        let calls = Arc::new(AtomicUsize::new(0));
        let router = native_router(
            antigravity_config(upstream.base_url.clone(), "project-held"),
            calls.clone(),
            "token-held",
            "project-held",
            "account-held",
            reqwest::Client::new(),
        );
        let response = router.oneshot(native_request(true)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let mut body = response.into_body().into_data_stream();
        let _ = tokio::time::timeout(Duration::from_secs(1), body.next())
            .await
            .unwrap();
        drop(body);
        tokio::time::timeout(Duration::from_secs(1), upstream.state.dropped.notified())
            .await
            .expect("cancellation releases request-local upstream ownership");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn antigravity_native_affinity_inflight_swap_keeps_a_tuple_immutable() {
        let upstream = start_native_upstream(true, "gemini-3.8-flash-medium").await;
        let calls = Arc::new(AtomicUsize::new(0));
        let tuple = Arc::new(Mutex::new(MutableNativeTuple {
            access_token: "token-a".into(),
            project_id: "project-a".into(),
            account_fingerprint: "account-a".into(),
        }));
        let resolver = Arc::new(SwappingNativeResolver {
            tuple: tuple.clone(),
            calls: calls.clone(),
        });
        let mut config = antigravity_config(upstream.base_url.clone(), "project-a");
        config.server.max_concurrent_requests = 2;
        let router = build_router_with_dependencies(config, reqwest::Client::new(), resolver)
            .unwrap()
            .0;
        let first = router.clone().oneshot(native_request(true)).await.unwrap();
        assert_eq!(first.status(), StatusCode::OK);
        let mut first_body = first.into_body().into_data_stream();
        let _ = tokio::time::timeout(Duration::from_secs(1), first_body.next())
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if upstream
                    .state
                    .requests
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|r| r.path_and_query.contains("streamGenerateContent"))
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("request A reaches inference");
        *tuple.lock().unwrap() = MutableNativeTuple {
            access_token: "token-b".into(),
            project_id: "project-b".into(),
            account_fingerprint: "account-b".into(),
        };
        let second = router.oneshot(native_request(true)).await.unwrap();
        assert_eq!(second.status(), StatusCode::OK);
        drop(second);
        drop(first_body);
        tokio::time::timeout(Duration::from_secs(1), upstream.state.dropped.notified())
            .await
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let requests = upstream.state.requests.lock().unwrap();
        let inference: Vec<_> = requests
            .iter()
            .filter(|r| r.path_and_query.contains("streamGenerateContent"))
            .collect();
        assert_eq!(inference.len(), 2);
        assert_eq!(
            inference[0].authorization.as_deref(),
            Some("Bearer token-a")
        );
        assert_eq!(inference[0].body["project"], "project-a");
        assert_eq!(
            inference[1].authorization.as_deref(),
            Some("Bearer token-b")
        );
        assert_eq!(inference[1].body["project"], "project-b");
    }

    #[tokio::test]
    async fn antigravity_native_affinity_production_default_resolver_once() {
        let previous = std::env::var_os("SHUNT_ANTIGRAVITY_AUTH_FILE");
        let missing =
            std::env::temp_dir().join(format!("shunt-antigravity-missing-{}", std::process::id()));
        std::env::set_var("SHUNT_ANTIGRAVITY_AUTH_FILE", &missing);
        let config = antigravity_config("http://127.0.0.1:1".to_string(), "default-project");
        let (router, _, state) = build_router(config).unwrap();
        let response = router.oneshot(native_request(false)).await.unwrap();
        assert!(!response.status().is_success());
        assert_eq!(
            state.default_resolver_calls.unwrap().load(Ordering::SeqCst),
            1
        );
        match previous {
            Some(value) => std::env::set_var("SHUNT_ANTIGRAVITY_AUTH_FILE", value),
            None => std::env::remove_var("SHUNT_ANTIGRAVITY_AUTH_FILE"),
        }
    }

    /// `Config::default()` with `[server.auth]` bound to a unique env var and
    /// `[server.usage]` enabled, plus the built-in `codex` provider given one
    /// explicit account so the request does not touch the account store
    /// (mirrors `usage::tests::state_with_auth_and_seeded_pool`).
    fn config_with_usage_enabled(label: &str) -> (Config, String) {
        // Per-test-unique name: tests share the process env, and one test's
        // `remove_var` must not race another's construction-time resolve.
        let env = format!("SHUNT_SERVER_TEST_TOKENS_{}_{label}", std::process::id());
        std::env::set_var(&env, "tester:tok-secret");
        let mut config = Config::default();
        config.server.auth = Some(InboundAuthConfig {
            header: "x-shunt-token".to_string(),
            tokens_env: env.clone(),
        });
        config.server.usage = Some(UsageEndpointConfig::default());
        config
            .providers
            .get_mut("codex")
            .expect("built-in codex provider")
            .accounts = vec![AccountConfig {
            name: "acct-a".to_string(),
            ..AccountConfig::default()
        }];
        (config, env)
    }

    /// The needs-re-login mark is process-lifetime pool health, not per-request
    /// config: a config reload re-snapshots `AppState` and must carry it
    /// forward. PR #389 is the precedent — `state.refreshed()` silently
    /// discarded a hand-set `admin_auth` field, so a field that must outlive a
    /// reload has to be pinned rather than assumed.
    #[tokio::test]
    async fn needs_relogin_mark_survives_a_config_re_snapshot() {
        let mut config = Config::default();
        let account = AccountConfig {
            name: "dead-account".to_string(),
            ..AccountConfig::default()
        };
        let provider = config
            .providers
            .get_mut("anthropic")
            .expect("built-in anthropic provider");
        // A pool account is only valid on an OAuth provider; the built-in
        // `anthropic` entry defaults to a plain API-key upstream.
        provider.auth = crate::config::AuthMode::ClaudeOauth;
        provider.accounts = vec![account.clone()];
        let (_router, _shared, state) =
            build_router(config).expect("router builds from the default config");

        state.accounts.mark_needs_relogin(
            "anthropic",
            &account,
            crate::accounts::ReloginCause::RefreshGrant,
        );
        assert!(state.accounts.needs_relogin("anthropic", &account));

        let reloaded = state.refreshed();
        assert!(
            reloaded.accounts.needs_relogin("anthropic", &account),
            "a config re-snapshot must not discard the needs-re-login mark"
        );
        // And the snapshot the admin dashboard reads still carries it.
        let snapshots =
            reloaded
                .accounts
                .snapshot("anthropic", std::slice::from_ref(&account), None, None);
        assert!(snapshots[0].needs_relogin);
    }

    #[tokio::test]
    async fn usage_route_is_registered_and_answers_when_enabled_with_valid_auth() {
        let (config, env) = config_with_usage_enabled("registered");
        let (router, _shared, _state) = build_router(config).unwrap();

        let request = Request::builder()
            .uri("/usage")
            .header("x-api-key", "tok-secret")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        std::env::remove_var(&env);

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn usage_route_is_404_when_server_usage_is_not_configured() {
        let (router, _shared, _state) = build_router(Config::default()).unwrap();

        let request = Request::builder()
            .uri("/usage")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn zero_max_concurrent_requests_keeps_requests_unlimited() {
        let mut config = Config::default();
        config.server.max_concurrent_requests = 0;
        let (router, _shared, _state) = build_router(config).unwrap();

        let first = router
            .clone()
            .oneshot(Request::get("/protocol").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);
        assert!(
            !first.body().is_end_stream(),
            "/protocol must keep its response body alive for the overlap assertion"
        );
        let mut responses = vec![first];
        // Exceed the documented default so this cannot pass if `0` is silently
        // replaced with that finite fallback instead of omitting the layer.
        for _ in 0..=1024 {
            let response = router
                .clone()
                .oneshot(Request::get("/protocol").body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            assert!(!response.body().is_end_stream());
            responses.push(response);
        }
        drop(responses);
    }

    #[tokio::test]
    async fn health_bypasses_saturated_concurrency_limit() {
        let mut config = Config::default();
        config.server.max_concurrent_requests = 1;
        let (router, _shared, _state) = build_router(config).unwrap();

        let first = Request::builder()
            .uri("/protocol")
            .body(Body::empty())
            .unwrap();
        let first = router.clone().oneshot(first).await.unwrap();
        assert_eq!(first.status(), StatusCode::OK);
        assert!(
            !first.body().is_end_stream(),
            "/protocol must keep its response body alive to saturate the limiter"
        );

        let limited = Request::builder()
            .uri("/protocol")
            .body(Body::empty())
            .unwrap();
        let response = router.clone().oneshot(limited).await.unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let health = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let response = router.clone().oneshot(health).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let root = Request::builder().uri("/").body(Body::empty()).unwrap();
        let response = router.oneshot(root).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        // `first` still owns the only permit; keeping it alive to here is what
        // makes the assertions above run against a saturated limiter.
        drop(first);
    }

    #[tokio::test]
    async fn oauth_usage_route_is_registered_and_answers_when_enabled() {
        // Loopback default bind: the route is unauthenticated, so no
        // `[server.auth]`/credential is needed for a 200.
        let mut config = Config::default();
        config.server.oauth_usage = Some(OauthUsageConfig::default());
        let (router, _shared, _state) = build_router(config).unwrap();

        let request = Request::builder()
            .uri("/api/oauth/usage")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn oauth_usage_route_is_404_when_server_oauth_usage_is_not_configured() {
        let (router, _shared, _state) = build_router(Config::default()).unwrap();

        let request = Request::builder()
            .uri("/api/oauth/usage")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
