---
id: evidence:kinekino/20261029113604
title: 실제 텍스트 결과 검토 패키지
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 텍스트 결과 검토 패키지

2026-10-19. 사용자에게 결과 확인 방법을 제공하지 못했던 공백을 해결하기 위한 첫 읽기 전용 패키지다. Gemma270·Llama1B·Qwen0.6의 기존 고정 manifest3개를 SHA256으로 확인하고, 교정·카드 문안·하이라이트의 실제9개 입력/출력을 그대로 표시한다. 새 모델 추론은0회다. 전체23개 역할의 통합 검토나 사용자 판정 저장 완료는 아니다.

[패키지 manifest](../../../artifacts/quality-review/text-model-results-r1/manifest.json), [실제 입력·출력 JSON](../../../artifacts/quality-review/text-model-results-r1/bundle.json), [정적 검토 화면](../../../artifacts/quality-review/text-model-results-r1/index.html)을 보존했다. bundle SHA256은 `9851ff6a33e15f0ed11c21a3ea7540690bd76eb3215dfb047e436eba66470f9c`, HTML SHA256은 `07c48bb5b193987448125504c077865bd2cc0bdaa0fd83c98df4e9acfece3ee0`이다.

## 실제 확인

[도구 EXE·감사 시점 소스·게이트 원문](../../../evidence/verification/model-review-bundle-2026-10-19-r1/)도 별도 보존했다. 이는 source-at-audit이며 전체 전이 compiler input attestation으로 주장하지 않는다.

Rust 도구 `tools/model-review-bundle`의 fmt·2tests·all-targets strict Clippy·locked offline release build가 통과했다. 원본 변조 거부와 모델 출력의 HTML 실행 방지 escaping을 시험했다. 사용한 EXE SHA256은 `b8c1e173df967fdd96b14bcfe4d8d3149ab99e5ec1eb3cbde2f9833bd939db3c`다. 원본 manifest의 모든9개 응답은 HTTP200이며, 구조 실패 출력도 숨기지 않고 그대로 포함했다.

내장 브라우저에서 `http://127.0.0.1:9617/`를 실제 열었다. 접근성 트리에서9개 표본·입력·출력·저장 구조판정·종료 사유를 확인했고, 첫 화면 스크린샷에서 글자·안내·탐색 링크·2열 비교 표시를 확인했다. 기본 viewport에 한정한 표시 확인이며 모든 크기의 시각 인수는 아니다. 서버가 종료되면 URL은 사용할 수 없고 재실행 시 포트가 달라진다.

## 사용과 잔여 범위

후속 [실제 SDK package 연결](text-review-sdk-actual.md)에서 같은9개 표본의 공통 계약 변환과 CLI inspect/list를 완료했다. 기존 정적 화면의 판정 저장 기능이 연결된 것은 아니다.

표본 링크를 선택하고 입력과 출력을 비교한다. 교정의 누락·반복·잘림, 카드 문안의 cue 근거 일치, 하이라이트의 선택 구간과 이유를 확인할 수 있다. JSON 구조 통과는 내용 품질 통과를 뜻하지 않는다. 이 페이지에는 승인 버튼·판정 저장·자동 적용 기능이 없다. 사용자가 승인해야 다음 개발이 진행되는 상태가 아니다.

[공통 검토 계약](../../player/application/model-quality-review.md)의 판정 저장·정밀한 조건/기준·자원·원본 미디어 연결은 후속이며 SDK 전담에서 구현 중이다. Pages·VM·DPI는 재개하지 않았다.
