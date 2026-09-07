# Antigravity: the daily backend host and effort-suffixed model ids

## Historical observations (not current availability evidence)

In the historical probe below, every request the built-in `antigravity` provider sent to
`cloudcode-pa.googleapis.com` failed with:

```json
{
  "error": {
    "code": 429,
    "message": "Resource has been exhausted (e.g. check quota).",
    "status": "RESOURCE_EXHAUSTED"
  }
}
```

The message reads as a rate limit, but the probe does not establish its cause.
The historical live observations below were not repeated for Phase 11 and are
not a current model-availability or quota guarantee.

## Probe matrix

| host | model | envelope | result |
| --- | --- | --- | --- |
| `cloudcode-pa.googleapis.com` (old default) | `gemini-3.8-flash` | shunt's `{model, project, request}` | 429 "check quota" |
| `cloudcode-pa.googleapis.com` | `gemini-3.8-flash-medium` | shunt's | 503 |
| `daily-cloudcode-pa.googleapis.com` | `gemini-3.8-flash-medium` | plus `userAgent`, `requestType`, `requestId`, `request.sessionId` | **200** |
| `daily-cloudcode-pa.googleapis.com` | `gemini-3.8-flash` | full envelope | 404 `NOT_FOUND` |

Read together: in that capture the tested **bare model slug did not exist** —
that row varied only the id, so the `404` is attributable to it. The
production `429` is *not*: that request differed from the working one in
both the model id (bare) and the envelope (plain Code Assist), so the
probes do not establish which input produced it. What the matrix does
establish is the working combination — suffixed id plus the full envelope
on the daily host — not a cause for each individual failure.

## The catalog

`POST {base_url}/v1internal:fetchAvailableModels` (body `{}`, bearer plus
the Antigravity `User-Agent`) answers `{"models": {"<catalog id>": {...}}}`.
The **keys** are the wire ids; the `model` values inside the entries are
internal placeholders (`MODEL_PLACEHOLDER_M322`) that 404 if sent.

The key set is **per account and changes over time**. One capture showed
effort-suffixed ids:

- `gemini-3.8-flash-{high,medium,low}`
- `gemini-3.7-flash-{high,medium,low}`
- `gemini-3.6-flash-{high,medium,low}`
- `gemini-3.1-pro-{high,low}` — Pro has **no** `-medium`
- `claude-sonnet-4-6`, `claude-opus-4-6-thinking`, `gpt-oss-120b-medium`
  (not Gemini, no suffix synthesis)

A later capture on the same account had no `gemini-3.8-flash-{high,medium,low}`
and no `gemini-3.7-flash-*` suffix ids at all — instead
`gemini-3.8-flash-tiered`, `gemini-3.7-flash-tiered`, `gemini-3.6-flash-tiered`
alongside the surviving `gemini-3.6-flash-*` and `gemini-3.1-pro-*` suffix
ids. Live: `gemini-3.8-flash-medium` → `404 Requested entity was not
found`, `gemini-3.8-flash-tiered` → `200`, `gemini-3.6-flash-medium` →
`200`. A `-tiered` id names no effort, so the effort goes in
`request.generationConfig.thinkingConfig.thinkingLevel` as
`"low" | "medium" | "high"` (case-insensitive; an unknown value is a `400`,
so the field is parsed). `thinkingLevel` and `thinkingBudget` together are
accepted (`200`).

`agy models` lists the catalog as it currently stands. Note that other
proxy implementations query `fetchAvailableModels` on more than one host,
so "only the daily host serves this catalog" is *not* established here —
what is established is that the `agy` client addresses the daily host for
both discovery and inference.

## Reference implementation

router-for-me/CLIProxyAPI,
`internal/runtime/executor/antigravity_executor_request.go`:

- `resolveAntigravityRequestBaseURL` sends inference to the `daily-` host.
- `geminiToAntigravity` adds `userAgent: "antigravity"`,
  `requestType: "agent"`, `requestId: "agent-<uuid v4>"`, and
  `request.sessionId: "-<up to 19 decimal digits>"`.
- `generateStableSessionID` derives the session id from the conversation
  rather than drawing it at random, so follow-up turns stay in one
  session.

The `agy` CLI itself also calls `loadCodeAssist` and
`fetchAvailableModels` on the daily host, so the daily host is the right
target for discovery as well as inference.

## Verified native contract — 2026-09-07

Phase 11 verification uses synthetic credentials, private temporary files, and
local router/HTTP fixtures. It does not authenticate against a live subscription
or claim that a model is currently available.

- **Origin:** canonical HTTPS roots are `daily-cloudcode-pa.googleapis.com`
  and `cloudcode-pa.googleapis.com` (plain production normalizes to daily).
  Configure no extra path, query, fragment, userinfo, or production port.
  Loopback fixtures retain their exact origin. Inference and every redirect
  retain the validated origin and exact `/v1internal:streamGenerateContent?alt=sse`
  target; lookalikes and off-origin redirects never receive the bearer.
- **Admission:** fresh `fetchAvailableModels` evidence is keyed by backend,
  account fingerprint, and project. Catalog redirects are refused before following
  any Location, including sibling paths. Successful snapshots are fresh within the
  ten-minute TTL. Stale/cold failures cannot authorize inference. The exact
  Gemini ID must be present; no suffix synthesis, nearest-effort selection,
  missing-model repinning, or Claude/GPT forwarding occurs. Authentication and
  catalog lookup can precede rejection, but unsupported tuples never reach inference.
  Route/provider effort takes precedence over request effort. Explicit effort
  must match a suffix; exact tiered IDs accept low/medium/high (default medium).
  Unknown efforts, including xhigh/max, are rejected rather than clamped.
- **Envelope:** one captured credential tuple supplies bearer, project, catalog,
  and account scope. Native requests carry model/project, `userAgent: "antigravity"`,
  `requestType: "agent"`, a fresh `requestId: "agent-<uuid>"`, and
  `request.sessionId`. The session derives from the account and canonical
  opening user turn, not the growing transcript. Identical openings in one
  account share a session. Plain Gemini Code Assist remains unchanged.
- **Transport:** both client modes use SSE upstream. Streaming relays incrementally;
  unary output uses bounded accumulation of the same checked decoder. Text,
  reasoning, ordered tools, usage, finish state, and embedded errors have parity.
  Malformed/truncated streams, incomplete tools, and invalid/duplicate terminals fail closed.
- **Tool history:** preserve opaque `call_antigravity_v2_` IDs, names,
  arguments, and order. Account/session/signature/call fields and conversation-wide
  ordinal are checked. These are unkeyed context tags, not cryptographic provenance:
  deliberately recomputed tags are not detected. Signatures are preserved, never
  synthesized. Legacy v1 native histories require a new conversation; non-native
  Gemini behavior is unchanged.
- **Recovery:** only an initial pre-downstream-header 401 permits one same-account
  in-memory refresh and one replay, retaining project/catalog/session/request ID/body.
  Only the bearer changes. A second 401, refresh failure, account change, ambiguous
  send, parser/body error, or failure after headers/output/tools does not replay.
  Legacy refresh-grant-derived identity rotation fails closed. This recovery does
  not persist refreshed credentials; existing login/expiry-refresh writes are unchanged.
- **Lifetime:** dropping a streaming body or cancelling unary work drops the
  upstream body and releases admission. An independent subsequent request is
  admitted; cancellation does not create an automatic retry.

### Executed evidence

The full serial all-feature workspace suite after Plan 11-06 passed **2,637 tests**
with **2 pre-existing ignored tests**. All **40** tests selected by
`antigravity_native` passed with **0 ignored**. These are hermetic results, not live captures.

Plan 11-07's review added
`antigravity_native_origin_catalog_refuses_redirects_before_following`: it failed
before catalog redirect refusal and passed afterward for same-origin and
off-origin redirects. Final release gates passed **2,638 tests, 2 ignored**;
the seven native filters selected **41 passing tests**, none ignored. Parallel
replay tests also passed after isolating configuration reads and retaining exact
one-request assertions. Format, warnings-denied Clippy, the exact scope gate,
and the four-language documentation build passed.

| Claim | Named evidence |
| --- | --- |
| Exact destinations and redirect safety | `antigravity_native_origin_accepts_only_exact_canonical_or_loopback_targets`; `antigravity_native_origin_rejects_off_origin_redirect_before_bearer`; `antigravity_native_origin_bounds_redirect_loops_at_ten` |
| Captured account/project/catalog tuple | `antigravity_native_affinity_injected_resolver_once_and_tuple_distinct`; `antigravity_native_affinity_inflight_swap_keeps_a_tuple_immutable`; `antigravity_native_affinity_same_project_different_accounts_have_isolated_catalogs` |
| Exact model/effort admission | `antigravity_native_affinity_exact_admission_covers_full_tuple_matrix`; `antigravity_native_affinity_rejected_catalog_tuple_has_zero_inference_hits` |
| Envelope and scoped history | `antigravity_native_envelope_session_survives_history_growth`; `antigravity_native_tool_signature_real_router_roundtrip_and_rejection`; `antigravity_native_tool_signature_parallel_streaming_matches_unary` |
| Always-SSE, parity, incremental output and bounds | `antigravity_native_sse_real_loopback_both_downstream_modes`; `antigravity_native_sse_streams_before_upstream_completion`; `antigravity_native_sse_strict_failures_are_closed_in_both_modes`; `antigravity_native_sse_decoder_accepts_cap_and_rejects_cap_plus_one` |
| One-shot refresh with no writeback | `antigravity_native_401_first_preheader_401_refreshes_same_account_and_replays_once`; `antigravity_native_401_second_401_terminates_without_a_third_attempt`; `antigravity_native_401_account_swap_never_exchanges_another_accounts_grant` |
| No replay after uncertain send or output | `antigravity_native_401_ambiguous_send_timeout_terminates_without_replay`; `antigravity_native_401_tool_output_boundary_terminates_without_replay`; `antigravity_native_401_text_then_auth_error_never_refreshes_or_replays` |
| Cancellation and actual admission release | `antigravity_native_lifetime_stream_drop_releases_upstream_and_capacity`; `antigravity_native_lifetime_unary_cancellation_releases_upstream_and_capacity` |

### Confidence boundaries and exclusions

This evidence establishes narrow request-local invariants, not a generalized
cache/storage platform, new global mutable-state architecture, durable signature
store, cross-account project reuse, or a no-progress cancellation heuristic.
Google AI Studio Web is excluded. No generated wiki content is changed.
The scope gate checks exact Git-visible changed paths, documentation contract
tokens, excluded implementation markers, and credential-shaped added values.
Its entropy scan is heuristic, not proof that every possible secret is absent.
