---
phase: 09-provider-conformance-foundation
plan: "02"
subsystem: responses-translation
tags: [rust, sse, websocket, bounds, terminal-state]
requires:
  - phase: 09-provider-conformance-foundation
    provides: monotonic redispatch commitment from plan 09-01
provides:
  - Trustworthy-terminal Responses translation for HTTP and defensive WebSocket relays
  - Strict bounded SSE parsing and bounded non-streaming collection
  - Bounded translated state with malformed tool-argument rejection
affects: [gemini-hardening, antigravity-hardening, cursor-hardening, openai-chat]
actuals:
  tokens: 13705
  tasks: 3
  commits: 14
plan_head_before: 0c50146196a024fb7b1651cf269024c12987fa6c
tech-stack:
  added: []
  patterns: [deferred-success-terminal, disposed-on-parser-error, checked-byte-accounting]
key-files:
  created: []
  modified:
    - src/model/responses.rs
    - src/adapters/responses/http.rs
    - src/adapters/responses/ws_stream.rs
    - tests/responses_translate.rs
    - tests/inbound_codex_endpoint.rs
    - tests/inbound_codex_websocket.rs
    - docs/m1-responses-translation.md
key-decisions:
  - "A provider success terminal is recorded immediately but its Anthropic message_stop is withheld until clean transport closure."
  - "Responses SSE is decoded from bytes with fixed 8 MiB event/residual, 256-frame feed, and 32 MiB non-streaming/translated-state limits."
patterns-established:
  - "Trustworthy terminal: EOF finalization succeeds only after one accepted provider success terminal."
  - "Disposed parser: malformed UTF-8/JSON or a breached bound clears state and rejects every later feed."
requirements-completed: [PRES-01, PRES-04, SAFE-01, SAFE-02, SAFE-03]
coverage:
  - id: D1
    description: HTTP and defensive WebSocket translation require one genuine provider terminal and reject conflicting or truncated transcripts.
    requirement: PRES-01
    verification:
      - kind: integration
        ref: cargo test --all-features responses_terminal
        status: pass
      - kind: integration
        ref: cargo test --all-features responses_transport_terminal
        status: pass
    human_judgment: false
  - id: D2
    description: Responses parser, non-streaming wire collection, and retained translation state are strictly decoded and byte-bounded.
    requirement: SAFE-01
    verification:
      - kind: unit
        ref: cargo test --all-features responses_bounds
        status: pass
    human_judgment: false
  - id: D3
    description: Native Codex ingress retains OpenAI-shaped gateway errors while translated Anthropic clients retain Anthropic-shaped outcomes.
    requirement: PRES-04
    verification:
      - kind: integration
        ref: cargo test --all-features --test inbound_codex_endpoint --test inbound_codex_websocket --test codex_websocket_fallback
        status: pass
    human_judgment: false
duration: 20min
completed: 2026-09-06
status: complete
---

# Phase 9 Plan 02: Trustworthy Responses Terminals and Bounds Summary

**Responses HTTP and WebSocket translation now rejects malformed, oversized, conflicting, or truncated transcripts without manufacturing a clean completion.**

## Performance

- **Duration:** 20 min
- **Started:** 2026-09-06T21:21:46Z
- **Completed:** 2026-09-06T21:41:10Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Deferred `message_stop` until transport closure validates one real `response.completed`, `response.done`, or `response.incomplete` event.
- Added strict incremental UTF-8/JSON SSE framing, exact boundary checks, bounded JSON collection, and bounded retained tool/content metadata.
- Preserved ordinary HTTP/SSE, inbound and outbound WebSocket v2, compression, continuation, cancellation, and ingress error behavior across the full workspace suite.

## Task Commits

1. **Task 1 RED: expose fail-open terminals** - `5ea4ef3`
2. **Task 1 GREEN: require trustworthy terminals** - `d55ccef`
3. **Task 2 RED: expose unbounded Responses state** - `1a0ed03`
4. **Task 2 GREEN: bound parser and collection** - `6e0550c`
5. **Task 3 RED: expose WebSocket parity gap** - `7a451ce`
6. **Task 3 GREEN: align WebSocket terminals and docs** - `e3859b9`
7. **Task 3 regression alignment** - `f63043a`
8. **Task 3 formatting** - `27af121`
9. **Bounds hardening follow-up** - `ed3d31e`

The measured ledger count is 14 because five non-overlapping Plan 09-04 commits landed on the shared branch during this plan's execution.

## Files Created/Modified

- `src/model/responses.rs` - Explicit terminal state, fallible transport finalization, aggregate accounting, and tool-argument validation.
- `src/adapters/responses/http.rs` - Strict bounded byte parser, bounded JSON collector, and one-error streaming failure path.
- `src/adapters/responses/ws_stream.rs` - HTTP-equivalent terminal handling for defensive WebSocket streaming and JSON collection.
- `tests/responses_translate.rs` - Terminal conflict, truncation, aggregate, and malformed-tool regressions.
- `tests/inbound_codex_endpoint.rs` - Codex gateway-owned error-shape cases included in the terminal sweep.
- `tests/inbound_codex_websocket.rs` - Inbound WebSocket malformed/early-terminal protocol regression included in the sweep.
- `docs/m1-responses-translation.md` - Correct strict terminal and internal-bound engineering contract.

## Decisions Made

- Kept the existing fixture-friendly `apply` and `final_json` entry points, while production HTTP/WebSocket relays use fallible `apply_checked`, `finish_checked`, and `final_json_checked` APIs.
- Kept unknown well-formed event compatibility, but made malformed UTF-8/JSON and known semantic data after a terminal fail closed.
- Added no config key or dependency; all limits are internal constants or test-only constructor inputs.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Counted streaming-only retained metadata against the aggregate limit**
- **Found during:** Final bounds review after Task 3
- **Issue:** Streaming disables content reconstruction, but tool arguments and some citation/web-search metadata can still be retained transiently.
- **Fix:** Applied aggregate accounting to those provider-derived values even when full content accumulation is disabled.
- **Files modified:** `src/model/responses.rs`, `tests/responses_translate.rs`
- **Verification:** `cargo test --all-features responses_bounds`
- **Committed in:** `ed3d31e`

---

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** The fix closes the planned streaming-state bound without expanding public behavior or configuration.

## Issues Encountered

- The shared no-isolation wave interleaved five Plan 09-04 commits between this plan's commits; file scopes did not overlap and the complete workspace test passed.
- Documentation review found stale synthesized-completion wording in `site/src/content/docs/reference/configuration.md` and `site/src/content/docs/reference/troubleshooting.md`. Those files are outside Plan 09-02's declared write set, so the Phase 9 orchestrator will route the required English and ko/ja/zh-cn correction after wave execution. README English/ko/ja/zh-CN claims already describe premature EOF as failure and need no edit. `wiki/` was not touched.

## User Setup Required

None - no external service configuration required.

## Verification

- `cargo test --all-features responses_terminal` - pass
- `cargo test --all-features responses_bounds` - pass
- `cargo test --all-features responses_transport_terminal` - pass
- `cargo test --all-features --test responses_translate --test inbound_codex_endpoint` - pass
- `cargo test --all-features --test inbound_codex_endpoint --test inbound_codex_websocket --test codex_websocket_fallback` - pass
- `cargo fmt --all --check` - pass
- `cargo clippy --all-targets --all-features -- -D warnings` - pass
- `cargo test --all-features --workspace` - pass (2 ignored pre-existing tests)

## Next Phase Readiness

- The strict checked terminal and bounds pattern is ready for later Gemini, Antigravity, Cursor, and OpenAI Chat work.
- Site documentation drift is explicitly routed to Phase 9 review/fix; no code blocker remains.

## Self-Check: PASSED

- All seven declared implementation/test/doc files exist.
- All nine Plan 09-02 commits listed above exist on the current branch.
- No stub, skipped test, new dependency, credential writeback, Google AI Studio Web surface, or generated wiki change was introduced.

---
*Phase: 09-provider-conformance-foundation*
*Completed: 2026-09-06*
