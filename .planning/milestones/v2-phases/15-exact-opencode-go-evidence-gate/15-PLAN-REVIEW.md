# Phase 15 independent plan review

Reviewer: `/root/go_plan_checker_glm`, `opencode-go/glm-5.3-flash`, high.
Result: ISSUES FOUND — 7 blockers, 4 warnings, 2 informational findings.
Initial independent review; revision iteration 1 dispatched to Luna/high.
No implementation or verification pass is implied.

## Blockers

- B1: Cargo accepts only one positional filter; draft commands both violate
  grammar and confuse binary names with test-name filters. Use actual targets,
  named tests and failure on zero selection.
- B2: The tracer depends on identity introduced only by the following task.
  Make the first vertical slice executable; RED must be behavioral, not compilation.
- B3: Integration binaries cannot access crate-private CredentialResolver and
  router injection seams. Prefer crate-local cfg(test) tests to prove zero
  credential lookups and outbound requests without broadening public APIs.
- B4: Decision coverage is only 5/11. Materially cover and cite D-01, D-04,
  D-05, D-07, D-10 and D-11 as well as the already covered decisions.
- B5: Existing check_cli coverage is not an owned Go negative CLI smoke.
  Specify synthetic state, unsupported response, zero outbound observations and
  cleanup; keep private credential-counter proof separate and accurately labeled.
- B6: Literal `&amp;&amp;` is not a valid shell command separator here.
- B7: Plan 15-02 owns fifteen files, exceeding the plan file-budget gate.
  Split evidence and documentation into focused dependency-ordered plans.

## Warnings

- W1: Use canonical structured `must_haves.prohibitions`, through the prescribed
  descriptor-less serializer; top-level prose is not the contract.
- W2: Give eight edge rows explicit dispositions. Classified cases need criteria;
  unclassified OGO-02 remains visibly flagged, never silently resolved.
- W3: Specify actual symbols, product identity, canonical Go URL, environment
  choice and count_tokens ordering. Existing generic semantics remain unchanged.
- W4: Account for engineering docs, provider-guide links, navigation and all four
  maintained locales. Generated wiki is not manually edited.

## Informational

- I1: Preserve warm-cache `RUSTFLAGS=-Dwarnings` spelling in verification commands.
- I2: Evidence ledger explicitly rejects failed, unknown, family-only,
  unsupported-effort and wrong-wire tuples. Source-only evidence admits none.

Checker suggestions are not authority to weaken zero-lookup assertions, export
test APIs or reclassify locked decisions. Revision must preserve all context locks.

## Revision 1 independent review

Reviewer: `/root/go_revision_checker_glm`, `opencode-go/glm-5.3-flash`, high.
Result: ISSUES FOUND — 4 blockers, 4 warnings, 2 informational findings.
Count decreased from 11 to 8; revision iteration 2/3 is permitted, not dispatched.
Luna/high produced four revised plans. Decision coverage now passes 11/11;
command-path and failure-direction probes return ok but do not validate Cargo
grammar or per-file documentation assertions. All four structure checks fail.

### Binding findings for revision 2

```yaml
issues:
  - plan: "15-01..15-04"
    dimension: task_completeness
    severity: blocker
    required_property: "Every plan has required autonomous frontmatter."
    description: "All four revised plans omit autonomous; verify.plan-structure reports invalid."
    fix_hint: "Restore autonomous: true."
  - plan: "15-01"
    dimension: task_completeness
    severity: blocker
    required_property: "Every new compiled module has an owned declaration."
    description: "src/proxy/opencode_go_tests.rs has no declaration; src/proxy.rs is not owned."
    fix_hint: "Own src/proxy.rs and declare the cfg(test) module there."
  - plan: "15-01"
    dimension: key_links_planned
    severity: blocker
    required_property: "Product identity is concretely implementable without breaking unowned full ProviderConfig literals."
    description: "An unnamed private field breaks external full literals; blanket no-public-API-widening also contradicts an additive ProviderKind variant."
    fix_hint: "Choose and name an approved additive identity representation; keep test seams private and own every necessary match change."
  - plan: "15-01"
    dimension: verification_derivation
    severity: blocker
    required_property: "The first RED fixture compiles against current code and fails a specified behavioral assertion."
    description: "The new preset is unknown before implementation; the plan specifies neither a concrete config-acceptance assertion nor a current-code-compatible router fixture."
    fix_hint: "Specify a compilable explicit config-acceptance assertion followed by gateway GREEN evidence; never claim compile failures or generic URL inference as the Go tracer."
  - plan: "15-01..15-04"
    dimension: key_links_planned
    severity: warning
    required_property: "Prohibitions use projectProhibitions structured descriptor-less output."
    description: "Revised prohibitions are still plain string lists."
    fix_hint: "Use canonical serializer with flagged-unverified records and no check descriptors."
  - plan: "15-02,15-04,15-VALIDATION"
    dimension: task_completeness
    severity: warning
    required_property: "Cargo commands preserve actual libtest thread options."
    description: "Doubled -- makes --test-threads=1 a free filter rather than an option."
    fix_hint: "Put the filter before Cargo's single -- separator."
  - plan: "15-03"
    dimension: scope_sanity
    severity: warning
    required_property: "File budget is met or coupling explicitly justified."
    description: "Twelve files exceed the ten-file warning threshold."
    fix_hint: "Split locale ownership or justify coupled docs scope."
  - plan: "15-03"
    dimension: verification_derivation
    severity: warning
    required_property: "Verification fails if any required surface is missing required content."
    description: "rg alternation across files succeeds with one match in one file; locale rg -l has the same gap."
    fix_hint: "Use per-file assertions in a small deterministic test."
```

Informational: explicitly account for docs/ engineering guidance; define the
ledger's machine-readable frame rather than unspecified Markdown parsing.
Root additionally notes navigation/guide file ownership must match promised
actions, and CLI smoke must exercise actual request rejection rather than merely
an unrelated missing-credential startup failure.

No runtime edits, tests, live calls or support admissions occurred. Revision 2
dispatch controls were not exposed after the checker returned. Preserve these
drafts; do not mark planned or execute until revision and independent check pass.

## Revision 2 — written, independent check pending

Planner `/root/go_revision2_glm`, `opencode-go/glm-5.3-flash`, high, returned
PLANNING COMPLETE. Five plans now specify ProviderKind::OpenCodeGo without new
ProviderConfig fields; config-acceptance behavioral RED; owned cfg(test) module;
actual HTTP CLI rejection smoke with synthetic key; structured prohibition
records; split English and locale docs with per-file assertions. These are planner
claims awaiting independent semantic review, not accepted findings dispositions.

Root checks: all five verify.plan-structure results valid with no warnings;
decision coverage 11/11. Root removed one trailing space found by git diff --check.
No implementation, build, or test suite ran.

Native subagent dispatch controls were no longer exposed after the planner
returned. The user-approved Cursor CLI fallback lists `gpt-5.6-luna-high`.
Attempted an independent checker in read-only `--mode ask`, explicit high model,
through the isolation wrapper. It exited 1 before review with ActionRequiredError:
usage limit reached; reset reported as 2026-09-18. No spend-limit changes made.
Wrapper confirmed production OpenCodex config mtime/SHA and backup inventory
unchanged after help, model discovery and the rejected review invocation.

No checker verdict exists for revision 2; no issue-count or review-iteration
advance is inferred from the quota failure. Resume independent checking with an
available approved high-thinking candidate. Do not rerun the producing revision
or implement blindly. Still inspect serializer conformance, fallback filtering
versus whole-chain rejection, native inbound rejection ordering and docs guide
ownership rather than trusting structure-only checks.

## Revision 2 independent check — native Luna/high

`/root/go_revision2_check` returned ISSUES FOUND: four blockers, three warnings.
Count 8 -> 7 permits final revision 3/3; dispatched to `/root/go_revision3`,
Luna/high. This is not an implementation or verification pass.

Required repairs:

1. Make RED assertion explicitly current-code-compilable: assert config loading
   succeeds without referring to the new enum before its introduction. The
   reviewer overstated that Task 1 expressly names the new Rust enum; its text
   names the serialized kind. Clarification still prevents an invalid RED.
2. Own a cfg(test), crate-visible server injection seam or relocate the harness.
   The resolver trait is actually pub(crate) in auth/mod.rs; the private server
   builder/counter access is the genuine missing link. No public test exports.
3. Filter unsupported Go fallback candidates while preserving a valid generic
   primary; reject Go primary before all credential access. Separate Go lookup
   counts from legitimate generic-control lookups in mixed-chain assertions.
4. Preserve existing exact native Go rejection by resolve_native_inbound rather
   than claim an unreachable shared gate ran. Explicitly guard pinned Go paths.
   Any gate used by codex_endpoint must be crate-visible, not pub(super).
5. Specify canonical-destination and API-key config validation and fixtures.
6. Include four provider overview guides and their assertions in owned docs scope.
7. Update and execute existing ordered-preset/public-name tests.

Additional root corrections passed to the planner: use actual projectProhibitions
serializer, not prior noncanonical house shape; correct contradictory no-public-
addition prose and preserve `RUSTFLAGS=-Dwarnings`. No runtime files changed.

## Revision 3 final checkpoint — ISSUES FOUND; user decision required

Planner `/root/go_revision3` (gpt-5.6-luna, explicit high) produced six plans.
This verdict is root inline semantic review under the previously recorded
delegation fallback, not a fresh independent subagent verdict. No runtime
implementation or execution verification occurred.

Three blockers remain:

1. **Smoke cannot boot under its own config contract.** 15-01 Task 2 rejects a
   noncanonical Go destination, but 15-04 Task 1 overrides that destination to a
   loopback fixture and requires successful startup and an HTTP admission error.
   Preserve canonical-destination validation. Design CLI rejection evidence and
   hermetic no-egress evidence so each is observable without a public bypass or
   falsely attributing zero requests to a fixture the process cannot use.
2. **Dependency wave conflict.** 15-06 declares wave 4 and depends on 15-05,
   also wave 4. Both edit tests/opencode_go_docs.rs. Serialize the dependency;
   also move downstream 15-04 after the resulting guide wave.
3. **Task verification is vacuous or precedes its tests.** 15-03 Task 1 uses
   always-successful echo. Locale and guide Task 1 commands select functions
   introduced only in their respective Task 2. Cargo ordinarily succeeds with
   zero selected tests, despite the plans declaring that selection must fail.
   Own executable nonzero-selection checks and create the assertions before or
   with the content they verify; preserve meaningful RED/GREEN behavior.

Four warnings also require disposition:

1. 15-VALIDATION.md lacks its opening YAML delimiter; canonical frontmatter.get
   returned an empty object. Restore parseable validation metadata before gates.
2. 15-01 must_haves still names a pub(super) helper and shared exact-native gate
   despite its action correctly specifying crate visibility and retaining the
   existing native routing rejection. Align the acceptance claims with actions.
3. 15-03 assertions demand URL/key/admission prose in every English file,
   including navigation source. Scope navigation checks to entries/labels and
   support-claim checks to Go sections, preserving legitimate other-provider docs.
4. 15-02 calls every candidate wire Chat Completions while the pinned research
   leaves absent Omen/Muse candidates unknown. Distinguish supported source hints
   from unknown candidate facts; zero admission does not justify invented fields.

Issue count remains seven (3 blockers, 4 warnings). All three revision cycles
are consumed, so no fourth automatic revision or implementation is authorized
by this workflow. Request an adjusted approach from the user; recommended is
root-led targeted corrections with subsequent fresh checking, not accepting
inconsistent plans. Phase 15 remains unexecuted and Phase 16 remains pending.

Read-only command-path/failure-direction probes reported no findings, but those
presence checks do not disprove the semantic issues above. Existing ordered
preset tests were confirmed in source, not executed. Historical Phase 14 test
results are not Phase 15 verification. Production state was not loaded or changed.

## Approved approach adjustment — fresh review PASSED

User approved root-led targeted corrections followed by fresh checking. Root
corrected the seven findings: canonical CLI config with separately attributed
crate-local counters; serialized waves 1/2/3/4/5/6; combined docs RED/GREEN tasks
with actual count and mutation checks; restored validation frontmatter; aligned
native/helper semantics; scoped Go-section assertions/nav exception; unknown
wire for absent source candidates. No safety gate was removed.

Fresh reviewer `/root/go_root_correction_check` (gpt-5.6-luna, explicit high)
returned PASSED: all six plans valid, acyclic, complete OGO-01..04 coverage and
locked-decision alignment; no substantive blocker. Its two advisories were
clarified: EDGE schema remains unchanged with planned verification references
and null unresolved OGO-02; Clippy now explicitly sets RUSTFLAGS=-Dwarnings.
Phase execution may begin. This is a planning pass, not implementation proof.
