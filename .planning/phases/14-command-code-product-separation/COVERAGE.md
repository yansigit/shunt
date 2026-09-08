# API Coverage — Command Code external surface

> Full coverage by default. Opt-outs are explicit, reasoned decisions.
> Deterministic API detector returned detected:true on the complete plan scope (Command Code hosted API,
> two distinct products). Source evidence: OpenCodex at
> 055c3ecf0de6c35f59195fc434d6b08525182b7f (dated 2026-08/09 facts; MIT
> attribution for substantial translated material).

## Product 1: `commandcode` API-key Chat (Phase 13 Chat surface reused)

| capability | decision | reason |
|---|---|---|
| POST /provider/v1/chat/completions | INTEGRATE | CCK-01/02 core turn path via preset over openai_chat |
| stream:true relay + keepalive | INTEGRATE | CCK-03 streaming scenarios inherit Phase 13 relay |
| tool calls/results (strict pairing) | INTEGRATE | CCK-03 reuses Phase 13 strict Chat pairing unchanged |
| usage/finish mapping | INTEGRATE | shared openai_chat response machine |
| GET /provider/v1/models catalog | OPT-OUT | public-only upstream catalog; account availability is not model capability; Phase 13 does not ship a model catalog surface and none is required by CCK/CCS |
| apiKeyValidation probe semantics | OPT-OUT | source labels validity unknown; a 401 probe cannot prove validity; no live probe in scope |
| parallelToolCalls:false discovery metadata | OPT-OUT | No catalog/discovery metadata port is required by CCK; generic Chat request/tool semantics remain unchanged. Do not claim this nonexistent preset field is enforced. |
| credential writeback/store copy | OPT-OUT | D-02 forbids creation/refresh/writeback of Command Code credential files |

## Product 2: `command-code` subscription NDJSON

| capability | decision | reason |
|---|---|---|
| POST /alpha/generate (stream, NDJSON) | INTEGRATE | CCS-01..08 core transport |
| text-delta / reasoning-delta events | INTEGRATE | CCS-05 incremental relay |
| tool-call events -> start/argument/end | INTEGRATE | CCS-04 tool assembly |
| finish-step + finish terminal lifecycle | INTEGRATE | D-08/grammar resolution: first terminal-class record authoritative, documented companion pair accepted, duplicates/conflicts fail closed |
| error events + error finishReason | INTEGRATE | CCS-06 explicit failure semantics |
| usage + cache detail integers | INTEGRATE | CCS-08 precision row (nonnegative integers only, checked arithmetic) |
| minimal constant config envelope | INTEGRATE | D-05/D-13: no cwd/git/workspace exfiltration; live acceptance labelled Phase 16 gate |
| envelope identity headers | INTEGRATE | D-05: UA cli, version 0.52.1, environment/taste/co-flag, x-session-id; dated source provenance |
| x-project-slug (cwd-derived) | OPT-OUT | workspace identity leakage; D-05/CONTEXT explicitly not needed |
| source SSE-prefix / null / junk tolerance | OPT-OUT | CCS-06 requires fail-closed; deliberately not ported (research §NDJSON) |
| source missing-finish EOF synthesis (done) | OPT-OUT | CCS-06: EOF without terminal fails, never synthesized |
| effort ladder enforcement (exact arrays, no alias/clamp/refresh) | INTEGRATE | D-05 pinned table from command-code-efforts.ts |
| effort alias/clamp/retry-without-effort | OPT-OUT | unknown/unsupported fails before generation; no remap |
| vision-exp/flash effort rows | OPT-OUT | reporter-only evidence #2647, unverified; not advertised |
| GET /alpha/whoami validation probe | OPT-OUT | source-side validation flow; Shunt validates via destination checks only, no live account probing |
| subscription env token source (SHUNT_COMMAND_CODE_TOKEN) | INTEGRATE | D-03 explicit env precedence, read-only |
| CLI file fallback ~/.commandcode/auth.json (read-only) | INTEGRATE | D-03 bounded read-only fallback, no write/copy/refresh |
| identity refresh / account rotation | OPT-OUT | D-10 forbids refresh/rotation |
| manual redirect following | OPT-OUT | D-04 refuse redirects; Policy::none |
| replay beyond ConnectOnly | OPT-OUT | D-10 conservative non-idempotent vocabulary only |

## Cross-product

| capability | decision | reason |
|---|---|---|
| product inference from bearer | OPT-OUT | D-01: never infer a product from a bearer |
| shared subscription acquisition with Chat | OPT-OUT | D-01: products never share acquisition |
| public version override config | OPT-OUT | research: no override justified merely because upstream has one; constant 0.52.1 only |

## Unresolved edge rows (flagged, not silently decided)

CCS-02, CCS-04, CCS-06 carry unclassified edge-probe rows in
14-EDGE-COVERAGE.json. Requirements are planned against explicit CONTEXT
decisions (D-04, D-07, D-08); the unclassified rows remain flagged for root
disposition at verification, not resolved by this planning.
