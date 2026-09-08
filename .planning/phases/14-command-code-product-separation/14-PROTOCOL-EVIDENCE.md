# Command Code protocol evidence — synthetic fixture contract

All source references below are the read-only adjacent OpenCodex checkout at
`055c3ecf0de6c35f59195fc434d6b08525182b7f`, accessed **2026-09-08**.
That revision and access date apply to each fact and table row. These are
source-derived compatibility facts, not current official API guarantees or live captures.

## Source-derived facts

- `src/providers/registry.ts:1981-2018`: API-key product `commandcode` uses
  Chat at `https://api.commandcode.ai/provider/v1/chat/completions`.
  `registry.ts:1221-1226`: subscription product `command-code` is distinct.
- `src/adapters/command-code.ts:620-659`: subscription POST destination is
  `https://api.commandcode.ai/alpha/generate`. Envelope: `config`, empty
  `memory`, null `taste`/`skills`, `permissionMode:standard`, `mode:agent`;
  `params` carries model, messages, tools, system, max_tokens (default 64000),
  stream:true always, optional temperature/reasoning_effort.
- Same adapter lines: bearer header; Content-Type application/json; User-Agent
  cli; x-command-code-version 0.52.1; x-cli-environment production;
  x-taste-learning false; x-co-flag false; x-session-id. Source optionally adds
  a working-directory-derived x-project-slug. `fetchCommandCode` refuses redirects.
- `src/oauth/command-code.ts:20-44`: CLI auth file `~/.commandcode/auth.json`
  contains required nonempty string `apiKey`, optional `userId`. Source imports
  it and validates through whoami; Shunt does neither import nor validation probe.
- `src/adapters/command-code.ts:253-278`: without an identity the source uses
  randomUUID; otherwise SHA-256 over an identity tag/value, rendered UUID-shaped.
  The source hash does not include credentials. First user text is its final
  identity fallback after explicit thread/replay/Cursor/cache identities.
- `src/adapters/command-code.ts:703-735` and
  `tests/command-code-provider.test.ts:643-655`: both finish-step and finish
  are terminal-class; source emits one done using the first. The concrete
  companion transcript is finish-step(finishReason:stop, usage) followed by
  finish(rawFinishReason:stop, totalUsage). finish also accepts usage.
  A finish reason of error is failure, with consumed usage retained, not success.
- `src/adapters/command-code.ts` parseStream/wireMessages: text-delta and
  reasoning-delta carry text; tool-call carries toolCallId/toolName/input or args;
  error carries error. Tool-call/result history is adjacent, missing results
  become non-executed error-text, orphan results become user text, result images
  follow as user image content. Source tolerates junk/SSE prefixes/unterminated EOF;
  those permissive behaviors are not adopted.
- `src/providers/command-code-efforts.ts:3-101`: the source rows below are not an
  account catalog. **Correction:** its reporter-only comment applies to all three
  adjacent entries at lines 39-50: deepseek/deepseek-v4-flash-vision-exp,
  gpt-5.6-luna, and google/gemini-3.7-flash. Git blame confirms all three were
  introduced together by e1e6ec04f4 and the explicit unverified notice by
  ad8ab4f702. The earlier plan incorrectly treated Luna and Google as admitted.
  All three remain excluded from subscription admission until better evidence.
  This does not constrain subagent model selection through other providers.

| Exact model ID | Exact efforts |
|---|---|
| deepseek/deepseek-v4-pro | high, max |
| deepseek/deepseek-v4-flash | high, max |
| zai-org/GLM-5 | high, max |
| zai-org/GLM-5.1 | high, max |
| zai-org/GLM-5.2 | high, max |
| zai-org/GLM-5.2-Fast | high, max |
| zai-org/GLM-5.3 | low, high, max |
| meta/muse-spark-1.2 | low, medium, high, xhigh, max |
| meta/muse-spark-1.2-contributor | low, medium, high, xhigh, max |
| meta/muse-spark-1.1 | low, medium, high, xhigh, max |

Attribution: OpenCodex MIT-licensed compatibility source. Its license notice:

    MIT License
    Copyright (c) 2026 opencodex contributors
    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:
    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.
    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.

## Gateway hardening

These are Shunt decisions (14-CONTEXT D-01..D-14, recorded 2026-09-08), not
claims about source permissiveness:

- Product selection is explicit: commandcode/OpenAiChat/API-key versus
  command-code/command_code/command_code_oauth. Distinct acquisition paths and envs.
- SHUNT_COMMAND_CODE_TOKEN precedes the CLI file; invalid explicit values fail
  closed. No refresh, repair, migration, credential-store copy, whoami or writeback.
  Plan 14-02 uses env only; 14-05 adds bounded read-only file fallback with private
  temporary-path injection for tests. OPENCODEX_HOME does not relocate the CLI home.
- Canonical HTTPS host/path is checked before credential lookup and again before
  dispatch. Invalid destinations cause zero lookups/requests; a returned redirect
  permits the initial credential snapshot only, never a follow-up/new lookup.
- Constant config:{} and no workspace/git/project metadata. Hash conversation
  identity together with credential identity; never use prompt text or shared
  cohorts as session identity. No-ID requests use a request-local UUID, stable
  across that request's replay only, explicitly not multi-turn stable.
- Exact case-sensitive model/effort admission; unknown or unsupported tuples fail
  before generation. No clamping, aliases, remote effort refresh or public version override.
- Strict bounded UTF-8 NDJSON and checked semantics. EOF without a supported
  terminal fails. First terminal is authoritative; one compatible finish-step→finish
  companion is allowed. Duplicate, conflicting or already-received trailing semantic
  content fails before success. Never claim inspection of future bytes after cancellation.
- Both streaming and bounded unary use the same checked machine. No synthetic
  success, JSON repair, ignored junk, or unclassified event tolerance. Pre-connect
  retries only; no replay after a possible send. Ownership releases transport/parser/capacity.

## Unverified live acceptance

- Minimal constant config:{} acceptance is unknown until the separate Phase 16
  live gate. Synthetic mocks cannot establish this.
- Version 0.52.1 is pinned source provenance, not a claim of current CLI version
  or current service acceptance. Phase 16 must validate it without automatic updates.
- Model and effort source rows do not establish this account's live availability,
  entitlements, quotas, tools, or long-context performance.
- Reporter-only unadmitted tuples: gpt-5.6-luna claims low/medium/high/xhigh/max;
  google/gemini-3.7-flash claims low/medium/high; deepseek/deepseek-v4-flash-vision-exp
  claims high/max. These are explicitly unverified upstream claims, not admitted facts.
- No live Command Code credentials or provider endpoint were read/called for this
  ledger. Successful private TLS router tests remain mock evidence, not GUI/live proof.
