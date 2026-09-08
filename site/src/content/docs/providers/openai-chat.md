---
title: OpenAI-compatible (Chat Completions)
description: Route mapped models to any OpenAI Chat Completions endpoint with an API key.
---

**OpenAI-compatible (Chat Completions)** is a generic provider kind, not a named provider:
`kind = "openai_chat"` points shunt at any backend that serves the OpenAI Chat Completions API
(`POST /chat/completions`), and shunt translates Claude Code's Anthropic Messages request into
that shape — streaming included. Custom backends declare `kind`, `base_url`, and an API-key
credential explicitly. The separate [Command Code API product](/providers/command-code/)
has a `commandcode` preset using this kind; it is not the subscription transport.

## Configure the upstream

```toml
[[upstreams]]
name = "anthropic"
provider = "anthropic"   # keep Anthropic as the default for unrouted models (e.g. claude-*)

[[upstreams]]
name = "chat"
kind = "openai_chat"
base_url = "https://api.example.com/v1"
auth = { mode = "api_key", env = "CHAT_API_KEY" }

[[routes]]
model = "gpt-5.4"
provider = "chat"
```

Ordered `[[upstreams]]` replace shunt's built-in providers, so the config must declare the
`anthropic` default it still falls back to (`server.default_provider` defaults to `anthropic`).

The legacy `[providers.chat]` table form remains supported: set `kind`, `base_url`,
`auth = "api_key"`, and `api_key_env = "CHAT_API_KEY"` instead of the auth map. Do not mix
`[[upstreams]]` and `[providers.*]` in one file.

`kind = "openai_chat"` accepts only `auth = "api_key"`. Startup refuses any other credential
mode — the adapter injects the configured key per request and has no other credential path.

## Credentials

```bash
export CHAT_API_KEY='...'
```

Never write the key into the config. `shunt check` validates the config's structure but does not
read the key's value — if `CHAT_API_KEY` is unset, the first request routed to `chat` returns an
authentication error.

## Base URL grammar

`base_url` must be a plain `http://` or `https://` root: no query string, fragment, userinfo
(`user:pass@`), dot path segments, whitespace, or backslashes. shunt appends exactly one
`/chat/completions` path — a root that already ends in `/chat/completions` is kept as-is — so
`https://api.example.com/v1` and `https://api.example.com/v1/chat/completions` are equivalent.
`shunt check` validates the same grammar at boot.

## What the adapter carries

The translation is a deny-by-default whitelist, so an unsupported request fails closed with a
typed 400 instead of silently degrading:

- **Text turns** (system, user, assistant) with any of `max_tokens`, `temperature`, `top_p`,
  and `stop_sequences`. Unsupported top-level request fields are refused. A single text payload is
  capped at 8 MiB of UTF-8.
- **Images** as base64 data or URLs in user messages.
- **Tools**: declarations, `tool_choice`, and paired `tool_use`/`tool_result` turns, with a
  request-local id registry so concurrent requests cannot share state.
- **Streaming** translated to Anthropic SSE, with usage preserved: streaming requests force the
  upstream `stream_options.include_usage` contract. Aggregated unary responses are capped at
  32 MiB, and the streaming machine retains at most 8 MiB of semantic bytes per turn.

## Failure and cancellation semantics

Generation POSTs are never re-dispatched once the request could have reached the upstream, and
redirects are refused outright so the injected bearer cannot be carried off the configured origin.
Non-success upstream statuses surface as gateway-owned errors; a read idle longer than 120 seconds
fails the turn. Cancelling a request closes the upstream connection and releases the admission
slot.

These guarantees are covered by synthetic conformance fixtures in the repository's test suite; no
live-provider behavior is claimed or verified here.

## Protocol limits

Tool arguments are buffered per request and parsed once at the finish boundary; tool blocks
follow `message_start` in first-arrival order. The limits are 128 calls and 1 MiB of arguments
per call. SSE event payload and incomplete residual payload each have a separate 8 MiB limit
(excluding framing delimiters). The cumulative 8 MiB semantic budget counts text, reasoning,
and tool identity/arguments, not the residual buffer.

Success requires a supported finish reason and `[DONE]`, not EOF. `stop`, `length`, and
`tool_calls` map to `end_turn`, `max_tokens`, and `tool_use`; unsupported reasons fail.
Already-decoded trailing frames/residual bytes fail; shunt does not wait for future bytes or
HTTP EOF after the terminal. Text already delivered cannot be retracted. Token counters must
be integers in `0..=i64::MAX`.

Plaintext thinking is supported; `reasoning` and `reasoning_content` response fields are
provider extensions, not a universal OpenAI contract. Conflicting, signed, or redacted
reasoning is refused. The read-idle timeout is fixed at 120 seconds with no new configuration
key. `count_tokens` always uses the local estimate for this adapter, even when another
provider strategy is configured. Credential files are never written by this adapter.

## Verify

```bash
shunt check    # -> config ok
shunt run
curl -sS http://127.0.0.1:3001/v1/messages \
  -H 'anthropic-version: 2023-06-01' \
  -H 'content-type: application/json' \
  -d '{"model":"gpt-5.4","max_tokens":16,"messages":[{"role":"user","content":"Reply with OK."}]}'
```
