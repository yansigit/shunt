# Phase 3: Quota-Aware Resilience - Context

**Gathered:** 2026-09-05
**Status:** Ready for planning
**Mode:** Smart discuss resolved from previously approved port recommendations

<domain>
## Phase Boundary

Distinguish transient request-rate throttling from hard quota exhaustion on the existing provider and account paths, then derive bounded cooldown/retry timing from `Retry-After`. Preserve unrelated failover behavior. This phase adds no public configuration keys, persistence, generalized health scoring, or credential writeback.

</domain>

<decisions>
## Implementation Decisions

### Classification Contract
- Treat HTTP 429 alone as a generic transient rate limit, not proof of exhausted quota.
- Recognize hard exhaustion only from bounded, successfully parsed provider error bodies with exact trusted code/type evidence such as `usage_limit_exceeded` or `insufficient_quota`; ambiguous, malformed, duplicate, aborted, or oversized bodies fail closed to the transient/unverified class.
- Provider-specific text heuristics may be used only where the provider has a stable structured status contract, and transient request-rate language wins over broad quota wording.
- Keep authentication, permission, server-error, policy, and unrelated failover classifications unchanged unless a focused regression proves they intersect RES-01.

### Retry Timing
- Use one shared `Retry-After` parser for both delta-seconds and HTTP-date forms rather than adding adapter-local copies.
- Accept non-negative decimal delta-seconds with safe rounding and standards-compliant HTTP dates; malformed or excessively long values are ignored.
- Convert zero or past deadlines to an immediate-but-nonzero retry boundary where internal cooldown bookkeeping requires a future instant, avoiding wall-clock underflow.
- Clamp every honored deadline to an explicit finite maximum; an excessive upstream value must never create an unbounded cooldown.

### Pool and Failover Behavior
- Transient throttles receive bounded short retry/cooldown behavior and may rotate through already-supported account/fallback paths before output.
- Proven hard quota exhaustion marks the affected account/provider unavailable until a safe bounded deadline and avoids futile cycling back through exhausted accounts in the same turn.
- Never replay after response bytes or WebSocket events become observable; Phase 1 and Phase 2 streaming and per-turn snapshot guarantees remain binding.
- Preserve the final upstream error body and safe `Retry-After` exposure when all eligible attempts are exhausted.

### Scope and Verification
- Port OpenCodex's behavior and adversarial fixtures, not its duplicate parsers, SQLite history, compatibility lab, cost scoring, or generalized circuit-breaker platform.
- Add table-driven unit tests for classification and time parsing plus integration coverage for account rotation, exhaustion, bounded inspection, and unchanged unrelated statuses.
- Run focused suites and the repository-wide format, strict Clippy, and all-feature workspace tests.
- Update English and maintained README/Nimbus translations only where the resulting behavior is observable; do not hand-edit `wiki/`.

### Claude's Discretion
Internal enum names, exact module placement, and the finite short/maximum constants are implementation discretion, provided they are centralized, tested at their boundaries, and do not become new public configuration.

</decisions>

<canonical_refs>
## Canonical References

- `.planning/ROADMAP.md` Phase 3 and `.planning/REQUIREMENTS.md` RES-01/RES-02.
- `.planning/notes/opencodex-port-audit.md` for the approved selective-port strategy.
- OpenCodex `src/codex/quota-rejection.ts`, `src/adapters/google-errors.ts`, `src/combos/failover.ts`, and their focused tests for observable behavior.
- Shunt `src/accounts.rs`, `src/retry.rs`, `src/adapters/responses/inbound.rs`, and `src/proxy/failover.rs` for existing pool, retry, and failover contracts.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/accounts.rs` already owns account cooldowns, quota-window state, selection ordering, and the existing `retry_after` parser.
- `src/retry.rs` already centralizes bounded retry policy and rejects server delays beyond its budget.
- `src/adapters/responses/inbound.rs` already rotates ChatGPT accounts before output and preserves the final upstream response.
- `src/proxy/failover.rs` already controls route advancement and remembered failure precedence.

### Established Patterns
- Keep retry and quota decisions typed, deterministic, table-driven, and provider-aware.
- Inspect only explicitly bounded bodies and preserve streaming once output begins.
- Use `Instant` for elapsed deadlines and convert wall-clock dates defensively.
- Prefer focused inline/unit fixtures plus real multi-account integration coverage.

### Integration Points
- Centralize retry-date parsing at the current `accounts::retry_after` boundary or a small shared module consumed there and by retry logic.
- Add a typed quota rejection classifier near account-pool policy, with thin provider adapters supplying bounded status/body evidence.
- Feed the decision into existing account cooldown and pre-output rotation paths without creating a second pool or retry driver.

</code_context>

<specifics>
## Specific Ideas

Use the conservative rule from the approved audit: exact structured hard-quota evidence changes pool behavior; broad 429s remain transient. Preserve `Retry-After: 0`, support future HTTP dates without underflow, and cap absurd values.

</specifics>

<deferred>
## Deferred Ideas

- General provider health scoring and persisted request history remain out of scope.
- Changes to public cooldown configuration remain out of scope and require separate approval.
- Capability-aware heterogeneous fallback remains Phase 6.

</deferred>

---

*Phase: 03-quota-aware-resilience*
*Context gathered: 2026-09-05*
