# OpenCode Go exact-tuple evidence ledger

Inspected 2026-09-08 at pinned OpenCodex revision
055c3ecf0de6c35f59195fc434d6b08525182b7f. This is source provenance, not a current
availability claim. See [research](15-RESEARCH.md) for exact source locations.
The destination on absent candidates is the evaluation target, not evidence
that the candidate is served there. Neither Omen nor Muse 1.3 inherits wire or
capability facts from another candidate.

## Machine-readable records

```opencode-go-ledger
{
  "candidates": [
    {
      "model": "glm-5.3-flash",
      "destination": "https://opencode.ai/zen/go/v1",
      "wire": "openai-chat",
      "headers": "source adapter: Content-Type application/json; Authorization Bearer when key exists; Go-specific mandatory headers unknown",
      "context": "unknown on Go",
      "modalities": "unknown on Go; other-provider VLM claims do not establish this tuple",
      "effort": "source advertised low/high/max; compatibility remapping is source behavior, not admitted Shunt semantics",
      "tools": "source Chat adapter with reasoning_content replay on tool turns; exact Go conformance unknown",
      "filtering": "source Go/Zen tool-schema sanitization; exact model filtering unknown",
      "terminal": "source provider enables complete-tool-JSON EOF recovery; Shunt rejects this behavior and requires authoritative terminal",
      "session": "unknown: x-opencode-session is a Shunt promotion requirement, not evidenced upstream",
      "provenance": {
        "class": "source",
        "revision": "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "inspected": "2026-09-08",
        "source": "15-RESEARCH.md; pinned adjacent OpenCodex"
      },
      "capture": "none",
      "live": "none"
    },
    {
      "model": "omen-alpha",
      "destination": "https://opencode.ai/zen/go/v1",
      "wire": "unknown",
      "headers": "unknown",
      "context": "unknown",
      "modalities": "unknown",
      "effort": "unknown",
      "tools": "unknown",
      "filtering": "unknown",
      "terminal": "unknown",
      "session": "unknown: x-opencode-session is a Shunt promotion requirement, not evidenced upstream",
      "provenance": {
        "class": "source",
        "revision": "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "inspected": "2026-09-08",
        "source": "15-RESEARCH.md; pinned adjacent OpenCodex"
      },
      "capture": "none",
      "live": "none"
    },
    {
      "model": "muse-spark-1.3-contributor",
      "destination": "https://opencode.ai/zen/go/v1",
      "wire": "unknown",
      "headers": "unknown",
      "context": "unknown",
      "modalities": "unknown",
      "effort": "unknown",
      "tools": "unknown",
      "filtering": "unknown",
      "terminal": "unknown",
      "session": "unknown: x-opencode-session is a Shunt promotion requirement, not evidenced upstream",
      "provenance": {
        "class": "source",
        "revision": "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "inspected": "2026-09-08",
        "source": "15-RESEARCH.md; pinned adjacent OpenCodex"
      },
      "capture": "none",
      "live": "none"
    },
    {
      "model": "deepseek-v4-flash",
      "destination": "https://opencode.ai/zen/go/v1",
      "wire": "openai-chat",
      "headers": "source adapter: Content-Type application/json; Authorization Bearer when key exists; Go-specific mandatory headers unknown",
      "context": {
        "input": 1000000,
        "output": 384000,
        "provenance": "source generated/model-metadata.ts opencode-go row; not observed"
      },
      "modalities": "text (source Go noVisionModels)",
      "effort": "source advertised low/high/max; compatibility remapping is source behavior, not admitted Shunt semantics",
      "tools": "source Chat adapter with reasoning_content replay on tool turns; exact Go conformance unknown",
      "filtering": "source Go/Zen tool-schema sanitization; exact model filtering unknown",
      "terminal": "source provider enables complete-tool-JSON EOF recovery; Shunt rejects this behavior and requires authoritative terminal",
      "session": "unknown: x-opencode-session is a Shunt promotion requirement, not evidenced upstream",
      "provenance": {
        "class": "source",
        "revision": "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "inspected": "2026-09-08",
        "source": "15-RESEARCH.md; pinned adjacent OpenCodex"
      },
      "capture": "none",
      "live": "none"
    }
  ],
  "rejected": [
    {
      "reason": "failed-evidence",
      "disposition": "No positive evidence is available. A failed future probe remains rejected, not repaired into a pass."
    },
    {
      "reason": "unknown-fields",
      "disposition": "Absent or unknown fields cannot establish an exact tuple."
    },
    {
      "reason": "wrong-wire",
      "disposition": "A wire observed on another model/product does not establish this candidate."
    },
    {
      "reason": "family-inference",
      "disposition": "Muse 1.2 is not Muse 1.3; family or host subagent availability is not wire evidence."
    },
    {
      "reason": "unsupported-effort",
      "disposition": "Source aliases/remapping do not admit an unverified effort."
    },
    {
      "reason": "wrong-terminal",
      "disposition": "EOF completion synthesis is rejected even with complete tool argument JSON."
    },
    {
      "reason": "unverified-live",
      "disposition": "All four records are source-only; no credential-safe capture/live evidence exists."
    }
  ],
  "admitted": []
}
```

## Admission and session policy

Zero tuples are admitted. All explicit Go selections reject before Go credential
resolution or dispatch; later Go fallback candidates are removed without
invalidating a generic primary. Existing exact-native routing rejection remains
authoritative, while pinned Go paths are guarded explicitly.

Promotion requires exact model/destination/wire/effort/capabilities, hermetic
wire conformance, and credential-safe captured or live evidence together.
Every future admitted tuple must reuse its matching existing wire contract,
preserve strict bounded terminals, retry/cancellation ownership, and prove a
stable opaque conversation-scoped x-opencode-session only at the canonical
destination. No such header producer ships with this empty set.

The unresolved OGO-02 manual evidence assumption remains visible in
15-EDGE-COVERAGE.json. Planning disposition statuses there are not executed
test results. No live call, pricing inference, credential inspection, or
production configuration change was used to populate this ledger.
