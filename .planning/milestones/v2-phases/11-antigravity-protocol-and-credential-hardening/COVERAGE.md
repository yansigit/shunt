# API Coverage — Google Cloud Code Assist (native Antigravity)

> Full coverage by default. Opt-outs are explicit, reasoned decisions. This matrix covers the repository-proven Cloud Code Assist surface used by Shunt; the internal API is not treated as a discoverable public API beyond those proven calls.

| capability | decision | reason |
|---|---|---|
| OAuth authorization (`accounts.google.com/o/oauth2/v2/auth`) | INTEGRATE | |
| OAuth token exchange and refresh (`oauth2.googleapis.com/token`) | INTEGRATE | |
| OAuth account identity (`www.googleapis.com/oauth2/v2/userinfo`) | INTEGRATE | |
| Project discovery (`v1internal:loadCodeAssist`) | INTEGRATE | |
| Project onboarding (`v1internal:onboardUser`) | INTEGRATE | |
| Onboarding operation polling (`v1internal/operations/*`) | INTEGRATE | |
| Exact model catalog (`v1internal:fetchAvailableModels`) | INTEGRATE | |
| Agent inference SSE (`v1internal:streamGenerateContent?alt=sse`) | INTEGRATE | |
| Unary upstream inference (`v1internal:generateContent`) | OPT-OUT | Phase 11 deliberately standardizes native Antigravity on the always-SSE upstream contract and renders downstream unary responses from bounded SSE accumulation. |
| Deprecated local `agy` CLI transport | OPT-OUT | Retained unchanged for compatibility; native protocol parity or removal is explicitly deferred. |
| Google AI Studio Web endpoints | OPT-OUT | Outside the authenticated Cloud Code Assist native Antigravity surface and excluded by the milestone scope. |
| Undocumented Cloud Code Assist methods not exercised by the reference evidence | OPT-OUT | No trustworthy capability contract exists; inventing or probing billable/internal methods would exceed the hermetic hardening scope. |
