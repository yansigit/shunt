# Phase 15 — Root plan preflight

Status: revision required; independent checker not run.

Luna/high produced two plans and passed frontmatter/structure checks. Root
then ran verify-command-paths, verify-failure-directions and decision-coverage.
The first two probes returned no findings, but they do not validate Cargo's
argument grammar or actual selection. Decision coverage failed: 5/11 covered.

## Required revisions before execution

1. 15-01 Task 1 has three positional Cargo filters. Cargo accepts one; use
   separately runnable actual targets/filters. Every filter must select nonzero
   tests. Missing compile symbols are not behavioral RED evidence.
2. The tracer task adds admission behavior before Task 2 supplies the product
   identity on which it depends. Make the first vertical task genuinely executable
   through config normalization and the actual gateway, including its identity.
3. Cite and implement D-01, D-04, D-05, D-07, D-10 and D-11 in scanned task/truth
   surfaces. Do not move locked requirements to discretionary/informational status.
4. Use runnable shell separators, not literal `&amp;&amp;`, in automated commands.
   Preserve exact warm-cache RUSTFLAGS spelling `-Dwarnings` for the full gate.
5. Specify concrete product-marker/gate symbols, canonical URL, env choice and
   exact test ownership. Verify the injectable resolver/client seams are usable
   from the planned integration targets; cfg(test) lib seams are not exported
   to integration test binaries. Include all module declarations in file ownership.
6. 15-02 claims an owned CLI smoke but only lists a general check_cli target.
   Specify and create a real synthetic isolated negative smoke with assertions
   for unsupported status, zero credential/network access where observable,
   and process cleanup. Do not label an unrelated existing test as smoke proof.
7. Restore canonical `must_haves.prohibitions` structured descriptor-less entries
   through the prescribed serializer; the top-level prose list is not that contract.
   Give all eight edge-probe items explicit dispositions, leaving unclassified
   OGO-02 flagged. Name actual produced symbols and correctly rooted key links.
8. Consider docs/ engineering note, four-locale provider-guide links and navigation
   alongside provider/config/README pages. State the disposition of each docs surface.

## Dispatch status

Planner completed successfully. Host spawn/follow-up controls were not exposed
when root proceeded to the independent checker. No GLM checker or Luna revision
was launched, no independent review pass is claimed, and no runtime edits/tests
were made in this preflight. Candidate models remain Omen/high, GLM/high and
Luna/high. Resume independent check then revision; do not execute these drafts.
