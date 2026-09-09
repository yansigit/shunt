# Preliminary smoke eligibility — 2026-09-09

Preparation only: plan 16-04 has not started and no generation request was dispatched. Global accounting remains **0/8 attempts, US$0 planned paid usage**. This is not live-pass evidence.

Root ran a presence-only Node check under the serialized isolated wrapper. It emitted only booleans, never credential values or credential-file contents. Production config mtime/SHA-256 and backup/invalid inventory were unchanged.

## Presence results

The following explicit environment sources were absent in the isolated process: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `XAI_API_KEY`, `MOONSHOT_API_KEY`, `ZHIPUAI_API_KEY`, `MINIMAX_API_KEY`, `SHUNT_COMMANDCODE_API_KEY`, `SHUNT_COMMAND_CODE_TOKEN`, `SHUNT_OPENCODE_GO_API_KEY`, and `AI_GATEWAY_API_KEY`. Cursor token overrides are removed by the isolation wrapper by design; their absence there says nothing about the user's CLI login.

Presence-only checks found no default Command Code auth file, Shunt Cursor auth file, Shunt Antigravity auth file, or Gemini OAuth credential file at the specific standard paths checked. The Codex auth file exists but was not opened. These observations do not establish contents, validity, subscription entitlement, custom credential paths, or absence of credentials elsewhere. No keychain, alternate stores, production configuration, or account directories were searched.

## Source blockers already established

- ChatGPT/Codex: `src/model/responses_request.rs` explicitly omits `max_output_tokens` for `ResponsesFlavor::Chatgpt` because that backend rejects it. Therefore the approved 128-output-token bound cannot be guaranteed for this translated path; do not read credentials or dispatch a request to test the bound.
- Cursor: the active destination is the pinned AgentService Run URL, not the legacy preset. No complete output/reasoning/billing bound was established from the current inspected request/transport source. An installed/logged-in CLI alone does not make a bounded Shunt smoke eligible.
- OpenCode Go: no admitted tuple; never bypass admission for a smoke.
- API-key and Command Code subscription paths: missing explicit/default sources prevent dispatch through the checked sources. An existing Codex auth file does not prove an OpenAI API key is present; do not inspect it solely to discover additional accounts.

Final plan 16-04 must reconcile these observations with the completed finite ledger and security review, record every relevant tuple's pass/skip/block, and preserve the no-retry budget. No provider pricing assumption or live success is inferred from development subagent availability.
