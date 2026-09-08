# M16 — Command Code product separation

## Scope and evidence

Command Code's API-key Chat product and proprietary subscription product are
separate transports. The implementation is source-derived from OpenCodex
055c3ecf0de6c35f59195fc434d6b08525182b7f, inspected 2026-09-08, with synthetic
canonical-host TLS and gateway fixtures. This is not a claim of live provider
availability. Minimal-envelope acceptance and the currency of the pinned CLI
version 0.52.1 remain the Phase 16 opt-in live gate.

The source-derived subsystem retains the full OpenCodex MIT notice in shipped
[THIRD-PARTY-NOTICES.md](../THIRD-PARTY-NOTICES.md). Existing provider settings and
credential-file writeback behavior are unchanged; wiki is generated separately.

## Products and configuration

| Ordered upstream preset | Kind | Auth | Fixed/default endpoint |
| --- | --- | --- | --- |
| `commandcode` | `openai_chat` | `api_key`, `SHUNT_COMMANDCODE_API_KEY` | `https://api.commandcode.ai/provider/v1/chat/completions` |
| `command-code` | `command_code` | `command_code_oauth` | `https://api.commandcode.ai/alpha/generate` |

These presets must be declared in `[[upstreams]]`; they do not create automatic
legacy provider entries. Explicit legacy subscription tables use
`kind = "command_code"`, `auth = "command_code_oauth"` and the canonical base URL.
The API-key product retains generic Chat behavior, including its normal base URL
grammar and API-key injection. It cannot use subscription auth.

The subscription uses `SHUNT_COMMAND_CODE_TOKEN`; only true absence permits
read-only fallback to `~/.commandcode/auth.json` (`apiKey` string, optional
`userId`). Empty or invalid explicit values fail closed. File reads are bounded
at 16 KiB, credential lookup waiting at five seconds. Tokens are nonempty,
header-safe ASCII, at most 16,384 bytes (the file envelope has its own bound).
No login, whoami, refresh, repair, copy, migration, account rotation, or file
writeback is introduced. Tests inject temporary paths instead of changing HOME.

Canonical HTTPS `api.commandcode.ai:443` is required, with no userinfo, query or
fragment and only empty, `/` or `/alpha/generate` path. Validate before credential
lookup and immediately before bearer header construction; redirects are refused.
Private TLS/DNS injection exists only in crate-local tests, not public config.

## Request and session contract

The exact ten case-sensitive model/effort rows are documented in the maintained
[provider page source](../site/src/content/docs/providers/command-code.md) and
implemented in `src/adapters/command_code/efforts.rs`. All three reporter-only
rows (Luna, Gemini 3.7 Flash, vision-exp) remain excluded. Omitted effort is
supported; explicit request effort overrides the route default. Unknown values
and aliases are rejected before lookup/network rather than clamped.

The workspace-free envelope carries supported text, plaintext reasoning, images,
tool catalogs/choices, and authentic tool history. Missing recorded results are
execution-unknown, never invented success; orphans have visible carriers. Images
do not break pending tool/result adjacency. Duplicate or opaque identities fail.
Plaintext subagent and continuation histories are supported without executing
tools inside Shunt. Tool identities remain request-local.

Headers are a fixed allowlist. No raw conversation ID, project slug, prompt path
or workspace metadata is sent. Explicit conversation identity is SHA-256 derived
with credential scoping, domain separation and length delimiters, then UUID-shaped.
Without identity, use a request-local UUIDv4. Both remain stable within retries,
without cross-request state or disk persistence.

## Responses and lifetime

One checked semantic machine drives unary accumulation and incremental SSE.
Streaming retains no final content collection. Authentic tool IDs and strict
object arguments, text, reasoning, usage and supported finish reasons translate
consistently. Inclusive input usage is checked, then cache-read/write tokens are
separated without clamping or double accounting. Both usage and totalUsage are
validated when present; totalUsage has source-defined precedence.

Success requires an authoritative supported finish and clean framed EOF. Only
one compatible finish-step/finish companion is allowed. Malformed, unknown,
duplicate, late, failed and truncated records fail sticky; error finishes retain
reported usage but are not successful. There is no repair or EOF synthesis.

| Bound | Subscription limit |
| --- | --- |
| JSON record | 1 MiB, excluding LF/CRLF |
| Residual framing | record limit plus one optional CR |
| Total NDJSON wire | 32 MiB |
| Semantic bytes | 8 MiB |
| Tool arguments / tools / content blocks | 512 KiB each / 128 / 4,096 |
| Read idle / complete-record progress | 120 seconds each |

Partial-byte drips do not reset the absolute record-progress deadline; complete
records do. These are not whole-turn deadlines. The existing configurable inbound
request byte cap defaults to 32 MiB. Token counting uses the local estimate.

Generation uses ConnectOnly retry classification: genuine pre-connect failure
may retry with the same captured credential/session; post-send timeout, redirects,
output and tool activity cannot replay. Real sockets prove cancellation before
headers and mid-body in both modes closes upstream work and allows a subsequent
request to reuse the same one-slot gateway. Shared retry vocabulary is unchanged.

## Verification boundary

Focused API-key, translation, framing, canonical TLS, credential, retry and
cancellation tests are hermetic. CLI/curl smokes use fresh isolated homes and
non-production ports; ordinary subscription-binary smoke uses an explicitly
invalid token, so it never reads CLI credentials or generates against the network.
No live or GUI acceptance may be inferred from these checks. README and site
content ship in English, Korean, Japanese and Simplified Chinese in the same PR.
