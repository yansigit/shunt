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
