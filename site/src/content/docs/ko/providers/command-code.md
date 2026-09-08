---
title: "Command Code: API 키와 구독"
description: "서로 다른 자격증명과 프로토콜을 사용하는 두 Command Code 제품."
---

이름이 서로 다릅니다. 두 이름은 순서형 업스트림 프리셋이며 레거시 프로바이더 테이블이 자동 생성되지는 않습니다.

| 프리셋 | 종류 / 인증 | 대상 |
| --- | --- | --- |
| `commandcode` | `openai_chat` / `api_key` | `https://api.commandcode.ai/provider/v1/chat/completions` |
| `command-code` | `command_code` / `command_code_oauth` | `https://api.commandcode.ai/alpha/generate` |

## 설정
```toml
[[upstreams]]
name = "anthropic"
provider = "anthropic"

[[upstreams]]
name = "cc-api"
provider = "commandcode"

[[upstreams]]
name = "cc-sub"
provider = "command-code"
effort = "high"

[[models]]
id = "cc-sub-glm"
[models.upstream_model]
cc-sub = "zai-org/GLM-5.3"
```

API 계정에서 확인한 모델 ID로 `cc-api` 매핑을 별도로 추가하세요. 아래 구독 표는 API 키 제품에 적용되지 않습니다. 매핑되지 않은 모델을 위해 Anthropic 업스트림을 유지하세요. 기존 프로바이더 설정은 바뀌지 않습니다.

## 자격증명

API 키 프리셋은 `SHUNT_COMMANDCODE_API_KEY`를 사용합니다. 구독은 `SHUNT_COMMAND_CODE_TOKEN`을 사용하며, 이 변수가 아예 없을 때만 기존 `~/.commandcode/auth.json`을 읽기 전용으로 읽습니다(`apiKey` 문자열, 선택적 `userId`). 명시한 토큰이 비어 있거나 잘못되면 파일로 대체하지 않고 실패합니다. 파일 한도는 16 KiB, 자격증명 조회 대기 한도는 5초입니다. 토큰은 비어 있지 않은 헤더 안전 ASCII여야 합니다.

Shunt는 이 구독 파일에 대해 로그인, whoami, 갱신, 복사, 복구, 쓰기를 하지 않습니다. 계정 순환이나 자격증명 경로·버전 재정의도 없습니다. `command_code_oauth`는 `command_code` 전용입니다. 구독 대상은 정규 HTTPS 호스트와 443 포트만 허용하고 사용자 정보·쿼리·프래그먼트를 금지합니다. 경로는 빈 값, `/`, `/alpha/generate`만 가능합니다. 조회 전과 bearer 헤더 생성 전에 검증하며 리디렉션을 따르지 않습니다.

## 정확한 구독 모델 / 노력 수준

| 모델 ID (대소문자 구분) | 명시 가능한 노력 수준 |
| --- | --- |
| `deepseek/deepseek-v4-pro` | `high`, `max` |
| `deepseek/deepseek-v4-flash` | `high`, `max` |
| `zai-org/GLM-5` | `high`, `max` |
| `zai-org/GLM-5.1` | `high`, `max` |
| `zai-org/GLM-5.2` | `high`, `max` |
| `zai-org/GLM-5.2-Fast` | `high`, `max` |
| `zai-org/GLM-5.3` | `low`, `high`, `max` |
| `meta/muse-spark-1.2` | `low`, `medium`, `high`, `xhigh`, `max` |
| `meta/muse-spark-1.2-contributor` | `low`, `medium`, `high`, `xhigh`, `max` |
| `meta/muse-spark-1.1` | `low`, `medium`, `high`, `xhigh`, `max` |

노력 수준 생략은 허용합니다. 요청의 명시적 값이 라우트 기본값보다 우선하며, `none`·`ultra` 등 미지원 값은 보정하지 않고 거부합니다. 알 수 없는 ID와 보고에만 존재하는 Luna, Gemini 3.7 Flash, vision-exp 항목은 허용하지 않습니다.

## 변환과 한도

구독은 지원되는 텍스트, 평문 추론, 이미지, 실제 도구 호출·결과 이력을 작업공간 정보 없는 요청으로 변환합니다. 평문 하위 에이전트 결과와 이어지는 대화 이력은 지원하지만 불투명 상태는 거부합니다. 기록되지 않은 도구 결과는 성공을 만들어 내지 않고 실행 상태 불명으로 표시합니다. 도구 목록과 선택은 명시적입니다. 원시 프로젝트 경로나 대화 ID를 보내지 않습니다. 명시적 대화는 자격증명에 범위가 한정된 불투명 세션을 사용하고, 그 외에는 요청별 무작위 세션을 사용합니다.

두 제품 모두 단일 응답과 Anthropic 스트리밍 출력을 지원합니다. 구독 NDJSON은 항상 점진적으로 읽으며 JSON 레코드 1 MiB(LF/CRLF 제외), 전체 전송 32 MiB, 의미 데이터 8 MiB, 도구당 인자 512 KiB, 도구 128개, 콘텐츠 블록 4,096개로 제한합니다. 수신 요청 기본 한도는 32 MiB이며 `server.limits.max_request_bytes`로 설정합니다. 토큰 수는 로컬 추정값을 사용합니다.

지원되는 권위 있는 종료 레코드와 완전한 프레이밍 EOF가 있어야 성공합니다. 잘못되거나 중복·종료 후·알 수 없는·잘린 레코드는 복구 없이 실패하며 EOF만으로 성공을 만들지 않습니다. 사용량은 검증된 정수 연산을 사용하고 캐시 토큰은 전체 입력에서 분리하여 중복 계산하지 않습니다. 공급자 오류 종료는 보고된 사용량을 유지한 실패입니다. 연결 전 실패만 자격증명과 세션을 유지하여 재시도할 수 있습니다. 전송 후 실패나 출력·도구 활동 후에는 재전송하지 않습니다. 취소는 업스트림을 닫고 요청 슬롯을 반환합니다.

구독 읽기 유휴 한도와 완전한 레코드 진행 기한은 각각 고정 120초입니다. 일부 바이트만 들어와서는 진행 기한이 초기화되지 않습니다. 전체 턴 기한을 뜻하지는 않습니다. 게이트웨이 오류는 수신 프로토콜의 형식을 따릅니다. API 키 제품은 [일반 Chat 계약](/ko/providers/openai-chat/)을 유지합니다.

## 검증 범위

이 표와 클라이언트 버전 `0.52.1`은 2026-09-08에 확인한 고정 OpenCodex `055c3ecf` 소스 기반입니다. 격리 TLS 및 CLI/모의 테스트는 실제 공급자 가용성 증명이 아닙니다. 최소 요청 형식의 수락과 버전의 최신성은 Phase 16의 선택적 실서비스 검증 항목입니다. 공개 대상 우회 설정은 없습니다. [설정 레퍼런스](/ko/reference/configuration/)를 참조하세요.

