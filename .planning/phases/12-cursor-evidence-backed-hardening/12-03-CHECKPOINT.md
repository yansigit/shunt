# Phase 12 Plan 03 — Architecture Decision

## Completed work

Plans 01 and 02 are summarized and verified. Plan 03 now preserves the authentic McpArgs field-3 tool_call_id in both JSON and SSE output and rejects missing identity instead of minting one. Three focused identity tests pass after two original red assertions exposed the random-ID behavior. This is a partial plan, not structured-history completion.

## Discovery

OpenCodex revision `055c3ecf0de6c35f59195fc434d6b08525182b7f`, inspected 2026-09-07:

- `src/adapters/cursor/protobuf-request.ts` `toolCallStep` encodes authentic IDs in McpArgs and pairs results within McpToolCall; `conversationTurns` stores those steps and turns as content-addressed blobs.
- Generated `agent_pb.ts`: AgentRunRequest.conversation_state=1; ConversationStateStructure.turns=8; AgentConversationTurnStructure.steps=2 contains blob IDs, not inline ConversationStep values.
- `native-exec.ts` `storeCursorBlob` uses raw SHA-256 IDs; `handleCursorNativeKv` handles get/set requests. AgentServerMessage.kv_server_message=4, AgentClientMessage.kv_client_message=3.
- The high-level request builder also has a text-rendering layer. That layer alone is NOT proof of structured history. Composer and external-model continuation forms differ; model-family inference is not safe.
- Shunt currently flattens history and generates a new conversation UUID on every Run. Its sender only emits prebuilt paced frames/heartbeats; it has no KV reply path or per-turn blob ownership.

## Why the reviewed plan is insufficient

The plan scoped implementation chiefly to request.rs and mod.rs. A faithful structured replay needs a new bidirectional protocol handler, bounded request-local blob storage and lifetime ownership, and changes to Run framing/read/write coordination. Putting structured IDs into prompt strings, or emitting inline bytes where the schema expects blob references, would falsely satisfy the plan.

GSD executor deviation Rule 4 requires a user decision before this significant structural change. The existing ban on durable stores and credential-file changes remains in force.

## Recommended change

Approve a narrowly scoped, bounded in-memory KV/blob exchange owned by a single active request, released on completion/error/cancellation, with no disk writeback and no cross-request cache. Reconstruct supported paired history from client-provided messages, preserve authentic call IDs, reject opaque/missing history before credentials/network, and prove hydration and roundtrip pairing through hermetic whole-router tests. Keep exact model support evidence-gated; do not assume Composer behavior for other models.

## Alternative

Reject structured tool-history continuation explicitly until such a transport is approved. This reduces supported behavior and cannot be reported as full completion of CUR-03 without revising the requirements and documentation.

## Awaiting

User approval for the bounded request-local KV/blob protocol addition. Do not create a complete 12-03 SUMMARY or advance phase verification while this decision is pending. Plans 04–08 and phases 13–16 remain pending.

## Verified partial implementation

- Three focused identity tests passed after the original JSON/SSE random-ID assertions failed red.
- Full all-feature workspace tests passed, exit 0 (library: 2,139 passed, 2 existing ignored; integration suites all passed).
- Format check and all-target/all-feature clippy with warnings denied passed.
- Rebuilt binary smoke driver passed all five checks on ports 31711/31712; owned processes and temporary config were cleaned up. This is generic gateway verification, not live Cursor-model verification.
- Production OpenCodex config mtime/SHA-256 and invalid/backup inventory remained unchanged across the isolated test trees.
- README, docs, and site behavior documentation still requires coordinated phase-08 updates before phase completion; generated wiki remains untouched.
