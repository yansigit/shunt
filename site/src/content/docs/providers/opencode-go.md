---
title: "OpenCode Go: evidence-gated, zero support"
description: "OpenCode Go remains unadmitted until an exact, hermetic and credential-safe tuple is proven."
---

# OpenCode Go is not supported today

The OpenCode Go admitted set is empty. Shunt rejects every explicit Go
selection at the pre-credential gate, before credential lookup or network dispatch, so no credential and no
`x-opencode-session` header are emitted today.

The opt-in configuration identity is `kind = "opencode_go"`, using
`SHUNT_OPENCODE_GO_API_KEY` and the canonical destination
`https://opencode.ai/zen/go/v1`. The current source-only candidate records are:

- `glm-5.3-flash`
- `omen-alpha`
- `muse-spark-1.3-contributor`
- `deepseek-v4-flash`

These are candidates, not supported or live-verified models. Unknown fields,
wrong wires, family inference, unsupported effort aliases, and failed evidence
all remain rejected. The gateway requires a strict authoritative terminal and
does not synthesize success from permissive EOF or repair a partial turn.

## Future promotion contract

A future admitted tuple must have exact model, destination, wire, effort, and
capability evidence that passes hermetic conformance plus credential-safe
captured or live verification. It must reuse the matching existing Chat
contract. Only then may it send a stable opaque, conversation-scoped
`x-opencode-session`, and only to `https://opencode.ai/zen/go/v1`; it may never
contain prompt-derived identity or raw account/session data. The empty admitted
set means that session producer is absent today.

See the [configuration reference](/reference/configuration/).
