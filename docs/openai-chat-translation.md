# OpenAI Chat Completions translation

Phase 13 adds a generic `kind = "openai_chat"` provider for configured backends implementing
the supported Chat Completions subset through Anthropic Messages ingress. This note records the
adapter design and its guarantees. Scope statement up front: the guarantees below are enforced
by hermetic conformance fixtures in `tests/openai_chat_conformance.rs`; nothing in this note is
live-provider evidence.

## Configuration

There is no built-in preset. An upstream sets `kind = "openai_chat"`, `base_url`, and an
API-key credential — ordered form `auth = { mode = "api_key", env = "CHAT_API_KEY" }` or legacy
`auth = "api_key"` with `api_key_env`. No other auth mode is accepted: validation
(`OpenAiChatRequiresApiKey`) refuses them at boot, because the adapter has exactly one
credential path.

`base_url` follows one shared grammar used by both config boot validation and request
construction (`chat_completions_endpoint`): explicit `http`/`https`, nonempty authority without
userinfo, no query, fragment, dot path segments, whitespace, controls, or backslashes. Exactly one
`/chat/completions` path is appended; a root already ending in `/chat/completions` is not
doubled. Builds are deterministic — repeated construction yields identical bytes.

## Adapter transport

The adapter is **ConnectOnly** (`OPENAI_CHAT_RETRY_SAFETY = RetrySafety::ConnectOnly`): a
generation POST is non-idempotent and reachable in one send, so shunt never re-dispatches after
the request could have reached the upstream, and the outer failover loop treats an upstream
response as terminal even when it is an error status. Redirects are refused outright
(`Policy::none`): a 3xx could otherwise move the injected bearer to a different origin than the
operator configured.

One shared `reqwest` client serves every Chat upstream with a 120-second read-idle timeout.
This is a private fixed bound, not a new public timeout setting. Token counting always
uses the local estimate strategy for Chat, regardless of the provider's `count_tokens` value.
Only `x-request-id` and `request-id` response headers, bounded and character-validated, are
carried into diagnostics; body metadata is untrusted provider content, never forwarded as headers.

## Request translation

`translate_request` is a deny-by-default whitelist. Carried: the resolved model, the
adapter-decided `stream` flag, `messages` (text, base64 or URL images in user messages, thinking
blocks, and paired `tool_use`/`tool_result` turns), `system`, `max_tokens`, `temperature`,
`top_p`, `stop_sequences`, `tools`, and `tool_choice`. Unknown or Responses-only fields
(`metadata` among them) fail closed with a typed Anthropic-shaped 400 rather than degrade. The
tool id registry is request-local, so concurrent translations cannot share state. A single text
payload is bounded by `MAX_TEXT_BLOCK_BYTES` (8 MiB of UTF-8), enforced before dispatch.

## Response machine

Both output modes share one response machine. Unary bodies are aggregated under
`MAX_OPENAI_CHAT_UNARY_RESPONSE_BYTES` (32 MiB). Streaming SSE runs through
`OpenAiChatSseMachine`, which retains at most `MAX_RETAINED_SEMANTIC_BYTES` (8 MiB) of semantic
bytes per turn, emits Anthropic-shaped events, and preserves usage accounting: streaming requests
force `stream_options.include_usage` upstream so usage survives translation. Incomplete UTF-8 at
a buffer boundary is held as residual and counted against the budget; a split character that
completes later stays valid.

## Terminal semantics and named bounds

- Non-success upstream statuses map to gateway-owned Anthropic error shapes; a response proves
  the request may have reached the provider, so ConnectOnly also prevents outer failover
  redispatch.
- Read-idle beyond 120 seconds fails the turn with a typed error.
- SSE success requires an authoritative supported finish reason and `[DONE]`; EOF alone
  fails. Already-decoded frames and residual bytes after `[DONE]` fail; the adapter does not
  wait for HTTP EOF or promise to inspect future bytes. Already-delivered text is not retracted.
- `stop`, `length`, and `tool_calls` map to `end_turn`, `max_tokens`, and `tool_use`.
  Unknown finish reasons, including `content_filter`, fail closed.
- Independent bounds: request text 8 MiB; SSE event payload and incomplete residual payload
  8 MiB each (framing delimiters excluded); cumulative semantic bytes 8 MiB (text, reasoning,
  tool identity and arguments, not the SSE residual); unary wire body 32 MiB; 128 tool calls;
  1 MiB arguments per tool. Event/residual/aggregate/unary/tool-count/tool-argument limits have
  explicit minus-one/at/plus-one probes. Token counters must be integers in `0..=i64::MAX`.
- Tool deltas are buffered request-locally, then parsed once at the finish boundary; tool
  blocks follow `message_start` in first-arrival order. Plaintext `reasoning` and
  `reasoning_content` are supported provider extensions, not official universal Chat fields;
  conflicting, signed, and redacted reasoning is refused.
- Cancellation while awaiting headers or mid-stream/unary collection closes the upstream
  transport and releases the admission permit within a bounded deadline; no byte guarantee is
  made about data already in flight.
