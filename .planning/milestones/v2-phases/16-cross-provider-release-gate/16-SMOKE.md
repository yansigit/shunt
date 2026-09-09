# Phase 16 bounded smoke results — 2026-09-09

**Disposition: completed preflight, all generation paths skipped. No live compatibility pass.** The approved bound permits zero live calls when eligibility cannot be established.

## Root-owned budget

```json
{"generation_attempt_limit":8,"generation_attempts":[],"generation_attempts_used":0,"planned_paid_usd":0,"paid_usd_limit":1,"per_request_timeout_seconds":60,"per_request_output_token_limit":128,"automatic_retries":0}
```

No outbound generation attempt, provider login, refresh, purchase, or credential-file read occurred. No attempt was interrupted or retried. Development subagent calls and local synthetic CLI fixtures are not Shunt live-provider evidence.

## Eligibility decisions

Each row below applies to every exact model row for that provider in `docs/provider-release-evidence.md`; the six unbound contracts are named separately. This is a finite preflight, not a claim about credentials elsewhere on the computer.

| Ledger provider/contract | Destination or contract | Credential presence check | Blocking preflight fact | Outcome |
| --- | --- | --- | --- | --- |
| anthropic | `https://api.anthropic.com/v1/messages` | Explicit API key absent; no incoming client credential supplied | No authorized request credential selected | Skip |
| codex | `https://chatgpt.com/backend-api/codex/responses` | Default file exists; contents uninspected | Translated ChatGPT path omits `max_output_tokens`; 128-token output bound cannot be guaranteed | Skip before credential read |
| openai | `https://api.openai.com/v1/responses` | Explicit API key absent; Codex fallback file not inspected to discover additional credentials | No eligible explicit API-key source established | Skip |
| xai | `https://api.x.ai/v1/responses` | Explicit API key absent | No request credential selected | Skip |
| grok | `https://cli-chat-proxy.grok.com/v1/responses` | Store not inspected | Safe no-refresh/writeback path and complete reasoning/billing bound not established | Skip before credential read |
| kimi | `https://api.moonshot.ai/anthropic/v1/messages` | Explicit API key absent | No request credential selected | Skip |
| zhipu | `https://open.bigmodel.cn/api/anthropic/v1/messages` | Explicit API key absent | No request credential selected | Skip |
| minimax-cn | `https://api.minimax.cn/anthropic/v1/messages` | Explicit API key absent | No request credential selected | Skip |
| vercel-anthropic | `https://ai-gateway.vercel.sh/v1/messages` | Explicit API key absent | No request credential selected | Skip |
| cursor | `https://agentn.global.api5.cursor.sh/agent.v1.AgentService/Run` | Default Shunt file absent; CLI login/keychain not inspected | Complete output/reasoning/billing bound not established; CLI login is not bounded Shunt-wire eligibility | Skip |
| antigravity | `https://daily-cloudcode-pa.googleapis.com/v1internal:streamGenerateContent?alt=sse` | Default Shunt file absent | No credential source; fresh account-catalog admission and complete billing bounds not established | Skip |
| command-code | `https://api.commandcode.ai/alpha/generate` | Explicit token and default CLI file absent | No request credential selected | Skip |
| commandcode (unbound) | `https://api.commandcode.ai/provider/v1/chat/completions` | Explicit API key absent | No credential or exact eligible model selected | Skip |
| custom-anthropic (unbound) | Operator-declared base plus `/v1/messages` | No custom credential/configuration read | No exact destination/model/price tuple selected | Skip |
| custom-openai-chat (unbound) | Operator-declared base plus `/chat/completions` | Example `CHAT_API_KEY` absent | No exact destination/model/price tuple selected | Skip |
| gemini (unbound) | Code Assist `v1internal` methods on `cloudcode-pa.googleapis.com` | Default Gemini file absent | No credentials; no-refresh path and complete bounds not established | Skip |
| kimi-code (unbound) | `https://api.kimi.com/coding/v1/messages` | Account stores uninspected | No exact account model, safe no-refresh path, or complete cost bound established | Skip |
| antigravity-cli (unbound) | Local `agy` subprocess | CLI/account stores uninspected | No complete generation/reasoning/billing bound established | Skip |
| opencode-go | No admitted tuple | Credentials not requested | Admission is empty; no bypass allowed | Skip |

For all skipped rows, generation input/output/reasoning usage is **not applicable: no request**. Intended input was a fixed synthetic no-tool prompt; no bound or price was assumed for an unselected tuple. Planned paid cost is zero because nothing was dispatched, not because any service was assumed free. The canonical destinations above are source facts, not observed network destinations or availability checks.

## Isolation and actual smoke check

Presence checks emitted booleans only. Explicit credential-path override variables were also checked for presence without displaying values. No alternate paths were opened. See `16-SMOKE-PREFLIGHT.md` for the exact initial source checks.

Root ran `node /tmp/shunt-phase16-serialized-run.cjs cargo test --all-features --test check_cli opencode_go_cli_negative -- --test-threads=1`: **1 selected, 1 passed, 5 filtered**, exit 0. This tests owned local CLI admission rejection only; it proves neither paid live success nor billing limits.

All stateful processes inherited fresh isolated homes and non-production ports through the wrapper. Production config mtime/SHA-256 and backup/invalid inventory remained unchanged. Since real source credentials were not opened or used, there was no source-credential mutation or recovery. Existing owner-only backups remain untouched.
