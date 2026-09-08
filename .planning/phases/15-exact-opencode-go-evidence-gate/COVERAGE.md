# API Coverage — OpenCode Go

> Full coverage by default. Opt-outs are explicit because the phase ships an empty admitted tuple set.

| capability | decision | reason |
|---|---|---|
| canonical Chat Completions request | INTEGRATE | Validate only through the existing Chat contract if a tuple is later promoted. |
| streaming response and authoritative terminal | INTEGRATE | Strict terminal behavior is part of the evidence gate. |
| bounded non-streaming response | INTEGRATE | Reuse the existing bounded Chat accumulation contract. |
| tools and paired tool results | INTEGRATE | Candidate evidence must prove exact tool behavior before admission. |
| usage and finish semantics | INTEGRATE | Required OGO-01 evidence fields. |
| authentication and destination binding | INTEGRATE | Credential-safe verification is required for promotion. |
| opaque x-opencode-session header | OPT-OUT | No candidate has captured/live acceptance evidence; record conditional OGO-03 contract only. |
| Responses wire | OPT-OUT | No Go candidate has exact Responses evidence; dormant multi-wire dispatch is out of scope. |
| Anthropic wire | OPT-OUT | No Go candidate has exact Anthropic evidence; dormant multi-wire dispatch is out of scope. |
| live model roster discovery | OPT-OUT | Live discovery is not sanitized wire evidence and broad sweeps are prohibited. |

## Source audit

| source | item | plan | status |
|---|---|---|---|
| GOAL | Exact OpenCode Go combinations are exposed only after proof; zero tuples valid | 15-01, 15-02 | COVERED |
| REQ | OGO-01 dated complete evidence records | 15-02 | COVERED |
| REQ | OGO-02 hermetic + credential-safe admission, empty set valid | 15-01, 15-02 | COVERED |
| REQ | OGO-03 matching wire/session contract after proof | 15-02 | COVERED |
| REQ | OGO-04 fail before credentials/network | 15-01 | COVERED |
| RESEARCH | Four candidates remain candidate-only; omen-alpha and muse-spark-1.3 have zero pinned evidence | 15-02 | COVERED |
| RESEARCH | Strict terminal policy rejects EOF even with complete JSON | 15-01, 15-02 | COVERED |
| RESEARCH | Explicit product marker must survive preset expansion | 15-01 | COVERED |
| RESEARCH | No speculative session header or multi-wire dispatch while empty | 15-02 | COVERED |
| RESEARCH | Isolated wrapper, non-10100 ports, and production-state invariants | all tasks | COVERED |
| CONTEXT | D-01 additive opt-in and backups | 15-01 | COVERED |
| CONTEXT | D-02/D-03 explicit identity and pre-credential exact gate | 15-01 | COVERED |
| CONTEXT | D-04/D-05 candidate/provenance separation | 15-02 | COVERED |
| CONTEXT | D-06 empty admission and no dormant dispatch | 15-01, 15-02 | COVERED |
| CONTEXT | D-07 conditional opaque session only after proof | 15-02 | COVERED |
| CONTEXT | D-08 strict terminal/retry/cancellation | 15-01, 15-02 | COVERED |
| CONTEXT | D-09 isolated commands and production-state safety | all tasks | COVERED |
| CONTEXT | D-10 tracer, gates, docs parity | 15-01, 15-02 | COVERED |
| CONTEXT | D-11 no live claim or credential inspection | 15-02 | COVERED |
