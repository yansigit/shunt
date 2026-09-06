# Phase 8: Bounded Shutdown - Context

**Gathered:** 2026-09-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Make process shutdown finite: the first SIGTERM/SIGINT stops new admission and
lets active HTTP, SSE, and WebSocket work drain until one documented deadline;
when it expires, the server future is cancelled so the Tokio runtime tears down
remaining tasks and drops their resource leases. Preserve the existing
second-signal immediate exit.

</domain>

<decisions>
## Implementation Decisions

### Deadline contract
- Add `[server] shutdown_timeout_seconds` with a 30-second default.
- Reject `0` and values above 3600 at config validation so shutdown is always
  bounded and configuration mistakes cannot recreate an indefinite drain.
- Start the deadline when the first shutdown signal is received, not at boot.
- One absolute deadline covers every active transport rather than granting a
  fresh timeout per connection or cleanup stage.

### Cancellation and resource release
- Reuse Axum graceful shutdown to stop accepting new connections and drain
  ordinary HTTP, SSE, and upgraded WebSocket connections.
- At the deadline, drop the server future and return from `run`; Tokio runtime
  teardown cancels remaining detached connection/background tasks and drops
  admission/account/subprocess guards.
- Terminate isolated Antigravity process groups on the first signal as today.
- Keep the second SIGTERM/SIGINT escape hatch and its conventional exit codes.

### Operator behavior
- Log clean drain completion at info level and deadline expiry at warning level.
- A forced deadline exit is an orderly process return, not `process::exit`, so
  tracing/telemetry guards still receive their normal drop opportunity.
- Do not add management APIs, temporary drain leases, or reusable in-process
  restart semantics; Shunt's lifecycle boundary is the process.

### Verification
- Unit-test deadline start timing, clean drain, timeout cancellation/drop, and
  the existing second-signal watcher.
- Exercise HTTP/SSE/WebSocket lifetime behavior through the server future and
  existing transport suites without weakening byte-streaming semantics.
- Run the full repository CI gate and document the key in every maintained
  locale.

### Claude's Discretion
- Exact helper names and internal channel type.
- Whether clean completion before a signal is represented separately, provided
  server errors still propagate unchanged.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/shutdown.rs` already owns SIGTERM/SIGINT selection, Antigravity cleanup,
  and the second-signal force-exit watcher.
- `axum::serve(...).with_graceful_shutdown(...)` already stops admission and
  tracks HTTP, streaming bodies, and upgraded WebSocket connection lifetimes.
- Rust future cancellation and Tokio runtime teardown already drop the request,
  pool-admission, and subprocess guards used throughout the gateway.

### Established Patterns
- Server-wide defaults live directly on `ServerConfig` with serde default
  functions and explicit validation errors.
- Lifecycle behavior is tested with injected futures/channels; only one
  serialized Unix test raises a real signal.
- Background pollers are detached process-lifetime tasks and stop when the Tokio
  runtime is dropped.

### Integration Points
- `src/main.rs` constructs and awaits the Axum server.
- `src/shutdown.rs` will notify the coordinator when the first signal starts the
  drain and enforce the absolute deadline.
- `src/config.rs` owns the public key, default, and validation.
- README, engineering docs, and Nimbus configuration/running references own the
  operator contract and translations.

</code_context>

<specifics>
## Specific Ideas

Adapt OpenCodex's most valuable lifecycle invariant—an irreversible admission
fence plus one absolute drain budget—to Shunt's process-oriented Axum runtime.
Do not port OpenCodex's larger reusable-server lifecycle registry or storage
worker graph where Rust task/runtime cancellation already supplies the needed
process-exit behavior.

</specifics>

<deferred>
## Deferred Ideas

- Reusable in-process stop/restart and management-triggered temporary drains.
- Per-worker shutdown hooks and durable final-flush guarantees beyond existing
  periodic/atomic persistence.

</deferred>
