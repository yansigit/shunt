# Phase 15: Exact OpenCode Go Evidence Gate - Pattern Map

**Mapped:** 2026-09-08  
**Files analyzed:** 8 likely implementation/test surfaces  
**Analogs found:** 8 / 8 (all paths below are tracked source)

Phase 15 is intentionally evidence-gated. The pinned-source records in
`15-RESEARCH.md` do not include a captured/live proof for any candidate (or for
the proposed session header), so the intended admitted tuple set is empty.
Patterns below describe the smallest future promotion seam; they do not justify
adding dormant wire/session infrastructure or changing generic provider
semantics.

## File Classification

| New/Modified File | Role | Data Flow | Closest Tracked Analog | Match Quality |
|---|---|---|---|---|
| `src/config/presets.rs` | config/preset | request-response config expansion | `src/config/presets.rs` (`commandcode`) | exact |
| `src/config/upstreams.rs` / `src/config.rs` | config identity | config transform | `src/config/upstreams.rs:198-245` | exact seam |
| `src/adapters/opencode_go/efforts.rs` (or equivalent crate-private gate) | utility/admission | request-response validation | `src/adapters/command_code/efforts.rs` | exact |
| `src/proxy/capability.rs` | middleware/admission | request-response fallback filtering | existing `CommandCode` branch | role + flow |
| `src/proxy/failover.rs` | controller/dispatcher | bounded request-response failover | `forward` and `count_tokens_response` | exact flow |
| `src/codex_endpoint.rs` / `src/routing.rs` | route/admission | inbound request-response | `resolve_native_inbound` + `forward_turn` | exact flow |
| `src/adapters/openai_chat/*` | provider adapter | streaming + unary transform | existing OpenAI Chat adapter | exact wire |
| `tests/*` (conformance, negative admission, inbound) | test | hermetic request/stream assertions | `tests/openai_chat_conformance.rs`, `tests/command_code_conformance.rs`, `tests/inbound_codex_endpoint.rs` | exact |

## Pattern Assignments

### Preset expansion and product identity

**Analogs:** `src/config/presets.rs:90-103`, `src/config/upstreams.rs:198-245`.

The existing `commandcode` preset demonstrates the additive table row:

```rust
ProviderPreset {
    name: "commandcode",
    kind: ProviderKind::OpenAiChat,
    base_url: "https://api.commandcode.ai/provider/v1",
    auth: AuthMode::ApiKey,
    api_key_env: Some("SHUNT_COMMANDCODE_API_KEY"),
}
```

`normalize` resolves `provider` to a preset, then copies only `kind`,
`base_url`, auth and env into `ProviderConfig` (`upstreams.rs:198-239`). A
bare `OpenAiChat` row therefore loses product identity after expansion; URL or
config-table-name inference would violate D-02. The smallest explicit opt-in
pattern is an additive product marker carried through the preset-to-provider
copy (a private, table-driven enum/field), defaulting to generic/none for all
existing providers. The marker must be immutable in the normalized snapshot and
checked by the Go admission gate; do not alter `AdapterKind::OpenAiChat` or
make all Chat providers inherit Go behavior. Preserve preset order tests at
`presets.rs:131-153` and add the new name only if the public preset is shipped.

### Exact tuple gate (pre-credential)

**Analog:** tracked `src/adapters/command_code/efforts.rs:1-54`.

The closest gate is an exact `(model, &[efforts])` table. It rejects unknown
models before any dispatch and validates explicit route/body effort without
case-folding, clamping, or silently dropping values:

```rust
let (_, supported) = MODEL_EFFORTS
    .iter()
    .find(|(id, _)| *id == model)
    .ok_or("unsupported Command Code model")?;
if effort.is_some_and(|e| !supported.contains(&e)) {
    return Err("unsupported Command Code reasoning effort");
}
```

For Go, use the same exact lowercase model + canonical wire/product marker,
with no family aliases. Because evidence is source-only and no capture/live
proof exists, the table should be empty (or return unsupported for every
candidate); do not pre-admit `deepseek-v4-flash`/`glm-5.3-flash` from the
research hints. Keep the gate crate-private and invoke it before
`resolve_route_credential` and before constructing outbound headers.

### Primary/fallback admission seam

**Analogs:** `src/proxy/failover.rs:56-84,102-146` and
`src/proxy/capability.rs:131-163`.

`forward` resolves the complete route chain, then calls
`capability::filter_fallbacks` before inbound auth/credential work
(`failover.rs:56-84`). The existing Command Code branch supplies the pattern for
route-specific validation:

```rust
if route.adapter == AdapterKind::CommandCode
    && crate::adapters::command_code::efforts::resolve(
        request, &route.upstream_model, route.effort.as_deref()
    ).is_err()
{
    reasons.push("model-or-reasoning-effort");
}
```

Add Go identity/wire admission at this same pre-credential stage, but ensure a
Go primary is rejected (not merely removed as a fallback) when its exact tuple
is unsupported. Fallback filtering must not permit a generic OpenAI Chat route
to impersonate Go, and must preserve the route chain's existing bounded retry
and cancellation behavior.

### Dispatch and credential boundary

**Analogs:** `src/proxy/failover.rs:272-334,356-400` and
`src/adapters/openai_chat/mod.rs:210-273`.

Count-token requests are truncated to the first route (`failover.rs:74-90`),
then `count_tokens_response` selects local Tiktoken/Estimate or dispatches that
single route (`:272-334`). The Go preset must make an explicit count-token
choice; with zero admitted tuples, the admission error must happen before this
choice can resolve a credential or send a request. Do not silently inherit a
new Chat counter.

Normal Chat dispatch resolves the provider credential, translates the body,
constructs the canonical `/chat/completions` endpoint, and only then sends a
Bearer request (`openai_chat/mod.rs:210-253`). Preserve redirect refusal,
ConnectOnly retry classification and strict terminal handling. A future Go
adapter branch may reuse this exact contract, but must not enable upstream's
bounded EOF synthesis: D-08 requires strict authoritative terminals.

### Inbound Codex path

**Analogs:** `src/routing.rs:136-215` and `src/codex_endpoint.rs:398-449`.

`resolve_native_inbound` performs exact model mapping, rejects ambiguous or
unsupported adapters, and returns `Pinned`, `Selected`, or `Rejected` before
`forward_turn` dispatch. `forward_turn` immediately turns `Rejected` into a
400 error (`codex_endpoint.rs:407-419`) and only then enters the adapter match
(`:447-449`). Route the same Go exact-tuple gate through this decision so an
inbound Codex Go selection gets the identical pre-credential rejection. Do not
relax the existing Responses/Anthropic-only native decision or add Go Responses
wire support absent evidence.

### Session/header pattern (conditional only)

**Closest tracked analog:** `src/adapters/command_code/request.rs:11-73`.

Command Code validates the provider again immediately before header construction,
bounds/validates the conversation id, hashes token + conversation into a stable
opaque UUID, and emits it only in its canonical header set. This is the pattern
to consult if a Go tuple is ever promoted. It is **not** permission to add a Go
session producer now: upstream evidence records `x-opencode-session` as absent,
and no captured/live acceptance exists. With an empty admitted set, emit no Go
session or credential header and add only negative assertions.

### Test structure

Use the tracked fixture organization in `tests/openai_chat_conformance.rs`
(streaming/unary twins and strict EOF/terminal cases),
`tests/command_code_conformance.rs` (exact model/effort and route-level
negative cases), and `tests/inbound_codex_endpoint.rs` (raw inbound Responses
routing, zero-dispatch assertions, and credential/header matchers). Add focused
tests for unknown/family-inferred/out-of-ladder/wrong-wire Go tuples asserting
zero credential lookups and zero mock-server requests on primary, fallback,
count-tokens, and inbound-Codex paths. Do not add positive admitted fixtures or
session assertions until a dated sanitized capture/live verification exists.

## Shared Invariants

- Exact model + product identity + canonical destination + wire + effort are an
  allowlist; no URL/name/family inference.
- Admission precedes credential resolution, header construction, network
  dispatch, and fallback retry.
- Empty admitted tuple set is a valid shipped state; generic OpenAI Chat,
  Responses, and Anthropic behavior remains unchanged.
- Strict terminal semantics and bounded retry/cancellation remain authoritative.
- No speculative multi-wire/session infrastructure, credential-file behavior,
  production config access, port `10100`, builds, or live calls belong in this
  mapping.
