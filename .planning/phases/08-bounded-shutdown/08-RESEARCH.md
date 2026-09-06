---
phase: 08-bounded-shutdown
status: complete
researched: 2026-09-06
requirements: [OPS-01]
---

# Phase 8 Research

## Current Shunt lifecycle

`src/main.rs` awaits `axum::serve(...).with_graceful_shutdown(shutdown_signal())`.
The first SIGTERM/SIGINT stops listener admission, terminates isolated
Antigravity process groups, and arms a second-signal `process::exit` watcher.
There is no first-signal deadline, so one open SSE response or inbound WebSocket
can hold the process forever. State/status/usage pollers are detached
process-lifetime tasks and naturally end only when the runtime drops.

Request, account-pool, and subprocess resources are represented by RAII guards.
Cancelling their owning futures releases them. This makes a process-level
server-future deadline sufficient; Shunt does not need OpenCodex's complete
in-process active-turn registry merely to exit a process.

## OpenCodex behavior worth adapting

`src/server/lifecycle.ts` uses an irreversible shutdown-draining latch, one
absolute timeout covering pre-existing drains and active turns, and forced abort
of survivors at the deadline. It also supports temporary drain owners,
reusable server references, explicit state-store flushes, and many worker
families. The invariant is portable; the TypeScript registry architecture is
not necessary for Shunt's single process lifetime.

## Recommended implementation

1. Add `shutdown_timeout_seconds` to `ServerConfig`, default 30, validate
   `1..=3600`.
2. Have the first-signal future notify a one-shot receiver immediately before
   it resolves into Axum's graceful-shutdown trigger.
3. Await the server normally until that notification, then wrap the still-pinned
   server future in `tokio::time::timeout` using the configured absolute budget.
4. On clean completion, propagate the server result. On timeout, log a warning
   and return successfully; dropping the server future and then the runtime
   cancels surviving HTTP/SSE/WebSocket/background tasks and releases RAII
   guards.
5. Preserve the continuously subscribed second-signal watcher.

## Risk analysis

- Starting the timeout before the signal would turn this into an uptime limit;
  tests must explicitly prove the timer starts at drain notification.
- Spawning the server into an unowned task could detach connection state; keep
  it pinned in the coordinator so timeout return drops the future directly.
- `shutdown_timeout_seconds = 0` must not mean unbounded, because that violates
  OPS-01. Reject it instead of inventing a legacy escape hatch.
- The timeout path should return normally rather than call `process::exit`,
  preserving telemetry guard drops. A second signal remains the intentional
  hard-exit path.
- Documentation must be clear that periodic state is best-effort; Phase 8 does
  not promise a new final persistence transaction.

## Verification seams

- Generic future completes within the post-signal budget: clean outcome.
- Pending future owns a drop sentinel: deadline outcome and sentinel release.
- Delay longer than the configured timeout before notifying: no premature
  timeout, proving the budget begins at signal time.
- Existing real-SIGTERM and second-signal watcher tests remain green.
- Full HTTP/SSE/WebSocket integration suites guard transport regressions; full
  workspace tests, fmt, clippy, diff, and wiki guards close the phase.
