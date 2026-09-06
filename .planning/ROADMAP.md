# Roadmap: Shunt OpenCodex Behavior Port

## Overview

The milestone first closes Shunt's missing inbound Responses WebSocket boundary,
then layers approved native routing, resilience, compaction, translation,
capability filtering, collaboration fidelity, and bounded shutdown. Each phase
keeps native passthrough opaque and adds only evidence-backed behavior.

## Phases

- [x] **Phase 1: Inbound Responses WebSocket** - Deliver authenticated, bounded, cancellable WS transport with focused conformance coverage. (completed 2026-09-05)
- [ ] **Phase 2: Native Responses Routing** - Route exact models to compatible Responses-native providers without translation. **Requires user approval before implementation because it changes documented provider semantics.**
- [ ] **Phase 3: Quota-Aware Resilience** - Distinguish transient throttling from exhausted quota and honor standards-compliant retry timing.
- [ ] **Phase 4: Native Compaction** - Forward Responses compaction opaquely without local history persistence.
- [ ] **Phase 5: Anthropic Translation** - Add a dedicated Responses-to-Anthropic vertical translation subsystem.
- [ ] **Phase 6: Capability-Aware Fallback** - Exclude incompatible targets before heterogeneous dispatch.
- [ ] **Phase 7: Collaboration Preservation** - Preserve opt-in routed collaboration and continuation semantics.
- [ ] **Phase 8: Bounded Shutdown** - Drain streaming transports to a deadline, then cancel safely.

## Phase Details

### Phase 1: Inbound Responses WebSocket

**Goal**: Codex clients can use the existing opt-in inbound Responses endpoint over a protocol-faithful WebSocket transport without changing HTTP behavior.
**Depends on**: Nothing (first phase)
**Requirements**: WS-01, WS-02, WS-03, WS-04, WS-05, WS-06, WS-07, CONF-01, CONF-02
**Success Criteria**:

  1. Authenticated clients can upgrade every registered inbound Responses path and unauthorized clients are rejected before upgrade.
  2. Warmup and live turns produce the expected ordered WebSocket events, including all terminal and error variants, while the existing HTTP passthrough remains unchanged.
  3. Replaced turns and disconnected sockets stop upstream work and never emit stale events.
  4. Tests demonstrate explicit frame/event/backpressure bounds and pass all repository quality gates.

**Plans**: TBD

- [x] 01-01-PLAN.md
- [x] 01-02-PLAN.md
- [x] 01-03-PLAN.md
- [x] 01-04-PLAN.md

### Phase 2: Native Responses Routing

**Goal**: Inbound model identifiers select exact compatible Responses-native routes while the existing pinned ChatGPT behavior remains the default.
**Depends on**: Phase 1 and explicit user approval
**Requirements**: ROUTE-01, ROUTE-02
**Success Criteria**:

  1. An exact configured model route reaches its native Responses provider with request bytes preserved.
  2. Ambiguous or incompatible targets fail before dispatch with an actionable Responses error.
  3. Existing pinned ChatGPT endpoint configurations continue to behave identically.

**Plans**: TBD — approval gated

### Phase 3: Quota-Aware Resilience

**Goal**: Retry and cooldown behavior reflects the difference between transient request limiting and hard quota exhaustion.
**Depends on**: Phase 2
**Requirements**: RES-01, RES-02
**Success Criteria**:

  1. Transient throttles use bounded short retry/cooldown behavior while exhausted quotas avoid futile account cycling.
  2. Delta-seconds and supported HTTP-date `Retry-After` values resolve to safe bounded deadlines.
  3. Existing failover behavior for unrelated statuses remains covered and unchanged.

**Plans**: TBD

### Phase 4: Native Compaction

**Goal**: Native Responses clients can compact long sessions without Shunt storing or interpreting request history.
**Depends on**: Phase 3
**Requirements**: COMP-01
**Success Criteria**:

  1. Compatible native routes accept `/v1/responses/compact` and preserve opaque continuation state.
  2. Unsupported routes fail clearly before dispatch.
  3. No request-history database or durable continuation store is introduced.

**Plans**: TBD

### Phase 5: Anthropic Translation

**Goal**: Responses clients can target Anthropic through a dedicated, fidelity-tested translation boundary.
**Depends on**: Phase 4
**Requirements**: TRANS-01, TRANS-02, TRANS-03
**Success Criteria**:

  1. Text, instructions, tools, tool results, images, and supported reasoning controls translate into valid Anthropic requests.
  2. Anthropic streams map to correctly ordered Responses events with accurate usage and terminal status.
  3. Unsupported or lossy requests fail before upstream dispatch rather than silently degrading.

**Plans**: TBD

### Phase 6: Capability-Aware Fallback

**Goal**: Heterogeneous fallback chains dispatch only to providers that can satisfy the request.
**Depends on**: Phase 5
**Requirements**: CAP-01
**Success Criteria**:

  1. Tool, image, structured-output, reasoning, and known context requirements exclude incompatible targets before network I/O.
  2. Exclusions are observable and actionable without adding a generalized manifest platform.

**Plans**: TBD

### Phase 7: Collaboration Preservation

**Goal**: Explicitly enabled routed collaboration maintains task identity and opaque continuation semantics across turns.
**Depends on**: Phase 6
**Requirements**: COLLAB-01
**Success Criteria**:

  1. Enabled collaboration metadata survives supported native and translated routes without corruption.
  2. Native ChatGPT traffic and installations without collaboration enabled remain unaffected.
  3. Recovery behavior is bounded and does not create hidden billable retries.

**Plans**: TBD

### Phase 8: Bounded Shutdown

**Goal**: Operators can restart Shunt without indefinite drain or uncontrolled loss of active streaming resources.
**Depends on**: Phase 7
**Requirements**: OPS-01
**Success Criteria**:

  1. Shutdown stops new admission and allows active HTTP/SSE/WS turns to drain up to a documented deadline.
  2. Work remaining at the deadline is cancelled and all leases/tasks are released.
  3. Automated lifecycle tests verify clean drain and forced-cancel paths.

**Plans**: TBD

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Inbound Responses WebSocket | 4/4 | Complete    | 2026-09-05 |
| 2. Native Responses Routing | 0/TBD | Approval gated | - |
| 3. Quota-Aware Resilience | 0/TBD | Not started | - |
| 4. Native Compaction | 0/TBD | Not started | - |
| 5. Anthropic Translation | 0/TBD | Not started | - |
| 6. Capability-Aware Fallback | 0/TBD | Not started | - |
| 7. Collaboration Preservation | 0/TBD | Not started | - |
| 8. Bounded Shutdown | 0/TBD | Not started | - |
