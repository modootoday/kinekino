---
id: evidence:kinekino/20261029113636
title: 원본 MiniLM의 독립 Rust 참조 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원본 MiniLM의 독립 Rust 참조 대조

현재 원본 경로 실제 추론은 총2회다. 아래 최초 R1 기록의 일반 자원 게이트 차이를 보존하고, 후속 R2에서는 ADR-015의 강제 관측 게이트를 적용했다. 두 실행을 같은 성능 조건으로 합산하거나 초회 기록을 덮어쓰지 않는다.

2026-10-18. 별도 Rust Candle0.9.2 CPU F32 어댑터로 공식 원본 safetensors를 실제 로드·추론하고, 기존 공식 ONNX FP32의33입력과 대조했다. 제한된 실행·수치 대조 완료이며 제품 검색 품질 승인이 아니다. [ONNX 관측](embedding.md), [모델 상태표](model-status.md), [별도 어댑터 준비 증거](../../../evidence/player/sentence-embedding-reference-preparation-2026-10-18.md)를 따른다.

## 실행 identity와 범위

repo `sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2`, revision `e8f8c211226b894fcb81acc59f3b34ba3efd5f42`, 원본 가중치470,641,600bytes/SHA256 `eaa086f0ffee582aeb45b36e34cdd1fe2d6de2bef61f8a559a1bbc9bd955917b`를 사용했다. 정확한 tensor 이름·shape·dtype를 실제 확인한 뒤 기존 Candle BERT를 로드했으며 추정 변환·엔진 수학 수정은 하지 않았다. 개발 코드는 Rust이며 tokenizers0.22.2에는 native Oniguruma 종속성이 있다.

최종 참조 EXE는6,216,704bytes/SHA256 `9047a6c1e0f59043674b81ea1e8f6c6a3c6a3f1f6bc29626bb647d4936816860`, [선보존 manifest](../../../evidence/player/sentence-embedding-reference-builds/0.1.0-9047a6c1/build-manifest.json)는SHA256 `7a2b4c5e279f313ca1a68f399dc4e3c48c8763e0ab9cc6403ae2baf8dcf67754`다. 준비 시7tests/include-ignored·fmt·Clippy-Dwarnings·release가 통과했다. 준비 세션의 추론0회와 root의 후속 실제1회를 구별한다. f5a95a03은 pre-final 후보이며 실제 모델 실행에 사용하지 않았다. 엔진 원문4파일과 비교 pooling 원문은 선택 보존이며 전체 dependency archive가 아니다.

root는 동결된 EXE를 별도 engines/0.1.0-9047a6c1에 복사하고 hash 일치를 확인했다. label `embedding-original-candle-r1`으로 기존 일반 measure를 사용했다. root orchestrator는SHA256 `e984bd3cf688091c143b6a4e0366f835bc77d0e5c27bea57749fffb4817db6a6`이며 선보존 manifest SHA256 `5d6e3eb09ec345ecbf57ac53c29306d354d1326b16fa2eaf315d3a50505476de`를 기록했다. CPU4·RAYON_NUM_THREADS=4·BelowNormal·오프라인·600초 제한·root 단일 worker였다. E5 worker가 종료된 뒤 시작했고 종료0/timeout=false,33개 출력 검사를 통과했다.

## 자원 게이트의 실제 차이

일반 preflight는ready=true·validation_apps=[]·여유34,757,873,664bytes·CPU63.76953125%였다. 원래 일반 게이트의80% 미만 조건은 충족했으나, 참조 준비에서 제안한 관측 전용60% 미만 조건은 충족하지 못했다. 이를60% 통과로 기록하거나 기존 원시 실행 기록을 수정하지 않는다. 사용자 앱·OS·메모리 캐시는 변경하지 않았다.

raw measurement는 standard_measurement/performance_comparison_eligible=true를 그대로 보존한다. 본 대조는 다른 시점·엔진·1회 표본이며 준비 조건도 달라 **분석 결과의 performance_comparison_eligible는false**로 제한한다. wall4.0790365초·표본 private peak1,023,844,352bytes·working set955,236,352bytes는 실행 상황 기록이다. 메모리·속도 우열이나 일반적인 순수 Rust 대 native 효과로 해석하지 않는다. 향후 관측용 재실행은60% 조건을 코드로 강제한 별도 고정 gate를 사용해야 한다.

## 토큰·벡터·순위 대조

원본 fixture SHA256 `b20ea540f1784e21acc5d78b6eb355ca465d39687b271a33ac8facff7cc237b8`와33개 cue 원문/순서가 일치했다. 참조 경로는 실제 ids·mask·types·128truncation·무 padding·각 출력shape/norm/float bits를 보존했다. 현재 비교기는 tokenizers0.22.2와 고정 tokenizer SHA를 사용해 과거 ONNX source의 encode(text,true)/padding 없음/128 설정을 신경망 없이 재현했다. 재현한 모든33개 토큰 배열이 Candle 실제 입력과 같았고, 과거 ONNX에 남아 있는 token count·truncation도 일치했다.

**과거 ONNX 실행은 토큰 배열 자체를 저장하지 않았다.** 위 배열 비교는 현재의 독립 재현이며 과거 프로세스 입력을 소급 캡처한 증거가 아니다. 이 제한을 벡터 대조와 함께 보존한다. 새 E5 adapter부터는 실제 입력 배열도 출력한다.

| 지표 | 실제 관측 |
|---|---:|
| 비교 벡터 |33 ×384 |
| 평균 vector cosine |1.0000000003856273 |
| 최소 vector cosine |0.9999999867505128 |
| 최대 성분 절대차 |2.3841857907380692e-7 |
| query top1 일치 |7/7 |
| 임시 의역5개 MRR |양쪽0.55 |
| 임시 hit@3 |양쪽0.8 |
| 관련 cue 순위 |양쪽1/1/3/3/12 |

정규화한 F32 벡터의 dot product를F64로 합산하므로 평균이 반올림 오차 수준에서1을 조금 초과한다. 값을 임의 clamp하거나 bit 동일로 표현하지 않는다.7개에는 무관 질의·원문 복사도 있으므로7/7은 의미 정확도100%가 아니다. 이33입력의 약한 retrieval 순위는 원본에서도 재현됐으며, 변환 차이가 이 실패의 주원인이라는 가설을 지지하지 않는다. 모든 입력의 변환 동등성·다른 언어·장문·다른 dtype까지 일반화하지 않는다. 사람 relevance·no-match·held-out 기준은 남는다.

root 비교 도구 build `1d4e69e1087623a83360e7555e87acd9dad509e16b39a50fae5e050b9c936361`은74파일 선보존 및17tests/fmt/Clippy-Dwarnings/build를 통과했다. 실제 stdout hash·모델/fixture identity·완료 measurement·행 대응·벡터 norm을 검사한 [대조 결과](../../../evidence/ai/model-spikes-2026-10-18/results/embedding-original-onnx-reference-comparison-r1.json)는SHA256 `67487f35c87d30da52baa4d2a8c98023c9b4ef4b01016c9470575ca656bc4841`다. [원시 출력](../../../evidence/ai/model-spikes-2026-10-18/results/embedding-original-candle-r1.stdout), [실행](../../../evidence/ai/model-spikes-2026-10-18/results/embedding-original-candle-r1.json), [임시 의미 평가](../../../evidence/ai/model-spikes-2026-10-18/results/embedding-original-candle-r1-semantic-evaluation.json)를 보존한다.

## 후속 R2: 강제 관측 게이트 실제 통과

[ADR-015](../../../adrs/player/adr-015-original-embedding-observation-gate.md)의 `observe-reference`를 구현하고18tests/fmt/Clippy-Dwarnings/locked build를 통과했다. CPU 경계 검사와 정확한 명령·threads4·추가 인자 거부를 검증했다. 불필요 참조 Clippy 오류를 수정한 뒤 전체 검사를 다시 통과했으며 실패 빌드를 실제 실행에 사용하지 않았다.

orchestrator build `b0ef04dd38617d68be5eb4b0708e950a154a0c2bc9a98a6ecb67d8fe6714356b`의75파일을 선보존했고 manifest SHA256은 `99460da488863cb42c9ba320a4d378f27aa53b826111fd06daca9ccbd445cad2`다. worker9047a6c1과 원본 가중치/fixture는 초회와 같다. label `embedding-original-observed-r2`에서 실제 preflight CPU23.22643343051506%·여유35,897,638,912bytes가 새 조건을 충족했다. 열린 player1개로 ready=false인 원문을 그대로 보존했고 앱을 종료하지 않았다.

33입력 추론은 종료0·timeout=false였다. wall12.4864027초·표본 private peak1,023,094,784bytes/working set950,689,792bytes였다. purpose original_embedding_correctness_not_performance·performance_comparison_eligible=false를 코드에서 기록했다. 종료 후 시스템 CPU70.60546875%도 원문 그대로 보존한다.60% admission은 시작 전 표본 조건이며 실행 중 사용자 작업에 대한 hard cap이 아니다. 초회와 다른 UI/동시 작업 상태의 wall을 속도 회귀 또는 개선으로 해석하지 않는다.

후속 비교기는 실행 성공·purpose·false eligibility·자원 admission을 다시 검사한 뒤 원본 cue/token replay/384벡터·순위를 대조했다. ONNX 대비 최대성분차2.3841857907380692e-7, 최소cosine0.9999999867505128, 의역순위1/1/3/3/12와 임시MRR0.55/hit@3 0.8로 초회와 같은 대조 지표였다. 이 지표만으로 초회와 후속 벡터의 bit 동일성을 주장하지 않는다.

실행 후 분석 build `65d0c28ce86f3ee7ec67a7737d24886772cfc26b165c74c3d0bfd8b85410d28f`도75파일 선보존·18tests/fmt/Clippy/build를 통과했다. [R2 원시 출력](../../../evidence/ai/model-spikes-2026-10-18/results/embedding-original-observed-r2.stdout), [실행 기록](../../../evidence/ai/model-spikes-2026-10-18/results/embedding-original-observed-r2.json), [평가](../../../evidence/ai/model-spikes-2026-10-18/results/embedding-original-observed-r2-semantic-evaluation.json), [독립 대조](../../../evidence/ai/model-spikes-2026-10-18/results/embedding-original-onnx-reference-comparison-r2.json)를 보존한다. 대조 SHA256은 `940071160e1823d23288cc53d37fcb9253e5502a4594ec6f00ef635e9d5c52ee`다.

별도 spec-maintenance Rust 검사에서 최종 manifest30파일과 pre-final29파일의 크기·SHA·경로·EXE/버전명을 root가 독립 확인했다. 이59파일 검사는 준비 후보를 실제 추론으로 계산하거나 전체 native dependency 패키징·모델 정확도를 증명하지 않는다.
