# Cursor request-owned history

Phase 12 plan 03 adds bounded, memory-only KV hydration to the active
AgentService/Run transport. Schema provenance is OpenCodex revision
`055c3ecf0de6c35f59195fc434d6b08525182b7f`, inspected read-only 2026-09-07:
`protobuf-request.ts`, `native-exec.ts`, and generated `agent_pb.ts`.

Run state field 1 references root JSON blobs (state field 1) and conversation
turn blobs (state field 8). Turns reference user messages and steps; MCP steps
carry the original call ID and paired result. Model-visible roots accompany
native state. AgentServerMessage KV field 4 receives client replies on field 3
of the same bounded request-body stream. Unknown gets return an empty result.
Server IDs are opaque; locally created IDs are SHA-256 digests. An existing key
cannot be replaced with different content.

Limits are 192 entries, 512 KiB per entry, 8 MiB total, and 128-byte keys. Overflow
fails explicitly. Client history preparation uses the bounded blocking pool
before credential resolution. The active turn owns its blob store; dropping the
turn stops and aborts its sender, including before response headers. KV reply
sends have a five-second timeout. No persistence or shared history cache exists.

Paired text history requires explicit stable metadata.session_id (or session_id
inside JSON-string metadata.user_id). A deterministic UUIDv8 partitions by exact
wire model and session; tool call IDs are never synthesized. Compaction/recovery
requires retained user context and paired blocks, not an opaque checkpoint.
Only composer-2.5 is admitted for structured tool history; other wire models,
image history, mixed user text/results, missing context, and malformed pairing
fail before dispatch. The exact-model explicit user-action continuation follows
the referenced implementation; no semantic repetition heuristic is imported.

Hermetic tests decode references and MCP arguments/results, perform actual
bidirectional hydration through TLS HTTP/2 and the full gateway router (JSON
and SSE), reject unsupported continuation with zero upstream dispatch, and
exercise store limits/isolation and sender cancellation. These fixtures are
protocol evidence, not live Cursor model or account availability evidence.

README and all four provider-page locales describe the client contract. The
generated wiki is intentionally unchanged.

## Input admission (plan 04)

The active Run adapter requires named, unique custom MCP tools with explicit
object input schemas and object arguments. Argument/schema nesting is limited
to fewer than 64 levels. Only automatic tool choice is represented; forced
selection and parallel-use guidance fail explicitly rather than being ignored.
Inline images require valid nonempty base64 and an explicit PNG/JPEG/GIF/WebP
media type. URL or malformed sources are rejected, not silently omitted. Image
decode and all validation occur before credential resolution. The old extractor
helpers remain available for benchmarks, but no legacy bridge is reachable from
the active dispatch path.

## Terminal and usage reporting (plan 05)

EOF and idle expiry without a wire terminal are errors. Duplicate Connect END
frames or bytes after END already received in the same batch fail before any
success terminal. Agent Run interaction field 14 also denotes turn-ended; an
immediately following Connect trailer in the same batch is inspected first.
There is no claim to inspect future bytes after cancellation or accepted end.

Interaction field 8 token deltas accumulate output tokens, rejecting negative
int32 and overflow. Checkpoint field 3 / token_details field 5 / used_tokens field
1 replaces absolute context occupancy; it is never added to previous context.
Input is derived as max(context minus output, 0). Streaming usage is emitted
incrementally, and JSON uses the same final counters. `usage.estimated: true`
labels this approximation. Required numeric fields use zero when unavailable;
zero without an upstream metric is not a measured count. Cache counts are
unavailable on the evidenced Run fields. No CLI aggregate cache values or retired
StreamUnifiedChatWithTools field mappings are imported as Run measurements.

## Strict wire decoding (plan 06)

The active Run path rejects malformed END JSON, corrupt gzip, invalid flags,
truncated protobuf, invalid UTF-8, and malformed nested MCP arguments instead of
synthesizing completion or substituting null/empty values. Empty END trailers
and valid gzip-compressed JSON remain valid; structured authentication errors
retain their status. Unknown protobuf fields with supported wire encodings
remain forward-compatible. The existing 64 MiB frame/decompression bound is
unchanged; decoded arguments also use that aggregate allocation budget and a
64-level depth boundary. Large argument decoding uses the existing bounded
response-work pool. No retry, configuration, or credential behavior changes.
