# Phase 2: Native Responses Routing - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-05
**Phase:** 02-native-responses-routing
**Areas discussed:** exact-selection source, fallback behavior, payload fidelity, failure boundary

---

## Exact-selection source

| Option | Description | Selected |
|--------|-------------|----------|
| Existing exact declarations | Reuse exact `[models.upstream_model]` and `[[routes]]` declarations without adding config | ✓ |
| Legacy routes only | Limit the endpoint to `[[routes]]` despite model maps being the recommended exact form | |
| Prefix routing too | Apply `[[route_prefixes]]` to inbound Responses traffic | |

**User's choice:** Continue according to the approved recommendations.
**Notes:** Exact existing declarations are eligible; prefix routing remains outside this phase.

---

## Fallback behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Preserve pinned default | Unmatched or unreadable models continue to `[server.codex_endpoint].provider` | ✓ |
| Reject unmatched models | Require every inbound model to have an exact declaration | |

**User's choice:** Continue according to the approved recommendations.
**Notes:** Compatibility with existing endpoint installations is a phase success criterion.

---

## Payload fidelity

| Option | Description | Selected |
|--------|-------------|----------|
| Opaque native passthrough | Select the provider without rewriting request or response payload bytes | ✓ |
| Rewrite mapped model | Mutate the JSON `model` field to the configured upstream id | |

**User's choice:** Continue according to the approved recommendations.
**Notes:** Mappings that imply translation are rejected as incompatible rather than silently mutating native payloads.

---

## Failure boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Fail closed before dispatch | Reject ambiguous or non-Responses-native exact targets before any upstream request | ✓ |
| Fall back to pinned provider | Ignore an invalid exact declaration and use the compatibility provider | |

**User's choice:** Continue according to the approved recommendations.
**Notes:** Fail-closed behavior prevents a configured exact intent from silently reaching the wrong provider.

## Claude's Discretion

- Internal resolver type and module placement.
- Precise actionable error-code names.
- Test fixture organization.

## Deferred Ideas

- Prefix/capability-aware fallback, compaction, translation, and quota classification remain in their existing roadmap phases.
