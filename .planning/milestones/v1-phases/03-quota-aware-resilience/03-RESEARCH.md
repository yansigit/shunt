# Phase 03: Quota-Aware Resilience - Research

**Researched:** 2026-09-05
**Domain:** Quota-Aware Resilience and Retry Scheduling
**Confidence:** HIGH

## Summary

Phase 03 implements quota-aware resilience for Shunt, addressing requirements RES-01 and RES-02. The core objective is to distinguish transient request-rate throttling from hard quota exhaustion on existing provider and account paths, and to derive standards-compliant, bounded cooldown/retry timing from the `Retry-After` header without altering unrelated failover behavior.

Currently, Shunt's account pool rotates every HTTP 429 as a generic failure without inspecting response bodies [VERIFIED: src/accounts.rs:3049-3063], while its `retry_after` parser only accepts integer seconds and HTTP dates, without supporting decimal delta-seconds, input length bounds, or finite maximum clamping [VERIFIED: src/accounts.rs:3065-3081]. OpenCodex solved this via a battle-tested pre-stream quota rejection classifier [VERIFIED: /Volumes/PortableSSD/Projects/opencodex/src/codex/quota-rejection.ts:1-280] and an RFC-compliant `Retry-After` parser [VERIFIED: /Volumes/PortableSSD/Projects/opencodex/src/combos/failover.ts:95-150].

The primary recommendation is to introduce a dedicated, focused resilience module (e.g. `src/quota.rs` or submodules under `src/accounts/`) containing:
1. A standards-compliant `Retry-After` parser supporting decimal delta-seconds with safe ceiling, RFC 7231/9110 HTTP dates (IMF-fixdate, RFC 850, asctime), immediate non-zero positive boundaries for zero/past values, and an explicit finite maximum clamp.
2. A typed pre-stream quota rejection classifier that inspects bounded response bodies (capped at 64 KiB), recognizes hard exhaustion only from exact structured code/type evidence (`usage_limit_exceeded`, `insufficient_quota`), rejects ambiguous/duplicate-key JSON payloads fail-closed to the transient class, and supports provider-specific status contracts (such as Google AIP-194).
3. Integration with `src/adapters/responses/pool.rs`, `src/adapters/responses/inbound.rs`, and `src/accounts.rs` to mark hard-exhausted accounts unavailable until their bounded deadline, preventing futile cycling back through exhausted accounts in the same turn while keeping streaming passthrough untouched.

**Primary recommendation:** Centralize `Retry-After` parsing and typed pre-stream quota classification in a dedicated module, wire them into the account pool cooldown and rotation loop before response streaming begins, and preserve verbatim upstream error bodies and headers upon pool exhaustion.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Retry-After Parsing | Core Gateway (Accounts/Retry) | — | Single shared standard parser prevents divergence across single-credential retries and pool cooldowns. |
| Pre-Stream Quota Classification | Upstream Adapter / Pool Boundary | — | Must execute before response bytes stream to client; inspects bounded error bodies. |
| Account Cooldown & Selection | Account Pool (`src/accounts.rs`) | — | State store already tracks `cooldown_until` and prioritizes headroom/availability during turn selection. |
| Pre-Output Account Rotation | Responses Inbound & Outbound Pool | — | Loops candidate accounts before first output byte, advancing on transient/exhaustion failures. |
| Route Chain Failover | Proxy Dispatch (`src/proxy/failover.rs`) | — | Multi-provider fallback chain; kept separate from account-pool quota classification per RES-01. |

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Classification Contract
- Treat HTTP 429 alone as a generic transient rate limit, not proof of exhausted quota.
- Recognize hard exhaustion only from bounded, successfully parsed provider error bodies with exact trusted code/type evidence such as `usage_limit_exceeded` or `insufficient_quota`; ambiguous, malformed, duplicate, aborted, or oversized bodies fail closed to the transient/unverified class.
- Provider-specific text heuristics may be used only where the provider has a stable structured status contract, and transient request-rate language wins over broad quota wording.
- Keep authentication, permission, server-error, policy, and unrelated failover classifications unchanged unless a focused regression proves they intersect RES-01.

#### Retry Timing
- Use one shared `Retry-After` parser for both delta-seconds and HTTP-date forms rather than adding adapter-local copies.
- Accept non-negative decimal delta-seconds with safe rounding and standards-compliant HTTP dates; malformed or excessively long values are ignored.
- Convert zero or past deadlines to an immediate-but-nonzero retry boundary where internal cooldown bookkeeping requires a future instant, avoiding wall-clock underflow.
- Clamp every honored deadline to an explicit finite maximum; an excessive upstream value must never create an unbounded cooldown.

#### Pool and Failover Behavior
- Transient throttles receive bounded short retry/cooldown behavior and may rotate through already-supported account/fallback paths before output.
- Proven hard quota exhaustion marks the affected account/provider unavailable until a safe bounded deadline and avoids futile cycling back through exhausted accounts in the same turn.
- Never replay after response bytes or WebSocket events become observable; Phase 1 and Phase 2 streaming and per-turn snapshot guarantees remain binding.
- Preserve the final upstream error body and safe `Retry-After` exposure when all eligible attempts are exhausted.

#### Scope and Verification
- Port OpenCodex's behavior and adversarial fixtures, not its duplicate parsers, SQLite history, compatibility lab, cost scoring, or generalized circuit-breaker platform.
- Add table-driven unit tests for classification and time parsing plus integration coverage for account rotation, exhaustion, bounded inspection, and unchanged unrelated statuses.
- Run focused suites and the repository-wide format, strict Clippy, and all-feature workspace tests.
- Update English and maintained README/Nimbus translations only where the resulting behavior is observable; do not hand-edit `wiki/`.

### Claude's Discretion
Internal enum names, exact module placement, and the finite short/maximum constants are implementation discretion, provided they are centralized, tested at their boundaries, and do not become new public configuration.

### Deferred Ideas (OUT OF SCOPE)
- General provider health scoring and persisted request history remain out of scope.
- Changes to public cooldown configuration remain out of scope and require separate approval.
- Capability-aware heterogeneous fallback remains Phase 6.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| RES-01 | Shunt distinguishes transient request-rate limiting from hard quota exhaustion using provider status, codes, and bounded body inspection. | Supported by OpenCodex's `classifyCodexPreStreamRejection` pattern [VERIFIED: /Volumes/PortableSSD/Projects/opencodex/src/codex/quota-rejection.ts:232-280] and Google AIP-194 status rules [VERIFIED: /Volumes/PortableSSD/Projects/opencodex/src/adapters/google-errors.ts:25-67]. Bounded body inspection (< 64 KiB) before stream output prevents streaming buffer leaks. |
| RES-02 | Retry scheduling honors both delta-seconds and standards-compliant HTTP-date `Retry-After` values without wall-clock underflow or unbounded cooldowns. | Supported by OpenCodex's `parseRetryAfterMs` [VERIFIED: /Volumes/PortableSSD/Projects/opencodex/src/combos/failover.ts:95-150] and Rust `httpdate` crate [VERIFIED: Cargo.toml:29]. Centralized at `accounts::retry_after` and capped at a finite maximum (e.g., 3600s). |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Write code in English, and documentation in English except for maintained translations (`README.<locale>.md` and `site/src/content/docs/<locale>/`). [VERIFIED: AGENTS.md:27-30]
- Keep Rust files focused and preferably under 500 lines. [VERIFIED: AGENTS.md:30-30]
- Preserve streaming semantics; do not buffer upstream SSE responses unless the client requested non-streaming output. [VERIFIED: AGENTS.md:31-31]
- Keep gateway-owned errors in the Anthropic error shape, except on the inbound Codex endpoint (`[server.codex_endpoint]`), where gateway-owned errors use the OpenAI Responses error shape. [VERIFIED: AGENTS.md:32-34]
- Prefer table-driven config additions over hardcoded provider logic. [VERIFIED: AGENTS.md:35-35]
- Ask before changing credential-file writeback behavior or public config keys/provider semantics; never weaken or remove tests. [VERIFIED: AGENTS.md:58-60]
- Update affected documentation surfaces in the same PR; do not hand-edit generated `wiki/`. [VERIFIED: AGENTS.md:36-53]

## Standard Stack

| Component | Library / Crate | Version | Provenance | Purpose |
|-----------|-----------------|---------|------------|---------|
| HTTP Date Parser | `httpdate` | `1` | [VERIFIED: Cargo.toml:29] | Standard RFC 7231 / RFC 9110 date parser (IMF-fixdate, RFC 850, asctime). |
| JSON Parser & AST | `serde_json` | `1` | [VERIFIED: Cargo.toml:50] | Structured JSON parsing for error response bodies. |
| Byte Buffers | `bytes` | `1` | [VERIFIED: Cargo.toml:23] | Zero-copy slicing and body reconstruction for bounded inspections. |
| HTTP Types & Headers | `axum` / `http` / `reqwest` | `0.8` / `1` / `0.12` | [VERIFIED: Cargo.toml:21,44] | Inbound and outbound HTTP request/response representations. |
| Time Primitives | `std::time::{Duration, Instant, SystemTime}` | stdlib | [VERIFIED: stdlib] | Monotonic deadlines (Instant) and wall-clock parsing (SystemTime). |

No new external dependencies are required. All primitives are already available in `Cargo.toml`.

## Architecture Patterns

### 1. Centralized Retry Timing (`Retry-After`)

The current implementation in `src/accounts.rs:3065-3082` is:
```rust
pub fn retry_after(headers: &HeaderMap) -> Option<Duration> {
    let value = headers.get(reqwest::header::RETRY_AFTER)?.to_str().ok()?;
    // RFC 7231 allows two forms: delta-seconds or an HTTP-date. Try the cheap
    // numeric form first, then fall back to the date form — a server that sends
    // `Retry-After: <HTTP-date>` would otherwise be silently ignored.
    if let Ok(seconds) = value.trim().parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }
    let deadline = httpdate::parse_http_date(value.trim()).ok()?;
    // Honor the wait until that instant; a deadline already in the past means
    // "retry now" (zero wait) rather than falling through to computed backoff.
    Some(
        deadline
            .duration_since(SystemTime::now())
            .unwrap_or(Duration::ZERO),
    )
}
```
[VERIFIED: src/accounts.rs:3065-3081]

This will be upgraded to:
1. **Length Cap:** Values with `trimmed.len() > 128` return `None` to prevent CPU/memory exploitation.
2. **Delta-Seconds:**
   - Parse decimal strings (e.g. `"1.5"`, `"0.25"`, `"0"`, `"60"`).
   - Round up (ceil) fractional seconds to avoid premature retries before upstream release.
   - Reject negative numbers, exponents, multiple decimals, and non-digit characters.
3. **HTTP Dates:**
   - Evaluated via `httpdate::parse_http_date` (covers IMF-fixdate, RFC 850, and ANSI asctime).
4. **Immediate-but-Nonzero Boundary:**
   - Zero seconds (`"0"`) or dates in the past resolve to `Duration::from_millis(1)` (or `Duration::from_secs(1)` for integer second contexts) when evaluated for internal cooldowns, preventing underflow while guaranteeing an immediate future timestamp.
   - Note: Client exposure of raw headers retains `"0"` if received from upstream.
5. **Finite Maximum Clamp:**
   - All durations are clamped to a finite ceiling (e.g. `MAX_RETRY_AFTER = Duration::from_secs(3600)`). An upstream date 10 years in the future will never produce an infinite cooldown.

### 2. Pre-Stream Bounded Body Inspection & Rejection Classification

Located at the boundary before response streaming begins (in `src/adapters/responses/pool.rs` and `src/adapters/responses/inbound.rs`).

#### Bounded Body Reading:
- Read up to `BOUNDED_BODY_MAX_BYTES = 65_536` (64 KiB) [VERIFIED: /Volumes/PortableSSD/Projects/opencodex/src/lib/bounded-body.ts:4: `export const BOUNDED_BODY_MAX_BYTES = 65_536;`].
- If body exceeds 64 KiB, contains invalid UTF-8, encounters transport timeout, or is aborted, fail closed: do NOT treat as hard quota exhaustion. Treat as unverified/transient.
- Buffer the raw bytes so that if all accounts fail, the exact final error body can be returned to the client without losing data [VERIFIED: src/adapters/responses/inbound.rs:248-290].

#### Discrete Values from OpenCodex Reference:
Exact reset-eligible exhaustion codes in OpenCodex:
```ts
const RESET_ELIGIBLE_CODE_VALUES = [
  "usage_limit_exceeded",
  "insufficient_quota",
] as const;
```
[VERIFIED: /Volumes/PortableSSD/Projects/opencodex/src/codex/quota-rejection.ts:3-6]

Rejection kinds in OpenCodex:
```ts
export type CodexPreStreamRejectionKind = 
  | "reset-eligible-exhaustion"
  | "generic-rate-limit"
  | "unverified-billing-or-quota"
  | "transient-server-error"
  | "authentication-error"
  | "permission-error"
  | "other";
```
[VERIFIED: /Volumes/PortableSSD/Projects/opencodex/src/codex/quota-rejection.ts:11-18]

Discrete denial codes for 403 in OpenCodex:
```ts
const WORKSPACE_DENIAL_CODES: ReadonlySet<string> = new Set([
  "codex_workspace_access_denied",
  "workspace_access_denied",
]);

const ENTITLEMENT_DENIAL_CODES: ReadonlySet<string> = new Set([
  "codex_entitlement_missing",
  "entitlement_missing",
]);
```
[VERIFIED: /Volumes/PortableSSD/Projects/opencodex/src/codex/quota-rejection.ts:37-45]

Cooldown constants in OpenCodex:
```ts
const DEFAULT_COOLDOWN_MS = 60_000;
const MAX_COOLDOWN_MS = 10 * 60_000;
export const COMBO_REQUEST_RATE_COOLDOWN_MS = 5_000;
```
[VERIFIED: /Volumes/PortableSSD/Projects/opencodex/src/combos/failover.ts:13-16]

Google-family transient rate limit patterns and quota needles:
```ts
const GOOGLE_QUOTA_EXHAUSTED_NEEDLES = [
  "quotafailure",
  "quota exceeded",
  "exceeded your current quota",
  "billing",
  "individual quota reached",
  "quota reached",
  "enable overages",
  "exhausted your capacity",
  "daily limit reached",
  "weekly limit reached",
];

const GOOGLE_TRANSIENT_RATE_LIMIT_PATTERNS = [
  "per minute",
  "per-minute",
  "per min",
  "rpm",
  "requests per minute",
  "too many requests",
  "rate limit",
  "retry after",
  "retry-after",
  "concurrent request limit",
];
```
[VERIFIED: /Volumes/PortableSSD/Projects/opencodex/src/adapters/google-errors.ts:20-43]

#### Shunt Implementation Pattern:
In Shunt, we introduce a typed enum:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreStreamRejection {
    HardExhaustion { code: String },
    TransientRateLimit,
    UnverifiedBillingOrQuota,
    TransientServerError,
    AuthenticationError,
    PermissionError { denial: Option<DenialKind> },
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenialKind {
    Workspace,
    Entitlement,
}
```

#### Rules for Hard Exhaustion:
1. Status must be 429 or 402.
2. The payload must be valid JSON with NO duplicate keys at any depth.
3. The exact structured `code` or `type` field must match `usage_limit_exceeded` or `insufficient_quota`.
4. If both `code` and `type` are present, they must match identically.
5. Ambiguous schemas (e.g. root discriminator AND nested error discriminator) fail closed to generic rate limit.
6. Near-misses (whitespace, case differences, words appearing only in `message` prose) fail closed to generic rate limit.

#### Provider-Specific Rules (e.g. Google AIP-194):
- Google error JSON: `{ "error": { "code": 429, "message": "...", "status": "RESOURCE_EXHAUSTED" } }`
- Transient patterns win: if message contains `"per minute"`, `"rpm"`, `"rate limit"`, `"concurrent request limit"`, it is classified as `TransientRateLimit` [VERIFIED: /Volumes/PortableSSD/Projects/opencodex/src/adapters/google-errors.ts:35-44].
- Hard quota patterns: if no transient pattern matches and text contains `"quota exceeded"`, `"quotafailure"`, `"daily limit reached"`, `"billing"`, it is classified as `HardExhaustion` [VERIFIED: /Volumes/PortableSSD/Projects/opencodex/src/adapters/google-errors.ts:20-31].

### 3. Account Pool Wiring and Turn-Level Exhaustion Tracking

1. **Transient Rate Limit:**
   - Cooldown: short bounded cooldown (e.g. `Retry-After` if present; if absent, short fallback e.g. 5s [VERIFIED: /Volumes/PortableSSD/Projects/opencodex/src/combos/failover.ts:16]).
   - Account Pool rotates to the next candidate in selection order.
2. **Hard Quota Exhaustion:**
   - Cooldown: set to honored `Retry-After` or default quota window deadline (e.g. clamped up to 1h).
   - Marked unavailable in pool.
   - **No futile cycling in the same turn:** The request turn tracks tried accounts and excludes hard-exhausted accounts from rotation retry during the remainder of the turn.
3. **Preserving Stream Safety:**
   - No retry or rotation can occur once any response body byte or WebSocket event has been dispatched to downstream client.

## Don't Hand-Roll

| Problem | Anti-Pattern to Avoid | Standard Solution |
|---------|----------------------|-------------------|
| Date Parsing | Custom regex or hand-rolled HTTP date parser | Use `httpdate::parse_http_date` which already conforms to RFC 7231 / 9110 [VERIFIED: Cargo.toml:29]. |
| Cooldown State | SQLite DB or external storage for cooldowns | Use existing memory-only `AccountHealth::cooldown_until` in `src/accounts.rs`. |
| Error Relaying | Synthetic error re-generation on pool exhaustion | Buffer the bounded error response bytes and relay the exact upstream status, headers, and body. |
| Duplicate Retries | Retrying after first byte delivered | Commit response on first byte; never retry mid-stream. |
| Circuit Breaker | General provider health scoring platform | Out of scope per CONTEXT.md; rely strictly on typed pre-stream rejections and bounded cooldowns. |

## Common Pitfalls

1. **Premature Body Consumption:**
   - *Gotcha:* Calling `upstream.bytes()` consumes the `reqwest::Response`, making it unavailable to relay if all accounts are exhausted.
   - *Mitigation:* Store the bounded bytes along with status and headers in a buffered structure (or rebuild the response via `axum::response::Response::builder()`).
2. **Duplicate Key Permissiveness:**
   - *Gotcha:* Standard `serde_json::Value` parsing silently overwrites duplicate keys, allowing an attacker or malformed server to hide `rate_limit_error` behind `usage_limit_exceeded`.
   - *Mitigation:* Scan tokens or validate key uniqueness recursively, failing closed to transient if duplicate keys are detected.
3. **Wall-Clock Underflow:**
   - *Gotcha:* A `Retry-After` date in the past or zero seconds resulting in `Duration::ZERO` can cause `Instant::now() + Duration::ZERO` to immediately expire or race.
   - *Mitigation:* Convert non-positive durations to an immediate positive boundary (e.g., 1ms or 1s).
4. **Unbounded Cooldowns:**
   - *Gotcha:* Upstream returning `Retry-After: 999999999` could lock an account out for decades.
   - *Mitigation:* Clamp all honored durations to `MAX_RETRY_AFTER = Duration::from_secs(3600)`.
5. **Streaming Leaks:**
   - *Gotcha:* Attempting to classify a stream after headers and initial SSE frames have already been sent to the client.
   - *Mitigation:* Classification must strictly occur before any bytes are forwarded.

## Code Examples

### 1. Robust Retry-After Parser
```rust
pub fn parse_retry_after(headers: &HeaderMap, now: SystemTime) -> Option<Duration> {
    let raw = headers.get(reqwest::header::RETRY_AFTER)?.to_str().ok()?;
    parse_retry_after_str(raw, now)
}

pub fn parse_retry_after_str(raw: &str, now: SystemTime) -> Option<Duration> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 128 {
        return None;
    }
    const MAX_COOLDOWN: Duration = Duration::from_secs(3600);
    const MIN_COOLDOWN: Duration = Duration::from_millis(1);

    // Delta-seconds (integer or decimal)
    if let Ok(seconds) = trimmed.parse::<f64>() {
        if seconds.is_finite() && seconds >= 0.0 {
            if seconds == 0.0 {
                return Some(MIN_COOLDOWN);
            }
            let duration = Duration::from_secs_f64(seconds).max(MIN_COOLDOWN);
            return Some(duration.min(MAX_COOLDOWN));
        }
        return None;
    }

    // HTTP-date (IMF-fixdate, RFC 850, asctime)
    if let Ok(deadline) = httpdate::parse_http_date(trimmed) {
        let duration = deadline.duration_since(now).unwrap_or(MIN_COOLDOWN);
        let duration = if duration.is_zero() { MIN_COOLDOWN } else { duration };
        return Some(duration.min(MAX_COOLDOWN));
    }

    None
}
```

### 2. Duplicate Key Detection Scanner
```rust
pub fn has_duplicate_json_keys(text: &str) -> bool {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    check_unique_keys(&mut deserializer).is_err()
}
```

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust / Cargo | Build & Test | ✓ | 1.84+ | — |
| `httpdate` crate | RES-02 HTTP date parsing | ✓ | 1.0.3 | — |
| `serde_json` crate | RES-01 Error body parsing | ✓ | 1.0.138 | — |
| `wiremock` crate | Integration tests | ✓ | 0.6.2 | — |

No missing dependencies.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (built-in + wiremock) |
| Config file | `Cargo.toml` |
| Quick run command | `cargo test --test codex_multi_account -- a_429` |
| Full suite command | `cargo test --all-features --workspace` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| RES-01 | Distinguish transient 429 from hard quota (`usage_limit_exceeded`, `insufficient_quota`) via bounded body | unit | `cargo test --lib quota::tests::test_classify_rejection` | ❌ Wave 0 (to be added) |
| RES-01 | Fails closed on duplicate keys, malformed JSON, UTF-8 errors, oversized bodies | unit | `cargo test --lib quota::tests::test_classify_fails_closed` | ❌ Wave 0 (to be added) |
| RES-01 | Google AIP-194 transient vs quota pattern distinction | unit | `cargo test --lib quota::tests::test_google_quota_patterns` | ❌ Wave 0 (to be added) |
| RES-01 | Multi-account rotation cools down exhausted account and avoids futile cycling in same turn | integration | `cargo test --test codex_multi_account -- hard_quota_avoids_futile_cycling` | ❌ Wave 0 (to be added) |
| RES-01 | All accounts exhausted relays verbatim final upstream error body and headers | integration | `cargo test --test codex_multi_account -- exhausted_pool_preserves_final_body` | ❌ Wave 0 (to be added) |
| RES-02 | Parse integer and decimal delta-seconds with ceiling rounding | unit | `cargo test --lib accounts::tests::parses_numeric_retry_after` | ✅ (needs decimal tests) |
| RES-02 | Parse HTTP dates (IMF-fixdate, RFC 850, asctime) | unit | `cargo test --lib accounts::tests::parses_http_date_retry_after` | ✅ |
| RES-02 | Zero or past dates resolve to immediate non-zero positive duration | unit | `cargo test --lib accounts::tests::past_http_date_retry_after_is_nonzero` | ❌ Wave 0 (to be updated) |
| RES-02 | Clamp excessive `Retry-After` to finite maximum | unit | `cargo test --lib accounts::tests::excessive_retry_after_clamped` | ❌ Wave 0 (to be added) |

### Sampling Rate
- **Per task commit:** `cargo test --lib quota && cargo test --test codex_multi_account`
- **Per wave merge:** `cargo test --all-features --workspace && cargo clippy --all-targets --all-features -- -D warnings`
- **Phase gate:** Full test suite, format check, and Clippy green before closing phase.

### Wave 0 Gaps
- [ ] Unit tests for decimal `Retry-After`, overlong input rejection, past date non-zero normalization, and max clamp.
- [ ] Unit tests for pre-stream rejection classifier with adversarial payloads (duplicate keys, near-miss strings, oversized padding).
- [ ] Integration tests in `tests/codex_multi_account.rs` asserting hard-exhaustion vs generic-throttle rotation and final body relay.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | Yes | Bound upstream error body reads to 64 KiB; enforce UTF-8 validity; parse JSON with duplicate key rejection; bound `Retry-After` string length to 128 chars. |
| V4 Access Control & Availability | Yes | Clamp maximum cooldown to 1 hour to prevent indefinite denial of service / account lockouts; prevent futile cycling in rotation loops. |
| V2 Authentication | Yes | Preserve distinct handling of 401 credential refresh vs 403 authorization denials (workspace/entitlement). |

### Known Threat Patterns for Gateway Resilience

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unbounded `Retry-After` DoS | Denial of Service | Clamp all honored retry-after values to 3600 seconds. |
| Slowloris / Memory Inflation on Error Body | Denial of Service | Impose 64 KiB ceiling (`BOUNDED_BODY_MAX_BYTES`) and stream timeout on error body buffering. |
| JSON Parser Smuggling (Duplicate Keys) | Tampering | Reject duplicate JSON keys fail-closed to generic rate limit; do not trust ambiguous payloads. |
| False Quota Exhaustion Injection | Denial of Service | Only trust structured `code` or `type` fields; do not trigger hard exhaustion on unstructured message text that might echo prompt inputs. |

## Sources

### Primary (HIGH confidence)
- OpenCodex codebase:
  - `/Volumes/PortableSSD/Projects/opencodex/src/codex/quota-rejection.ts:1-280` - pre-stream rejection classifier and duplicate key detection.
  - `/Volumes/PortableSSD/Projects/opencodex/src/combos/failover.ts:1-150` - `parseRetryAfterMs`, cooldown constants, and transient vs quota codes.
  - `/Volumes/PortableSSD/Projects/opencodex/src/adapters/google-errors.ts:1-75` - Google AIP-194 error message heuristics.
  - `/Volumes/PortableSSD/Projects/opencodex/tests/codex-quota-rejection.test.ts:1-350` - adversarial fixtures for quota rejection.
- Shunt codebase:
  - `src/accounts.rs:3049-3082` - existing `classify_codex` and `retry_after` implementations.
  - `src/adapters/responses/pool.rs:480-495, 650-710` - `rotate_cooldown` and `classify_first`.
  - `src/adapters/responses/inbound.rs:160-290` - passthrough send and account rotation loop.
  - `src/retry.rs:320-335` - `next_backoff` single-credential retry logic.
  - `src/proxy/failover.rs:388-397` - route failover advance statuses.

### Secondary (MEDIUM confidence)
- RFC 7231 / RFC 9110 Section 10.2.3: `Retry-After` header specifications.
- Google AIP-194: Error handling and Resource Exhausted guidelines.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All required dependencies already present and verified in `Cargo.toml`.
  - `httpdate`: [VERIFIED: Cargo.toml:29]
  - `serde_json`: [VERIFIED: Cargo.toml:50]
  - `bytes`: [VERIFIED: Cargo.toml:23]
- Architecture: HIGH - Integration seams in `src/accounts.rs`, `src/adapters/responses/pool.rs`, and `src/adapters/responses/inbound.rs` identified and verified.
- Pitfalls & Security: HIGH - Ported directly from OpenCodex production lessons and verified test suites.

**Research date:** 2026-09-05
**Valid until:** 2026-10-05 (stable domain)

