---
phase: 13-generic-openai-chat-completions
reviewed: 2026-09-08T00:00:00Z
depth: standard
files_reviewed: 27
files_reviewed_list:
  - src/adapters/mod.rs
  - src/adapters/openai_chat/diagnostic_tests.rs
  - src/adapters/openai_chat/mod.rs
  - src/adapters/openai_chat/sse.rs
  - src/adapters/openai_chat/timeout_tests.rs
  - src/auth/slots/tests.rs
  - src/config.rs
  - src/model/mod.rs
  - src/model/openai_chat_request.rs
  - src/model/openai_chat_request/endpoint.rs
  - src/model/openai_chat_request/tools.rs
  - src/model/openai_chat_response.rs
  - src/model/openai_chat_response/assembly.rs
  - src/model/openai_chat_response/checked.rs
  - src/model/openai_chat_response/unary_tools.rs
  - src/model/openai_chat_response/validation.rs
  - src/proxy/capability.rs
  - src/proxy/failover.rs
  - src/routing.rs
  - tests/failover.rs
  - tests/openai_chat_conformance.rs
  - tests/openai_chat_conformance/lifetime.rs
  - tests/openai_chat_conformance/matrix.rs
  - tests/openai_chat_translate.rs
  - tests/openai_chat_translate/assembly.rs
  - tests/openai_chat_translate/caps.rs
  - tests/retry.rs
findings:
  critical: 1
  warning: 0
  info: 2
  total: 3
status: issues_found
---

# Phase 13: Code Review Report

**Reviewed:** 2026-09-08T00:00:00Z
**Depth:** standard
**Files Reviewed:** 27
**Status:** issues_found

## Summary

Standard-depth adversarial review of the generic OpenAI Chat Completions
slice: request translation, response semantic machine, bounded SSE decoder,
adapter transport, config/routing/failover wiring, and the new test suites.

Overall this is a well-hardened implementation. The deny-by-default request
whitelist fails closed with typed Anthropic errors; the response machine
correctly gates success on a provider-declared finish_reason (EOF, duplicate
[DONE], and residual frames cannot synthesize success); the SSE decoder is
byte-bounded including CRLF-split frames; ConnectOnly retry plus
failure=None on post-send errors correctly prevents both in-attempt retry and
outer failover redispatch after a response is possible, with connect-phase
errors (failure=BeforeHeaders) still allowed to advance failover; credentials
are resolved fresh per request, never logged, and redirects are refused so the
bearer cannot leave the configured origin. The 8 MiB cumulative streaming
semantic cap and the strict single trailing usage chunk are intentional and
pinned by matrix tests.

One blocker was found: the semantic machine can emit tool content-block
events before message_start, violating the Anthropic Messages SSE event
ordering for the common tool-call streaming shape. No warnings rose to the
severity bar; two info notes remain.

## Critical Issues

### CR-01: message_start emitted after tool content-block events in tool-first streams

**File:** `src/model/openai_chat_response.rs:204-239`

**Issue:** In `process_chunk_checked`, `message_start` is gated on
`!parts.is_empty() || reasoning.is_some() || finish_reason.is_some()` — but
the loop that emits tool-use `content_block_start`/`content_block_delta`/
`content_block_stop` events runs regardless of whether the machine has
started. A streaming chunk whose delta carries only `tool_calls` (with
`content: null` or no `content` key — the normal OpenAI tool-call shape) has
`parts` empty, so the stream begins with tool content-block events and
`message_start` is only emitted later, on the finish_reason chunk. Anthropic
Messages SSE clients require `message_start` to precede all content blocks,
so this produces a protocol-invalid event sequence on the most common
tool-calling stream shape. The unary path is unaffected (the whole completion
is processed in one call and its finish_reason triggers start first). The
conformance suites (including the interleave test at
`tests/openai_chat_conformance.rs:81-120` and the cancellation tools stage in
`tests/openai_chat_conformance/lifetime.rs:69`) never assert the relative
order of `message_start` and tool block events, so the gap is unpinned.

**Fix:**

```rust
// In process_chunk_checked, before emitting any content-block events, include
// tool-delta presence in the started condition (or emit message_start ahead of
// the tool loop unconditionally when any payload exists):
let has_tool_deltas = !checked.tools.is_empty() || tool_bytes > 0;
if !self.started
    && (!checked.parts.is_empty()
        || checked.reasoning.is_some()
        || checked.finish_reason.is_some()
        || has_tool_deltas)
{
    self.started = true;
    events.push(self.message_start_event());
}
```

Then add a conformance assertion that a tool-first stream begins with
`message_start` before any `content_block_start`.

## Info

### IN-01: Operator count_tokens config silently overridden for openai_chat providers

**File:** `src/proxy/failover.rs:285-297`

**Issue:** `count_tokens` for OpenAI Chat routes always uses
`CountTokens::Estimate`, ignoring any operator-configured value for that
provider. The rationale (no Chat-calibrated tiktoken counter) is sound, but
the override is silent at runtime; an operator who set
`count_tokens = "tiktoken"` gets different behavior than configured with no
boot-time signal.

**Fix:** Prefer a boot-time validation rejecting a non-default `count_tokens`
on `kind = "openai_chat"` providers (mirroring `OpenAiChatRequiresApiKey`),
or document the per-adapter override wherever `count_tokens` is documented.

### IN-02: Chat adapter hardcodes a 120s upstream read timeout

**File:** `src/adapters/openai_chat/mod.rs:80-91`

**Issue:** `chat_client()` fixes `read_timeout(120s)` while every other
Chat timeout (`upstream_ttfb_ms`, SSE keepalive) is config-driven. Long
provider stalls between stream frames are bounded by an invisible constant
that operators cannot tune, and the value can drift from sibling adapters.

**Fix:** Source the idle read timeout from `server.timeouts` (a new key or an
existing one), falling back to 120s, and thread it through the once-initialized
client (or build the client per resolved timeout).

---

_Reviewed: 2026-09-08T00:00:00Z_
_Reviewer: gsd-code-reviewer (standard depth)_
_Depth: standard_

## Root adjudication after independent review

CR-01 is not reproduced. `checked.rs` returns an empty `tools` vector for delta
payloads; `process_chunk_checked` only fills `streamed_tools` when a finish reason
is present. That same finish reason triggers `message_start` before the tool loop.
The proposed immediate-start change is therefore unnecessary. Root added explicit
first-event and exactly-one-message-start assertions to the tool-first interleaved
real-router fixture `openai_chat_assembly_interleave_at_limit_multibyte`; it passed
against unchanged production code (1 passed, 46 filtered). This is new coverage,
not a RED/GREEN production fix. Original reviewer findings above are preserved.

IN-01 and IN-02 are documented intentional limitations: Chat always estimates
token counts, and its read-idle bound is fixed at 120 seconds. The engineering
note and all four provider pages now say so. No new public key, boot rejection,
or timeout semantics were introduced as a result of these informational notes.

Disposition: no reproduced blocking source defect; final documentation and
post-assertion gates still need to complete before phase closure.
