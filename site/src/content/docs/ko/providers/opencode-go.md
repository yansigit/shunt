---
title: "OpenCode Go: 증거 게이트, 지원 없음"
description: "정확하고 hermetic하며 자격 증명 안전한 튜플이 입증될 때까지 OpenCode Go는 허용되지 않습니다."
---

# OpenCode Go는 현재 지원되지 않습니다

허용된 OpenCode Go 집합은 비어 있습니다(빈 허용 집합). shunt는 자격 증명 전 게이트에서 모든 명시적 Go 선택을 자격 증명 조회나 네트워크 디스패치 전에 거부하므로 현재 자격 증명이나 `x-opencode-session` 헤더가 전송되지 않습니다.

옵트인 구성 ID는 `kind = "opencode_go"`이며 `SHUNT_OPENCODE_GO_API_KEY`와 표준 대상 `https://opencode.ai/zen/go/v1`을 사용합니다. 현재 소스 전용 후보는 다음과 같습니다.

- `glm-5.3-flash`
- `omen-alpha`
- `muse-spark-1.3-contributor`
- `deepseek-v4-flash`

이들은 후보일 뿐 지원되거나 실시간 검증된 모델이 아닙니다. 알 수 없는 필드, 잘못된 wire, 패밀리 추론, 지원되지 않는 effort 별칭 및 실패한 증거는 거부됩니다. 게이트웨이는 엄격한 권위 있는 터미널을 요구하며 허용적인 EOF에서 성공을 합성하거나 불완전한 턴을 복구하지 않습니다.

## 향후 승격 계약

향후 승격에는 정확한 모델·대상·wire·effort·capability 증거와 hermetic 적합성, 자격 증명 안전 캡처 또는 라이브 검증이 필요합니다. 일치하는 Chat 계약을 재사용할 때만 `https://opencode.ai/zen/go/v1`에 안정적인 불투명 conversation-scoped `x-opencode-session`을 보낼 수 있습니다. 현재 빈 허용 집합에서는 세션 생산자가 없습니다.

구성 세부 사항은 [구성 레퍼런스](/ko/reference/configuration/)를 참고하세요.
