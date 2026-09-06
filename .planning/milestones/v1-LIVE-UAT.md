---
status: completed_with_findings
milestone: v1
base_commit: 342e9d176e4364fadc4f7bab0233c67091ee2d68
started: 2026-09-06T17:16:53-06:00
completed: 2026-09-06
live_provider: codex (ChatGPT OAuth)
---

# v1 Live Inference and Smoke UAT

## Scope and isolation

The release binary was run on loopback with an isolated, ignored config under
`target/`. The config enabled the built-in `codex` provider, the inbound Codex
endpoint, and exact `gpt-5.6-sol` / `gpt-5.6-luna` routes. The existing
`~/.codex/auth.json` credential was read by Shunt; no credential value was
printed, copied, or changed. No ordinary user Shunt process was running or
stopped.

## Live results

| Scenario | Result | Evidence |
|---|---|---|
| Release build and config validation | PASS | `cargo build --release`; isolated `shunt check`; clean loopback boot |
| Liveness and discovery surfaces | PASS | `/`, `/health`, `/protocol`, `/routes`, and `/v1/models` returned 200; 13 advertised models |
| Native Responses SSE | PASS | 200; ordered created/in-progress/content/terminal events; one `response.completed`; exact marker |
| Native Responses JSON request | EXPECTED REJECT | ChatGPT subscription backend requires `stream:true`; its 400 was relayed verbatim |
| Messages to Responses, non-streaming | PASS | 200 Anthropic message; exact marker; 17 input / 8 output tokens |
| Messages to Responses, streaming | PASS | 200 chunked SSE; ordered Anthropic lifecycle and one terminal; exact marker |
| Nested-schema forced tool call | PASS | `inspect_plan` returned valid nested objects, enum, arrays, required fields, and stable call id |
| Tool-result continuation | PASS | Paired `tool_result` completed with exact marker |
| Parallel tool calls | PASS | Two requested tools returned in one turn with correct names, ids, and arguments |
| Image plus requested reasoning effort | PASS | Base64 PNG translated and accepted; exact image marker returned. This response exposed no reasoning summary block. |
| Long context | PASS | Local count 50,416 tokens; live request reported 50,442 input tokens and returned both beginning/end markers in 2 s |
| Native WebSocket warmup | PASS | All three paths returned local `response.created` then `response.completed`, empty ids |
| Native WebSocket live inference | PASS | Ordered frames and exact marker, one terminal |
| Explicit WebSocket `generate:true` | FIXED/PASS | Initially relayed the WS-only flag and received upstream 400. Shunt now strips it; focused tests and rebuilt live binary returned 200/event completion. |
| Real Codex CLI through Shunt | PASS WITH FINDING | Codex 0.152.0 executed a shell read and completed the final response through `/v1/responses` |
| Actual spawned subagent through Shunt | PASS | Parent spawned exactly one child, waited, and received the child's independent nonce result |
| Real-client context/tool load | PASS | 81,996 input tokens (47,104 cached), 410 output tokens, 106 reasoning tokens across the client tool/subagent loop |
| Client disconnect | PASS | Client timed out after two SSE events and before terminal; `/health` stayed 200; immediate follow-up returned exact marker |
| Inbound auth | PASS | Missing/bad credentials returned 401. Configured header and Bearer worked on Responses; configured header/Bearer/`x-api-key` worked on Messages and model discovery. Responses deliberately does not authenticate from `x-api-key`. |
| Analytics discard sinks | PASS | Both documented paths returned 200 `{}` |
| Malformed JSON | PASS | OpenAI endpoint returned bounded 400 and process stayed healthy |
| Idle graceful shutdown | PASS | SIGINT completed the drain and exited 0 |
| Active-stream shutdown deadline | PASS | After five seconds Shunt cancelled the remaining stream and exited 0; the client received partial events and no false terminal |

## Findings and unavailable live paths

1. **Open: Codex model-discovery schema mismatch.** Codex CLI 0.152.0 repeatedly
   requested Shunt's model catalog and expected a top-level `models` field,
   while Shunt's Claude gateway discovery endpoint returned the documented
   OpenAI-style `data` list. Inference, tool execution, and subagent execution
   still completed, but the client logged the mismatch on each refresh.
2. **Provider/environment limitation: native compact.** A real
   `POST /v1/responses/compact` reached the ChatGPT edge and returned its 404.
   Local routing/body/auth preservation remains covered by the deterministic
   inbound endpoint tests; no live compact success is claimed.
3. **Not run live: Responses-to-Anthropic translation and its collaboration
   namespace bridge.** This host had no Anthropic credential. The full JSON,
   SSE, WebSocket, encrypted-task rejection, collaboration-name restoration,
   and no-network rejection cases passed against controlled upstreams.
4. **Not run live: other provider kinds and multi-account quota rotation.** Only
   the existing Codex credential was available. Retry, failover, quota,
   capability filtering, header safety, WebSocket replacement/disconnect, and
   compaction routing were exercised with deterministic controlled upstreams.

## Deterministic regression evidence

- `tests/inbound_codex_endpoint.rs`: 40 passed
- `tests/inbound_codex_websocket.rs`: 12 passed
- `tests/inbound_anthropic_translation.rs`: 10 passed
- `tests/responses_translate.rs`: 83 passed
- `tests/failover.rs`: 20 passed
- `tests/retry.rs`: 7 passed
- shutdown-focused library/binary tests: 12 passed
- Final required gate: `cargo fmt --all --check`, Clippy with warnings denied,
  and `cargo test --all-features --workspace`

## Disposition

Live inference is a go for the configured Codex provider across Messages,
native Responses SSE, inbound WebSocket, complex tool loops, a real Codex CLI,
one actual spawned subagent, and 50k-token explicit context. The model-catalog
schema mismatch remains a non-fatal compatibility issue and live remote
compaction is unavailable on the tested ChatGPT endpoint.
