# Phase 15 runtime spot-review

Independent reviewer: `/root/go_runtime_review`,
`opencode-go/glm-5.3-flash`, explicit high thinking, 2026-09-08.
Scope: read-only review of plans 01/02, their summaries, source diffs and
surrounding call paths. This is not the final whole-phase review.

No meaningful runtime bug or safety hole found. Confirmed:

- Messages/count_tokens gate before credentials and dispatch.
- Exact native routing rejects Go independently; pinned/unknown native routing
  reaches the shared gate. WebSocket turns reuse that native forwarding path.
- Identity is explicit provider kind; canonical destination/auth/env validation
  is also applied on reload.
- Real router requests reach injected credential counters; generic controls
  exercise real HTTP traffic. Eight focused lib tests and two ledger tests exist.
- Ledger admits zero tuples and retains the unresolved OGO-02 assumption.

Low-severity evidence correction: canonical Go is HTTPS, while the fixture is
plain HTTP. Its HTTP-path counter is not independently discriminating against
a leaked TLS attempt. Root corrected comments and the 15-01 summary in
`3fd0149`. Credential counters and response assertions remain discriminating;
DNS pinning contains attempted provider traffic to loopback. No assertion was
removed or weakened.

Intended behavior: a Go primary rejects the entire chain rather than falling
through to a generic fallback. This follows the approved fail-closed decision.

Limitations: reviewer did not run tests, inspect credentials, make network calls,
or revalidate the pinned external source facts. Execution evidence remains in
the individual summaries; no live or GUI acceptance is claimed.
