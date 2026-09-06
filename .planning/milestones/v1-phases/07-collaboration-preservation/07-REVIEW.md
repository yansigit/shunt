---
phase: 07-collaboration-preservation
status: passed
reviewed: 2026-09-06
---

# Phase 7 Code Review

Reviewed activation boundaries, request authority construction, schema
sanitization, task conversion, JSON/SSE restoration, native isolation, and
integration coverage.

## Checks

- Collaboration parsing is reachable only on exact Anthropic translations and
  only when the endpoint flag is true; native Responses dispatch is unchanged.
- Tool restoration requires a request-scoped authority-map match, so a provider
  cannot gain the namespace by emitting a prefix-shaped unrequested name.
- Duplicate logical or wire names fail locally instead of resolving
  ambiguously.
- Schema traversal removes only schema annotations, preserves properties and
  literals named `encrypted`, and stops at a fixed nesting limit.
- Plaintext agent tasks retain the routing envelope as a distinct user turn;
  malformed envelopes and ciphertext-only trailing tasks fail before dispatch.
- JSON and SSE paths share the same authority and preserve incremental stream
  delivery without buffering the upstream body.
- Focused HTTP, SSE, WebSocket, disabled-mode, zero-dispatch, and native
  byte-fidelity fixtures cover the new seams.

## Result

No open correctness or quality findings.
