---
id: evidence:kinekino/20261029113730
title: 같은 ONNX 패키지 FP32와 INT4 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 같은 ONNX 패키지 FP32와 INT4 대조

2026-10-18. [이전 혼합 INT4 관측](qwen-onnx-actual.md)은 Candle와 엔진·전처리가 달라 양자화만의 효과를 판단하지 않았다. 여기서는 동일 HF revision·Rust fork·실행 파일·CPU 설정에서 ONNX decoder 변형을 명시적으로 선택했다. 아래 순서를 실제 실행 전에 고정했고 총8회가 정상 종료·EOS도달·timeout=false로 끝났다. 자원 gate 대기1회는0추론으로 별도 보존했다. 제품 품질 승인은 아니다.

## 사전 고정 프로토콜

| 음성 | 첫 변형 | 둘째 변형 | 목적 |
|---|---|---|---|
| KO12.48초 | fp32 | int4 | 기존한국어결과·raw/EOS·CER 대조 |
| JA1516 16.74초 | int4 | fp32 | 기존오인식잔존입력 대조 |
| JA1634 12.12초 | fp32 | int4 | 숫자·문장 정확도 대조 |
| JA1578 9초 | int4 | fp32 | 짧은고유명사발화 대조 |

총8회로 각 입력·변형을 한 번 실행한다. 순서를 AB/BA로 번갈아 쓰되 OS cache를 비우거나 기존 앱을 종료하지 않는다. 각 실행의 admission CPU<60%, INT4 여유RAM≥12GiB·FP32≥20GiB를 강제한다. 한 worker씩 BelowNormal·CPU intra4/inter1·순차/no-spin·128token·timeout600초로 실행하며 각 process 종료 후 다음을 시작한다. admission 실패는0추론으로 보존하고 새 label로만 재시도한다.

이것은 제한된 입력의 같은 package 변형 대조다. 시스템의 다른 부하를 격리한 통계적 성능 benchmark가 아니며 `performance_comparison_eligible=false`를 유지한다. encoder는 두 변형 모두 같은 FP32 SHA, 저장 embedding은 같은 FP16→F32 cache, 같은 tokenizer/language prompt를 사용한다. decoder graph/weight 변형에 따른 결과 차이를 관측하되 원본 checkpoint·모든 export 경로의 일반적인 양자화 오차로 확장하지 않는다.

## 준비 관측

worker `58598154fb2d389b65c0a702eb81256de6c4958ff51325bea6ed7380d8797f36`과 [62파일 동시 보존](../../../evidence/player/qwen-onnx-builds/58598154fb2d389b65c0a702eb81256de6c4958ff51325bea6ed7380d8797f36/)을 사용한다. typed Variant는 `fp32`/`int4`만 받고 auto/default fallback을 허용하지 않는다. 각 variant의7개 파일 SHA를 loader 전에 확인한다. adapter fmt·4tests·Clippy all-targets Dwarnings·locked offline build를 통과했다.

FP32 decoder graph 두 개와3,006,500,864byte data를 같은 HF revision `4fc24a1402e74db89c4d2ef256875e71680128c4`에서 `qwen-onnx-fp32-weights-r1`로 순차 취득했다. 모두 LFS SHA 대조를 통과했으며 다운로드는0추론이다. data SHA256은 `98ea738a3453fada779aa6c0f6e05489f6ca4da26f3861c8af74637d5d03a3b0`이다.

기본1GiB/180초 한도를 전역 확대하지 않았다. 정확한 repo/revision/파일/LFS SHA 네 값이 맞는 이 data만3.1GB/600초로 허용하며 불일치는 기본값을 유지하는 Rust 회귀 검사를 추가했다. 다운로드 orchestration의34tests·fmt·Clippy와 실제 실행을 연결한 frozen build를 보존한다.

공유 encoder는 기존 `encoder.int4.onnx`가 실제 FP32이며 `encoder.onnx`와 SHA가 같다는 고정 metadata·카드에 따른다. Rust `prepare-encoder`가 같은 폴더의 encoder.onnx를 hard link로 준비하고 양쪽 hash를 검사한다. 기존 target이 다르면 중단하고 덮어쓰지 않는다. 새 encoder 다운로드를 하지 않는다. 결과는 runtime results의 `qwen-onnx-shared-encoder-r1.json`에 보존한다.

## 실제 결과

메모리 값은100ms 간격으로 측정한 해당 worker 프로세스 private 표본의 최댓값이다.

| 입력·변형 | 문자 오류 / 정답 문자 | CER | peak private bytes | inference ms | 전체 wall 초 |
|---|---:|---:|---:|---:|---:|
| KO FP32 | 3 / 53 | 5.66% | 7,053,258,752 | 6955 | 30.5680503 |
| KO INT4 | 3 / 53 | 5.66% | 3,030,020,096 | 3483 | 21.7988560 |
| JA1516 FP32 | 15 / 61 | 24.59% | 7,471,206,400 | 8704 | 48.9511769 |
| JA1516 INT4 | 8 / 61 | 13.11% | 3,444,867,072 | 3660 | 16.4649440 |
| JA1634 FP32 | 0 / 43 | 0% | 6,776,229,888 | 4485 | 45.4361334 |
| JA1634 INT4 | 0 / 43 | 0% | 2,753,253,376 | 2940 | 20.3354257 |
| JA1578 FP32 | 3 / 25 | 12% | 6,844,788,736 | 2828 | 28.6611844 |
| JA1578 INT4 | 0 / 25 | 0% | 2,820,526,080 | 3257 | 19.1124813 |

최대 private의 관측 범위는 FP32 약6.78~7.47GB, INT4 약2.75~3.44GB(십진 bytes 기준)였다. 이4입력에서는 각 쌍 모두 INT4가 더 낮았으나 모든 장치·입력·GPU의 절감률로 일반화하지 않는다. 마지막 JA1578의 실제 inference 함수는 INT4가 더 오래 걸렸다. 전체 wall에는 파일 hash·tokenizer 검사·load가 포함되며 실행 중 다른 부하도 있어 속도 우열의 통계적 benchmark로 표시하지 않는다.

모든 쌍의 raw 문장·생성 token IDs는 달랐다. 같은 CER도 같은 출력이 아니다. KO는 FP32 `다리 및`, INT4 `다리미`처럼 오류 내용이 달랐고, JA1634의0%도 숫자 표기·문장부호까지 같은 문자열이라는 뜻은 아니다. CER의 Unicode alphanumeric/lowercase·공백/문장부호 제외 규칙은 사전에 쓰던 evaluator와 동일하다. JA1516은 INT4에도 명백한 오인식이 남았다.

## 집계 무결성과 실행 증거

실제8회는 모두 orchestrator `7610dd71c94547fda9bc06ce8ef8b63285469f7576ad7e286ef5c005464ba804`의 [동시 보존89파일](../../../evidence/ai/model-spike-builds/7610dd71c94547fda9bc06ce8ef8b63285469f7576ad7e286ef5c005464ba804/)을 사용했다. 해당 manifest SHA256은 `1cf46772f0c2dde530fe39e47fe974341c60eec8f759280a6b36e2909e83c51c`다. 모든 worker는 위58598154 EXE이며 variant 외 실행 경로·prompt 설정은 유지했다.

`qwen-onnx-pair-ja1634-fp32-r1`은 CPU98.6328125%에서 worker 시작 전에 차단됐고 preflight SHA는 `4289d6ed187b962fc8e9f3ebd2e0f754d9aba950184df877a9d9262979cc3e3b`다. measurement/stdout이 생기지 않았음을 Rust 집계기가 확인했다. 새 label r2의 actual admission CPU7.2195%·free26,229,043,200bytes에서 실행했다. 나머지 입력은r1이다. 실패·대기 기록을 덮어쓰거나 품질 실패를 제외한 것이 아니다.

Rust `qwen-onnx-paired-summary`는 입력·변형마다 시작된 worker가 정확히1개인지 확인한다. 재시도 목록에서 좋은 결과를 고르지 않으며, 시작되지 않은 시도는 CPU/RAM gate 실패까지 확인해야 대기로 분류한다. 원본 measurement/stdout·개별 evaluation 일치·worker SHA·variant·같은 음성/실제 prompt/설정·공유4개 model component SHA/bytes·동일 gold를 대조했다. 4개 쌍 모두 통과했다. 집계 자체는 새 신경망0회다.

집계 도구는 [79169ff8·90파일](../../../evidence/ai/model-spike-builds/79169ff827e03314f4078c35a2048c94e01dc30cf2a97ee4453bd6d95bd8ac93/)로 실행했다. 새 입력/다른 prompt의 잘못된 pairing을 거부하는 검사 포함35tests·fmtcheck·Clippy all-targets Dwarnings·locked offline build를 통과했다. 결과는 `runtime/ai-spike/results/qwen-onnx-paired-summary-v1.json`, SHA256 `aef360f0d7780e2b2aed60ae9f9a7298498d72057e58d090736046ad7fa4d295`다. 각 개별 결과의 해시와 원문은 공통 evidence snapshot에도 보존한다.

## 남은 품질 범위

원본 checkpoint의 정확한 revision·변환 계보, 여러 독립 애니메이션 발화·화자·잡음/음악 혼합, 사람 검토, 자막 cue/화자/시간 정보 연결과 native 재배포는 별도 게이트다. 이 대조는 제품 GUI/CLI의 ASR 지원을 추가하는 작업이 아니다. INT4를 경량 후보로 유지하되 이번4개 짧은 발화만으로 자동 자막 기본 모델·품질 통과를 확정하지 않는다.
