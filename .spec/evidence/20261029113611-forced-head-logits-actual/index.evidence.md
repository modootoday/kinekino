---
id: evidence:kinekino/20261029113611
title: 동일 mel 전체 타임스탬프 head 실제 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 동일 mel 전체 타임스탬프 head 실제 대조

2026-10-19. 원본 Torch와 Rust에서 동일한 공식 NumPy mel, expanded token IDs, attention mask, feature mask 및 타임스탬프 위치를 사용했다. 한국어1개·일본어2개 입력의 선택 위치 전체 5000-class logits가 고정 허용오차를 통과하고 raw argmax도 일치했다. 이는 수치 대조이며 사람 단어 경계 품질이나 zero-duration 문제 해결을 의미하지 않는다.

## 완료한 세 입력

| 입력 | 타임스탬프 행 | 비교 원소 | 최대 절대 오차 | 허용오차 밖 | raw 클래스 일치 |
|---|---:|---:|---:|---:|---:|
| KO1883 | 38 | 190000 | 1.811981201171875e-5 | 0 | 38/38 |
| JA1516 | 56 | 280000 | 2.384185791015625e-5 | 0 | 56/56 |
| JA1634 | 62 | 310000 | 2.574920654296875e-5 | 0 | 62/62 |

합계780000원소·156개 raw 클래스다. 새 cohort에서 원본 forward3회와 native forward3회를 실제 완료했다. 비교기 실행은 신경망 호출을 추가하지 않는다. JA1634 비교 결과 `head-ja1634-paired-r1-head-comparison.json`과 원본 결과 SHA256 `a76e57544c7d4002ecaae0028a80f5e64ee58614e270c1e86043545839890a99`도 동일 묶음에 보존했다.

기준은 각 원소 `abs(diff) <= 1e-4 + 1e-4 * abs(reference)`이며 bitwise 동일하지 않다. 독립 비교기는 원본 전체 `[1,S,5000]`에서 같은 위치 p의 행을 다시 선택해 저장 행과 byte 일치를 검사했다. p-1로 이동하지 않는다. 양쪽 선택 클래스는 finite 5000값에서 최저 인덱스 우선 argmax로 재계산한다. native 내부 position/rotary와 비타임스탬프 전체 logits는 캡처하지 않았으므로 그 단계의 직접 대조 완료로 확대하지 않는다.

원본은 [708tensor 전체 strict CPU F32 로드](forced-head-full-load-actual.md), eager/eval/no_grad, use_cache=false를 사용하고 실제 thinker forward를 호출했다. 토큰 임베딩·결합 입력·text layer28개·norm·head 및 선택 logits를 관측했다. 원본 Python 파일을 수정하거나 대체 모델 구현을 만들지 않았다. Rust는 [별도 참조 입력 어댑터](../application/forced-head-reference-preparation-2026-10-18.md)의 고정 입력 경로를 사용한다.

## 고정 결과와 범위

비교 결과는 `.spec/artifacts/spikes/2026-10-18/results/`의 `head-ko-paired-r2-head-comparison.json`, `head-ja1516-paired-r1-head-comparison.json`이며 원본·native 결과 및 행별 통계 해시를 포함한다. 결과의 날짜 디렉토리는 최초 스파이크 묶음 식별자이며 현재 검증일과 구별한다. 원본 KO 결과 SHA256은 `a696a194287f4e4be3737f087cf0f919aa488b77994404fd46de41874e339fc5`, JA1516은 `6fd8ef5c95c35d21d6514d2a76b67078f6fe53026c4deae61cb2d38a34465d0d`이다.

KO 원본 raw word pair의 equal/reversed는 각각0개지만 인접 타임스탬프 감소는1개다. native inherited repair는 index28을 변경했다. 후속 [원본 fix_timestamp/parse_timestamp·최종 F64 실제 대조](forced-timestamp-functions-actual.md)에서 세 입력의 보정 결과까지 일치했으며 일본어 길이0 단어5개도 재현됐다. 과거 PCM 경로의 정렬17회·구조3통과/14실패는 다른 동결 cohort이며 이 새 동일 mel 실행으로 소급 변경하지 않는다. 품질 승인0개를 유지한다.

## 다음 게이트

독립 사람 경계 평가와 길이0 결과의 제품 실패 계약이 남는다. JA1634 native repair의 변경 위치는10·12이며 이 변경 자체는 올바른 경계의 증거가 아니다. SDK 참조 어댑터의 전체 보존 입력115개·patch7개·입력6개·설정·EXE1을 중앙 무결성 검사에 연결해 통과했다. 모델 기본값이나 GUI/CLI 지원 기능은 이번 수치 대조로 승격하지 않는다. 비교기 변경 후 Rust60tests·fmt check·clippy all-targets -D warnings·locked offline build를 통과했다.
