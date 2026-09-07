---
phase: 10-gemini-semantic-hardening
verified: 2026-09-07T01:44:25Z
status: gaps_found
score: 9/14 must-haves verified
covered_files:
  - .planning/REQUIREMENTS.md
  - .planning/ROADMAP.md
  - .planning/phases/10-gemini-semantic-hardening/10-01-PLAN.md
  - .planning/phases/10-gemini-semantic-hardening/10-01-SUMMARY.md
  - .planning/phases/10-gemini-semantic-hardening/10-02-PLAN.md
  - .planning/phases/10-gemini-semantic-hardening/10-02-SUMMARY.md
  - .planning/phases/10-gemini-semantic-hardening/10-03-PLAN.md
  - .planning/phases/10-gemini-semantic-hardening/10-03-SUMMARY.md
  - .planning/phases/10-gemini-semantic-hardening/10-04-PLAN.md
  - .planning/phases/10-gemini-semantic-hardening/10-04-SUMMARY.md
  - .planning/phases/10-gemini-semantic-hardening/10-05-PLAN.md
  - .planning/phases/10-gemini-semantic-hardening/10-05-SUMMARY.md
  - .planning/phases/10-gemini-semantic-hardening/10-CONTEXT.md
  - .planning/phases/10-gemini-semantic-hardening/10-RESEARCH.md
  - .planning/phases/10-gemini-semantic-hardening/10-REVIEW-FIX.md
  - .planning/phases/10-gemini-semantic-hardening/10-REVIEW.md
  - .planning/phases/10-gemini-semantic-hardening/10-SOURCE-AUDIT.md
  - .planning/phases/10-gemini-semantic-hardening/10-VALIDATION.md
  - docs/upstreams-failover.md
  - docs/v2-gemini-semantic-hardening.md
  - scripts/check_phase10_scope.sh
  - site/src/content/docs/ja/reference/configuration.md
  - site/src/content/docs/ja/reference/troubleshooting.md
  - site/src/content/docs/ko/reference/configuration.md
  - site/src/content/docs/ko/reference/troubleshooting.md
  - site/src/content/docs/reference/configuration.md
  - site/src/content/docs/reference/troubleshooting.md
  - site/src/content/docs/zh-cn/reference/configuration.md
  - site/src/content/docs/zh-cn/reference/troubleshooting.md
  - src/adapters/gemini/mod.rs
  - src/adapters/gemini/sse.rs
  - src/model/gemini.rs
  - src/model/gemini_request.rs
  - src/model/gemini_request/tests.rs
  - tests/gemini_conformance.rs
  - tests/gemini_translate.rs
covered_digest: "v1:sha256:611bce60b256cfdd8e53a8b2e51abee577d21aac01f456bead1bc7dd85e737c4"
behavior_unverified: 1
overrides_applied: 0
decision_coverage:
  honored: 16
  total: 16
  not_honored: []
gaps:
  - truth: "Only authentic, exactly paired tool identities and thought signatures survive a call/result round trip."
    status: failed
    reason: "Parallel tool results are consumed positionally before their tool_use_id is matched, so an unambiguous reversed result order is rejected instead of pairing by identity."
    artifacts:
      - path: src/model/gemini_request.rs
        issue: "Lines 282-293 pop the first outstanding call and reject a different next ID; the ID is sufficient to select the matching call."
      - path: tests/gemini_translate.rs
        issue: "Parallel round-trip coverage returns results only in call order; the existing negative table treats reversed results as invalid."
    missing:
      - "Pair each adjacent result batch by unique tool_use_id while preserving the required Gemini call order on emission."
      - "Add reversed parallel-result coverage plus duplicate, missing, and foreign ID cases."
  - truth: "The same captured transcript produces equivalent ordered content/tool/error meaning without silently discarding an ambiguous supported shape."
    status: failed
    reason: "Known Gemini Part kinds other than text/functionCall/functionResponse/thoughtSignature fall through to Metadata, and can also coexist with functionCall, causing silent semantic loss."
    artifacts:
      - path: src/model/gemini.rs
        issue: "Lines 407-414 check only text+functionCall, while lines 503-508 accept every remaining object as metadata."
      - path: tests/gemini_translate.rs
        issue: "Strictness fixtures do not cover inlineData, executableCode, codeExecutionResult, fileData, or conflicts between those known variants and functionCall."
    missing:
      - "Whitelist explicitly evidenced metadata-only keys and reject known unsupported semantic Part variants or incompatible combinations atomically."
      - "Add stream and unary fixtures proving no known Part semantic is silently discarded."
  - truth: "Malformed, oversized, invalid-UTF-8, prematurely terminated, duplicate-terminal, or post-terminal events fail without synthesized success."
    status: failed
    reason: "After [DONE], comment-only and empty-data frames parse to None and are accepted; clean EOF then releases the deferred success terminal despite bytes after the terminal boundary."
    artifacts:
      - path: src/adapters/gemini/sse.rs
        issue: "Lines 64-76 reject post-DONE JSON and duplicate DONE only, not other completed frames."
      - path: tests/gemini_conformance.rs
        issue: "Lines 663-685 cover post-DONE JSON and duplicate DONE but omit comment and empty-data frames."
    missing:
      - "Dispose or fail the decoder on every completed frame after [DONE], allowing only the explicitly chosen raw-whitespace tail policy."
      - "Add split/coalesced comment-only and empty-data post-DONE regression cases."
  - truth: "A Gemini turn keeps the selected Google OAuth bearer and project in its Code Assist envelope for its full lifetime."
    status: partial
    reason: "The implementation is structurally request-local, but the promised real-gateway identity evidence exercises API-key /v1beta only, not Google OAuth /v1internal Code Assist."
    artifacts:
      - path: tests/gemini_conformance.rs
        issue: "Lines 50-70 force AuthMode::ApiKey and every conformance request uses the v1beta model endpoint."
      - path: docs/v2-gemini-semantic-hardening.md
        issue: "Lines 116-123 claim loopback Code Assist identity/envelope coverage that the suite does not provide."
    missing:
      - "Add a hermetic GoogleOauth credential seam/fixture that captures /v1internal unary and streaming requests."
      - "Assert the same synthetic bearer, project, endpoint, and byte-equivalent envelope through a permitted pre-header retry and cancellation, with no credential writeback."
behavior_unverified_items:
  - truth: "A Gemini turn keeps the selected Google OAuth bearer and project in its Code Assist envelope for its full lifetime."
    test: "Drive a hermetic GoogleOauth Code Assist turn through /v1internal in unary, streaming, pre-header retry, and cancellation scenarios."
    expected: "Every attempt uses one selected synthetic bearer and project-bearing envelope; cancellation does not redispatch or retain capacity."
    why_human: "Production wiring is present, but current behavioral tests replace Google OAuth with API-key authentication and never exercise the Code Assist endpoint."
deferred:
  - truth: "Comprehensive Antigravity response compatibility after changes to the shared Gemini adapter/state machine."
    addressed_in: "Phase 11"
    evidence: "Phase 11 owns Antigravity SSE consumption, semantic unwrapping, identity affinity, and retry behavior (ANT-01 through ANT-08)."
---

# Phase 10: Gemini Semantic Hardening Verification Report

**Phase Goal:** Gemini users receive equivalent, strict Google Code Assist semantics in streaming and non-streaming modes without unsafe replay.
**Verified:** 2026-09-07T01:44:25Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|---|---|---|
| 1 | One selected Google OAuth identity and project remain paired for the Code Assist turn lifetime. | PRESENT_BEHAVIOR_UNVERIFIED | Credential resolution and envelope construction occur once before the retry closure (`src/adapters/gemini/mod.rs:207-355`), but gateway tests force API-key auth (`tests/gemini_conformance.rs:50-70`). |
| 2 | Stream and unary clients receive equivalent ordered text, reasoning, tools, usage, finish, and provider-error meaning. | FAILED | Tested supported fixtures agree, but known unsupported Part variants are silently converted to metadata/no-op by `src/model/gemini.rs:407-508`. |
| 3 | Malformed, oversized, invalid-UTF-8, or prematurely terminated events fail without silent loss or synthesized success. | FAILED | Bounds/UTF-8/EOF cases pass, but a comment or empty-data frame after `[DONE]` is accepted by `src/adapters/gemini/sse.rs:64-76`. |
| 4 | Generation retries only on proven replay-safe pre-header failure and never after output or tool activity. | VERIFIED | `RetrySafety::NonIdempotentPost` is wired at `src/adapters/gemini/mod.rs:331-357`; exact-hit status/body tests pass. |
| 5 | Authentic tool identities and signatures round-trip with exact call/result pairing. | FAILED | Signature preservation works, but `src/model/gemini_request.rs:282-293` pairs parallel results by position instead of unique ID. |
| 6 | SSE framing preserves arbitrary byte/UTF-8 splits, CRLF, comments, and multiline data in order. | VERIFIED | Six decoder tests and eight real-gateway framing tests pass. |
| 7 | Streaming stays lazy; only explicit unary mode uses bounded full-body collection. | VERIFIED | `bytes_stream` remains body-owned; unary/error collection is capped at 32 MiB; exact/cap-plus-one tests pass. |
| 8 | Client-visible output or replay-unsafe tool activity closes redispatch monotonically. | VERIFIED | Body failures, embedded errors, tool paths, and returned statuses each produce exactly one upstream hit. |
| 9 | Dropping a downstream response releases upstream work and gateway capacity promptly. | VERIFIED | `gemini_response_drop_releases_upstream_and_gateway_capacity` observes upstream drop and one-slot reacquisition within one second. |
| 10 | English docs describe the implemented Gemini contract accurately. | FAILED | The docs claim all ambiguous supported shapes fail and that Code Assist identity/envelopes have loopback coverage; both claims overstate current behavior/evidence. |
| 11 | Docs preserve existing Code Assist configuration and explicitly exclude writeback, Antigravity policy, and AI Studio Web. | VERIFIED | Engineering/site docs state exclusions and the phase diff does not change public config/provider declarations. |
| 12 | English, Korean, Japanese, and Simplified Chinese site pages carry equivalent Gemini semantics. | VERIFIED | All eight configuration/troubleshooting pages contain the same stable contract marker and corresponding localized prose. |
| 13 | Focused and repository-required release gates pass. | VERIFIED | Scope audit, focused Gemini/retry/failover suites, formatting, warnings-denied Clippy, and the full serial all-feature workspace suite all exited 0. |
| 14 | The phase adds no public config/provider expansion, credential writeback, AI Studio Web implementation, dependency, secret, or wiki change. | VERIFIED | `scripts/check_phase10_scope.sh` passes for `d39b64c..HEAD`; direct path diff checks agree. |

**Score:** 9/14 truths verified (1 present, behavior-unverified)

### Deferred Items

| # | Item | Addressed In | Evidence |
|---|---|---|---|
| 1 | Full Antigravity response regression coverage for the shared Gemini semantic/transport path | Phase 11 | ANT-01 through ANT-08 explicitly own Antigravity identity, SSE, translation, terminal, and retry behavior. |

### Required Artifacts

All 16 declared artifacts exist and pass GSD's substantive checks. Four are behaviorally incomplete:

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `src/model/gemini.rs` | Checked request-local semantic state | PARTIAL | Substantive/wired; silently accepts known unsupported Part variants. |
| `src/model/gemini_request.rs` | Exact call/result reconstruction | PARTIAL | Substantive/wired; rejects unambiguous reversed parallel result order. |
| `src/adapters/gemini/sse.rs` | Bounded incremental byte SSE decoder | PARTIAL | Substantive/wired; post-DONE no-op frames escape the terminal check. |
| `tests/gemini_conformance.rs` | Framing, identity, retry, parity, cancellation evidence | PARTIAL | Strong API-key gateway coverage; no Google OAuth `/v1internal` Code Assist case. |
| `src/adapters/gemini/mod.rs` | Lazy bounded transport and immutable dispatch | VERIFIED | Streaming/unary paths consume the checked machine and explicit retry safety. |
| Documentation and locale artifacts | Accurate synchronized contract | PARTIAL | Locale parity/exclusions are sound, but English evidence claims inherit the implementation gaps. |
| `scripts/check_phase10_scope.sh` | Deterministic exclusion audit | VERIFIED | Passed independently; pins Antigravity retry behavior and forbidden paths/content. |

### Key Link Verification

| From | To | Via | Status | Details |
|---|---|---|---|---|
| Gemini SSE decoder | semantic state machine | decoded JSON in wire order | VERIFIED | Production adapter calls `process_chunk_checked`. |
| Gemini adapter | retry policy | `NonIdempotentPost` | VERIFIED | Explicitly wired for API-key and Google OAuth, with Antigravity left idempotent for Phase 11. |
| Gemini adapter | conformance tests | gateway capture/hit counts | PARTIAL | API-key path is covered; promised Code Assist OAuth identity/envelope link is absent. |
| Gemini request translator | semantic tool IDs | opaque signature/call-result round trip | PARTIAL | Authentic signature link works; positional result matching breaks valid parallel reordering. |
| Engineering docs | conformance suites | named executable evidence | PARTIAL | Files are named, but claimed Code Assist coverage is absent. |
| English docs | maintained locales | stable contract marker | VERIFIED | All eight affected pages contain the exact marker. |

### Data-Flow Trace (Level 4)

| Artifact | Data | Source | Produces Real Data | Status |
|---|---|---|---|---|
| `src/adapters/gemini/mod.rs` | bearer/project/payload | credential resolver + translated inbound request | Yes | FLOWING structurally; OAuth behavior untested |
| `src/adapters/gemini/sse.rs` | decoded provider events | upstream `reqwest::bytes_stream` | Yes | FLOWING, with post-DONE no-op gap |
| `src/model/gemini.rs` | Anthropic events/final JSON | decoded Gemini response values | Yes | FLOWING, with unsupported-Part loss gap |
| Locale docs | Gemini contract prose | English reference pages + stable marker | Yes | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| Scope exclusions | `bash scripts/check_phase10_scope.sh` | pass | PASS |
| Pure Gemini semantics | `cargo test --all-features --test gemini_translate` | 26 passed | PASS |
| Real gateway Gemini paths | `cargo test --all-features --test gemini_conformance` | 22 passed | PASS |
| Signature round trip | `cargo test --all-features gemini_tool_signature_roundtrip` | 4 passed across unit/integration targets | PASS |
| SSE bounds | `cargo test --all-features gemini_sse_bounds` | 6 passed | PASS |
| Retry/failover regressions | `cargo test --all-features --test retry --test failover` | 29 passed | PASS |
| Repository gates | `cargo fmt --all --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features --workspace -- --test-threads=1` | exit 0; one pre-existing ignored measurement test | PASS |

### Probe Execution

No conventional `probe-*.sh` or phase-declared probe exists. The phase-declared deterministic scope audit was run directly and passed.

### Requirements Coverage

| Requirement | Source Plans | Status | Evidence |
|---|---|---|---|
| GEM-01 | 10-03, 10-04, 10-05 | PARTIAL | Production capture is structurally immutable, but no OAuth Code Assist gateway test exercises bearer/project lifetime. |
| GEM-02 | 10-01, 10-02, 10-04, 10-05 | BLOCKED | Ordered supported content passes, but positional parallel-result matching rejects valid ID-addressable results. |
| GEM-03 | 10-01, 10-02, 10-03, 10-04, 10-05 | BLOCKED | Fixture parity passes; unsupported known Part semantics can be silently discarded. |
| GEM-04 | 10-01, 10-02, 10-04, 10-05 | BLOCKED | Most negative cases pass; completed no-op frames after `[DONE]` still permit success. |
| GEM-05 | 10-03, 10-04, 10-05 | SATISFIED | Explicit non-idempotent retry safety, one-hit post-header cases, and cancellation release tests pass. |

No Phase 10 requirement is orphaned from the plans.

### Decision Coverage

All 16 trackable CONTEXT.md decisions are honored by shipped artifacts according to `check.decision-coverage-verify`. This heuristic does not override the behavioral gaps above.

### Test Quality Audit

| Test File | Linked Requirements | Active | Skipped | Circular | Assertion Level | Verdict |
|---|---|---:|---:|---|---|---|
| `tests/gemini_translate.rs` | GEM-02/03/04 | 26 | 0 | No | Value/behavioral | PARTIAL — misses known Part variants and reversed parallel results |
| `tests/gemini_conformance.rs` | GEM-01/02/03/04/05 | 22 | 0 | No | Behavioral | PARTIAL — all gateway fixtures force API-key `/v1beta`, so GEM-01's OAuth claim is not exercised |
| `src/model/gemini_request/tests.rs` | GEM-02/04 | active unit tests | 0 | No | Value/behavioral | PASS except shared positional-result gap |
| `src/adapters/gemini/sse.rs` unit tests | GEM-02/04 | 6 focused | 0 | No | Boundary/value | PARTIAL — no post-DONE comment/empty-data case |

No disabled requirement test or circular fixture generator was found. The stale `10-VALIDATION.md` ledger still says `status: ready` and marks all 13 task rows `pending`; update it during gap closure so planning evidence matches execution.

### Anti-Patterns Found

No unreferenced `TBD`, `FIXME`, or `XXX` marker, placeholder implementation, new dependency, secret-bearing fixture, or forbidden scope path was found in the Phase 10 diff.

### Human Verification Required

The provider hardening phase has no user-facing UX requiring manual UAT. The OAuth/project lifetime invariant remains behavior-unverified, but it should be closed with a hermetic injected-credential gateway test rather than live credentials.

### Gaps Summary

Phase 10 is not ready to advance. Green suites prove the implemented supported fixtures, bounds, retry safety, and cancellation, but they miss three protocol contradictions and the core Google OAuth Code Assist identity path. Close the structured gaps, correct the overstated docs/validation ledger, then re-run verification. No credential writeback, AI Studio Web, dependency, public config, or wiki change is needed.

---

_Verified: 2026-09-07T01:44:25Z_
_Verifier: Codex (gsd-verifier)_
