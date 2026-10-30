---
id: evidence:kinekino/20261029113715
title: EmbeddingGemma 2 실제 자막33입력 FP32 실행
status: active
created: 2026-10-29
updated: 2026-10-29
---
# EmbeddingGemma 2 실제 자막33입력 FP32 실행

후속 [같은 worker FP32/q4 대조](embeddinggemma2-quantized.md)를 완료했다. 아래 양자화 잔여 항목은 첫FP32 단계의 상태이며 후속 근거와 구분한다.

2026-10-18. [준비한 텍스트 export](embeddinggemma2-preparation.md)를 Rust ORT CPU에서 실제 실행했다. **26개 실제 한국어 자막과7개 query/control, 총33입력 모두768차원 finite·정규화 벡터를 반환했다.** 실행 완료이며 모델 품질·제품 채택·자동 검색 판정 승인은 아니다. 같은 Sintel 자료와 assistant 임시 relevance를 재사용한 실험으로 독립 인간 gold가 아니다.

## 입력과 실행 계약

fixture SHA256 `b20ea540f1784e21acc5d78b6eb355ca465d39687b271a33ac8facff7cc237b8` 및 고정 graph/data/tokenizer 해시를 재검증했다. repo/revision은 준비 문서와 같으며 원본 Google framework parity는 미검증이다. corpus는 `title: none | text: `, query는 `task: search result | query: ` 접두사를 사용하고 원문을 자르거나 trim하지 않았다.

고정 tokenizer.json의 TemplateProcessing을 Rust tokenizers0.22로 실행했다. 모든 입력에서 BOS2/EOS1과 최대256 tokens를 검사하고 media marker255999/258881/258884를 거부했다. batch1, padding 없음,3개 media features는 Float32[0,512]다. token IDs·mask·실제 model_input·truncation 여부를 행마다 보존했다. 실제 잘린 입력은0개다.

Session의 sentence_embedding을 그대로 사용해768차원과 norm 오차1e-4 이내를 확인했다. token 출력[1,T,768]을 mask-aware mean/L2로 재계산한 diagnostic은33개 모두 사전 허용치1e-5 이내였고 최대차이는1.7881393432617188e-7이었다. 이 재계산은 worker 내부이며 원본 framework나 외부 독립 replay가 아니다. raw token tensor 전체는 보존하지 않았고 sentence vector·입력·진단값을 보존했다.

## 내용 평가와 실패 사례

| 지표 | EmbeddingGemma 2 | 같은 fixture 기존 E5 |
|---|---:|---:|
| 임시 positive5개 MRR | 0.8285714285714285 | 0.825 |
| 임시 hit@3 | 0.8 | 0.8 |
| blade/blood/land/notfound 첫 정답 순위 | 각각1 | 각각1 |
| thanks 첫 정답 순위 | 7 | 8 |

고마움 query는 여전히 관련 자막을 상위3개에서 찾지 못했다. exact-copy control은1위였고 positive 집계에서 제외했다. 무관 query의 top cosine은0.5962036644678697, cue16이며 정답이 없는 질의를 positive 실패율에 섞거나 자동 match로 승인하지 않았다. E5와 작은 MRR 차이는 이 임시5질의에서 thanks 순위8→7에 따른 것이며 일반 한국어 검색 우위·no-match 임계값·추천 품질을 입증하지 않는다.

기존384차원 평가 함수를768로 무조건 바꾸지 않았다. 전용 진입점에서768차원 및 graph/data identity·Gemma prefix를 검사하며 같은 relevance/ranking 계산을 공유한다. 기존 MiniLM/E5의384차원 계약을 유지한다. [평가 JSON](../../../evidence/ai/model-spikes-2026-10-18/results/embeddinggemma2-fp32-r1-semantic-evaluation.json)에 모든26개 순위와 제외한 control을 보존했다.

## 자원 관측

`embeddinggemma2-fp32-r1`은 preflight CPU25.364431486880466%·free RAM36038959104B로 admission을 통과했다. 원래 ready=false와 다른 검토 player 존재를 보존했으며 해당 앱을 종료하지 않았다. [worker 측정](../../../evidence/ai/model-spikes-2026-10-18/results/embeddinggemma2-fp32-r1.json)은 exit0,7.4328302초,100ms 간격64표본의 peak private893104128B·working set893878272B, BelowNormal·4threads·timeout=false였다.

성능 비교 적격false다. 이 값은 로드·검증·추론·행별 저장을 포함한 프로세스 관측이며 모델만의 지연이나 device 전체 RAM이 아니다. OS cache를 비우지 않았고 GPU·NPU·노트북 전력 조건을 측정하지 않았다. 기존 공통 worker의600초 기한과 자체·자식 표본 수집을 사용했으며 사설Job/1MiB stream cap을 사용한 JER reference 실행과 혼합하지 않는다.

## 실행과 증거 보존

실행 전 EXE SHA256 `6c6811d8b7c7ad0c69f5c1a4065aed327a6e2b5d3083d3933e9990bac78013d5`의 컴파일입력158개와EXE1개를 [보존](../../../evidence/ai/model-spike-builds/6c6811d8b7c7ad0c69f5c1a4065aed327a6e2b5d3083d3933e9990bac78013d5/build-manifest.json)했다. [실제 stdout](../../../evidence/ai/model-spikes-2026-10-18/results/embeddinggemma2-fp32-r1.stdout) SHA256은 `0ae3f7f556fd0c250f97961f9abdf789e655e674b476c948cecaa067028c28a8`, 측정 JSON은 `498b3c2b94b49cc0a61fc0e5cf1645ca80a7da56ae272f493ccfc85064abec97`이다.

[행별 진행 기록](../../../evidence/ai/model-spikes-2026-10-18/results/embeddinggemma2-fp32-r1-rows/progress.json)은 started33/completed33이다. 각 행을 즉시 새 디렉터리에 저장하여 중간 실패 시 완료 출력을 잃지 않도록 했다. 모델 묶음1개의33행 실행이며33개 모델이나 독립33자료로 세지 않는다.

별도 평가 EXE `67eceff2500e3acfda87a3a88bb78cb345d59b4225498c6afe4bd8ce0c6a6abb`도 컴파일입력158개와EXE1개를 [선보존](../../../evidence/ai/model-spike-builds/67eceff2500e3acfda87a3a88bb78cb345d59b4225498c6afe4bd8ce0c6a6abb/build-manifest.json)했다. 평가 JSON SHA256은 `deb4164ba752c0e14670160f4f847fe0cea5d09c385961f0b04f449282996bdc`. 평가와 보고서 갱신은 새 신경망 실행0회다. snapshot1168개에 입력/진행/출력/평가를 보존했다.

## 검사와 다음 작업

Rust 테스트48개·fmt check·clippy all-targets -D warnings·locked offline build 통과. prefix의 원문 공백 보존과 unknown role 거부를 검사했고 최종 테스트 모듈 순서 lint를 수정했다. 실제 forward와 평가 JSON 검사는 합성 unit 검사와 구별한다.

다음은 같은33입력의 q4 또는q8 한 조건과 자원·벡터·rank 대조다. MRL 절단은 별도 후처리 실험이다. 원본 framework oracle·export/quantization 계보·독립 인간 relevance·다른 작품/언어·no-match 보정·제품 API/job 통합·장치별 실행은 남아 있다. 모델 품질 승인0개는 유지한다.
