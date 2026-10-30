---
id: adr:kinekino/20261029113532
title: "ADR-013: 고정 의미 벡터 모델의 관측 전용 비교"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# ADR-013: 고정 의미 벡터 모델의 관측 전용 비교

상태: accepted, 2026-10-18. 사용자 앱을 보존하면서 실제 자막 입력의 FP32/양자화 출력을 확인한다. idle 성능 비교를 승인하는 결정이 아니다.

## 결정

`observe-embedding`은 source manifest가 먼저 동결된 현재 model-spike 실행 파일의 `infer embedding`만 호출한다. 검토된 FP32/AVX2 ONNX SHA 두 개와 실제33개 자막/질의 fixture·tokenizer·pooling 설정 SHA를 고정한다. 다른 executable·role·추가 인자·변경된 파일은 거부한다. 공통 관측 게이트 RAM≥12GiB/0≤CPU<60%, CPU4/interop1·BelowNormal·오프라인·root 단일 worker·timeout/종료 후 unload를 유지한다. 열린 UI와 ChatGPT/Terminal은 변경하지 않는다.

## 해석

실행 결과에 worker뿐 아니라 orchestrator EXE와 보존 manifest SHA도 기록한다. 원 preflight.ready 및 앱 목록은 유지한다. 출력은 embedding_correctness_not_performance·performance_comparison_eligible=false로 표시한다. 표본 private/working set과 wall은 실행 상황 기록이며, 서로 다른 UI 부하 아래 속도 순위나 일반적인 양자화 효과의 통제 실험으로 해석하지 않는다.

의역5개 임시 관련 ID는 독립적인 사람의 정답이 아니다. exact-copy와 no-match 질의는 양성 검색 집계에서 제외하고 no-match rejection threshold는 별도 검증한다. 임시 MRR/hit@3·벡터 유사도·실행 성공은 제품 적용이나 전체 의미 검색 품질 승인과 구분한다. 일반 benchmark/다른 모델 실행은 기존 ready 정책을 유지한다.
