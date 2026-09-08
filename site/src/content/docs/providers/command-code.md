---
title: "Command Code: API key and subscription"
description: "Two separate Command Code products with independent credentials and protocols."
---

The names are deliberately different. These are ordered-upstream presets, not automatically created legacy provider tables.

| Preset | Kind / auth | Destination |
| --- | --- | --- |
| `commandcode` | `openai_chat` / `api_key` | `https://api.commandcode.ai/provider/v1/chat/completions` |
| `command-code` | `command_code` / `command_code_oauth` | `https://api.commandcode.ai/alpha/generate` |

## Configuration
```toml
[[upstreams]]
name = "anthropic"
provider = "anthropic"

[[upstreams]]
name = "cc-api"
provider = "commandcode"

[[upstreams]]
name = "cc-sub"
provider = "command-code"
effort = "high"

[[models]]
id = "cc-sub-glm"
[models.upstream_model]
cc-sub = "zai-org/GLM-5.3"
```

Add a separate model mapping to `cc-api` using a model ID verified for your API account. The subscription table below does not describe the API-key product. Keep the Anthropic upstream for unmapped models. Existing provider settings are unchanged.

## Credentials

API-key preset: set `SHUNT_COMMANDCODE_API_KEY`. Subscription preset: set `SHUNT_COMMAND_CODE_TOKEN`, or, only when that variable is absent, use the existing read-only `~/.commandcode/auth.json` file (`apiKey` string and optional `userId`). Empty or invalid explicit tokens fail without file fallback. The file is capped at 16 KiB; credential lookup waits at most five seconds. Tokens must be nonempty header-safe ASCII.

Shunt never logs in, calls whoami, refreshes, copies, repairs or writes this subscription credential file. There is no account rotation or credential-path/version override. `command_code_oauth` is exclusive to `command_code`. Subscription credentials require the canonical HTTPS host and port 443, with no userinfo, query or fragment; the configured path may be empty, `/` or `/alpha/generate`. Validation runs before lookup and before bearer-header construction; redirects are not followed.

## Exact subscription model / effort matrix

| Model ID (case-sensitive) | Explicit efforts |
| --- | --- |
| `deepseek/deepseek-v4-pro` | `high`, `max` |
| `deepseek/deepseek-v4-flash` | `high`, `max` |
| `zai-org/GLM-5` | `high`, `max` |
| `zai-org/GLM-5.1` | `high`, `max` |
| `zai-org/GLM-5.2` | `high`, `max` |
| `zai-org/GLM-5.2-Fast` | `high`, `max` |
| `zai-org/GLM-5.3` | `low`, `high`, `max` |
| `meta/muse-spark-1.2` | `low`, `medium`, `high`, `xhigh`, `max` |
| `meta/muse-spark-1.2-contributor` | `low`, `medium`, `high`, `xhigh`, `max` |
| `meta/muse-spark-1.1` | `low`, `medium`, `high`, `xhigh`, `max` |

Omitting effort is accepted. An explicit request effort overrides the route default; unsupported values, including `none` and `ultra`, fail rather than being clamped. Unknown IDs and reporter-only Luna, Gemini 3.7 Flash and vision-exp rows are not admitted.

## Translation and limits

Subscription requests carry supported text, plaintext reasoning, images and authentic tool call/result history in a workspace-free envelope. Plaintext subagent results and continuation history are supported; opaque state is rejected. Missing recorded tool results are marked execution-unknown, not invented successes. Tool catalogs and choices are explicit. No raw project path or conversation identifier is sent; an explicit conversation has a credential-scoped opaque session, otherwise a request-local random session is used.

Both products support unary and streaming Anthropic output. The subscription always consumes NDJSON incrementally: 1 MiB per JSON record (excluding LF/CRLF), 32 MiB total wire, 8 MiB semantic bytes, 512 KiB per tool's arguments, 128 tools and 4,096 content blocks. The default inbound request cap is 32 MiB, configurable with `server.limits.max_request_bytes`. Counting uses a local estimate.

Success requires a supported authoritative finish and clean framed EOF. Malformed, duplicate, late, unknown or truncated records fail; there is no repair or EOF-synthesized success. Usage is checked integer arithmetic; cache tokens are separated from inclusive input without double counting. Provider error finishes remain failures with reported usage. Only proven pre-connect failures may retry, retaining credential and session. Post-send failures and output/tool activity cannot replay. Cancellation closes upstream work and releases admission capacity.

The subscription has a fixed 120-second read-idle limit and a 120-second complete-record progress deadline; partial-byte drips do not reset the latter. No whole-turn deadline is implied. Gateway errors use the inbound protocol's error shape. The API-key product retains the [generic Chat contract](/providers/openai-chat/).

## Evidence boundary

This compatibility matrix and client version `0.52.1` are source-derived from pinned OpenCodex `055c3ecf`, inspected 2026-09-08. Hermetic TLS and CLI/mock tests are not live-provider availability proof. Minimal-envelope acceptance and version currency remain the Phase 16 opt-in live verification gate. There is no public origin bypass. See the [configuration reference](/reference/configuration/).

