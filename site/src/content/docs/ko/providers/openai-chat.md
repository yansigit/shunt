---
title: OpenAI 호환 (Chat Completions)
description: API 키만으로 매핑된 모델을 임의의 OpenAI Chat Completions 엔드포인트로 라우팅하기.
---

**OpenAI 호환 (Chat Completions)**은 이름이 붙은 프로바이더가 아니라 범용 프로바이더 종류입니다:
`kind = "openai_chat"`은 OpenAI Chat Completions API(`POST /chat/completions`)를 제공하는
어떤 백엔드든 shunt에 연결하며, shunt는 Claude Code의 Anthropic Messages 요청을 그 형태로
변환합니다 — 스트리밍도 포함됩니다. 내장 프리셋이 없으므로 업스트림이 `kind`, `base_url`,
API 키 자격증명을 직접 선언해야 합니다.

## 업스트림 구성

```toml
[[upstreams]]
name = "anthropic"
provider = "anthropic"   # keep Anthropic as the default for unrouted models (e.g. claude-*)

[[upstreams]]
name = "chat"
kind = "openai_chat"
base_url = "https://api.example.com/v1"
auth = { mode = "api_key", env = "CHAT_API_KEY" }

[[routes]]
model = "gpt-5.4"
provider = "chat"
```

순서 있는 `[[upstreams]]`는 shunt의 내장 프로바이더를 대체하므로, 설정이 여전히 폴백하는
`anthropic` 기본값을 직접 선언해야 합니다(`server.default_provider`의 기본값은 `anthropic`입니다).

레거시 `[providers.chat]` 테이블 형식도 계속 지원됩니다: auth 맵 대신 `kind`, `base_url`,
`auth = "api_key"`, `api_key_env = "CHAT_API_KEY"`를 설정하세요. 한 파일에서
`[[upstreams]]`와 `[providers.*]`를 섞어 쓰지 마세요.

`kind = "openai_chat"`은 `auth = "api_key"`만 받습니다. 시작 시 다른 자격증명 모드는 거부됩니다 —
어댑터가 요청마다 설정된 키를 주입하며, 그 외의 자격증명 경로는 없습니다.

## 자격증명

```bash
export CHAT_API_KEY='...'
```

키를 설정 파일에 직접 쓰지 마세요. `shunt check`는 설정 구조를 검증할 뿐 키 값을 읽지 않습니다 —
`CHAT_API_KEY`가 설정되어 있지 않으면 `chat`으로 라우팅되는 첫 요청이 인증 오류를 반환합니다.

## base URL 문법

`base_url`은 일반 `http://` 또는 `https://` 루트여야 합니다: 쿼리 문자열, 프래그먼트, userinfo
(`user:pass@`), dot 경로 세그먼트, 공백, 백슬래시는 허용되지 않습니다. shunt는 정확히 하나의
`/chat/completions` 경로를 붙입니다 — 이미 `/chat/completions`로 끝나는 루트는 그대로 유지됩니다 —
따라서 `https://api.example.com/v1`과 `https://api.example.com/v1/chat/completions`은 동일합니다.
`shunt check`가 부팅 시 같은 문법을 검증합니다.

## 어댑터가 전달하는 것

변환은 기본 거부 화이트리스트 방식이라 지원되지 않는 요청은 조용히 저하되지 않고 타입이 지정된 400으로
실패 닫힘(fail closed)됩니다:

- **텍스트 턴**(system, user, assistant)과 `max_tokens`, `temperature`, `top_p`,
  `stop_sequences`. 지원되지 않는 최상위 요청 필드는 거부됩니다. 단일 텍스트 페이로드는 UTF-8 기준
  8 MiB로 제한됩니다.
- **이미지**: user 메시지의 base64 데이터 또는 URL.
- **도구**: 선언, `tool_choice`, 쌍을 이루는 `tool_use`/`tool_result` 턴. 요청 로컬 id
  레지스트리를 사용하므로 동시 요청이 상태를 공유할 수 없습니다.
- **스트리밍**: Anthropic SSE로 변환되며 사용량이 보존됩니다. 스트리밍 요청은 업스트림
  `stream_options.include_usage` 계약을 강제합니다. 집계된 unary 응답은 32 MiB로 제한되고,
  스트리밍 머신은 턴당 최대 8 MiB의 의미 바이트를 유지합니다.

## 실패 및 취소 의미론

요청이 업스트림에 도달했을 가능성이 있는 이후에는 생성 POST가 다시 발송되지 않으며, 리다이렉트는
아예 거부되어 주입된 bearer가 설정된 원본 외부로 유출될 수 없습니다. 업스트림의 비성공 상태는
게이트웨이 소유 오류로 표면화되고, 120초를 넘는 읽기 유휴는 해당 턴을 실패시킵니다. 요청을
취소하면 업스트림 연결이 닫히고 어드미션 슬롯이 해제됩니다.

이 보장들은 저장소 테스트 스위트의 합성 컨포먼스 피처로 검증됩니다. 실제 프로바이더 동작은 여기서
주장하거나 검증하지 않습니다.

## 프로토콜 제한

도구 인수는 요청별로 버퍼링한 뒤 종료 경계에서 한 번 파싱합니다. 도구 블록은
`message_start` 뒤에 최초 도착 순서로 출력됩니다. 최대 128개 호출과 호출당 1 MiB의
인수를 허용합니다. SSE 이벤트 페이로드와 미완성 잔여 페이로드는 각각 별도의 8 MiB
제한을 적용하며 프레이밍 구분자는 제외합니다. 누적 의미 데이터의 8 MiB 예산에는 텍스트,
추론, 도구 식별자와 인수가 포함되지만 잔여 버퍼는 포함되지 않습니다.

성공에는 지원되는 종료 사유와 `[DONE]`이 모두 필요하며 EOF만으로는 성공하지 않습니다.
`stop`, `length`, `tool_calls`는 각각 `end_turn`, `max_tokens`, `tool_use`로
변환되고 다른 사유는 거부됩니다. 이미 디코딩된 후속 프레임과 잔여 바이트는 오류입니다.
종료 후 미래 바이트나 HTTP EOF를 기다리지 않으며 이미 전달된 텍스트를 회수하지 않습니다.
토큰 카운터는 `0..=i64::MAX` 범위의 정수여야 합니다.

일반 텍스트 thinking을 지원합니다. 응답의 `reasoning`과 `reasoning_content`는
프로바이더 확장이며 보편적인 OpenAI 계약이 아닙니다. 충돌하거나 서명·삭제된 추론은 거부합니다.
읽기 유휴 제한은 새 설정 키 없이 120초로 고정됩니다. 이 어댑터의 `count_tokens`는
다른 전략을 설정해도 항상 로컬 추정값을 사용합니다. 자격증명 파일에는 쓰지 않습니다.

## 검증

```bash
shunt check    # -> config ok
shunt run
curl -sS http://127.0.0.1:3001/v1/messages \
  -H 'anthropic-version: 2023-06-01' \
  -H 'content-type: application/json' \
  -d '{"model":"gpt-5.4","max_tokens":16,"messages":[{"role":"user","content":"Reply with OK."}]}'
```
