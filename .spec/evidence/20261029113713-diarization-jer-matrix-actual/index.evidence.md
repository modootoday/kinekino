---
id: evidence:kinekino/20261029113713
title: 원본 JER 교차 행렬 모든 원소 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원본 JER 교차 행렬 모든 원소 대조

2026-10-18. [이전 JER 함수 대조](diarization-jer-functions-actual.md)에 남았던 교차 행렬 검증을 완료했다. 같은6개 고정 실제 출력에서 원본 NumPy/SciPy 경로와 수정한 Rust 경로의 **78개 원소가 모두 정확히 일치**했다. frame counts와 JER의 기존 통과 조건도 유지했다. 새 모델 추론0회이며 모델 품질 승인은 아니다.

## 실제 대조 범위

| 고정 출력 | 행×열 | 비교 원소 | 결과 |
|---|---:|---:|---|
| diarize-ami-fp32-r1 | 3×8 | 24 | 정확히 일치 |
| diarize-ami-int8-r1 | 3×6 | 18 | 정확히 일치 |
| diarize-ami-fp32-local-r2 | 3×6 | 18 | 정확히 일치 |
| diarize-ami-int8-local-r1 | 3×4 | 12 | 정확히 일치 |
| diarize-holdout-fp32-pcm16-r1 | 2×1 | 2 | 정확히 일치 |
| diarize-holdout-int8-pcm16-r1 | 2×2 | 4 | 정확히 일치 |

각 입력은6000 scored frames다. Rust 평가기의 `Scores.jer_contingency_matrix`로 실제 계산에 사용한 정수 행렬을 노출하고, reference 입력 유도 단계에서 corrected matrix를 전달한다. 새 계산식을 별도 복제하거나 최종 JER에서 행렬을 역추정하지 않았다. application이 원본과 corrected 행렬의 전체 배열을 대조하며 `external_jer_function_parity_verified`에 이 조건을 추가했다. 과거 보고서의 재현 경로와 파일은 유지한다.

## 시도와 실행 증거

`jer-functions-matrix-r4`는 CPU63.57421875%로 admission 기준60% 미만을 충족하지 못해 worker 시작 전에 종료코드1로 끝났다. 가용 RAM은34965123072B였고 preflight의 ready=true를 별도 자원 기준의 통과로 오해하지 않았다. 사용자 앱은 종료하지 않았다. [사전 검사 기록](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/jer-functions-matrix-r4/preflight.json)을 보존했다.

기본 빌드 검사 종료 후 새 label `jer-functions-matrix-r5`는 CPU18.395303326810176%·RAM33874575360B로 통과했다. [감시 결과](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/jer-functions-matrix-r5/observation.json)는 exit0,19.5273173초,timeout=false,stream 초과false다. 기존 private Job·worker admission gate·BelowNormal·4threads·60초·각1MiB를 유지했다. 속도나 최대 메모리 비교 결과가 아니다.

EXE SHA256 `79a876e985d80b2f478228184450764e8f0c785d14957ab29d7d29b3eeb6ce6e`의 [선보존 build manifest](../../../evidence/player/jer-function-reference-builds/79a876e985d80b2f478228184450764e8f0c785d14957ab29d7d29b3eeb6ce6e/build-manifest.json) SHA256은 `3b3f401df95e808efb6945565768464add4b01eb087d30496966b89f15f50bb4`다. 원본/수정 행렬과 비교 boolean을 담은 [worker stdout](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/jer-functions-matrix-r5/worker.stdout) SHA256은 `8b516b1ce73187162497b011789a0ee9e4b05b814b06485e7dc766e541b80fba`다.

## 검사와 한계

평가기20tests·reference준비4tests·C-API실행기1test 및 각 crate fmt/clippy를 통과했다. 기존 경계 회귀 검사에 실제 교차 행렬941도 확인하도록 추가했다. 실제6개 대조와 합성 unit 검사 결과는 구분한다.

공유 library 변경 후 root spike46tests·fmt/clippy·locked offline build도 통과했다. root EXE `600c74cbdcf7410288a49864b5a4138bc971da59baf3a484b5c20e9df5c27775`의 컴파일입력154개와 EXE1개를 [선보존](../../../evidence/ai/model-spike-builds/600c74cbdcf7410288a49864b5a4138bc971da59baf3a484b5c20e9df5c27775/build-manifest.json)한 뒤 보고서를 갱신했다. snapshot1108개에 이번 admission 거절과 성공 관측을 포함했다. root 보고서 수집 실행을 원본 함수의 별도 재실행으로 합산하지 않는다.

기존 JER 최대차이 약1.42e-14 percentage points와 사전 허용치0.00000051은 그대로다. 이 결과는 같은 AMI의 두 시간 구간에 한정한다. 독립 경계/overlap corpus, 전체 dscore CLI의 Windows Perl 문제, 화자 분리 모델 오류 개선과 제품 job 통합은 여전히 남는다. [잔여 품질 게이트](../../../plans/player/ai-quality-gates.md)를 따른다.
