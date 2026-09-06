---
title: Inbound Codex Endpoint
description: Point the OpenAI Codex CLI itself at shunt and load-balance it across a ChatGPT/Codex OAuth account pool.
---

Every other guide on this site routes **Claude Code** to another backend. shunt can also run the opposite direction: an opt-in raw OpenAI Responses passthrough that lets the **Codex CLI** point its own `base_url` at shunt and be load-balanced across a ChatGPT/Codex OAuth account pool. It is opt-in: when `[server.codex_endpoint]` is absent, none of those routes are registered and shunt's default HTTP surface is unchanged.

This builds on the same account pool as [Codex Multi-Account](/guides/codex-multi-account/) — selection, cooldowns, and refresh are shared unchanged. See the [M11 behavior specification](https://github.com/pleaseai/shunt/blob/main/docs/m11-inbound-codex-endpoint.md) for the full spec, including the exact failover table and reload semantics.

For the end-to-end setup walkthrough — enabling the endpoint, pointing the Codex CLI at shunt, client auth, account provisioning, and picking an entitled model — follow [Connect the Codex CLI](/guides/connect-codex-cli/). This page focuses on *what the endpoint does*; that guide is the *how to connect* checklist.

## Enable the endpoint

```toml
[server.codex_endpoint]   # all keys optional; default shown
provider = "codex"        # must be a chatgpt_oauth provider
collaboration = false     # opt-in bridge for translated V2 collaboration
```

```bash
shunt check
shunt run
```

Startup validation rejects an unknown `provider` or one that doesn't use `auth = "chatgpt_oauth"` — the endpoint injects the operator's Codex bearer, so only a `chatgpt_oauth` provider qualifies. See the [configuration reference](/reference/configuration/#servercodex_endpoint-optional) for every key and default, and [HTTP Endpoints](/reference/endpoints/) for the registered routes.

## Client analytics sink

The Codex CLI also posts product analytics to the base URL. shunt accepts both paths the CLI can produce:

- `POST /backend-api/codex/analytics-events/events`
- `POST /codex/analytics-events/events`

These routes use the same `[server.auth]` policy as the Responses routes but never forward telemetry upstream, because choosing one pooled account would misattribute the client event to that account. They always return `200 {}` after authentication, including for malformed, unreadable, or oversized bodies.

The payload and event properties are neither logged nor exported. shunt records only the sanitized `event_type` as the `event` attribute on the opt-in `shunt.codex_client_events` counter: names may contain lowercase ASCII letters, digits, `.`, `_`, and `-`, up to 64 bytes; invalid names become `other`, and unrecognized batches become `unparsed`. Without Sentry or OpenTelemetry metrics enabled, this is a pure discard sink.

## Point the Codex CLI at shunt

The Codex CLI always appends `/responses` to whatever base URL it uses, so either `~/.codex/config.toml` shape works:

**Mirror the ChatGPT backend's base URL:**

```toml
chatgpt_base_url = "http://127.0.0.1:3001/backend-api/codex"
```

**Or a custom model provider** (the top-level `model_provider` must select it, or the CLI keeps its built-in provider):

```toml
model_provider = "shunt"

[model_providers.shunt]
base_url = "http://127.0.0.1:3001/v1"
wire_api = "responses"
```

With the custom provider (add `requires_openai_auth = false` so the CLI needs no local login), the Codex CLI's own `~/.codex/auth.json` becomes irrelevant once pointed at shunt — the account comes from shunt's pool on every request. The `chatgpt_base_url` shape instead keeps the CLI in ChatGPT-login mode, so it still needs its local login and works only against an **ungated** endpoint: its ChatGPT bearer is not the configured shunt token, so `[server.auth]` rejects it.

## Client authentication

If shunt has [`[server.auth]`](/guides/shared-gateway/) configured — recommended for anything beyond loopback — present the client token **either** as an OpenAI-style Bearer key (`OPENAI_API_KEY` / a custom provider's `env_key`, the LiteLLM/llmgateway idiom) **or** as the `x-shunt-token` header:

```toml
# A. Bearer — built-in openai provider. Set the base URL in ~/.codex/config.toml,
#    NOT via the OPENAI_BASE_URL env var: the env var leaves the CLI's Responses
#    WebSocket pointed at wss://api.openai.com, so it bypasses shunt. See
#    "Point the Codex CLI at shunt" in the connect guide.
openai_base_url = "http://127.0.0.1:3001/v1"
```

```bash
export OPENAI_API_KEY="<shunt-token>"      # sent as Authorization: Bearer
```

```toml
# B. Header — a custom provider carries it (use env_http_headers to keep it out of the file):
[model_providers.shunt]
base_url = "http://127.0.0.1:3001/v1"
wire_api = "responses"
http_headers = { "x-shunt-token" = "<token>" }
```

Without `[server.auth]`, the endpoint is open to anyone who can reach it — acceptable for loopback or personal use, not for a shared gateway. The client's presented credential is used **only** to authenticate to shunt: it (and any `Authorization` the CLI happens to send) is stripped and never forwarded upstream. The `[server.admin]` credential header — `x-shunt-admin-token` by default, or whatever `[server.admin] header` names — is stripped too, since the admin surface authenticates on that slot and an admin credential can provision upstream accounts. So is the whole `cookie` header, because the admin surface also accepts a write-tier session cookie there; shunt keeps no cookie jar, so nothing upstream depends on it. `x-api-key` is stripped unconditionally too — even when `[server.auth]` is not configured — since the target provider is validated `chatgpt_oauth`-only at boot, so no inbound `x-api-key` value can ever be a valid upstream credential; a client whose `apiKeyHelper` sets both `Authorization` and `x-api-key` to the same key (as Claude Code's does) does not leak that key through the second slot. Because the inbound client is a real Codex CLI, the passthrough forwards its request headers verbatim (`version`, `originator`, `OpenAI-Beta`, `x-codex-*`, …) and swaps in **only** the selected pool account's `Authorization` bearer + `chatgpt-account-id`. See [Connect the Codex CLI](/guides/connect-codex-cli/#3-present-the-shunt-client-token-when-serverauth-is-set) for the full auth walkthrough.

## WebSocket transport

The three Responses paths accept both HTTP `POST` and authenticated WebSocket `GET` upgrades. Authentication completes before `101 Switching Protocols`. On a socket, `generate: false` warmups complete locally; live `response.create` frames reuse the HTTP account pool, force upstream streaming, and forward each SSE `data:` payload as a WebSocket text frame through the first terminal event. Replacing a turn or closing the socket cancels the active upstream body. Client frames and SSE events are limited to 4 MiB, sends apply bounded backpressure, and protocol/upstream failures use standalone `type: "error"` frames with only safe response metadata.

## Account provisioning

Reuses the same pool as [Codex Multi-Account](/guides/codex-multi-account/#configure-the-pool):

```bash
codex login
shunt login codex --name main
```

```toml
[[providers.codex.accounts]]
name = "main"
```

With no `[[providers.codex.accounts]]` configured **and an empty shunt account store**, the endpoint falls back to the single default `~/.codex/auth.json` credential — no pooling, no failover — so a single Codex login works the moment `[server.codex_endpoint]` is set. (The handler first scans the account store and pools any accounts it discovers, so imported store accounts still enable pooling.)

## What's different from `/v1/messages`

- **Native routes stay opaque.** For a Responses-native route, the inbound body and upstream response remain byte-for-byte passthrough. Translation is isolated from this path.
- **Compressed request bodies pass through.** Current Codex releases zstd-compress the request body when they talk to the ChatGPT backend, which includes the `chatgpt_base_url` shape pointed at this endpoint. The bytes and their `content-encoding: zstd` header are forwarded unchanged; shunt additionally decodes a copy in-memory only to read the request's `model` for its metrics, logs, and spans. A body shunt cannot decode still relays fine — only the `model` label degrades to `unknown`, with a warning naming the reason.
- **Exact model routing.** A unique exact `[models.upstream_model]` or `[[routes]]` declaration may select one Responses-native or Anthropic Messages provider. Prefix-only, non-exact, and unmatched models use the pinned native `[server.codex_endpoint].provider`; ambiguous declarations reject before any upstream request.
- **Anthropic translation is strict and bounded.** Exact Anthropic routes accept instructions, text and URL/data-URL images, function tools/calls/results, tool choice, generation controls, and reasoning effort. HTTP and WebSocket share one bounded JSON/SSE translator. It maps `end_turn`, `stop_sequence`, and `tool_use` to completed, `max_tokens` to incomplete, and malformed output, unknown stop reasons, stream errors, or premature EOF to failed. Input usage includes cache-read and cache-write tokens.
- **Collaboration is explicit.** `collaboration = true` lets exact Anthropic routes bridge declared V2 `collaboration` tools and plaintext `agent_message` task envelopes. Authorized calls are restored to the collaboration namespace in JSON and SSE responses. Native Responses traffic stays byte-for-byte opaque regardless of the flag.
- **Lossy inputs fail before dispatch.** Translated routes reject `previous_response_id`, encrypted reasoning or compaction state, hosted/custom tools, remote file ids, malformed tool relationships, and unsupported fields instead of guessing. Native Responses compaction and native passthrough are unaffected.
- **There is no hidden recovery.** Ciphertext-only translated agent tasks fail before credential resolution or network dispatch. shunt never decrypts, caches, persists, or makes a billable recovery request; provide plaintext history or use a native Responses route.
- **Exhaustion relays verbatim.** If every pooled account is tried and at least one upstream response came back, shunt relays that last response unchanged rather than re-shaping it into an Anthropic-style error, since a Responses client expects the raw shape it would have gotten from the real ChatGPT backend.
- **Quota-aware rotation is bounded.** A plain `429` is transient throttling. shunt suppresses an account for a finite internal cooldown only when a bounded response body contains exact structured hard-quota evidence (`usage_limit_exceeded` or `insufficient_quota`); malformed, ambiguous, oversized, or aborted bodies remain unverified. Valid `Retry-After` delta-seconds (including decimals, rounded safely) and HTTP-date values are honored within finite limits. When candidates are exhausted, the final upstream status, body, and safe `Retry-After` metadata remain visible. Account rotation is eligible only before output starts; unrelated route-level failover behavior is unchanged.
- **Native compaction stays opaque.** HTTP-only `POST /v1/responses/compact` uses the same authentication, request limits, native route decision, and account pool, forwarding the body and continuation fields byte-for-byte to a verified ChatGPT/Codex or canonical OpenAI compact endpoint. Malformed, ambiguous, translated, or unsupported targets fail before network dispatch. shunt does not decrypt continuation state, synthesize summaries, or store request history.
- **Gateway-owned errors are OpenAI-shaped.** When the failure is shunt's own — a bad or missing client token (`401`), an unresolvable pool with no upstream response (`502`), an oversized request body, or an unconfigured endpoint — shunt returns it in the OpenAI Responses error shape (`{"error":{"message":…,"type":…,"code":null}}`) with the same status code, so the Codex CLI parses it through its own error path instead of the Anthropic `{"type":"error",…}` envelope. Relayed *upstream* errors (429/4xx/5xx from the backend) still pass through verbatim.
- **Two inbound transports.** HTTP `POST` remains byte-faithful; WebSocket `GET` adds bounded event delivery and does not depend on the provider's outbound `websocket = true` setting.
- **Shared dispatch boundary.** HTTP and WebSocket use the same exact resolver and provider-aware credential filtering. Once output begins, there is no provider hop or replay; only account rotation before output is eligible.

## Security

- Gate this endpoint with `[server.auth]` on anything beyond loopback — the provider injects a real Codex bearer on every request.
- Nothing about the client's own credential reaches the Codex backend; the passthrough forwards the Codex CLI's own request headers verbatim and swaps in only the selected pool account's bearer + `chatgpt-account-id` (the shunt client-token header, the `[server.admin]` credential header, the whole `cookie` header, the internal `x-shunt-inbound-client` label, the client's `Authorization`/`chatgpt-account-id`, and `x-api-key` are all stripped, never forwarded).
- The route set is decided once at boot. Toggling `[server.codex_endpoint]` on or off at runtime logs a warning that a restart is required; a reload can still change which provider it targets.
