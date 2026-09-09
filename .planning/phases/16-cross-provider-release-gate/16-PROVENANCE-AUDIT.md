# Phase 16 preliminary provenance audit

Read-only review by opencode-go/omen-alpha, high effort; root confirmed the
OpenCodex notice wording and Cursor source headers. This is planning input,
not release sign-off. No live calls or credential access occurred.

## Required correction: notice scope

THIRD-PARTY-NOTICES.md currently scopes its reproduced OpenCodex MIT notice
to Command Code. Cursor usage.rs, history.rs and kv.rs cite schema adaptation
from the same pinned revision 055c3ecf0de6c35f59195fc434d6b08525182b7f,
inspected 2026-09-07. Extend the shipped attribution to cover this material
and preserve distinct inspection dates. Plan 16-02 must implement and test
the actual notice coverage, not merely assert generic MIT text is present.

## Additional source to verify

src/adapters/cursor/agent.rs credits the MIT-licensed 1jehuang/jcode agent
transport. Establish the original license/source provenance before adding
its notice; do not invent copyright text or a pinned revision. Distinguish
pre-existing derivative code from milestone additions.

## Reviewed boundaries

The reviewer found no new live-capture claim, real secret in translated
fixtures, AI Studio Web implementation, or generated wiki edit. These are
preliminary source-review observations; final scope and sanitization tests
remain pending. Do not blanket-reject pre-existing OAuth writeback or
dependencies: compare the actual milestone changes and preserve approved
provider semantics.
