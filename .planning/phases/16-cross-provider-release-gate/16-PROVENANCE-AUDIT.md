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

## Independent original-source follow-up (2026-09-09)

The root-delegated `release_license_source_check` (Omen, high) verified the
authoritative jcode repository and reported MIT copyright
`Copyright (c) 2025 Jeremy Huang`. Current inspected master revision:
`e65e47c31af2ab79346458ff1511bea533930b59`. The credited
`crates/jcode-provider-cursor-runtime/src/agent_transport.rs` first appeared at
`8a91c8fda939a648820d29edbb78d3d802a70a44` (2026-07-09), before Shunt's
pre-existing Cursor transport introduction `2039c27` (2026-07-16).

Use the inspected revision as present-day verification provenance, not a
claim about which revision the original Shunt author copied. That historical
copy revision remains unknown. The exact original license is available at
[jcode LICENSE](https://github.com/1jehuang/jcode/blob/e65e47c31af2ab79346458ff1511bea533930b59/LICENSE).

The reviewer also confirmed the local OpenCodex checkout at the already
pinned `055c3ecf0de6c35f59195fc434d6b08525182b7f` has the existing reproduced
MIT copyright text unchanged. Cursor inspection is 2026-09-07 and Command Code
inspection is 2026-09-08. This follow-up supports the plan 16-02 notice edit;
it is not final sign-off on the as-yet-unwritten release ledger or security
test artifact. No credentials or live requests were used in this review.
