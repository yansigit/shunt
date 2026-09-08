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
