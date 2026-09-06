# Phase 10 Source Coverage Audit

| Source | ID | Feature / requirement | Plan | Status | Notes |
|--------|----|-----------------------|------|--------|-------|
| GOAL | — | Equivalent strict Code Assist semantics without unsafe replay | 10-01..05 | COVERED | Shared machine, strict transport, replay-safe dispatch, localized contract, final gates. |
| REQ | GEM-01 | OAuth identity/project affinity | 10-03 | COVERED | Captured request lifetime and cancellation evidence. |
| REQ | GEM-02 | Ordered incremental text/reasoning/tools/results/usage/terminal | 10-01, 10-02, 10-03 | COVERED | Calls stream outward; client tool results relay as exact next-request functionResponses; unsupported assistant-side functionResponses fail explicitly. |
| REQ | GEM-03 | Streaming/unary semantic equivalence | 10-01..03 | COVERED | Pure normalized oracle and gateway parity. |
| REQ | GEM-04 | Malformed/oversized/UTF-8/truncated protocol failures | 10-01, 10-02 | COVERED | Checked state and byte decoder/collector. |
| REQ | GEM-05 | Replay-safe generation retry only | 10-03 | COVERED | Non-idempotent safety and hit counts. |
| RESEARCH | — | Checked shared semantic machine | 10-01 | COVERED | Direct/wrapped, terminal, errors, tools. |
| RESEARCH | — | Bounded byte SSE and unary collection | 10-02 | COVERED | Exact/cap-plus-one and disposed parser. |
| RESEARCH | — | Immutable identity, retry, cancellation gateway evidence | 10-03 | COVERED | Synthetic markers, hit counts, body drop. |
| RESEARCH | — | Documentation audit and exclusions | 10-04, 10-05 | COVERED | English engineering/site sources, all maintained locale copies, deterministic exclusion audit. |
| CONTEXT | D-01..D-04 | Request identity and lifetime | 10-02, 10-03 | COVERED | Immutable capture, body ownership, no writeback. |
| CONTEXT | D-05..D-08 | Framing, failure, terminal, and bounds | 10-01, 10-02 | COVERED | Strict decoder/machine/collector. |
| CONTEXT | D-09..D-12 | Shared parity, order, authentic tools, provider errors | 10-01..03 | COVERED | One semantic oracle and gateway matrix. |
| CONTEXT | D-13..D-16 | Retry/recovery and preserved public contract | 10-03, 10-04 | COVERED | Non-idempotent pre-header retry only; exclusions audited. |
| CONTEXT | deferred | Antigravity, other providers, AI Studio Web, credential writeback, durable repair/history | NONE | EXCLUDED | Explicitly owned by later phases or prohibited. |

All goal, requirement, research, and locked-context items are covered. There are no unplanned in-scope items.
