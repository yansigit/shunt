---
phase: 09-provider-conformance-foundation
verified: 2026-09-06T22:43:33Z
status: passed
score: 8/8 must-haves verified
covered_files:
  - .planning/REQUIREMENTS.md
  - .planning/ROADMAP.md
  - .planning/STATE.md
  - .planning/phases/09-provider-conformance-foundation/09-01-PLAN.md
  - .planning/phases/09-provider-conformance-foundation/09-01-SUMMARY.md
  - .planning/phases/09-provider-conformance-foundation/09-02-PLAN.md
  - .planning/phases/09-provider-conformance-foundation/09-02-SUMMARY.md
  - .planning/phases/09-provider-conformance-foundation/09-03-PLAN.md
  - .planning/phases/09-provider-conformance-foundation/09-03-SUMMARY.md
  - .planning/phases/09-provider-conformance-foundation/09-04-PLAN.md
  - .planning/phases/09-provider-conformance-foundation/09-04-SUMMARY.md
  - .planning/phases/09-provider-conformance-foundation/09-CONTEXT.md
  - .planning/phases/09-provider-conformance-foundation/09-PATTERNS.md
  - .planning/phases/09-provider-conformance-foundation/09-RESEARCH.md
  - .planning/phases/09-provider-conformance-foundation/09-REVIEW-FIX.md
  - .planning/phases/09-provider-conformance-foundation/09-REVIEW.md
  - .planning/phases/09-provider-conformance-foundation/09-VALIDATION.md
  - docs/m1-responses-translation.md
  - site/src/content/docs/ja/reference/configuration.md
  - site/src/content/docs/ja/reference/troubleshooting.md
  - site/src/content/docs/ko/reference/configuration.md
  - site/src/content/docs/ko/reference/troubleshooting.md
  - site/src/content/docs/reference/configuration.md
  - site/src/content/docs/reference/troubleshooting.md
  - site/src/content/docs/zh-cn/reference/configuration.md
  - site/src/content/docs/zh-cn/reference/troubleshooting.md
  - src/adapters/mod.rs
  - src/adapters/responses/codex_continuation.rs
  - src/adapters/responses/codex_ws.rs
  - src/adapters/responses/http.rs
  - src/adapters/responses/mod.rs
  - src/adapters/responses/websocket.rs
  - src/adapters/responses/ws_stream.rs
  - src/auth/mod.rs
  - src/concurrency.rs
  - src/model/responses.rs
  - src/model/responses_request.rs
  - src/proxy/failover.rs
  - src/retry.rs
  - tests/codex_multi_account.rs
  - tests/codex_websocket_fallback.rs
  - tests/failover.rs
  - tests/inbound_codex_endpoint.rs
  - tests/inbound_codex_websocket.rs
  - tests/passthrough.rs
  - tests/responses_translate.rs
  - tests/retry.rs
covered_digest: "v1:sha256:7a3723a3fb67e6863622b7ef64ea12ef3158ed05668dadb61813472c96539c2d"
behavior_unverified: 0
overrides_applied: 0
decision_coverage:
  honored: 16
  total: 16
  not_honored: []
human_verification: []
---

# Phase 9: Provider Conformance Foundation Verification Report

**Phase Goal:** Operators can trust that existing providers and every later compatibility slice share explicit, testable safety and preservation guarantees.
**Verified:** 2026-09-06T22:43:33Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

The four ROADMAP success criteria were merged with clearly restating PLAN truths. Four additional plan-specific contracts remain independently scored.

| # | Truth | Status | Evidence |
|---|---|---|---|
| 1 | Existing ChatGPT/Codex and generic-Anthropic Vercel paths retain HTTP, SSE, WebSocket v2, auth, account, compaction, collaboration, error, cancellation, and terminal behavior. | ✓ VERIFIED | The full all-features workspace suite passes. Real-gateway coverage includes `tests/inbound_codex_endpoint.rs` (40 tests), `tests/inbound_codex_websocket.rs` (12), `tests/codex_websocket_fallback.rs` (9), `tests/codex_multi_account.rs` (21), and `tests/passthrough.rs` (32, including six named Vercel cases). |
| 2 | Exercised streams have explicit size/time/state bounds, at most one authoritative terminal, and bounded cancellation/resource release. | ✓ VERIFIED | `src/adapters/responses/http.rs:27-29` defines event/feed/non-streaming caps; `src/model/responses.rs:27-39,351-376,1010-1069` enforces one fail-closed terminal and 32 MiB retained state; `src/adapters/responses/codex_ws.rs:93-118` bounds queues/connections and terminal types; focused cancellation tests in `src/concurrency.rs`, `src/adapters/mod.rs`, and `tests/codex_multi_account.rs` pass under one/two-second timeouts. |
| 3 | Redispatch occurs only before client-visible output or replay-unsafe tool activity, while a safe recovery preserves paired tools and authentic continuation metadata exactly once. | ✓ VERIFIED | `Commitment` is monotonic at `src/retry.rs:102-130`; production WebSocket fallback/recovery consults it at `src/adapters/responses/websocket.rs:144-224`. Exact-hit gateway tests `replay_commitment_tool_before_text_does_not_fallback_to_http` and `continuation_recovery_preserves_tool_pair_and_opaque_state_once` pass. HTTP retry is structurally pre-body (`src/retry.rs:195-221`), while route failover's post-acceptance cases are stopped by typed adapter outcomes and exact zero-hit traps in `tests/failover.rs:753-975`. |
| 4 | Credentials are provider/destination bound, inbound slots are stripped, diagnostics are redacted, response-owned resources end with the response, and fixtures contain no secrets. | ✓ VERIFIED | Manual `Credential` debug at `src/auth/mod.rs:67-85` exposes structure only; its all-variant redaction test passes. `route_selected_credentials_are_rebound_per_destination` and the Vercel bearer/x-api-key tests assert exact selected headers and absence of inbound slots. Fixtures construct synthetic split markers; no credential store/writeback code changed. |
| 5 | The shared commitment vocabulary does not broaden existing retry or failover behavior. | ✓ VERIFIED | Generic creation POST retry remains limited to pre-response transport failures and never retries statuses (`tests/retry.rs`, seven passing tests). Review fix `03c343e` removed vacuous generic commitment plumbing and retained the shared predicate only at production-reachable WebSocket fallback/recovery seams. |
| 6 | Malformed input, provider failure, duplicate/post-terminal input, and premature EOF fail closed without synthesized success. | ✓ VERIFIED | `AnthropicSseMachine::apply_checked/finish_checked` and `SseParser::finish` reject these states. Named duplicate-terminal, post-terminal, bare-EOF, malformed-WebSocket, unterminated-SSE, provider-failure, and `response.done` regressions pass. HTTP streaming stops immediately after provider failure at `src/adapters/responses/http.rs:165-179`. |
| 7 | Gateway-owned Codex ingress errors remain Responses-shaped while other ingress errors remain Anthropic-shaped. | ✓ VERIFIED | `responses_terminal_gateway_owned_401_body_is_openai_shaped`, `responses_terminal_gateway_owned_502_body_is_openai_shaped`, and the concurrency path-shape matrix pass; ordinary `/v1/messages` overload remains Anthropic-shaped. |
| 8 | Continuation is retained only after genuine completion, within aggregate limits, and with authentic tool/reasoning identities; invalid state is rejected rather than synthesized. | ✓ VERIFIED | `src/adapters/responses/codex_ws.rs:930-1035,1124-1223` clears stale state, captures only reusable success, and atomically discards overflow. `src/adapters/responses/codex_continuation.rs:58-88,172-276` bounds and validates the full transcript. Production uses the fallible translator at `src/adapters/responses/mod.rs:140-155`; authentic-identity and exact/plus-one bound tests pass. |

**Score:** 8/8 truths verified (0 present-but-behavior-unverified)

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `src/retry.rs` | Monotonic replay commitment and pre-body retry boundary | ✓ VERIFIED | Substantive, unit-tested, and consumed by the Responses WebSocket transport. |
| `tests/codex_websocket_fallback.rs` | Real-gateway commitment and recovery matrix | ✓ VERIFIED | Nine active behavioral tests; exact dispatch/tool/metadata assertions. |
| `tests/failover.rs` | HTTP failover, post-acceptance stop, and credential rebinding | ✓ VERIFIED | Twenty-two active behavioral tests; zero-hit trap upstreams prove no redispatch. |
| `src/model/responses.rs` | Strict bounded terminal state machine | ✓ VERIFIED | Checked parser API is used by HTTP and defensive WebSocket relays. |
| `src/adapters/responses/http.rs` | Bounded incremental SSE and non-streaming collection | ✓ VERIFIED | Explicit caps, fail-closed residual handling, no content accumulation in streaming mode. |
| `src/adapters/responses/ws_stream.rs` | HTTP-equivalent terminal handling for WebSocket relay | ✓ VERIFIED | Uses the same checked machine finish/finalization path. |
| `docs/m1-responses-translation.md` | Accurate fail-closed terminal contract | ✓ VERIFIED | Genuine provider terminal plus clean transport close is required. |
| `src/adapters/responses/codex_continuation.rs` | Bounded authentic continuation construction | ✓ VERIFIED | Checked item/byte/metadata limits and identity validation precede cloning/storage. |
| `src/adapters/responses/codex_ws.rs` | Atomic continuation capture and discard | ✓ VERIFIED | Wired into the live connection reader and pool lifecycle. |
| `src/model/responses_request.rs` | Strict request-history identity validation | ✓ VERIFIED | Fallible production path rejects missing IDs and malformed tool inputs. |
| `tests/passthrough.rs` | Generic Vercel conformance and cancellation proof | ✓ VERIFIED | Six named Vercel cases pass, including incremental SSE and response drop. |
| `src/auth/mod.rs` | Secret-safe credential diagnostics | ✓ VERIFIED | Manual formatter covers all credential variants; no secret fields are formatted. |
| `src/adapters/mod.rs` + `src/concurrency.rs` | Response-lifetime account/global permit ownership | ✓ VERIFIED | RAII body wrappers are exercised through partial-stream drop tests. |

All 13 PLAN artifact declarations pass `query verify.artifacts` (13/13).

### Key Link Verification

| From | To | Via | Status | Details |
|---|---|---|---|---|
| `src/adapters/responses/websocket.rs` | `src/retry.rs` | Shared commitment predicate | ✓ WIRED | Both first-event fallback and previous-response recovery call `may_redispatch`. |
| `src/proxy/failover.rs` | `src/retry.rs` | Redispatch safety boundary | ✓ WIRED (narrowed) | The literal vacuous commitment parameter was intentionally removed by reviewed fix `03c343e`; generic route advancement remains pre-handoff or is stopped by non-advance typed adapter failures. Real zero-hit regressions prove the intended boundary. |
| `src/retry.rs` | `tests/retry.rs` | Same-provider pre-response retry | ✓ WIRED (narrowed) | Creation POST status retry is forbidden and transport retry cannot observe a response body; focused and full suites pass. |
| `src/adapters/responses/http.rs` | `src/model/responses.rs` | Bounded parser into checked machine | ✓ WIRED | Every complete event uses `apply_checked`; EOF uses `finish_checked`. |
| `src/adapters/responses/ws_stream.rs` | `src/model/responses.rs` | Shared terminal finalization | ✓ WIRED | Bare close and terminal behavior use checked finalization. |
| `tests/inbound_codex_endpoint.rs` | gateway error mapping | Codex/OpenAI shape assertions | ✓ WIRED | Two gateway-owned terminal error cases plus full endpoint suite pass. |
| `src/adapters/responses/websocket.rs` | `src/adapters/responses/codex_continuation.rs` | One-shot full-input recovery | ✓ WIRED | `previous_response_missing` starts one fresh turn with continuation disabled; real gateway asserts exact frames. The static query missed this because the implementation names differ from its textual pattern. |
| `src/adapters/responses/codex_ws.rs` | `src/adapters/responses/codex_continuation.rs` | Clean-terminal bounded storage | ✓ WIRED | `ContinuationCapture::into_stored` calls `build_transcript_with_limits` only for reusable success terminals. |
| `tests/passthrough.rs` | `src/adapters/anthropic/mod.rs` | Ordinary Anthropic configuration | ✓ WIRED | Exact body/path/auth and incremental SSE are observed through the real router. |
| `src/adapters/mod.rs` | `src/concurrency.rs` | Lazy response-owned admission/permit RAII | ✓ WIRED | Pending body retains capacity; EOF/error/drop releases it within the asserted timeout. |

### Data-Flow Trace (Level 4)

This foundation phase renders no UI or dynamic page data. The relevant transport data flows were traced instead:

| Artifact | Data | Source | Produces real data | Status |
|---|---|---|---|---|
| Responses HTTP/WS relays | Provider events | `reqwest::Response::bytes_stream` / bounded WebSocket channel | Yes; checked event machine emits downstream frames incrementally | ✓ FLOWING |
| Continuation store | Provider response ID, output items, turn state | Genuine reusable terminal on the live Codex connection | Yes; validation and aggregate accounting precede pool storage | ✓ FLOWING |
| Credential headers | Route-selected credential | `resolve_credential` / per-route failover resolution | Yes; inbound credential slots are stripped before injection | ✓ FLOWING |
| Admission/global capacity | RAII guards | Request admission and concurrency middleware | Yes; guard ownership moves into the lazy response body | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| Monotonic commitment | `cargo test --all-features retry::tests::replay_commitment_transitions_are_monotonic -- --exact` | 1 passed | ✓ PASS |
| Exact one-shot continuation recovery | `cargo test --all-features continuation_recovery_preserves_tool_pair_and_opaque_state_once -- --exact` | 1 passed | ✓ PASS |
| Tool activity closes WebSocket fallback | `cargo test --all-features replay_commitment_tool_before_text_does_not_fallback_to_http -- --exact` | 1 passed | ✓ PASS |
| Strict terminal and bounds cases | Named `responses_terminal_*`, `responses_bounds_*`, and `authentic_tool_identity_*` tests | All selected cases passed | ✓ PASS |
| Vercel conformance | `cargo test --all-features --test passthrough vercel_anthropic` | 6 passed | ✓ PASS |
| Credential rebinding/redaction | Named failover integration plus credential unit test | Both passed | ✓ PASS |
| Response-drop cleanup | Named global, account, and upstream cancellation tests | All passed within bounded timeouts | ✓ PASS |
| Final Rust quality gate | `cargo fmt --all --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features --workspace` | All passed; two pre-existing ignored tests only | ✓ PASS |

The documentation site dependencies are not installed in this worktree, so Astro typecheck/build was not runnable without an out-of-scope dependency installation. All eight changed English/ko/ja/zh-cn Markdown pages were inspected and consistently describe a fail-closed stream error with no clean completion.

### Probe Execution

No phase plan or summary declares a probe, and no conventional `scripts/*/tests/probe-*.sh` exists. Probe execution is not applicable.

### Requirements Coverage

| Requirement | Source Plan | Status | Evidence |
|---|---|---|---|
| PRES-01 | 09-02 | ✓ SATISFIED | Full native HTTP/SSE/inbound+outbound WS, compression, continuation, cancellation, and terminal suites pass. |
| PRES-02 | 09-03 | ✓ SATISFIED | Account selection, quota, compaction, collaboration, and continuation regressions pass without durable history changes. |
| PRES-03 | 09-01 | ✓ SATISFIED | Commitment and exact-hit no-redispatch tests pass for output/tool/post-acceptance paths. |
| PRES-04 | 09-02 | ✓ SATISFIED | Codex gateway-owned errors are Responses-shaped; other ingress remains Anthropic-shaped. |
| PRES-05 | 09-04 | ✓ SATISFIED | Six generic-Anthropic Vercel cases pass; no adapter or preset was added. |
| SAFE-01 | 09-02 | ✓ SATISFIED | Request/decompression pre-existing bounds remain green; new event, residual, feed, translated state, and continuation limits have exact/plus-one coverage. |
| SAFE-02 | 09-02 | ✓ SATISFIED | Streaming disables content accumulation and emits the first SSE chunk before upstream terminal; only explicit non-streaming collection is capped/buffered. |
| SAFE-03 | 09-02 | ✓ SATISFIED | Checked terminal state machine and HTTP/WS regressions prove one terminal and no clean completion after failure/cut. |
| SAFE-04 | 09-01 | ✓ SATISFIED | One monotonic predicate governs semantic WebSocket redispatch; generic retry/failover are proven structurally pre-response or non-advancing after acceptance. |
| SAFE-05 | 09-04 | ✓ SATISFIED | Redacted diagnostics, exact credential injection, inbound stripping, destination rebinding, and synthetic-only fixtures are tested. |
| SAFE-06 | 09-03 | ✓ SATISFIED | Paired tool/result IDs and opaque metadata survive one safe recovery; missing/malformed identities fail closed and overflow discards atomically. |
| SAFE-07 | 09-04 | ✓ SATISFIED | Downstream drop cancels upstream work and releases global permits, account admission, parser/continuation state, and response-owned credentials within bounded tests. |

No Phase 9 requirement is orphaned: all 12 are declared across the four plans.

### Decision Coverage

All 16 trackable `09-CONTEXT.md` decisions are honored by shipped artifacts (`check.decision-coverage-verify`: 16/16, non-blocking gate clean).

### Test Quality Audit

| Test group | Linked requirements | Active coverage | Disabled | Circular | Strongest assertion | Verdict |
|---|---|---|---|---|---|---|
| Responses translation/terminal/bounds | PRES-01, PRES-04, SAFE-01..03, SAFE-06 | 99 integration tests plus unit transport tests | 0 | 0 | Behavioral/value | ✓ PROVES |
| Retry/failover/WebSocket recovery | PRES-03, SAFE-04, SAFE-06 | 7 + 22 + 9 integration tests | 0 | 0 | Behavioral exact-hit/state transition | ✓ PROVES |
| Vercel/auth/cancellation | PRES-05, SAFE-05, SAFE-07 | 6 named Vercel tests plus failover/account/concurrency unit and integration tests | 0 | 0 | Behavioral/value with bounded timeouts | ✓ PROVES |
| Existing native/account/compaction/collaboration sweep | PRES-01, PRES-02 | Endpoint, inbound WS, multi-account, and translation suites | 0 phase-linked disabled tests | 0 | Behavioral/value | ✓ PROVES |

Expected values are explicit protocol fixtures and independent mock-upstream observations, not generated by the system under test. No linked test is skipped or circular.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|---|---|---|---|---|
| `src/retry.rs` | 34, 175 | `TODO(#126, cursor)` | ℹ️ Info | Formally tracked pre-existing Cursor idempotency work; not introduced as untracked Phase 9 debt. |
| `src/auth/mod.rs` | 26 | `TODO(M2)` | ℹ️ Info | Pre-existing milestone note outside the changed diagnostic implementation. |

No `TBD`, `FIXME`, `XXX`, disabled phase-linked test, placeholder implementation, secret fixture, or untracked completion blocker was found.

### Scope and Prohibition Checks

- Google AI Studio Web remains excluded. No AI Studio/MakerSuite provider, adapter, auth flow, preset, or documentation was added; the only `GoogleOauth` diff is the safe formatter arm for an already-existing credential variant.
- No credential discovery, persistence, refresh, or writeback behavior changed. The only production auth change is `Credential`'s manual redacted `Debug` implementation.
- No public configuration key, provider preset, dependency, generated wiki file, or Phase 10 implementation changed.
- The reviewed narrowing of generic retry/failover commitment plumbing changes no documented provider semantics; it removes unreachable state while preserving the stronger pre-response and typed post-acceptance boundaries.

### Human Verification Required

N/A — infrastructure/foundation phase with no user-facing elements. Every state transition, cancellation/cleanup invariant, ordering guarantee, and protocol behavior in the merged must-haves has a passing behavioral test.

### Disconfirmation Pass

- **Potential partial requirement checked:** SAFE-04 could have been only symbolic if commitment remained in tests; production use was confirmed at WebSocket fallback and continuation recovery, while generic retry/failover were separately proven at their actual structural boundary.
- **Potential misleading test checked:** static `query verify.key-links` missed the continuation link because it searched plan wording; manual trace plus the real-gateway recovery test proves the link. Its generic retry/failover matches were also manually checked rather than trusted, revealing the intentional review narrowing.
- **Potential uncovered error path checked:** malformed Codex WebSocket frames, unterminated SSE terminals, provider failure followed by pending/EOF, duplicate terminals, and response-ID aggregate overflow all have passing review-added regressions.

### Gaps Summary

No blocking gaps, behavior-unverified truths, or human-verification items remain. Phase 9 achieves its provider-conformance foundation goal and is ready for transition. The absent local site dependency install is recorded as a verification-environment limitation, not an implementation gap; Rust code gates and Markdown semantic review are clean.

---

_Verified: 2026-09-06T22:43:33Z_
_Verifier: Codex (gsd-verifier)_
