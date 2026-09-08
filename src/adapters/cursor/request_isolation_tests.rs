// ------------------------------------------------------------------
// 12-02: Run destination pin (CUR-01/D-01) and request-local fact
// isolation (CUR-02/D-02). Adapter-dispatch level: these fixtures drive
// the CursorAgentClient directly and do NOT cover the whole gateway
// router (whole-router parity remains the 12-01 tracer job). The
// loopback is a TLS+h2 server rather than wiremock because the proven
// destination constant is an https h2-only URL that the wiremock
// harness used for error mapping cannot serve.
// ------------------------------------------------------------------

/// The proven Run destination (12-CONTEXT D-01). Tests assert the shipped
/// constants byte-for-byte against this local copy, so replacing
/// AGENT_BASE_URL or the endpoint suffix fails these tests until fresh
/// captured/live evidence updates this file.
const D01_PROVEN_URL: &str = "https://agentn.global.api5.cursor.sh";
const D01_PROVEN_PATH: &str = "/agent.v1.AgentService/Run";

/// Three distinct model ids with no byte-overlap, so the token embedded
/// in a captured request body identifies its originating request
/// unambiguously (D-02).
const ISOLATION_MODELS: [&str; 3] = ["composer-2.5", "gpt-5.6", "claude-sonnet-4-5"];

#[derive(Clone, Copy)]
struct RunSpec {
    model: &'static str,
    fast: bool,
    mode: u64,
    tool: Option<&'static str>,
}

fn run_spec(spec: RunSpec) -> agent::AgentRunParams {
    let RunSpec {
        model,
        fast,
        mode,
        tool,
    } = spec;
    agent::AgentRunParams {
        prompt: "fixture".to_string(),
        model_id: model.to_string(),
        fast,
        cwd: "/".to_string(),
        mode,
        images: Vec::new(),
        tools: tool
            .map(|name| {
                vec![agent::AgentTool {
                    name: name.to_string(),
                    description: "fixture tool".to_string(),
                    input_schema: serde_json::json!({"type": "object"}),
                }]
            })
            .unwrap_or_default(),
    }
}

/// One Run POST observed on the loopback transport.
struct RunObservation {
    method: String,
    path: String,
    request_id: String,
    body: Vec<u8>,
}

// Schema-derived request projection; unknown fields remain forward-compatible.
#[derive(prost::Message)]
struct ObservedRunInput {
    #[prost(message, optional, tag = "1")]
    request: Option<ObservedRun>,
}
#[derive(prost::Message)]
struct ObservedRun {
    #[prost(message, optional, tag = "2")]
    action: Option<ObservedAction>,
    #[prost(message, optional, tag = "9")]
    model: Option<ObservedModel>,
}
#[derive(prost::Message)]
struct ObservedAction {
    #[prost(message, optional, tag = "1")]
    user_action: Option<ObservedUserAction>,
}
#[derive(prost::Message)]
struct ObservedUserAction {
    #[prost(message, optional, tag = "1")]
    user: Option<ObservedUser>,
}
#[derive(prost::Message)]
struct ObservedUser {
    #[prost(uint64, tag = "4")]
    mode: u64,
}
#[derive(prost::Message)]
struct ObservedModel {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(message, repeated, tag = "3")]
    parameters: Vec<ObservedParameter>,
}
#[derive(prost::Message)]
struct ObservedParameter {
    #[prost(string, tag = "1")]
    key: String,
    #[prost(string, tag = "2")]
    value: String,
}

fn haystack_contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

/// Identify which request model token is embedded in the captured body.
fn model_token_in_body(body: &[u8]) -> Option<&'static str> {
    ISOLATION_MODELS
        .iter()
        .find(|model| haystack_contains(body, model.as_bytes()))
        .copied()
}

/// Aborts the loopback server task when the test ends.
struct LoopbackAbort {
    handle: tokio::task::JoinHandle<()>,
}
impl Drop for LoopbackAbort {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Starts the adapter-level loopback: TLS+h2 server for the Run stream,
/// plus an h2 reqwest client configured so the proven host resolves to
/// that server (TLS identity and the Run URL stay real). No live
/// credentials: the loopback does not validate the bearer.
async fn start_run_loopback() -> (
    CursorAgentClient,
    tokio::sync::mpsc::UnboundedReceiver<RunObservation>,
    LoopbackAbort,
) {
    let host = "agentn.global.api5.cursor.sh";
    let cert = rcgen::generate_simple_self_signed(vec![host.to_string()]).unwrap();
    let key = rustls::pki_types::PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());
    let mut tls = rustls::ServerConfig::builder_with_provider(std::sync::Arc::new(
        rustls::crypto::aws_lc_rs::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(vec![cert.cert.der().clone()], key.into())
    .unwrap();
    tls.alpn_protocols = vec![b"h2".to_vec()];
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let client = reqwest::Client::builder()
        .no_proxy()
        .add_root_certificate(reqwest::Certificate::from_der(cert.cert.der()).unwrap())
        .resolve(host, addr)
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .unwrap();
    let (observation_tx, observation_rx) = tokio::sync::mpsc::unbounded_channel();
    let server = tokio::spawn(async move {
        let acceptor = tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(tls));
        let mut connections = tokio::task::JoinSet::new();
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let acceptor = acceptor.clone();
            let observation_tx = observation_tx.clone();
            connections.spawn(async move {
                let socket = acceptor.accept(socket).await.unwrap();
                let mut connection = h2::server::handshake(socket).await.unwrap();
                let mut requests = tokio::task::JoinSet::new();
                while let Some(request) = connection.accept().await {
                    let Ok((request, mut respond)) = request else {
                        break;
                    };
                    let observation_tx = observation_tx.clone();
                    requests.spawn(async move {
                        // Dispatch-level assertion of the pinned destination: the
                        // exact method+path of every Run POST (CUR-01/D-01).
                        assert_eq!(request.method(), axum::http::Method::POST);
                        let path = request.uri().path().to_string();
                        assert_eq!(path, D01_PROVEN_PATH);
                        let headers = request.headers().clone();
                        assert_eq!(request.uri().host(), Some("agentn.global.api5.cursor.sh"));
                        for (name, expected) in [
                            ("authorization", "Bearer synthetic-loopback-token"),
                            ("connect-protocol-version", "1"),
                            ("connect-accept-encoding", "gzip,br"),
                            ("x-cursor-client-type", "cli"),
                            ("x-ghost-mode", "true"),
                            ("user-agent", "connect-es/1.6.1"),
                        ] {
                            assert_eq!(headers.get(name).unwrap(), expected);
                        }
                        assert!(!headers.get("x-cursor-client-version").unwrap().is_empty());
                        assert_eq!(
                            headers.get("content-type").and_then(|v| v.to_str().ok()),
                            Some("application/connect+proto")
                        );
                        let request_id = headers
                            .get("x-request-id")
                            .and_then(|value| value.to_str().ok())
                            .expect("x-request-id header")
                            .to_string();
                        assert_eq!(
                            headers.get("x-original-request-id"),
                            headers.get("x-request-id"),
                            "request-id headers must pair per request",
                        );

                        // Collect the paced request stream. The model token rides
                        // in frame 0, so respond as soon as it is observed; then
                        // keep draining until the client ends the request.
                        let mut body_stream = request.into_body();
                        let mut body = Vec::new();
                        let mut observed_model: Option<&'static str> = None;
                        while let Some(data) = body_stream.data().await {
                            let data = match data {
                                Ok(data) => data,
                                Err(_) => break,
                            };
                            body_stream
                                .flow_control()
                                .release_capacity(data.len())
                                .unwrap();
                            body.extend_from_slice(&data);
                            observed_model =
                                observed_model.or_else(|| model_token_in_body(&body));
                            if let Some(model) = observed_model {
                                let echo = text_turn_frames(&format!("echo:{model}"));
                                let response = axum::http::Response::builder()
                                    .status(200)
                                    .header("content-type", "application/connect+proto")
                                    .body(())
                                    .unwrap();
                                let mut send = respond.send_response(response, false).unwrap();
                                send.send_data(Bytes::from(echo), true).unwrap();
                                break;
                            }
                        }
                        if observed_model.is_none() {
                            panic!("loopback never observed a request model token");
                        }
                        // Finish reading the request body (heartbeats and pacing
                        // markers) until the client half-closes.
                        while let Some(data) = body_stream.data().await {
                            let data = match data {
                                Ok(data) => data,
                                Err(_) => break,
                            };
                            body_stream
                                .flow_control()
                                .release_capacity(data.len())
                                .unwrap();
                            body.extend_from_slice(&data);
                        }
                        let _ = observation_tx.send(RunObservation {
                            method: "POST".to_string(),
                            path,
                            request_id,
                            body,
                        });
                    });
                }
            });
        }
    });
    (
        CursorAgentClient::new(client),
        observation_rx,
        LoopbackAbort { handle: server },
    )
}

/// Dispatches one turn through the adapter client and collects its decoded
/// events. The turn guard is owned by the response reader; no background
/// task outlives this future.
async fn dispatch_one(
    client: &CursorAgentClient,
    spec: RunSpec,
) -> Vec<super::response::CursorStreamEvent> {
    let frames = build_run_frames_async(run_spec(spec))
        .await
        .expect("agent frames should build");
    let turn = client
        .open_turn("synthetic-loopback-token", frames)
        .await
        .expect("adapter dispatch should reach the loopback");
    assert!(
        turn.status().is_success(),
        "Run dispatch status should be success",
    );
    let mut events = Vec::new();
    let mut stream = std::pin::pin!(turn.into_event_stream());
    while let Some(event) = stream.next().await {
        events.push(event.expect("loopback response should decode"));
    }
    events
}

/// Waits for exactly `count` observations, with a bounded drain.
async fn collect_observations(
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<RunObservation>,
    count: usize,
) -> Vec<RunObservation> {
    let mut observations = Vec::new();
    for _ in 0..count {
        let observation = tokio::time::timeout(std::time::Duration::from_secs(10), rx.recv())
            .await
            .expect("observation should arrive in bounded time")
            .expect("observation channel should stay open until the server aborts");
        observations.push(observation);
    }
    observations
}

#[tokio::test]
async fn cursor_destination_pin_regression_enforces_run_url_for_mixed_models() {
    let _observer = offload::OFFLOAD_OBSERVER.lock().unwrap_or_else(|e| e.into_inner());
    // Byte-level constant regression (D-01): replacement of the proven Run
    // destination must fail here first.
    assert_eq!(super::agent::AGENT_BASE_URL, D01_PROVEN_URL);
    assert_eq!(super::agent::AGENT_PATH, D01_PROVEN_PATH);

    let (client, observation_rx, _loopback) = start_run_loopback().await;
    // Concurrent mixed batch: three models; the destination must not
    // depend on model, mode, fast flag, or client version.
    let dispatches = ISOLATION_MODELS.map(|model| {
        dispatch_one(
            &client,
            RunSpec {
                model,
                fast: true,
                mode: 1,
                tool: None,
            },
        )
    });
    let results = futures_util::future::join_all(dispatches).await;
    assert_eq!(
        results.len(),
        ISOLATION_MODELS.len(),
        "every dispatch should be observed",
    );

    let mut observation_rx = observation_rx;
    let observations = collect_observations(&mut observation_rx, ISOLATION_MODELS.len()).await;
    assert_eq!(observations.len(), ISOLATION_MODELS.len());
    let mut request_ids = Vec::new();
    for observation in &observations {
        assert_eq!(observation.method, "POST");
        assert_eq!(observation.path, D01_PROVEN_PATH);
        request_ids.push(observation.request_id.clone());
    }
    let unique_ids: std::collections::HashSet<_> = request_ids.iter().collect();
    assert_eq!(
        unique_ids.len(),
        observations.len(),
        "every Run POST must carry its own request id",
    );
    // TLS identity plus resolved DNS tie the loopback to the proven host:
    // any other destination constant would fail the rustls trust chain
    // before a Run POST could complete.
}

#[tokio::test]
async fn cursor_request_isolation_runs_concurrent_mixed_facts_without_shared_state() {
    let _observer = offload::OFFLOAD_OBSERVER.lock().unwrap_or_else(|e| e.into_inner());
    let (client, observation_rx, _loopback) = start_run_loopback().await;
    let specs = [
        RunSpec {
            model: ISOLATION_MODELS[0],
            fast: false,
            mode: 1, // AGENT
            tool: None,
        },
        RunSpec {
            model: ISOLATION_MODELS[1],
            fast: true,
            mode: 2, // ASK
            tool: None,
        },
        RunSpec {
            model: ISOLATION_MODELS[2],
            fast: false,
            mode: 3, // PLAN; capability marker tool rides only on this request
            tool: Some("cap_tool_marker"),
        },
    ];
    // Interleaved dispatch under tokio join: no detached tasks (the
    // bounded-concurrency DoS mitigation in the plan threat model).
    let (events0, events1, events2) = tokio::join!(
        dispatch_one(&client, specs[0]),
        dispatch_one(&client, specs[1]),
        dispatch_one(&client, specs[2]),
    );
    let mut observation_rx = observation_rx;
    let observations = collect_observations(&mut observation_rx, specs.len()).await;
    assert_eq!(observations.len(), specs.len());

    // Per-request fact isolation: each captured stream carries exactly its
    // own model token and none of the other request tokens.
    let mut observed_models = Vec::new();
    for observation in &observations {
        let model = model_token_in_body(&observation.body).expect("model token in body");
        let expected = specs.iter().find(|spec| spec.model == model).unwrap();
        let length = u32::from_be_bytes(observation.body[1..5].try_into().unwrap()) as usize;
        let run =
            <ObservedRunInput as prost::Message>::decode(&observation.body[5..5 + length])
                .unwrap()
                .request
                .unwrap();
        let observed_model = run.model.unwrap();
        assert_eq!(observed_model.name, expected.model);
        assert!(observed_model
            .parameters
            .iter()
            .any(|parameter| parameter.key == "fast"
                && parameter.value == expected.fast.to_string()));
        assert_eq!(
            run.action.unwrap().user_action.unwrap().user.unwrap().mode,
            expected.mode
        );
        assert_eq!(
            haystack_contains(&observation.body, b"cap_tool_marker"),
            expected.tool.is_some()
        );
        for other in ISOLATION_MODELS {
            if other != model {
                assert!(
                    !haystack_contains(&observation.body, other.as_bytes()),
                    "cross-request fact leak outside its own stream",
                );
            }
        }
        if model != ISOLATION_MODELS[2] {
            assert!(
                !haystack_contains(&observation.body, b"cap_tool_marker"),
                "capability fact leaked across requests",
            );
        }
        observed_models.push(model);
    }
    observed_models.sort_unstable();
    let mut expected_models = ISOLATION_MODELS.to_vec();
    expected_models.sort_unstable();
    assert_eq!(
        observed_models, expected_models,
        "one stream per expected model"
    );

    // Request ids pair unambiguously: all distinct, one per observation.
    let ids: std::collections::HashSet<_> =
        observations.iter().map(|o| o.request_id.as_str()).collect();
    assert_eq!(ids.len(), observations.len());

    // Server echoes the observed model token back through the same stream,
    // so each response pairs with the request that carried that token.
    for (events, expected) in [events0, events1, events2].into_iter().zip(specs) {
        let text: String = events
            .iter()
            .filter_map(|event| match event {
                super::response::CursorStreamEvent::TextDelta { text } => Some(text.clone()),
                _ => None,
            })
            .collect();
        let decoded_model = text.trim_start_matches("echo:");
        assert_eq!(
            decoded_model, expected.model,
            "response must pair to its own request"
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event, super::response::CursorStreamEvent::End)),
            "each response should end with the wire terminal",
        );
    }
}
