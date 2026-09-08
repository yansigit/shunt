# Plan 13-05 execution evidence

## New regression coverage

Commit `09de192` adds 22 tests: five cancellation ownership fixtures, eight
router matrix/boundary fixtures, and nine decoder/aggregate cap probes.
All passed against the existing implementation; no production change was needed
and no RED failure is claimed for those tests. The conformance suite passed all
47 cases; the nine named byte-cap probes passed independently.

## AI-SPEC scenario map

Names below are executable fixtures, not assertions inferred from a passing build.
Conformance fixtures are in `tests/openai_chat_conformance.rs` and its `matrix.rs`
and `lifetime.rs` modules; pure cases are in `tests/openai_chat_translate.rs` and
its `assembly.rs` and `caps.rs` modules.

| AI-SPEC scenario | Executable evidence |
|---|---|
| 1. Unary text | `openai_chat_tracer_unary_happy_path` |
| 2. Streamed text | `openai_chat_tracer_streaming_relay` |
| 3. Tool-heavy unary | `openai_chat_matrix_tool_heavy_unary_roundtrip`, `openai_chat_translate_wire_tool_pairing` |
| 4. Interleaved tool streaming | `openai_chat_assembly_interleave_at_limit_multibyte`, `assembly_interleave_preserves_first_arrival_order` |
| 5. Truncation / EOF | `openai_chat_tracer_streaming_eof_failclosed`, `openai_chat_tracer_finish_without_done_is_error` |
| 6. Malformed frames | `openai_chat_terminal_malformed_json_rejected`, `openai_chat_terminal_residual_after_done_rejected` |
| 7. Embedded provider errors | `openai_chat_terminal_unary_embedded_200_error_request_id`, `openai_chat_terminal_error_after_text_single_terminal` |
| 8. Request/tool pairing violations | `openai_chat_translate_wire_tool_pairing`, pure tool-pair rejection cases |
| 9. Long context / bounds | `openai_chat_cap_aggregate_{minus_one,at,plus_one}_wire`, `openai_chat_cap_unary_{minus_one,at,plus_one}_wire`, nine `cap_*` probes, six tool `bound_*` probes |
| 10. Credential destination | `openai_chat_auth_redirect_refusal`, `openai_chat_auth_concurrent_keys_survive_no_redirect`, `openai_chat_tracer_unary_isolation` |
| 11. Cancellation | All five `openai_chat_cancel_*` fixtures in `lifetime.rs`: upstream EOF, admission reclamation, joined task |
| 12. Replay after possible acceptance | `openai_chat_auth_postsend_timeout_single_attempt`; `tests/failover.rs` post-send status/timeout rejection and pre-connect positive control; `tests/retry.rs` Chat 503 single attempt |

Additional plan scenarios: `openai_chat_matrix_image_wire_mapping` proves image
input on the actual wire; `openai_chat_terminal_usage_only_trailing_chunk_relays_usage`
proves trailing usage. All evidence is hermetic; this is not live-provider or
Computer UI verification.

## CLI/curl smoke

The isolated driver `/tmp/shunt-chat-cli-SSL2Yq/smoke.cjs` exercised the built
debug gateway on port 31983 and mock upstream on port 31984. It passed CLI config
validation, curl unary text/stop translation, curl SSE with exactly one
`message_stop`, malformed-model HTTP 400, and exactly two upstream requests.
Captured requests used `/v1/chat/completions`, model `fixture-chat`, and only the
configured fixture API key; the inbound authorization and x-api-key tripwires
were not forwarded. The stream request included `stream_options.include_usage`.
Both servers were shut down by owned handles. The enclosing isolation wrapper
verified production OpenCodex config mtime/SHA and backup inventory unchanged.

## Documentation RED gate

Commit `7e91f8f` persists the native Node TAP failure and documentation contract
test. GSD `check tdd-red-evidence` returned `RED_EVIDENCE_OK` before documentation
implementation began. This check covers all maintained locales and identical
ordered-configuration examples; it does not substitute for reviewing prose.

## Final gates

Format and warnings-denied Clippy passed. An initial full-test invocation failed
before Cargo launched because `RUSTFLAGS=-D warnings` was unquoted; it was rerun
with the entire environment assignment quoted. Final full-suite and site-build
results passed after the capability correction: **2,875 passed, 0 failed, 2
ignored**, across 28 Cargo result groups. The two ignored tests predate Chat.
Format and warnings-denied Clippy passed again; Astro built 165 pages, and all
four built Chat provider routes contained the contract. The Node docs test
passed, and the shared documented TOML example passed CLI `check`.

## Review follow-through

The independent review's alleged tool-first ordering blocker was disproved by
source tracing and a new real-router ordering assertion (green without changing
production code). The original finding and root adjudication remain in
13-REVIEW.md. Two informational limits were documented, not converted into new
public settings: estimated token counting and fixed 120-second read-idle.

Root discovered a separate incomplete capability branch: Chat was still allowed
as a fallback for structured output and explicit effort, which its completed
translator rejects. The named test failed with the Chat route wrongly retained;
the native output and truthful TAP adapter are in evidence/13-05-red-capability.json.
GSD returned RED_EVIDENCE_OK before the fix. Commits `733fbc5` (RED) and `00f1ce1`
(GREEN) preserve the sequence. All five capability tests passed; supported tools
and base64/URL images remain eligible, and the configured primary stays intact.

The documentation test initially used the wrong `.md` suffix for existing
provider guides. Root restored their original `.mdx` filenames and corrected
the assertion to inspect those same maintained pages; no coverage was removed.
The docs subagent briefly wrote new locale pages to the default checkout and
moved them into this worktree; root confirmed the default checkout had only its
pre-existing `.gsd/` entry afterward. No production settings were touched.

The 13-04 key-link scanner initially missed `tool_assembly` because its planned
regex used `assemble`. Root traced apply/finish calls and adapter consumers,
corrected the pattern to the actual member name, and reran successfully. All
25 declared artifact checks and seven key-link checks pass. Decision coverage:
11/11 honored. Schema and UI safety checks did not block; codebase map staleness
remains the previously recorded non-blocking advisory. Wiki diff and Chat
disabled-test scans were empty. Computer UI remains blocked, never marked passed.
