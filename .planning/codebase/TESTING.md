# Testing Patterns

**Analysis Date:** 2026-09-05

## Test Framework

**Runner:**
- Rust standard test harness via Cargo (`cargo test`)
- Tokio asynchronous runtime test macros: `tokio = { version = "1", features = ["full", "test-util"] }`
- Benchmark runner: `divan = { version = "5.0.1", package = "codspeed-divan-compat" }` for CPU-bound hot spots in `benches/`
- Config: Configured in `Cargo.toml` and GitHub Actions workflow (`.github/workflows/ci.yml`)

**Assertion Library:**
- Standard library macros: `assert!`, `assert_eq!`, `assert_ne!`, `assert!(body.contains(...))`
- JSON assertions: `serde_json::json!` and `assert-json-diff` (via Wiremock / dev-dependencies)

**Run Commands:**
```bash
cargo test --all-features --workspace                # Run all unit and integration tests
cargo test --test passthrough                         # Run specific integration test
cargo test <test_name_filter>                         # Run tests matching name filter
cargo llvm-cov --all-features --workspace --lcov      # Generate test coverage report
```

## Test File Organization

**Location:**
- Unit tests: Co-located in `src/`, either in-file via `#[cfg(test)] mod tests { ... }` or in dedicated submodule `tests.rs` / subdirectories (e.g., `src/codex_endpoint/tests.rs`, `src/config/upstreams/tests.rs`, `src/auth/slots/tests.rs`)
- Integration tests: Dedicated files in `tests/` directory (e.g., `tests/passthrough.rs`, `tests/responses_translate.rs`, `tests/inbound_codex_endpoint.rs`, `tests/multi_account.rs`)
- Smoke & driver scripts: Shell-based hermetic drivers in `.claude/skills/run-shunt/smoke.sh` and `.claude/skills/run-claude-gateway-ref/driver.sh`

**Naming:**
- Integration test files: `tests/<subsystem>.rs` (e.g., `tests/failover.rs`, `tests/inbound_auth.rs`, `tests/kimi_multi_account.rs`)
- Unit test modules: `tests.rs` or `mod tests`
- Test functions: Descriptive snake_case naming stating the behavior under test (e.g., `reshapes_shunt_error_401_to_openai_shape`, `stream_chunks_forward_as_they_arrive`, `account_uuid_is_rewritten_for_each_account_during_rotation`)

**Structure:**
```
shunt/
├── benches/
│   ├── gateway.rs                     # CodSpeed microbenchmarks
│   └── perf_issues.rs
├── src/
│   ├── <module>.rs                    # Inlined #[cfg(test)] mod tests { ... }
│   └── <module>/
│       └── tests.rs                   # Dedicated unit test submodules
└── tests/
    ├── add_cli.rs                     # CLI integration tests
    ├── check_cli.rs
    ├── codex_multi_account.rs         # Provider / pool integration tests
    ├── inbound_codex_endpoint.rs
    ├── passthrough.rs                 # Gateway proxy & header preservation tests
    └── responses_translate.rs         # Protocol translation tests
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn reshapes_shunt_error_401_to_openai_shape() {
        let response = ShuntError::new(
            StatusCode::UNAUTHORIZED,
            "authentication_error",
            "missing client token",
        )
        .into_response();
        let reshaped = into_openai_error_shape(response).await;
        assert_eq!(reshaped.status(), StatusCode::UNAUTHORIZED);
        let body = body_json(reshaped).await;
        assert!(body.get("type").is_none());
        assert_eq!(body["error"]["message"], "missing client token");
        assert_eq!(body["error"]["type"], "authentication_error");
        assert!(body["error"].get("code").is_some_and(Value::is_null));
    }
}
```

**Patterns:**
- **Setup pattern:** Build test config using helper functions (`test_config(...)`, `retry_test_config(...)`), start mock servers with Wiremock (`MockServer::start().await`), and spawn real Axum test server on an ephemeral loopback port (`start_gateway_with(config).await`)
- **Teardown pattern:** RAII cleanup via `Drop` implementations on guard structs:
  - `TestGateway` implements `Drop` to abort the server background task (`self.task.abort()`)
  - `TempDir` implements `Drop` to delete temporary SQLite databases and account directories
- **Assertion pattern:** Strict status code checks, header checks (both inclusion and absence), exact JSON payload matching, and stream body assertions

## Mocking

**Framework:** `wiremock = "0.6"` for HTTP upstream mocking.

**Patterns:**
```rust
let upstream = MockServer::start().await;
Mock::given(method("POST"))
    .and(path("/v1/messages"))
    .and(header("x-api-key", "secret-token"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
        "id": "msg_123",
        "type": "message",
        "role": "assistant",
        "content": [{"type": "text", "text": "hello"}]
    })))
    .expect(1)
    .mount(&upstream)
    .await;

// Verify expectations at end of test:
upstream.verify().await;
```

**What to Mock:**
- External LLM provider endpoints (Anthropic, OpenAI, Codex, Kimi, Cursor, Gemini)
- OAuth token discovery and exchange endpoints
- HTTP telemetry collector targets

**What NOT to Mock:**
- Gateway HTTP server stack: real Axum router and middleware are executed over loopback TCP listeners
- Request routing and translation logic: tested with real data structures and byte streams
- Account pool rotation and concurrency limits: run against live in-memory state

## Fixtures and Factories

**Test Data:**
```rust
fn account(name: &str, token_env: &str) -> AccountConfig {
    AccountConfig {
        name: name.to_string(),
        token_env: Some(token_env.to_string()),
        ..Default::default()
    }
}

fn test_config(upstream_base_url: &str, accounts: Vec<AccountConfig>) -> Config {
    let mut config = Config::default();
    config.server.bind = "127.0.0.1:0".to_string();
    config.providers.get_mut("anthropic").unwrap().base_url = upstream_base_url.to_string();
    config
}
```

**Location:**
- Helper factory functions defined near the top of each test file (`tests/passthrough.rs`, `tests/inbound_codex_endpoint.rs`, `tests/codex_multi_account.rs`)
- Custom Wiremock matchers implementing `wiremock::Match` (`ExactHeader`, `HeaderAbsent`, `ExactBody`) defined in integration test suites
- Constant string fixtures for raw protocol envelopes (`INBOUND_BODY`, `HEADER_JSON`)

## Coverage

**Requirements:** Enforced in CI via `cargo-llvm-cov` with reporting to Codecov and SonarQube Cloud Scan (`.github/workflows/ci.yml`).

**View Coverage:**
```bash
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

## Test Types

**Unit Tests:**
- Scope: Pure functions, error conversions, protocol serializers/deserializers, token counters, and state transitions
- Located inside `src/` modules and submodules (`mod tests`)
- Fast, synchronous or lightweight tokio async tests with zero network I/O

**Integration Tests:**
- Scope: End-to-end HTTP pipeline across Axum routes, middleware, protocol translation, account pool rotation, and upstream proxying
- Located in `tests/*.rs`
- Spin up real Axum listeners on `127.0.0.1:0` and Wiremock upstream servers

**E2E Tests:**
- Standalone smoke test script: `.claude/skills/run-shunt/smoke.sh` drives the compiled `shunt` binary with a local Python HTTP mock upstream and `curl` assertions
- Gateway protocol reference harness: `.claude/skills/run-claude-gateway-ref/driver.sh` runs live against Postgres + Dex containers

## Common Patterns

**Async Testing:**
```rust
#[tokio::test]
async fn stream_chunks_forward_as_they_arrive() {
    let upstream = MockServer::start().await;
    // ... configure stream response ...
    let gateway = start_gateway(upstream.uri()).await;
    let client = reqwest::Client::new();
    let mut response = client
        .post(format!("{}/v1/messages", gateway.base_url))
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .body(json!({"model": "claude-sonnet-4-direct", "stream": true}).to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    // Process streaming chunks
}
```

**Error Testing:**
```rust
#[tokio::test]
async fn reshapes_upstream_error_502_to_openai_shape() {
    let response = UpstreamError::from_message("all Codex OAuth accounts failed").into_response();
    let reshaped = into_openai_error_shape(response).await;
    assert_eq!(reshaped.status(), StatusCode::BAD_GATEWAY);
    let body = body_json(reshaped).await;
    assert!(body.get("type").is_none());
    assert_eq!(body["error"]["message"], "all Codex OAuth accounts failed");
    assert_eq!(body["error"]["type"], "api_error");
    assert!(body["error"].get("code").is_some_and(Value::is_null));
}
```

---

*Testing analysis: 2026-09-05*

