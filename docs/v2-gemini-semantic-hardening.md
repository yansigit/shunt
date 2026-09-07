# Provider Compatibility v2: Gemini semantic hardening

Status: **implemented** in Phase 10. This record describes the existing
`kind = "gemini"` plus `auth = "google_oauth"` Google Code Assist path after
semantic hardening. It adds no provider, public configuration, endpoint, model
entitlement, authentication source, or dependency.

The phase keeps three locked compatibility decisions explicit:

- **D-02:** retain the Code Assist destinations and request envelope; do not
  borrow Antigravity identity, catalog, session, header, or model-tier policy.
- **D-03:** retain the existing Google OAuth validation and destination policy,
  with no credential-file writeback or migration.
- **D-16:** retain existing HTTP status and Anthropic error-envelope behavior
  where it was already defined. Newly strict malformed, truncated, and embedded
  provider-error cases fail explicitly, but no public provider mode or config
  key is added.

## Identity and request lifetime

Gemini keeps the existing Google Code Assist destinations
`v1internal:generateContent` and `v1internal:streamGenerateContent`, the
`{model, project, request}` envelope, and the existing `google_oauth` credential
source. The adapter resolves one token and selected project before dispatch and
keeps that pair, the endpoint, and the translated request immutable through
every permitted attempt and until the response completes or is cancelled.
Dropping the downstream body drops the upstream response, parser state, and
request-owned resources through the existing RAII/body ownership path.

The bearer remains subject to the existing Gemini destination policy. No token,
project, account, or request is rediscovered or rotated during a turn, and this
phase performs no credential writeback, creation, migration, or refresh-write.

## Bounded framing and collection

Streaming responses use an incremental byte-oriented SSE decoder. It preserves
event order across arbitrary transport and UTF-8 splits, accepts LF, CRLF,
comments, unknown SSE fields, and supported multi-line `data:` fields, and
decodes JSON only after an event is complete. The production limits are 8 MiB
per SSE event and 256 completed events per decoder feed. Client-requested unary
responses and non-success HTTP bodies use a checked 32 MiB wire limit; semantic
retention is independently bounded to 32 MiB, with bounded tool signatures,
tool IDs, and content-block counts.

Invalid UTF-8, malformed JSON, malformed supported fields, multiple candidates,
oversized events or bodies, amplification beyond the event-count limit,
unterminated non-whitespace residuals, duplicate terminals, and semantic data
after a terminal are protocol failures. Unknown JSON fields remain ignorable for
forward compatibility, but an ambiguous supported shape is never guessed or
silently discarded.

## One checked semantic contract

Direct Gemini response objects and the Code Assist `response` wrapper feed the
same request-local state machine. Streaming emits validated text, reasoning,
function calls, and usage incrementally; unary mode reconstructs its bounded
Anthropic message from the same ordered state. Both modes therefore share the
same content, tool, usage, finish, and provider-error meaning.

A successful response requires exactly one supported provider finish
(`STOP`, `MAX_TOKENS`, or `SAFETY`) followed by clean transport closure.
Transport EOF and `[DONE]` are framing boundaries, not substitutes for a
provider finish. Provider finish maps to `end_turn`, `max_tokens`, or
`stop_sequence` respectively, except that a valid final function call retains
the Anthropic `tool_use` stop reason. Embedded provider errors are authoritative
and produce one Anthropic error outcome rather than a successful HTTP 200 result
or a synthetic terminal.

## Tools and authentic signatures

Candidate parts are validated atomically before any part is emitted. Text,
reasoning, and function calls retain provider order. A function call requires a
non-blank name and valid JSON arguments. Gemini 3 calls also require the
authentic non-empty `thoughtSignature` attached to that exact part; shunt bounds
and carries it in the opaque `call_gemini_v1_...` tool-use ID. It never invents,
moves, or repairs a signature. Foreign, malformed, duplicate, orphaned, or
ambiguous tool identities fail before dispatch.

GEM-02's function-result direction is the faithful Anthropic round trip: a
Gemini `functionCall` becomes a client-visible `tool_use`, and the client's
matching `tool_result` becomes the next Code Assist request's
`functionResponse`, preserving its name, result, pairing, and authentic
signature metadata. An assistant-side Gemini `functionResponse` is rejected
because Anthropic assistant output has no faithful function-result
representation; it is not rewritten as text or a tool block.

## Retry, failover, and cancellation

Gemini generation is a non-idempotent POST and is dispatched with
`RetrySafety::NonIdempotentPost`. A bounded retry may reissue only a clearly
transient connection or timeout failure resolved before response headers. The
closure reuses the same token, project-bearing envelope, endpoint, and payload.
A returned status, malformed or truncated body, embedded provider error,
client-visible output, or replay-unsafe tool activity is never retried against
the same upstream. Body processing contains no redispatch, account switch,
project discovery, route hop, or partial-response repair seam.

The ordered cross-upstream chain remains governed by
[`docs/upstreams-failover.md`](upstreams-failover.md): it may advance on its
documented pre-body status classes, but no fallback is possible after a 2xx
response's headers are returned. Cancellation does not start recovery; it drops
the owned upstream work and releases gateway capacity.

## Hermetic evidence

The contract is executable without real credentials or provider traffic:

```text
cargo test --all-features --test gemini_translate
cargo test --all-features --test gemini_conformance
cargo test --all-features --test retry --test failover --test gemini_conformance
```

`tests/gemini_translate.rs` checks direct/wrapped semantic parity, strict
terminal behavior, bounds, ordered reasoning/text/tools, authentic signature
round trips, and invalid or orphaned tool metadata. `tests/gemini_conformance.rs`
drives the real gateway against loopback upstreams and checks Code Assist
identity/envelopes, stream/unary parity, framing and collection limits,
provider-error handling, exact attempt counts, tool-result round trips, and
cancellation. All identity, project, signature, and prompt fixtures are
synthetic.

## Explicit exclusions

This phase deliberately does not change Antigravity destination, discovery,
OAuth, account/project, catalog, session, model-tier, header, or signature
policy. It does not add credential writeback or durable tool/signature history,
general response repair, new dependencies, or generated wiki edits.

Google AI Studio Web is excluded. There is no cookie or SAPISIDHASH
authentication, browser extension, daemon, session synchronization, MakerSuite
parser, browser/WebKit dependency, implementation, fixture, or support claim.
Direct Gemini API-key and Vertex behavior are likewise not documented by this
Code Assist contract.
