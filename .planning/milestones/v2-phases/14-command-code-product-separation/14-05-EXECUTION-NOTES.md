# Plan 14-05 execution notes

## Test-first evidence limitation

The GLM/high auth executor reported an initial compile-failing run (missing
resolve_sources/FILE_READS and a fixture type error) as RED. This is **not** an
observed behavioral assertion failure and does not satisfy the established GSD
RED-evidence gate. No valid auth RED artifact or gate pass is claimed. The main
agent caught and rejected an intermediate empty-env-falls-back draft by source
inspection; it was corrected before acceptance. That correction also has no
separately observed pre-fix assertion failure and must not be presented as one.

Post-implementation verification is independent of that process gap: four
auth fixtures pass, including strict empty-env rejection, actual read-only
temporary files, malformed/oversized/non-regular inputs, and FIFO handling.
The main agent added a private async mutex for file-read-counter assertions,
then reran the four tests with parallel test execution successfully.

## Lifetime characterization

The existing response/retry ownership already met the added cancellation and
replay fixtures: their first executable runs passed, so no artificial failure
or behavioral RED is claimed. Four real-socket cancellation fixtures cover
before headers and mid-body for unary and SSE, actual upstream closure, saturated
admission, then a successful follow-up on the same one-slot gateway.

Four real canonical TLS replay fixtures cover a genuine refused TCP connection
then one retry, pinned credential/session despite changing the synthetic env
after capture, post-send TTFB timeout, same-origin 302 with no follow-up, and
text/tool output followed by missing finish without replay. The test DNS seam
exists only under cfg(test); all sockets are loopback ephemeral, never 10100.

The at-dispatch validation was moved next to bearer-header construction, with
a private helper and a mutated-snapshot characterization test. Initial
pre-credential validation remains intact. The 13-test lifetime filter and
warnings-denied all-targets Clippy pass. Full workspace gate is recorded in the
final summary only after observing completion.

## Safety and scope

The auth source is fixed ~/.commandcode/auth.json in production; tests inject
temporary paths without changing HOME. Only VarError::NotPresent falls back.
The read is capped at 16 KiB and waiting is capped at five seconds; the blocking
read task may outlive a timeout on a stalled filesystem, but cannot write or
retain unbounded data. Unix nonblocking open plus fstat rejects FIFOs and other
non-regular files. No refresh, copy, repair, whoami, or credential writeback was
added. All stateful commands use the isolation wrapper and completed commands
report production OpenCodex mtime/SHA/inventory unchanged.

The GLM/high response reviewer returned no review before interruption; no
independent response review pass is claimed. GLM/high auth execution succeeded;
no Luna fallback run is claimed. Root owns integration and remaining gates.
