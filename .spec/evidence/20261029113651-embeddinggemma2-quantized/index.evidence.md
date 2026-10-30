---
id: evidence:kinekino/20261029113651
title: EmbeddingGemma 2 같은 입력 FP32와 q4 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# EmbeddingGemma 2 같은 입력 FP32와 q4 대조

2026-10-18. [첫 FP32 실행](embeddinggemma2-fp32.md)에 이어 같은 Rust worker로 새 FP32 기준과 q4를 순서대로 실행했다. **각33입력의 cue·prefix·token IDs·mask·truncation이 모두 같았다.** q4는 이번 표본에서 private memory가 낮았지만 벡터 차이와 임시 검색 순위 손실도 관측했다. 모델 품질·자동 채택은 승인하지 않는다.

## 실제 관측

| 항목 | 새 FP32 | q4 |
|---|---:|---:|
| label | embeddinggemma2-fp32-paired-r2 | embeddinggemma2-q4-r1 |
| 완료 입력 | 33 | 33 |
| text external data bytes | 1084170240 | 174028800 |
| worker 전체 관측 초 | 8.4830909 | 4.9063231 |
| 표본 peak private bytes | 872968192 | 447537152 |
| 표본 peak working set bytes | 880443392 | 454680576 |
| 100ms 표본 수 | 70 | 40 |
| 임시 positive5개 MRR | 0.8285714285714285 | 0.825 |
| 임시 hit@3 | 0.8 | 0.8 |
| thanks 첫 정답 순위 | 7 | 8 |
| token mean/L2 diagnostic 최대차이 | 1.7881393432617188e-7 | 2.086162567138672e-7 |

네 개의 다른 positive query는 모두 첫 정답1위였고 exact-copy control도1위였다. q4 무관 query의 top은 cue16/cosine0.6121439033629016이다. 이는 보정된 확률이나 자동 match 기준이 아니다. 임시 정답은 assistant 작성5개이며 독립 인간 gold·다른 작품/언어를 대표하지 않는다.

33개 정규화 벡터의 FP32↔q4 cosine은 최소0.956500456004164, 평균0.9828747954560688이었다. 각 행의 최대 성분 차이와 전체 검색 순위를 [비교 결과](../../../evidence/ai/model-spikes-2026-10-18/results/embeddinggemma2-fp32-q4-comparison-r1.json)에 보존했다. 같은 shape/norm과 내부 pooling diagnostic 통과를 FP32 수치 동일성으로 승격하지 않는다. 양쪽 모두 잘린 입력0개,768차원 finite/norm 검사를 통과했다.

## 입력과 artifact 고정

repo `onnx-community/embeddinggemma-2-ONNX`, revision `daa72c51243991dfcaf9f9137d2c573d8f7790c0`는 같다. q4 graph490742B의 SHA256은 `f9eeba97acddf139b8ee2ddf04bc30dceafa88de93fadf74d7644e0d61a477a9`, data174028800B는 `c3975f2d1ab7a1878ae31a7d7a9b7804a827aff3800b60dfceafce21cac3df49`다. [취득 manifest](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/embeddinggemma2-text-q4-r1/result.json) SHA256 `d208930e50c868a29c938e8810f0a30058c89e1a22820f77c0244caa1e9d6072`로 실제 LFS 대조를 보존했다.

FP32 파일과 tokenizer·원문 card/config 해시는 [준비 정본](embeddinggemma2-preparation.md)을 따른다. domain의 명시적 Variant가 고정 파일/hash를 선택하며 임의 경로나 미검토 정밀도는 허용하지 않는다. q4 실패를 FP32로 대체하는 fallback은 없다. q4는 배포물 이름이며 모든 initializer가4bit라는 inventory나 quantizer/exporter의 완전한 계보를 검증한 결과는 아니다.

같은 실제 자막 fixture SHA256 `b20ea540f1784e21acc5d78b6eb355ca465d39687b271a33ac8facff7cc237b8`, tokenizer, batch1/최대256/BOS2/EOS1/빈Float32[0,512] features3개와 CPU intra4/inter1을 사용했다. 비교기는 정확한 graph/data pair, 같은 executable/orchestrator hash,33개 전처리 필드, finite768벡터/norm 및 평가 stdout hash를 검증했다. 예전 FP32 결과를 새 실행 파일의 결과로 재귀속하지 않고 새 기준을 실행했다.

## 자원과 재현 근거

둘 다 자원 admission·BelowNormal·4threads·600초 기한에서 exit0/timeout=false였고 한 번씩 실행했다. FP32 preflight CPU22.94921875%·free RAM35537113088B, q4는49.70703125%·33897914368B였다. 사용자 앱을 종료하거나 OS cache를 비우지 않았다. 성능 비교 적격false이며 이 시간은 hash 검증·로드·tokenization·추론·행별 저장을 포함한다. 표본 private는 장치 전체 RAM이나 모델만의 메모리가 아니며 반복 성능·항상 빠름·다른 장치 적합성을 승인하지 않는다.

동일 EXE `0cd52d70c56ceb418e07c00bfa3e00c0e57327126eb41e35901cc47edd99d6c8`의 컴파일입력159개와EXE1개를 [선보존](../../../evidence/ai/model-spike-builds/0cd52d70c56ceb418e07c00bfa3e00c0e57327126eb41e35901cc47edd99d6c8/build-manifest.json)했다. 실제 [FP32 stdout](../../../evidence/ai/model-spikes-2026-10-18/results/embeddinggemma2-fp32-paired-r2.stdout) SHA256은 `c57b40c974685c92c5e5909cf145a2dee137f43faf3520861c54c551dbf79251`, [q4 stdout](../../../evidence/ai/model-spikes-2026-10-18/results/embeddinggemma2-q4-r1.stdout)은 `462a299ebe634d7221ac4d67a9ed7fc7822095ea7b17683c7481635b5abb6736`다. 비교 JSON SHA256은 `b243e5eff9e6d50dd02f7c5f28c28fbd12044c38513cf94cd2ec1c2dee33f971`이다.

각 실행의 행별 진행과 벡터를 보존했다. 새 기준33행과q4 33행은 같은 모델 묶음의 두 조건이며 모델 수를 늘리지 않는다. 평가/비교/보고서 수집은 새 신경망 실행0회다. Rust48tests·fmt check·clippy all-targets -D warnings·locked offline build를 통과했다.

보고서 snapshot1258개에 취득 manifest, 두 worker의 원시 출력·측정·행별 진행 및 평가/비교를 포함했다. 새 실행 전에 보존한 소스·바이너리와 이후 수집한 결과의 범위를 구분한다.

## 다음 게이트

독립 relevance와 no-match 보정, thanks 실패 원인, 다른 작품/언어·문맥 길이, 원본 framework/변환 계보와 initializer/operator inventory가 남는다. MRL 차원 절단과 가중치 양자화는 별도 실험이며 자동 조합하지 않는다. 제품 모델 기본값·SDK job·GUI 연결·품질 승인은 이 비교로 변경하지 않는다.
