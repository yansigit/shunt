# Phase 16: Cross-Provider Release Gate - Pattern Map

**Mapped:** 2026-09-08
**Files analyzed:** 7 likely artifact/test/doc surfaces (no runtime source changes are authorized by this phase)
**Analogs found:** 7 / 7 (every named analog below was verified with "git ls-files" as tracked source; the only non-tracked dependency, /tmp/shunt-phase12-isolated-run.cjs, is a process-isolation boundary, not a pattern source)

Phase 16 is a verification phase. It inventories, tests, documents, and
evidence-gates the already-implemented provider surface; it does not add
adapters, presets, or runtime behavior. Per D-02, the OpenCode Go rows stay at
zero admitted tuples: the Phase 15 "admitted: []" ledger is evidence of
rejection policy, not a support claim. Per the orchestrator constraint, the
research preset label "Cursor" describes the preset inventory; the actual
Cursor adapter wire destination is "https://agentn.global.api5.cursor.sh/agent.v1.AgentService/Run"
(src/adapters/cursor/agent.rs:58-59), and the ledger must record the wire destination,
never the research shorthand.

## File Classification

| New/Modified File | Role | Data Flow | Closest Tracked Analog | Match Quality |
|---|---|---|---|---|
| .planning/phases/16-cross-provider-release-gate/16-RELEASE-MATRIX.md (new) | evidence/ledger artifact | static transform (markdown + fenced JSON) | .planning/phases/15-exact-opencode-go-evidence-gate/15-EVIDENCE.md (validated by tests/opencode_go_evidence.rs) | exact |
| tests/release_matrix.rs (new; or a named extension in an existing release test file — prefer the smallest tracked seam) | test | request-response assertions over a static artifact | tests/opencode_go_evidence.rs | exact |
| site/src/lib/i18n.ts (modify) | config/i18n label table | static transform | itself; fix the OpenCode Go sidebar translations at site/src/lib/i18n.ts:70-74 | exact |
| site/src/content/docs/{,ko/,ja/,zh-cn/}guides/providers.mdx and providers/opencode-go.* (modify, wording only) | docs | static transform | per-file assertions in tests/opencode_go_docs.rs | exact |
| README/docs surfaces (conditional modify; only if a support claim or default is corrected) | docs | static transform | README.md, README.ko.md, README.ja.md, README.zh-CN.md (all tracked) | role-match |
| .planning/phases/16-cross-provider-release-gate/16-SMOKE.md (new; safe owned live-smoke preflight/records) | evidence artifact + harness notes | event-driven process records | tests/check_cli.rs::opencode_go_cli_negative (real-process, RAII-owned gateway) | role + flow |
| 16-VERIFICATION.md, 16-SECURITY.md, 16-UI-REVIEW.md (new phase records) | evidence records | static transform | same-named Phase 15 files | exact format |

## Pattern Assignments

### 16-RELEASE-MATRIX.md (evidence ledger, static)

**Analog:** .planning/phases/15-exact-opencode-go-evidence-gate/15-EVIDENCE.md
as parsed by tests/opencode_go_evidence.rs:1-140.

The release ledger should reuse the Phase 15 shape: a markdown artifact with
one fenced JSON block that a focused Rust test parses and validates. The
existing validator shows the conventions to copy — exact identity fields, a
required key list, provenance class/revision/date, and explicit
"capture: none" / "live: none" until real evidence exists:

 Rust excerpt (tests/opencode_go_evidence.rs:19-40):
 for key in ["wire", "headers", "context", "modalities", "effort", "tools",
             "filtering", "terminal", "session", "provenance", "capture", "live"] {
     if row.get(key).is_none() || row[key].is_null() {
         return Err(format!("missing {id}/{key}"));
     }
 }
 if row["provenance"]["class"] != "source"
     || row["provenance"]["revision"] != REV
     || row["provenance"]["inspected"] != "2026-09-08"
     || row["capture"] != "none"
     || row["live"] != "none"
 {
     return Err("false provenance".into());
 }

For Phase 16 the tuple identity is (provider, auth path, exact model, wire),
seeded from the preset table in src/config/presets.rs:19-110 (for example the
cursor preset: name "cursor", kind ProviderKind::Cursor, base_url
"https://api2.cursor.sh", auth AuthMode::CursorOauth, api_key_env None). This is
only a preset seed, not a runtime endpoint or exhaustive support inventory;
Cursor's active agent endpoint is separately pinned in src/adapters/cursor/agent.rs.
Include custom Gemini/Antigravity and generic routes from their actual contracts.

Each row must point at named tests and record "not_applicable" with rationale
for inapplicable scenarios (normal, stream, tools, terminal, malformed,
truncation, auth, cancellation, retry). Do not infer coverage from a passing
full suite or from model/subagent dispatch (D-02, D-11). The Go section keeps
the empty-admission assertion pattern (tests/opencode_go_evidence.rs:41-44):
"if v["admitted"] != json!([]) { return Err("admission must remain empty".into()); }"

### Focused release-matrix test

**Analog:** tests/opencode_go_evidence.rs (whole file, under 400 lines).

Copy its structure: a PHASE/path constant resolved from
env!("CARGO_MANIFEST_DIR"), fence-splitting parse, a pure validate(&Value)
function, a happy-path test, and a mutation-rejection test that removes or
corrupts each required field and asserts the validator fails
(tests/opencode_go_evidence.rs:60-73 shows the fence-splitting parse).

Prefer reusing existing provider-local suites as the matrix's evidence links
rather than duplicating them: tests/gemini_conformance.rs,
tests/openai_chat_conformance.rs, tests/command_code_api_conformance.rs /
tests/command_code_conformance.rs,
src/adapters/cursor/{protocol,request_isolation,history_lifetime,cancellation,router_parity}_tests.rs,
tests/{codex_websocket_fallback,failover,retry,passthrough,inbound_codex_endpoint,inbound_codex_websocket}.rs,
tests/opencode_go_{evidence,docs}.rs, and
tests/check_cli.rs::opencode_go_cli_negative. Minimal existing test reuse;
no generalized framework (D-09).

### Sidebar label fix (D-07)

**Analog:** site/src/lib/i18n.ts:70-74 (tracked).

The Phase 15 UI review flagged locale-suffixed OpenCode Go sidebar labels.
Current source:

 label: "OpenCode Go",
 translations: { ko: "OpenCode Go (한국어)", ja: "OpenCode Go (日本語)", "zh-cn": "OpenCode Go (简体中文)" },
 slug: "providers/opencode-go",

Fix the translations to natural locale labels (no suffix), matching the
pattern of adjacent entries such as MiniMax China (i18n.ts:63-67). Any
change must keep tests/opencode_go_docs.rs green; extend its per-file
assertion style only if the fixed label needs a regression token.

### Redundant-emptiness wording fix (D-07)

**Analog:** per-file locale assertions in tests/opencode_go_docs.rs:1-80.

The existing test locks exact zero-support tokens per locale (e.g. the ko
assertion pins "현재 지원되지 않습니다", "허용 집합은 비어", "자격 증명 조회",
and the negative token "OpenCode Go는 지원됩니다" which must stay absent).

Cosmetic rewording of the "empty admitted set is empty" phrasing must (a) keep
the zero-support meaning, (b) update the corresponding tokens/assertions in
tests/opencode_go_docs.rs (and tests/opencode_go_evidence.rs where it asserts
wording), and (c) preserve English/ko/ja/zh-cn parity. Do not weaken or
remove assertions (AGENTS.md boundary). Verify final anchors from built
site/dist/<locale>/... HTML, not from source intent.

### Safe owned live-smoke records (D-04–D-06)

**Analog:** tests/check_cli.rs::opencode_go_cli_negative (tracked;
tests/check_cli.rs:199-230).

The existing real-process smoke demonstrates the ownership pattern to copy for
any harness validation against an owned local destination: reserve a loopback
address, RAII-kill the child process, use an isolated temp home, and assert
explicit negative outcomes. Its ownership excerpt:

 struct Gateway(std::process::Child);
 impl Drop for Gateway {
     fn drop(&mut self) { let _ = self.0.kill(); let _ = self.0.wait(); }
 }
 let dir = TempDir::new("opencode-go-negative");
 let listener = TcpListener::bind("127.0.0.1:0").expect("reserve gateway address");

Every stateful invocation must run through the verified isolation wrapper
(node /tmp/shunt-phase12-isolated-run.cjs or an equivalently verified
wrapper), which snapshots production config mtime/SHA and backup inventory,
uses a fresh temp OPENCODEX_HOME and OPENCODEX_PORT=31987, and exits 90 on
mutation (16-RESEARCH.md, D-10). The wrapper itself lives outside the
repository; it is a process boundary, not a tracked analog, and PATTERNS.md
must not present it as source to copy. Live requests require full preflight
(credential presence without exposing contents, canonical destination, complete
cost/reasoning/input bounds, 60 s max, 128 output tokens max, 8 requests max
total, US$1 planned max); any missing bound produces an explicit skip/blocked
record, never a fabricated pass. A client-side max_tokens=128 does not bound
provider-side reasoning or billing.

### Phase verification records

**Analog:** .planning/phases/15-exact-opencode-go-evidence-gate/15-{VERIFICATION,SECURITY,UI-REVIEW}.md
(tracked).

Reuse the Phase 15 record format verbatim: per-requirement status with named
commands and outputs, explicit evidence-class separation (source-derived
fixture vs. capture vs. live vs. static/visual), and honest unavailability
records. Final gates mirror D-09 exactly (fmt, warnings-denied Clippy,
all-features workspace tests, site build/validation) with every command run
through the isolation wrapper and each using a named nonzero test filter.

## Shared Patterns

### Evidence-class separation
**Source:** 15-EVIDENCE.md + tests/opencode_go_evidence.rs
**Apply to:** every Phase 16 artifact — source-derived fixtures, actual
captures, live results, and static/visual checks are distinct fields that must
never be merged.

### Exact-tuple identity
**Source:** src/config/presets.rs:19-110
**Apply to:** ledger rows — no family inference; "wire" is the canonical
upstream URL/protocol, and the Cursor row records
https://agentn.global.api5.cursor.sh/agent.v1.AgentService/Run
(src/adapters/cursor/agent.rs), not the older preset base URL.

### Isolation and mutation tripwire
**Source:** tests/check_cli.rs::opencode_go_cli_negative + verified wrapper
**Apply to:** all stateful commands — fresh temp OPENCODEX_HOME, non-10100
port, RAII process ownership, byte-for-byte credential immutability checks,
stop on unexpected mutation.

### Docs parity and regression locks
**Source:** tests/opencode_go_docs.rs, site/src/lib/i18n.ts, AGENTS.md
Documentation rules
**Apply to:** all doc edits — English plus ko/ja/zh-cn updated in the same
change, built-anchor verification, wiki untouched, zero-support tokens
preserved.

## No Analog Found

None. All Phase 16 surfaces have tracked in-repo analogs; the only external
dependency (isolation wrapper) is classified as a process boundary above.

## Metadata

**Analog search scope:** tests/, src/config/, site/src/lib/,
site/src/content/docs/, .planning/phases/15-exact-opencode-go-evidence-gate/
**Files scanned:** ~20; 7 analogs selected (early stop)
**Pattern extraction date:** 2026-09-08
