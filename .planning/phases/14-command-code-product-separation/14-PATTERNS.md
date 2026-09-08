# Phase 14: Command Code Product Separation — Pattern Map

**Mapped:** 2026-09-08
**Files analyzed:** 15 (7 new, 8 modified)
**Analogs found:** 12 / 15 (12 exact or near-exact, 0 partial, 3 no-analog)

All analog paths verified git-tracked via git ls-files (tracked-source gate
#3645). Line numbers reference the worktree at
/Users/user/.codex/worktrees/0466/shunt.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| src/adapters/command_code/mod.rs (new) | adapter (controller) | streaming + request-response | src/adapters/openai_chat/mod.rs | exact |
| src/adapters/command_code/ndjson.rs (new) | parser (middleware) | streaming | src/adapters/openai_chat/sse.rs | exact |
| src/adapters/command_code/request.rs (new) | model/transform | transform | src/model/openai_chat_request/endpoint.rs | role-match |
| src/model/command_code_response.rs (new) | model (checked machine) | streaming | src/model/openai_chat_response/checked.rs | exact |
| src/auth/command_code.rs (new) | auth resolver | request-response, read-only file I/O | src/auth/mod.rs (resolve_api_key, resolve_kimi_account) | exact |
| tests/command_code_translate.rs (new) | test | transform | tests/openai_chat_translate.rs | exact |
| tests/command_code_conformance.rs (new) | test | streaming + request-response | tests/openai_chat_conformance.rs | exact |
| subscription tracer (new CLI slice, D-12) | utility/diagnostic | request-response | none in-repo (see No Analog Found) | no analog |
| src/config.rs (modified) | config/enums | config | existing ProviderKind/AuthMode enums in the same file | exact |
| src/config/presets.rs (modified) | config/table | config | same file (preset table + tests) | exact |
| src/config/upstreams.rs (modified) | config/auth-map | config | same file (AuthMap, absorb_oauth_scope) | exact |
| src/auth/mod.rs (modified) | auth dispatch | request-response | same file (resolver dispatch arms) | exact |
| src/routing.rs (modified) | route/config | request-response | same file (AdapterKind, From of ProviderKind) | exact |
| src/proxy/failover.rs + src/proxy/capability.rs + src/adapters/mod.rs (modified) | dispatch/eligibility | streaming | same files (dispatch arm, incompatibilities arm, module list) | exact |
| docs: README.md + 3 locale copies, docs/ milestone note, site provider pages + locales, reference/configuration.md + locales | docs | n/a | site/src/content/docs/providers/openai-chat.md (Phase 13 precedent) | exact |

## Pattern Assignments

### src/adapters/command_code/mod.rs (adapter, streaming + request-response)

**Analog:** src/adapters/openai_chat/mod.rs — copy its overall shape verbatim:
module layout with cfg(test) submodules, an Adapter impl delegating to a free
forward function, a single OnceLock reqwest client with redirects refused, a
module-level RetrySafety constant, and two error mappers.

**Adapter trait + delegation** (analog lines 62-75):

    pub struct OpenAiChatAdapter;

    impl Adapter for OpenAiChatAdapter {
        fn forward<'a>(
            &'a self,
            state: AppState,
            route: Route,
            uri: &'a Uri,
            headers: &'a HeaderMap,
            body: RequestBody,
        ) -> AdapterFuture<'a> {
            Box::pin(async move { forward(state, route, uri, headers, body).await })
        }
    }

**Redirect-hardened client** (analog lines 77-91): redirect::Policy::none()
plus a read_timeout on one shared client — D-04's "refuse redirects" and D-09's
read deadlines are already enforced by this pattern.

**Retry safety constant** (analog lines 56-60): generation is non-idempotent,
so the Command Code subscription adapter must declare the same
RetrySafety::ConnectOnly (per D-10):

    pub(crate) const OPENAI_CHAT_RETRY_SAFETY: crate::retry::RetrySafety =
        crate::retry::RetrySafety::ConnectOnly;

src/retry.rs line 155 defines ConnectOnly; line 173 shows it fails over only
when error.is_connect() — pre-connect failures only.

**Gateway-owned error shapes** (analog lines 93-136): local_openai_chat_error
and map_openai_chat_error build typed AdapterError bodies and map statuses (429
-> rate_limit_error, 401/403 -> authentication_error, 4xx ->
invalid_request_error, else api_error). Per AGENTS.md issue #127, the error
shape must follow the inbound endpoint: Anthropic error JSON on /v1/messages,
OpenAI Responses error shape on the inbound Codex endpoint. The Chat adapter
currently emits Anthropic-shaped errors; copy the pattern, but let the planner
specify the inbound-protocol shape decision explicitly.

**Streaming/unary split** (analog line 33, lines 138-160+): unary accumulation
is bounded by MAX_OPENAI_CHAT_UNARY_RESPONSE_BYTES (32 MiB); streaming is
rendered by re-emitting translated SSE via append_sse_events and
append_protocol_error over crate::model::gemini::SseEvent. The command-code
upstream is NDJSON, but client-facing rendering can reuse those helpers.

### src/adapters/command_code/ndjson.rs (parser, streaming)

**Analog:** src/adapters/openai_chat/sse.rs — the closest existing bounded
incremental decoder. Copy the whole safety envelope, retargeted from SSE frames
to newline-delimited JSON records:

**Disposed-parser fail-closed** (analog lines 37-41, 49-51): once fail() clears
the buffer and sets disposed, every further push errors. Copy for D-08
"malformed -> fail closed, no recovery".

**Byte-bound enforcement** (analog lines 56-61, 76-81): over-limit frames error
with the limit in the message; the retained_candidate_len trick (lines 102-111)
catches a record that would exceed the cap before its delimiter lands — exactly
the D-09 residual-byte bound. For NDJSON the delimiter is a single newline, so
the partial-delimiter logic simplifies to tracking the current record length.

**EOF discipline** (analog lines 86-99): finish() fails when the required
terminal never arrived and when trailing unterminated bytes remain. Command
Code's strict contract (CCS-06: premature EOF is failure, never success) maps
directly: require a valid finish/finish-step record before finish() returns Ok,
and fail on any residual bytes. Deliberately do NOT port upstream's "synthesize
done at EOF" behavior (14-RESEARCH.md).

**UTF-8/JSON strictness** (analog lines 120-147): invalid UTF-8 and invalid
JSON are distinct errors. Command Code must additionally reject junk/null lines
and give unknown parseable records an explicit bounded classification (planner
decision), where the SSE decoder simply skips comment/empty lines.

### src/adapters/command_code/request.rs (model/transform)

**Analog:** src/model/openai_chat_request/endpoint.rs — the canonical
destination-grammar builder. Reuse this pattern for canonical
https://api.commandcode.ai/alpha/generate construction and for CCS-02 origin
validation:

**Canonical URL grammar** (analog lines 8-43): explicit scheme check, authority
nonempty without "@" (userinfo rejection, line 20), dot-segment rejection (lines
23-30), no query string (line 35), no fragment (line 38). D-04's "canonical
HTTPS origin and /alpha/generate, refuse redirects" wants this exact validation
plus an https-only policy and host pinning — extend, do not weaken.

**Envelope/params construction:** no direct analog builds a proprietary
envelope; the closest pattern is the whitelist translator approach documented at
the top of tests/openai_chat_translate.rs (lines 1-6): every accepted field must
map, every unsupported representation must be a typed request error, and the
outbound body must never carry a key outside the whitelist. Build the
config/memory/taste/skills/permissionMode/mode/params envelope as a fixed
constant config (never scan the filesystem — 14-RESEARCH.md), and emit params
from the translated Anthropic request: exact model, messages, tools as
name/description/input_schema triples, joined system, max_tokens, stream:true,
optional temperature and reasoning_effort. Validate model/effort tuples against
the exact evidence-backed table (D-05) before any credential access, failing
closed on unknown combinations.

**Headers** (from 14-RESEARCH.md source evidence): Authorization Bearer,
Content-Type application/json, User-Agent "cli", x-command-code-version,
x-cli-environment: production, x-taste-learning: false, x-co-flag: false, and
x-session-id. Never emit x-project-slug or any cwd-derived identity.

### src/model/command_code_response.rs (checked semantic machine)

**Analog:** src/model/openai_chat_response/checked.rs — copy the
CheckedChunk-returning validate_chunk pattern: every record is validated against
a strict schema, protocol violations are typed
(OpenAiChatSemanticError::protocol), and ambiguity fails closed (analog lines
48-52: "multiple OpenAI Chat choices have ambiguous ordering" becomes "duplicate
terminal is ambiguous" for command-code). Apply to:

- validate_usage (analog line 36) -> integer-only inputTokens/outputTokens plus
  cache-read/cache-write details.
- provider-error carrier (analog lines 16-34) -> the NDJSON error record and the
  credit-depletion body (success:false, error with code/status/message/docs),
  rendered as neutral redacted gateway errors.
- explicit lifecycle: exactly one trustworthy finish; finish-step then finish
  ordering must be resolved with grammar evidence before encoding (14-RESEARCH.md
  warns against self-confirming fixtures).
- tool assembly: bounded per-tool argument accumulation (D-09), authentic
  toolCallId/toolName, duplicate identities rejected, missing results carried as
  explicit non-executed error-text, orphan results retained (D-07).

Reuse the sibling-module structure of src/model/openai_chat_response/
(assembly.rs, checked.rs, unary_tools.rs, validation.rs) if a single file would
exceed the 500-line guideline in AGENTS.md.

### src/auth/command_code.rs (auth resolver, read-only)

**Analog:** src/auth/mod.rs. Two patterns to copy and one to explicitly not
copy:

**Env-first with typed absence error** (resolve_api_key, file lines ~511-531):

    fn resolve_api_key(name: &str, provider: &ProviderConfig) -> Result<String, AdapterError> {
        let env_name = provider.api_key_env.as_deref().ok_or_else(|| {
            auth_error(format!(
                "provider {name} uses auth = \"api_key\" but api_key_env is not set"
            ))
        })?;
        if let Ok(value) = env::var(env_name) {
            if !value.is_empty() {
                return Ok(value);
            }
        }
        ...
        Err(auth_error(format!("{env_name} is not set")))
    }

This is the template for D-03: an explicitly configured subscription env
variable is consulted first; empty is treated as absent; a present-but-invalid
explicit value must fail closed instead of silently falling through to the CLI
file. This differs deliberately from the analog, which falls through — the
planner must specify the distinction explicitly.

**Bounded read-only file fallback** (resolve_kimi_account, file lines ~445-464):
an account.credentials path override, then a store read via
KimiAuthStore::get_valid(). Command Code needs a much narrower variant: read
~/.commandcode/auth.json once, validate the schema (apiKey: nonempty string,
userId: optional string), and return the bearer — no store, no refresh, no
writeback (D-02). Wrap the read with with_credential_timeout (file lines
~539-557) so a stuck read fails the request, mirroring the
ANTIGRAVITY_CREDENTIAL_TIMEOUT discipline.

**Do not copy:** force_refresh_credential (file lines ~237-285) and any
writeback path. The subscription resolver must be read-only end to end.

### tests/command_code_translate.rs (pure translation tests)

**Analog:** tests/openai_chat_translate.rs. Copy the module-declaration style
(analog lines 10-14: path-attribute module includes for fixture subdirectories),
the header doc-comment convention (lines 1-6), and the machine-level assertions,
e.g. lines 47-58: drive process_chunk_checked, then transport_close_checked,
then final_json_checked, asserting the exact content-block JSON. Command Code
equivalents: envelope/params translation, tool-history carriers (D-07), and
effort-tuple rejection (D-05) — all without transport.

### tests/command_code_conformance.rs (wiremock conformance)

**Analog:** tests/openai_chat_conformance.rs. Copy:

- wiremock setup with MockServer and ResponseTemplate (lines 16-19, 28-34).
- Env-guard pattern for fixture credentials (lines 25, 52:
  EnvVarGuard::set with a mutex lock).
- The redirect-refusal test (lines 22-46): a 307 from the backend must yield
  BAD_GATEWAY with exactly one upstream request and zero redirect-target
  requests — a direct template for the CCS-02 redirect negative.
- The post-send timeout single-attempt test (lines 49-60): one send, no failover
  after the request may have reached upstream — template for CCS-07.

For CCS-01 (env/CLI matrix with file bytes/mtime invariance) and CCS-08 (real
socket cancellation), no direct analog exists inside the Chat conformance file;
borrow the repo's Phase 13 cancellation/redaction patterns noted in
14-RESEARCH.md, and run everything through the D-13 isolation wrapper
(node /tmp/shunt-phase12-isolated-run.cjs, non-10100 ports).

### Subscription tracer (D-12, new)

**No in-repo analog.** The closest harness style is
tests/openai_chat_conformance.rs, but a real-gateway tracer reaches the actual
host and must be a separately labeled, opt-in diagnostic — not a unit test. The
planner should scope it as a narrow owned-CLI slice that exercises
auth/destination/models for both products first (tracer-first delivery,
14-RESEARCH.md), never touching /Users/user/.opencodex or port 10100 (D-13).

### src/config.rs (modified — new kinds/auth modes)

**Analog:** the same file. Add a ProviderKind::CommandCode variant (subscription
NDJSON) following the documented-variant style at file lines 1765-1809; the
OpenAiChat variant (lines ~1797-1801) documents the endpoint and the auth
pairing in its doc-comment — do the same. Add AuthMode::CommandCodeOauth
following the KimiOauth style (lines ~1829-1833): document acquisition/storage
in the variant doc-comment, and state explicitly that it is read-only with no
refresh and no writeback (D-02). Keep the pairing invariant (subscription kind
requires the subscription auth mode) in config validation, the same way "Only
valid for kind = gemini" is documented for GoogleOauth.

### src/config/presets.rs (modified — two table rows)

**Analog:** the same file. Add rows following the preset struct at lines 19-90:

- commandcode: kind ProviderKind::OpenAiChat, base_url
  https://api.commandcode.ai/provider/v1, auth AuthMode::ApiKey, api_key_env
  Some(...) (CCK-01; Chat translation and the endpoint path-prefix support are
  reused untouched).
- command-code: kind ProviderKind::CommandCode, base_url
  https://api.commandcode.ai, auth AuthMode::CommandCodeOauth, api_key_env None.

Copy the test discipline at lines 152-174
(kimi_code_preset_uses_the_kimi_oauth_surface_and_leaves_kimi_untouched): each
new preset gets a test asserting its own surface AND that the sibling product
sharing its name prefix is untouched — directly applicable to commandcode vs
command-code. Also update the ordered-name assertions at lines 118-138.

### src/config/upstreams.rs (modified — auth map variants)

**Analog:** the same file. Add an AuthMap::CommandCodeOauth variant shaped like
KimiOauth (lines 87-92) only if the planner keeps account-scope syntax; the
subscription resolver is single-credential, so the simplest faithful variant is
a flag-less variant like XaiOauth (line 93). Route it through absorb (lines
106-137) — if account scoping is supported, reuse absorb_oauth_scope (lines
140-178) including its account-xor-accounts conflict and empty-list errors.
normalize (lines 180-246) needs no structural change: preset lookup already
flows kind, base_url, auth, and api_key_env into ProviderConfig.

### src/auth/mod.rs (modified — resolver dispatch)

**Analog:** the same file. Add a Credential::CommandCode variant carrying one
access token, alongside Credential::KimiOauth (Debug impl, lines 79-90), plus a
dispatch arm calling the new read-only resolver. Use the token_env early-return
shape from resolve_kimi_account (file lines ~444-464) for the env-source path
and the plain path read for the CLI-file fallback.

### src/routing.rs (modified — adapter kind)

**Analog:** the same file. Add AdapterKind::CommandCode to the enum (after
OpenAiChat, lines 10-21) and the From-of-ProviderKind arm (lines 23-40; line 30,
ProviderKind::OpenAiChat => AdapterKind::OpenAiChat, is the exact template).
Route (lines 42-50) already carries effort, which D-05 validation consumes.

### src/proxy/failover.rs, src/proxy/capability.rs, src/adapters/mod.rs (modified — dispatch + eligibility)

**Analog:** the existing arms in the same files.

- src/proxy/failover.rs lines 360-386: add the AdapterKind::CommandCode arm
  dispatching to crate::adapters::command_code::CommandCodeAdapter. Lines 285-291
  (the count-tokens adapter list plus the OpenAiChat-specific
  CountTokens::Estimate override at lines 22-27 of that excerpt) decide whether
  the new kind joins token counting and which honest estimator applies.
- src/proxy/capability.rs lines 75-80+: add the CommandCode arm to
  incompatibilities. The eligibility matrix test at line 236
  (eligibility_matrix_matches_existing_adapter_fidelity) must be extended, not
  bypassed. Command Code knowns: tools yes, effort yes (with exact-tuple gating),
  1m-context unknown (the same conservative "1m-context-unknown" answer).
- src/adapters/mod.rs lines 10-15: add pub mod command_code; alongside the other
  adapters. The with_admission wrapper (lines 24-41) already ties admission
  guards to streamed bodies — D-11's "capacity through ownership" is satisfied
  by routing the new adapter's responses through it like every other adapter.

### Docs surfaces (modified, same PR)

**Analog:** site/src/content/docs/providers/openai-chat.md (Phase 13's provider
page) for structure; site/src/content/docs/reference/configuration.md for
config-key documentation; a new docs/ milestone note following
docs/m15-kimi-oauth.md style for the subsystem record. Per AGENTS.md, ship
English plus README.ko.md, README.ja.md, README.zh-CN.md and the site
ko/ja/zh-cn copies in the same PR; fragment links in locale pages must be
verified against built site/dist locale ids, else link the English page. Do not
hand-edit wiki/ (generated).

## Shared Patterns

### Fail-closed bounded decoding
**Source:** src/adapters/openai_chat/sse.rs (Decoder, lines 13-100)
**Apply to:** src/adapters/command_code/ndjson.rs and the checked response
machine — disposed-on-error, byte bounds (record plus cumulative), strict
UTF-8/JSON, EOF-is-failure.

### ConnectOnly retry plus redirect refusal
**Source:** src/adapters/openai_chat/mod.rs lines 56-91; src/retry.rs lines
142-173
**Apply to:** the subscription adapter (D-04/D-10); the API-key preset inherits
Chat's existing safety automatically.

### Env-first, file-fallback, read-only credential resolution
**Source:** src/auth/mod.rs resolve_api_key (~511-531), resolve_kimi_account
(~445-464), with_credential_timeout (~539-557)
**Apply to:** src/auth/command_code.rs; explicit-invalid must fail closed
(differs deliberately from the env/file fall-through in resolve_api_key).

### Canonical destination grammar
**Source:** src/model/openai_chat_request/endpoint.rs lines 8-60
**Apply to:** /alpha/generate construction and CCS-02 origin validation (extend
to https-only plus host pinning; never relax the userinfo/query/fragment
rejections).

### Gateway-owned typed errors with inbound-protocol shape
**Source:** src/adapters/openai_chat/mod.rs lines 93-136
**Apply to:** all command-code local errors; select Anthropic vs OpenAI
Responses error shape by inbound endpoint per AGENTS.md issue #127.

### Table-driven config plus sibling-preservation tests
**Source:** src/config/presets.rs lines 19-90 (table), 152-174 (tests)
**Apply to:** both new presets; no hardcoded provider logic anywhere.

### Conformance harness conventions
**Source:** tests/openai_chat_conformance.rs lines 1-60 (env guards, mutex lock,
wiremock, redirect/timeout tests); the D-13 isolation wrapper for every
cargo/test/smoke tree.
**Apply to:** tests/command_code_conformance.rs and the tracer.

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| subscription tracer (D-12) | diagnostic utility | request-response (live) | No in-repo live-gateway tracer exists; it must be newly scoped, opt-in, and isolation-wrapped. Borrow harness style from tests/openai_chat_conformance.rs. |
| proprietary envelope/NDJSON wire fixtures | fixtures | transform/streaming | The command-code wire protocol (envelope, finish-step/finish lifecycle, credit-depletion body) has no Shunt-side precedent; fixtures must be built from dated upstream source evidence with explicit provenance, never labeled as live captures (14-RESEARCH.md). |
| effort-tuple admission table | config/validation | transform | Existing effort handling passes the value through Route; no exact model-to-effort ladder validation exists yet. Keep it narrow and table-driven per AGENTS.md, sourced from the pinned upstream evidence in 14-RESEARCH.md. |

## Metadata

**Analog search scope:** src/config.rs, src/config/{presets,upstreams,session}.rs, src/auth/mod.rs, src/routing.rs, src/proxy/{failover,capability}.rs, src/retry.rs, src/adapters/ (including openai_chat/), src/model/openai_chat_request/ and openai_chat_response/, tests/
**Files scanned:** ~25 candidates, 12 selected as named analogs
**Tracked-source gate:** all named analogs verified with git ls-files (non-empty)
**Pattern extraction date:** 2026-09-08
