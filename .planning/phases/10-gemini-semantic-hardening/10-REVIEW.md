---
phase: 10-gemini-semantic-hardening
reviewed: 2026-09-07T00:52:19Z
depth: deep
files_reviewed: 18
files_reviewed_list:
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
findings:
  critical: 8
  warning: 2
  info: 0
  total: 10
status: issues_found
---

# Phase 10: Code Review Report

**Reviewed:** 2026-09-07T00:52:19Z
**Depth:** deep
**Files Reviewed:** 18
**Status:** issues_found

## Summary

The focused Gemini suites pass, but deep cross-file review found eight release-blocking correctness, protocol, and resource-bound defects. The most consequential are an SSE feed that can retain roughly 2 GiB despite its advertised 8 MiB bound, Gemini 3 signature checks keyed to the client alias instead of the resolved model, rejection of valid first-call-only parallel signatures, ambiguous tool-result replay, and an out-of-scope retry-policy change to the shared Antigravity transport. Two additional warnings affect request validation and compatibility terminal helpers.

Verification performed: `cargo test --all-features --test gemini_translate --test gemini_conformance` passed (21 + 19 tests). That result confirms the findings are coverage gaps rather than already-failing regressions.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Per-event bounds permit multi-gigabyte SSE feed amplification

**File:** `src/adapters/gemini/sse.rs:47-94` (consumed at `src/adapters/gemini/mod.rs:392-430`)
**Issue:** `Decoder::push` accepts up to 256 independently bounded 8 MiB frames and retains every parsed `serde_json::Value` in `items` until the entire reqwest chunk has been decoded. A single legal HTTP chunk containing 256 near-limit events therefore retains about 2 GiB of raw payload plus parsed JSON. The adapter then serializes all translated events into another `Vec<u8>` before yielding. The per-frame and per-feed-count checks do not provide the documented bounded-memory guarantee and can OOM the gateway.
**Fix:** Decode and emit at most one frame (or a tightly bounded aggregate byte budget) per downstream yield. Enforce a cumulative decoded/output-byte cap before retaining parsed values, and add a small-limit regression with many near-cap frames in one transport chunk.

### CR-02: Gemini 3 signature policy is selected from the client alias

**File:** `src/adapters/gemini/mod.rs:380,491` and `src/model/gemini.rs:405-419`
**Issue:** Both streaming and unary machines receive `route.model`, while request translation and upstream dispatch use `route.upstream_model`. With a normal alias such as `claude-via-gemini -> gemini-3.1-pro-preview`, a missing Gemini 3 `thoughtSignature` is accepted and replaced by a random unsigned join ID. The next request is then rejected by `translate_request_for_model`, which correctly checks the resolved upstream model. The inverse alias can spuriously impose Gemini 3 rules on a legacy upstream.
**Fix:** Give the semantic machine separate public-response and resolved-upstream model values: emit `route.model` in the Anthropic response, but select model-specific validation from `route.upstream_model`. Add real-gateway alias tests in both directions.

### CR-03: Valid Gemini 3 parallel calls are rejected

**File:** `src/model/gemini.rs:405-419`, `src/model/gemini_request.rs:209-214`, and `tests/gemini_translate.rs:205-229`
**Issue:** Every Gemini 3 `functionCall` is required to carry a signature. The established provider contract records first-call-only signature placement for parallel calls (`docs/plans/gemini-provider/TICKETS.md:73`), so a valid `[signed call A, unsigned call B]` response fails, as does faithful echoed history. The test named `parallel_calls_keep_signature_on_first_call` fabricates and asserts a different signature on both calls, masking the regression.
**Fix:** Track function-call position per model content/parallel batch, require and preserve the authentic signature on the first call, and allow later calls in that same batch to remain unsigned. Replace the misleading fixture with first-call-only response and round-trip cases; keep sequential batches independently signed.

### CR-04: Unary accumulation is quadratically expensive within accepted limits

**File:** `src/model/gemini.rs:519-539,601-618`
**Issue:** Adjacent unary thinking/text parts are merged with `format!("{}{}", s, text)`, copying the complete accumulated string for every part. A response accepted by the 32 MiB semantic and 4,096-part limits can cause roughly 64 GiB of cumulative copying, making the nominal memory bound a CPU/amplification denial-of-service vector.
**Fix:** Mutate the retained `String` with `push_str` (using an owned-string access path) or accumulate fragments and join once. Add an adversarial many-part test that verifies linear work at the accepted boundary.

### CR-05: Tool-result identities are reusable and parallel ordering is ambiguous

**File:** `src/model/gemini_request.rs:201-253`
**Issue:** Tool IDs are inserted into a global name map but are only read, never consumed. The same ID can therefore produce multiple `functionResponse` parts. For parallel same-name calls, returning results in a different ID order emits indistinguishable same-name responses in client order, losing the exact call/result association that Phase 10 and the user docs claim. The current tests cover only unknown IDs and a happy ordered pair.
**Fix:** Track outstanding calls as ordered batches, consume each ID exactly once, reject duplicates and cross-batch/out-of-order ambiguity, and render parallel responses in the original call order (or fail when the wire cannot preserve identity). Add duplicate, reordered same-name, missing-result, and cross-batch tests.

### CR-06: Phase 10 silently changes Antigravity retry policy

**File:** `src/adapters/gemini/mod.rs:332-360` (shared mapping at `src/routing.rs:20-33`; ineffective gate at `scripts/check_phase10_scope.sh:40-45`)
**Issue:** `ProviderKind::Antigravity` routes through `GeminiAdapter`, so the unconditional switch to `RetrySafety::NonIdempotentPost` changes Antigravity returned-status retry behavior too. Phase 10 explicitly excludes Antigravity policy and its docs claim Antigravity remains unchanged. The scope audit checks separate Antigravity paths but omits the actual shared adapter/model files, so it reports a false pass.
**Fix:** Select retry safety explicitly by provider/auth kind, preserving the pre-Phase-10 Antigravity behavior until Phase 11 owns and tests its policy. Expand the scope audit to inspect shared Gemini adapter/model diffs for Antigravity-reachable changes, or replace path-only enforcement with behavior tests.

### CR-07: A malformed later frame discards valid earlier frames based on chunking

**File:** `src/adapters/gemini/sse.rs:51-85` and `src/adapters/gemini/mod.rs:392-403`
**Issue:** `Decoder::push` accumulates parsed items locally but returns only `Err` if a later frame in the same reqwest chunk is malformed or over limit. The adapter then emits only the error, discarding the valid prefix. Identical wire bytes split into two transport chunks emit the valid text first and then the error. Observable semantics therefore depend on arbitrary TCP/HTTP chunk boundaries, violating the phase's split-invariance requirement.
**Fix:** Return/yield the validated prefix plus a terminal parser error, or decode one frame at a time so committed prefix events are emitted before the failure. Add one-chunk versus split-chunk equivalence tests for valid-frame-plus-invalid-frame input.

### CR-08: Late usage metadata breaks streaming/unary parity

**File:** `src/model/gemini.rs:202-213,485-500`
**Issue:** Streaming emits `message_start` as soon as the first content arrives, using the current `input_tokens`. Gemini commonly provides `usageMetadata` only on the final chunk. In that case streaming permanently reports `input_tokens: 0`; the later update is never represented because `message_delta` carries only output tokens, while unary returns the correct final input count. Existing parity coverage places content, finish, and usage in one object and misses this normal multi-chunk shape.
**Fix:** Define a protocol-correct strategy for late input usage (for example, delay the first visible event until usage is known within a bounded prelude, or emit an allowed final usage update if the Anthropic client contract supports it), then test first-content/no-usage followed by final-usage in both modes.

## Warnings

### WR-01: Content block direction is not validated against the message role

**File:** `src/model/gemini_request.rs:117-135,180-254`
**Issue:** Block translation is independent of role: user `tool_use` becomes a user `functionCall`, assistant `tool_result` becomes a model `functionResponse`, and an unknown role defaults to user. These invalid directions are dispatched upstream instead of failing locally, weakening the phase's fail-closed request contract.
**Fix:** Reject unknown roles and enforce `tool_use` only in assistant/model history and `tool_result` only in user history before building Gemini contents. Add negative pre-dispatch tests for each reversed direction.

### WR-02: Compatibility terminal helpers can emit invalid or repeated terminals

**File:** `src/model/gemini.rs:673-700`
**Issue:** `finish()` does not treat `ProtocolFailed` as already terminal, so repeated calls append repeated error outcomes. `finish_stream("STOP")` on a fresh machine emits `message_delta` and `message_stop` without `message_start`; unsupported reasons silently leave it open. These helpers are not used by the production adapter, but they are public and retained as compatibility APIs.
**Fix:** Make all terminal states idempotent, ensure a start event precedes any success terminal, and return a typed error for unsupported reasons. Alternatively make/remove the helpers if no supported caller needs them.

---

_Reviewed: 2026-09-07T00:52:19Z_
_Reviewer: Codex (gsd-code-reviewer)_
_Depth: deep_
