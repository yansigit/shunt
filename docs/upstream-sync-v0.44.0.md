# Upstream v0.44.0 integration

Date: 2026-09-09. Development branch: `codex/opencodex-provider-compatibility`.
Pre-sync tip and retained `v2` tag: `d51161e3808bf1059a106108a8a07aa7a28c614b`.
Fetched upstream: `e1de716a887124cbc3de276ea2a150a531c6998b` (`pleaseai/shunt`, main).

## Scope and decisions

The preflight found 16 upstream commits and 25 conflicted paths. A normal merge
preserves both histories; no reset, rebase, force-push or tag movement is used.
Unrelated `.planning/config.json`, `.planning/state.json` and `.gsd/` are excluded.

- Explicit byte-exact `server.codex_endpoint.routes` entries take priority.
  Unmatched requests retain the existing global models/routes resolver, including
  context-hint normalization, and then the pinned provider fallback.
- Explicit native routes share upstream bounded decoding/model rewriting and
  credential-specific forwarding. Third-party Responses requests use the fresh
  header allowlist; ChatGPT routes retain their pool semantics.
- The existing strict Anthropic/collaboration bridge, continuation and cancellation
  protections, compaction admission, and deny-by-default Go gate remain intact.
- PR481's extra translation modules remain core only, as the PR states. This
  merge does not invent Chat or broader Anthropic endpoint-local dispatch.
- Upstream usage aggregation, per-provider usage, WebSocket quota-event recording,
  schema compatibility and SSO fixes are included. No credential-file refresh or
  writeback implementation was changed to resolve conflicts.

Four high-effort Luna subagents handled separate config, endpoint, transport and
documentation conflict scopes. Root reviewed their outputs, completed endpoint
dispatch, restored the upstream WebSocket quota integration test, fixed compile
integration errors, added precedence/Go regressions, and reconciled stale docs.
This is not a claim that the initial marker-free merge was buildable.

## Verification history

The first integrated warnings-denied full serial workspace run passed: 3,124
tests, zero failed, two existing ignored. A further Go endpoint-route admission
test was then added; final library coverage passed all 2,375 non-ignored tests.

During the subsequent full run, the unchanged
`antigravity_process::streaming_turn_translates_stub_events_to_sse` timed out at
its original 20-second guard. It passed in isolation in 0.38 seconds. This named
timeout predates the sync (Phase15 and v2 archival records). Earlier Phase10
diagnostics demonstrated an HTTP/1 EOF stall in a different test in this fixture
family, but the cause of this occurrence has not been established. No assertion,
test timeout or production code was changed to conceal it. A passing rerun does
not constitute a reliability fix.

The final full run passed **3,125 tests, zero failed, two existing ignored**.
Format and warnings-denied Clippy passed. All five isolated smoke checks passed;
the site built 173 pages in four locales. All 552 article links on the 48 changed
pages passed, as did the 12-page four-locale Go navigation/fragment check.
See [gate results](upstream-sync-v0.44.0-gates.json) for exact commands and result
lines, including the prior failure. The smoke script's SIGTERM diagnostic is its
intentional mock-server cleanup; the command exited zero.

The shared Nimbus template still emits nine broken English section-breadcrumb
instances across these pages (`/getting-started/`, `/guides/`, `/reference/` have
no section index). These were also observed in the pre-sync built site; they
are reported separately from edited article links, not counted as passing links
or fixed by this merge. Existing Vite deprecation and Pagefind CJK stemming
warnings remain non-fatal. No new runtime blocker was found.

Every stateful command uses the serialized
isolated runner: fresh OPENCODEX_HOME, non-production port, and before/after live
config mtime/SHA plus backup/invalid-file inventory checks. No live provider
generation, credential login, refresh, purchase, deployment or publication is
part of this sync.

## Maintenance research

See [upstream-maintenance.md](upstream-maintenance.md) for the GitHub (`gh`),
OpenAlex and web-source research and the proposed future-sync procedure.
`rerere` and scheduled checks are recommendations, not newly enabled settings.
README, engineering docs and all affected site locales were considered and
updated; generated wiki content was not edited.
