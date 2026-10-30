---
id: adr:kinekino/20261029113527
title: "ADR-014: 고정 E5 retrieval의 관측 전용 실행"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# ADR-014: 고정 E5 retrieval의 관측 전용 실행

상태: accepted, 2026-10-18. [ADR-013](adr-013-embedding-correctness-observation.md)의 MiniLM 허용 자산을 바꾸지 않고 별도 `observe-retrieval` 경로로 제한된 실제 자막 검증을 수행한다.

현재 source manifest를 먼저 동결한 model-spike 자신의 `infer embedding-e5`만 허용한다. 고정 E5 FP32 ONNX SHA256 `ca456c06b3a9505ddfd9131408916dd79290368331e7d76bb621f1cba6bc8665`, tokenizer `0b44a9d7b51c3c62626640cda0e2c2f70fdacdc25bbbd68038369d14ebdf4c39`, pooling `987f7a67a38fa564c849bb5d277c52ab9088a84368fc0be31a354125aebb12a0`, 원본33cue fixture `b20ea540f1784e21acc5d78b6eb355ca465d39687b271a33ac8facff7cc237b8`를 검사한다. 다른 worker·role·추가 인자·다른 파일은 거부한다.

CPU4/interop1·BelowNormal·오프라인·RAM≥12GiB·0≤CPU<60%·root 단일 worker·600초 제한 및 원래 preflight.ready/앱 목록 보존을 유지한다. 사용자 앱과 열린 UI는 종료하지 않는다. 결과는 retrieval_correctness_not_performance·performance_comparison_eligible=false다. UI 부하와 다른 모델의 wall/private 표본을 통제 성능 우열로 사용하지 않는다.

모델별 domain 정책으로 query/passage 접두어를 원문과 별도로 구성한다. graph는 검토된 세 입력 이름과 첫 token output 이름을 확인하고, ORT dtype 검사 및 실제 출력[1,n,384]·유한값·mean/L2 검사를 통과해야 완료다. 예상과 다르면 실패 증거를 보존하며 임의 입력 이름/수학으로 보정하지 않는다. 이 검사는 원본 모델 변환 parity나 검색 정답 승인과 다르다.

공식 최대512tokens와 이번 비교 제한128tokens를 구별한다. 임시 relevance와 원문 복사·무관 질의는 기존 평가 범위를 유지하며 사람 gold·no-match threshold·전체 언어/장문 품질은 미검증으로 둔다. 자세한 상태는 [E5 검증](../../sots/verification/ai/retrieval-e5.md)을 따른다.
