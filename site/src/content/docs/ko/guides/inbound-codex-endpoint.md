---
title: 인바운드 Codex 엔드포인트
description: OpenAI Codex CLI 자체를 shunt로 향하게 하고 ChatGPT/Codex OAuth 계정 풀에 걸쳐 로드 밸런싱하기.
---

이 사이트의 다른 가이드는 모두 **Claude Code**를 다른 백엔드로 라우팅합니다. shunt는 그 반대 방향으로도 동작할 수 있습니다: 옵트인 원시 OpenAI Responses 패스스루로, **Codex CLI**가 자체 `base_url`을 shunt로 향하게 하고 ChatGPT/Codex OAuth 계정 풀에 걸쳐 로드 밸런싱되도록 합니다. 옵트인이므로 `[server.codex_endpoint]`가 없으면 관련 라우트가 하나도 등록되지 않고 shunt의 기본 HTTP 화면은 그대로 유지됩니다.

이는 [Codex 멀티 계정](/ko/guides/codex-multi-account/)과 동일한 계정 풀 위에 만들어졌습니다 — 선택, 쿨다운, 갱신은 변경 없이 공유됩니다. 정확한 페일오버 표와 리로드 시맨틱을 포함한 전체 명세는 [M11 동작 명세](https://github.com/pleaseai/shunt/blob/main/docs/m11-inbound-codex-endpoint.md)를 참고하세요.

엔드포인트 활성화, Codex CLI를 shunt로 향하게 하기, 클라이언트 인증, 계정 프로비저닝, 자격이 부여된 모델 선택까지 처음부터 끝까지의 설정 안내는 [Codex CLI 연결](/ko/guides/connect-codex-cli/)을 따르세요. 이 페이지는 *엔드포인트가 무엇을 하는지*에 집중하고, 그 가이드는 *어떻게 연결하는지*의 체크리스트입니다.

## 엔드포인트 활성화

```toml
[server.codex_endpoint]   # 모든 키 선택; 기본값 표시됨
provider = "codex"        # chatgpt_oauth 프로바이더여야 함
collaboration = false     # 변환된 V2 collaboration 브리지는 명시적으로 활성화
```

```bash
shunt check
shunt run
```

시작 검증은 알 수 없는 `provider`나 `auth = "chatgpt_oauth"`를 쓰지 않는 프로바이더를 거부합니다 — 이 엔드포인트는 운영자의 Codex bearer를 주입하므로 `chatgpt_oauth` 프로바이더만 자격이 있습니다. 모든 키와 기본값은 [구성 레퍼런스](/ko/reference/configuration/#servercodex_endpoint-선택)를, 등록된 라우트는 [HTTP 엔드포인트](/ko/reference/endpoints/)를 참고하세요.

이 옵트인은 Codex CLI 모델 탐색도 파싱 가능하게 만듭니다. `GET /models`와 `GET /backend-api/codex/models`는 유효한 폴백 `{"models":[]}`를 반환합니다. 공유 `GET /v1/models` 경로에서는 `client_version` 쿼리 필드가 Anthropic 형태의 헤더보다 우선하여 Codex 형태를 선택하고, 그 필드가 없으면 기존 Anthropic 탐색 응답은 변경되지 않습니다. 이 요청들은 기존 모델 탐색 인증 게이트를 거치며, shunt는 불완전한 Codex 모델 행을 만들지 않습니다.

## 클라이언트 analytics sink

Codex CLI는 제품 analytics도 base URL로 전송합니다. shunt는 CLI가 만들 수 있는 두 경로를 모두 받아들입니다:

- `POST /backend-api/codex/analytics-events/events`
- `POST /codex/analytics-events/events`

이 라우트들은 Responses 라우트와 동일한 `[server.auth]` 정책을 따르지만, 텔레메트리를 업스트림으로 전달하지는 않습니다. 풀링된 계정 하나를 고르면 클라이언트 이벤트가 그 계정에 잘못 귀속되기 때문입니다. 인증 후에는 본문이 잘못됐거나, 읽을 수 없거나, 너무 크더라도 항상 `200 {}`을 반환합니다.

페이로드와 이벤트 속성은 로깅되지도 내보내지지도 않습니다. shunt는 정제된 `event_type`만 옵트인 `shunt.codex_client_events` 카운터의 `event` 속성으로 기록합니다: 이름에는 소문자 ASCII 문자, 숫자, `.`, `_`, `-`가 최대 64바이트까지 올 수 있고, 유효하지 않은 이름은 `other`로, 해석되지 않은 배치는 `unparsed`가 됩니다. Sentry나 OpenTelemetry 메트릭이 활성화되어 있지 않다면 이는 순수한 폐기 sink입니다.

## Codex CLI를 shunt로 향하게 하기

Codex CLI는 사용하는 base URL이 무엇이든 항상 그 뒤에 `/responses`를 붙이므로, 다음 두 가지 `~/.codex/config.toml` 형태 모두 동작합니다:

**ChatGPT 백엔드의 base URL을 흉내 내기:**

```toml
chatgpt_base_url = "http://127.0.0.1:3001/backend-api/codex"
```

**또는 커스텀 모델 프로바이더**(최상위 `model_provider`가 이를 선택해야 하며, 그렇지 않으면 CLI는 내장 프로바이더를 유지합니다):

```toml
model_provider = "shunt"

[model_providers.shunt]
base_url = "http://127.0.0.1:3001/v1"
wire_api = "responses"
```

커스텀 프로바이더를 쓰면(CLI에 로컬 로그인이 필요 없도록 `requires_openai_auth = false`를 추가하세요) shunt를 가리키는 순간 Codex CLI 자체의 `~/.codex/auth.json`은 무의미해집니다 — 계정은 매 요청마다 shunt의 풀에서 옵니다. 반면 `chatgpt_base_url` 형태는 CLI를 ChatGPT 로그인 모드로 유지하므로 여전히 로컬 로그인이 필요하고, **게이팅되지 않은** 엔드포인트에서만 동작합니다: CLI의 ChatGPT bearer는 구성된 shunt 토큰이 아니므로 `[server.auth]`가 이를 거부합니다.

## 클라이언트 인증

shunt에 [`[server.auth]`](/ko/guides/shared-gateway/)가 구성되어 있다면 — 루프백을 넘어서는 모든 경우에 권장됩니다 — 클라이언트 토큰을 OpenAI 스타일 Bearer 키(`OPENAI_API_KEY` 또는 커스텀 프로바이더의 `env_key`, LiteLLM/llmgateway 방식)로 제시하**거나** `x-shunt-token` 헤더로 제시하세요:

```toml
# A. Bearer — 내장 openai 프로바이더. base URL은 ~/.codex/config.toml에 설정하고
#    OPENAI_BASE_URL 환경 변수로는 설정하지 마세요: 환경 변수를 쓰면 CLI의 Responses
#    WebSocket이 wss://api.openai.com을 계속 가리켜 shunt를 우회합니다. 연결 가이드의
#    "Codex CLI를 shunt로 향하게 하기"를 참고하세요.
openai_base_url = "http://127.0.0.1:3001/v1"
```

```bash
export OPENAI_API_KEY="<shunt-token>"      # Authorization: Bearer로 전송됨
```

```toml
# B. 헤더 — 커스텀 프로바이더가 이를 실어 보냅니다(파일 밖에 두려면 env_http_headers 사용):
[model_providers.shunt]
base_url = "http://127.0.0.1:3001/v1"
wire_api = "responses"
http_headers = { "x-shunt-token" = "<token>" }
```

`[server.auth]`가 없으면 엔드포인트는 도달할 수 있는 누구에게나 열려 있습니다 — 루프백이나 개인 용도에는 받아들일 만하지만 공유 게이트웨이에는 적절하지 않습니다. 클라이언트가 제시한 자격 증명은 shunt에 인증하는 데에**만** 사용됩니다: 이 값은(그리고 CLI가 보내는 어떤 `Authorization`이든) 제거되며 업스트림으로 전달되지 않습니다. `[server.admin]` 자격 증명 헤더(기본값 `x-shunt-admin-token`, 또는 `[server.admin] header`가 지정한 이름)도 제거됩니다 — 관리 화면이 바로 그 슬롯에서 인증하며, 관리 자격 증명은 업스트림 계정을 프로비저닝할 수 있기 때문입니다. `cookie` 헤더도 통째로 제거됩니다: 관리 화면은 쓰기 등급 세션 쿠키 역시 그 슬롯에서 수락하고, shunt는 쿠키 저장소를 두지 않으므로 업스트림이 이에 의존할 일이 없습니다. `x-api-key`도 `[server.auth]`가 설정되지 않은 경우를 포함해 무조건 제거됩니다 — 대상 프로바이더는 부팅 시점에 `chatgpt_oauth` 전용으로 검증되므로, 인바운드 `x-api-key` 값은 이 업스트림에 대해 결코 유효한 자격 증명이 될 수 없습니다. Claude Code의 `apiKeyHelper`처럼 `Authorization`과 `x-api-key`에 같은 키를 넣는 클라이언트라도 두 번째 슬롯을 통해 그 키가 새어 나가지 않습니다. 인바운드 클라이언트가 실제 Codex CLI이므로, 패스스루는 그 요청 헤더를 그대로 전달하고(`version`, `originator`, `OpenAI-Beta`, `x-codex-*`, …) 선택된 풀 계정의 `Authorization` bearer와 `chatgpt-account-id`**만** 바꿔 넣습니다. 전체 인증 안내는 [Codex CLI 연결](/ko/guides/connect-codex-cli/#3-shunt-클라이언트-토큰-제시-serverauth가-설정된-경우)을 참고하세요.

## WebSocket 전송

세 Responses 경로는 HTTP `POST`와 인증된 WebSocket `GET` 업그레이드를 모두 받습니다. 인증은 `101 Switching Protocols` 전에 끝납니다. 소켓에서는 `generate: false` 워밍업을 로컬에서 완료하고, 실제 `response.create`는 기존 HTTP 계정 풀을 재사용해 업스트림 스트리밍을 강제하며 첫 종료 이벤트까지 각 SSE `data:` 페이로드를 WebSocket 텍스트 프레임으로 전달합니다. 턴 교체나 소켓 종료는 활성 업스트림 본문을 취소합니다. 클라이언트 프레임과 SSE 이벤트는 4 MiB로 제한되고, 전송에는 유계 백프레셔가 적용되며 오류 프레임에는 안전한 응답 메타데이터만 포함됩니다.

## 계정 프로비저닝

[Codex 멀티 계정](/ko/guides/codex-multi-account/#풀-구성)과 동일한 풀을 재사용합니다:

```bash
codex login
shunt login codex --name main
```

```toml
[[providers.codex.accounts]]
name = "main"
```

`[[providers.codex.accounts]]`가 구성되지 않았고 **shunt 계정 스토어도 비어 있으면**, 엔드포인트는 기본 `~/.codex/auth.json` 자격 증명 하나로 폴백합니다 — 풀링도 페일오버도 없습니다 — 따라서 `[server.codex_endpoint]`를 설정하는 즉시 Codex 로그인 하나만으로도 동작합니다. (핸들러는 먼저 계정 스토어를 스캔해 발견한 계정을 풀에 넣으므로, 가져온 스토어 계정은 여전히 풀링을 활성화합니다.)

## 모델을 다른 업스트림으로 라우팅하기

기본적으로 모든 요청은 `[server.codex_endpoint]`에 지정된 프로바이더 하나로 갑니다. 선택적인 `[[server.codex_endpoint.routes]]` 테이블을 쓰면 Codex CLI가 모델 id로 **다른** Responses 호환 업스트림을 고를 수 있습니다. 라우트가 없는 모델은 기존의 고정 프로바이더 동작을 그대로 유지합니다.

여러 벤더가 Codex CLI용 네이티브 Responses 엔드포인트를 문서화하고 있습니다: Z.ai GLM(`https://api.z.ai/api/v1`), DeepSeek(`https://api.deepseek.com`), Kimi Code(`https://api.kimi.com/coding/v1`), MiniMax(`https://api.minimax.io/v1`), Mimo(`https://api.xiaomimimo.com/v1`), OpenRouter(`https://openrouter.ai/api/v1`), Vercel AI Gateway(`https://ai-gateway.vercel.sh/codex/v1`), 그리고 순정 OpenAI. shunt는 프로바이더의 `base_url` 뒤에 `/responses`를 붙이므로, 벤더가 Codex용으로 안내하는 그 base URL을 그대로 적으면 됩니다. 업스트림은 Responses API를 네이티브로 구현해야 합니다 — Responses → Chat Completions 어댑터는 없습니다.

```toml
[providers.glm]
kind = "responses"
auth = "api_key"
api_key_env = "GLM_API_KEY"
base_url = "https://api.z.ai/api/v1"

[providers.deepseek]
kind = "responses"
auth = "api_key"
api_key_env = "DEEPSEEK_API_KEY"
base_url = "https://api.deepseek.com"

[server.codex_endpoint]
provider = "codex"

[[server.codex_endpoint.routes]]
model = "glm-5.3"
provider = "glm"

[[server.codex_endpoint.routes]]
model = "deepseek-v4-flash"
provider = "deepseek"
```

`upstream_model`은 선택 사항이며 기본값은 `model`입니다. CLI에 입력하는 id와 벤더가 실제로 제공하는 id가 다를 때 지정하세요. 라우팅 대상 프로바이더는 실제 자격 증명을 가져야 합니다 — 클라이언트 자신의 `Authorization`은 항상 제거되므로, 자격 증명이 없는 인증 모드(`passthrough` 또는 `none`)는 부팅 시 거부됩니다. shunt에 내장된 `kimi` 프리셋은 `kind = "anthropic"`이므로, Kimi Code로 향하는 Codex 라우트에는 별도의 `kind = "responses"` 프로바이더가 필요합니다 — Anthropic 종류의 프리셋으로 Codex 모델을 라우팅하면 부팅 시 거부됩니다.

CLI 쪽에서는 Codex가 **shunt**를 바라보게 하고 `model`로 라우트를 선택합니다:

```toml
# ~/.codex/config.toml
model = "glm-5.3"
model_provider = "shunt"
model_catalog_json = "~/.codex/models.json"

[model_providers.shunt]
base_url = "http://127.0.0.1:3001/v1"
wire_api = "responses"
env_key = "SHUNT_TOKEN"
```

shunt는 Codex용 모델 카탈로그를 제공하지 않습니다 — `GET /v1/models` 디스커버리 목록은 Anthropic 형태이며 Codex 라우트를 노출하지 않습니다. CLI는 이들 벤더가 안내하는 대로 `model_catalog_json`이 가리키는 `~/.codex/models.json` 카탈로그에서 슬러그 메타데이터를 얻습니다. shunt 라우트를 고르는 것은 오직 `model` 값입니다.

**ChatGPT가 아닌** 업스트림으로 라우팅된 요청에서 달라지는 점:

- **헤더 허용 목록.** 클라이언트에서 가져오는 것은 `content-type`과 `accept`뿐이고, 여기에 해석된 자격 증명과 라우팅 대상 업스트림이 요구하는 identity만 더해집니다 — `OpenAI-Beta: responses=experimental`(xAI/Grok에서는 생략), 그리고 `xai_oauth` 라우트의 경우 Grok CLI identity 헤더. `authorization`, `x-api-key`, `chatgpt-account-id`, `originator`, `version`, `user-agent`, `session-id`, `x-codex-*`, `x-shunt-*`는 어느 것도 서드파티에 닿지 않습니다.
- **본문 `model` 재작성.** `upstream_model`이 요청된 모델과 다르면 shunt가 최상위 `model`만 바꾸고 나머지 필드는 그대로 둡니다. JSON 객체가 아닌 본문은 그대로 보내지 않고 `400`으로 거부합니다.
- **identity 인코딩.** zstd 요청 본문은 먼저 디코딩되며 — 순정 Responses API는 그 인코딩을 받지 않습니다 — `content-encoding`은 전달되지 않습니다.
- **자격 증명 하나, 페일오버 없음.** 라우팅된 서드파티 뒤에는 풀이 없으므로 429나 5xx는 회전을 유발하지 않고 `retry-after`와 함께 그대로 릴레이됩니다.

매칭은 정확 일치이며 대소문자를 구분하고 문자 집합 제한이 없습니다. 따라서 `MiniMax-M3`, `openai/gpt-5.6-sol`, `~openai/gpt-latest` 같은 벤더 슬러그도 적은 그대로 라우팅됩니다. 다른 `chatgpt_oauth` 프로바이더로 향하는 라우트라면 전체 풀 패스스루가 그대로 유지됩니다. 라우트는 라이브 구성에서 읽으므로 리로드 시점에 반영됩니다.

## `/v1/messages`와 다른 점

- **네이티브 경로는 불투명합니다.** Responses 네이티브 경로에서는 요청 본문과 업스트림 응답이 바이트 단위로 그대로 전달되며, 변환 경로와 분리됩니다.
- **압축된 요청 본문도 그대로 통과합니다.** 현재 Codex 릴리스는 ChatGPT 백엔드와 통신할 때 요청 본문을 zstd로 압축하며, 여기에는 이 엔드포인트를 가리키는 `chatgpt_base_url` 형태도 포함됩니다. 바이트와 그 `content-encoding: zstd` 헤더는 변경 없이 전달되고, shunt는 메트릭·로그·스팬에 쓸 요청의 `model`을 읽기 위해 메모리에서 사본만 추가로 디코딩합니다. shunt가 디코딩할 수 없는 본문도 릴레이에는 문제가 없으며 — `model` 레이블만 `unknown`으로 낮아지고, 그 이유를 밝히는 경고가 남습니다.
- **정확한 모델 라우팅.** 고유한 정확 일치 선언은 Responses 네이티브 또는 Anthropic Messages 프로바이더 하나를 선택할 수 있습니다. 프리픽스 전용·비정확·불일치 모델은 고정 네이티브 프로바이더로 폴백하며, 모호한 선언은 업스트림 요청 전에 거부됩니다.
- **Anthropic 변환은 엄격하고 유계입니다.** 지침, 텍스트 및 URL/data-URL 이미지, 함수 도구·호출·결과, 도구 선택, 생성 제어, reasoning effort를 지원합니다. HTTP와 WebSocket은 동일한 JSON/SSE 변환기를 사용합니다. `end_turn`·`stop_sequence`·`tool_use`는 completed, `max_tokens`는 incomplete이며, 잘못된 출력·알 수 없는 종료 이유·스트림 오류·조기 EOF는 failed입니다. 캐시 읽기/쓰기 입력 토큰도 usage에 포함됩니다.
- **Collaboration은 명시적으로 활성화합니다.** `collaboration = true`이면 정확한 Anthropic 경로가 선언된 V2 `collaboration` 도구와 평문 `agent_message` task 봉투를 연결합니다. 승인된 호출은 JSON과 SSE 응답에서 collaboration namespace로 복원됩니다. 네이티브 Responses 트래픽은 이 플래그와 무관하게 바이트 단위로 불투명하게 유지됩니다.
- **손실성 입력은 전송 전에 실패합니다.** `previous_response_id`, 암호화 reasoning/compaction 상태, hosted/custom 도구, 원격 파일 id, 잘못된 도구 관계와 미지원 필드는 추측하지 않고 거부합니다. compaction V2 트리거를 포함한 네이티브 Responses 투명 전달에는 영향이 없습니다.
- **숨은 복구는 없습니다.** 암호문만 있는 변환 agent task는 자격 증명 확인이나 네트워크 전송 전에 실패합니다. shunt는 복호화·캐시·영속화·과금 복구 요청을 하지 않습니다. 평문 이력을 제공하거나 네이티브 Responses 경로를 사용하세요.
- **소진 시에도 그대로 릴레이합니다.** 풀링된 모든 계정을 시도했고 업스트림 응답이 최소 한 번 돌아왔다면, shunt는 그것을 Anthropic 형식의 오류로 다시 만들지 않고 마지막 응답을 변경 없이 릴레이합니다. Responses 클라이언트는 실제 ChatGPT 백엔드에서 받았을 원시 형태를 기대하기 때문입니다.
- **쿼터 인식 회전은 제한됩니다.** 일반 `429`는 일시적 스로틀링입니다. shunt는 제한된 응답 본문에 정확한 구조화 하드 쿼터 증거(`usage_limit_exceeded` 또는 `insufficient_quota`)가 포함된 경우에만 계정을 유한한 내부 쿨다운 동안 억제하며, 잘못되었거나 모호하거나 너무 크거나 중단된 본문은 미검증 상태로 남습니다. 유효한 `Retry-After` 델타 초(소수 포함, 안전하게 올림)와 HTTP 날짜 값은 유한한 한도 내에서 준수됩니다. 후보가 소진되면 최종 업스트림 상태, 본문 및 안전한 `Retry-After` 메타데이터가 그대로 노출됩니다. 계정 회전은 출력이 시작되기 전에만 적격하며, 관련 없는 라우트 수준 페일오버 동작은 변경되지 않습니다.
- **Compaction은 실패 폐쇄형이며 불투명하게 유지됩니다.** 레거시 HTTP 전용 `POST /v1/responses/compact`는 본문과 연속 상태를 정식 OpenAI API 키 백엔드에만 바이트 단위 그대로 전달합니다. ChatGPT/Codex 백엔드는 이 별도 엔드포인트를 더 이상 제공하지 않으므로 ChatGPT OAuth 대상은 자격 증명 확인이나 네트워크 디스패치 전에 로컬에서 실패합니다. 현재 Codex compaction V2는 일반 Responses 스트림에 마지막 `compaction_trigger`를 보내며 네이티브 경로는 이를 변경 없이 전달합니다. 잘못되었거나 모호하거나 변환이 필요하거나 지원되지 않는 대상도 네트워크 디스패치 전에 거부됩니다. shunt는 연속 상태를 복호화하거나 요약을 합성하거나 요청 기록을 저장하지 않습니다.
- **게이트웨이 소유 오류는 OpenAI 형식입니다.** 실패가 shunt 자신의 것일 때 — 잘못됐거나 없는 클라이언트 토큰(`401`), 업스트림 응답 없이 풀을 해석할 수 없는 경우(`502`), 지나치게 큰 요청 본문, 구성되지 않은 엔드포인트 — shunt는 이를 동일한 status 코드와 함께 OpenAI Responses 오류 형태(`{"error":{"message":…,"type":…,"code":null}}`)로 반환합니다. 그러면 Codex CLI가 Anthropic의 `{"type":"error",…}` 봉투가 아니라 자체 오류 경로로 이를 해석합니다. 릴레이되는 *업스트림* 오류(백엔드의 429/4xx/5xx)는 여전히 그대로 통과합니다.
- **두 가지 인바운드 전송.** HTTP `POST`는 바이트 충실도를 유지하고, WebSocket `GET`은 유계 이벤트 전달을 추가하며 프로바이더의 outbound `websocket = true` 설정과 무관합니다.
- **공유 디스패치 경계.** HTTP와 WebSocket은 동일한 정확 일치 해석기와 프로바이더별 자격 증명 필터링을 사용합니다. 출력이 시작되면 프로바이더 이동이나 재생은 없으며, 출력 전 계정 교체만 허용됩니다.


## 보안

- 루프백을 넘어서는 모든 경우에 `[server.auth]`로 이 엔드포인트를 게이팅하세요 — 프로바이더가 매 요청마다 실제 Codex bearer를 주입합니다.
- 클라이언트 자신의 자격 증명은 어떤 것도 Codex 백엔드에 닿지 않습니다. 패스스루는 Codex CLI 자체의 요청 헤더를 그대로 전달하고 선택된 풀 계정의 bearer와 `chatgpt-account-id`만 바꿔 넣습니다(shunt 클라이언트 토큰 헤더, `[server.admin]` 자격 증명 헤더, `cookie` 헤더 전체, 내부용 `x-shunt-inbound-client` 라벨, 클라이언트의 `Authorization`/`chatgpt-account-id`, 그리고 `x-api-key`는 모두 제거되며 전달되지 않습니다).
- 부팅 시 한 번 결정되는 것은 엔드포인트의 **HTTP 라우트 등록**뿐입니다. 런타임에 `[server.codex_endpoint]`를 켜거나 끄면 해당 경로를 추가·제거하려면 재시작이 필요하다는 경고가 로깅됩니다. 테이블이 *담고 있는* 내용은 모두 핫 리로드됩니다 — 대상 `provider`와 `[[server.codex_endpoint.routes]]` 모델 테이블 전체를 매 요청마다 라이브 config에서 읽으므로, 라우트를 추가·수정·삭제하면 리로드 시점에 반영됩니다.
