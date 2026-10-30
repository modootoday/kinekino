---
id: evidence:kinekino/20261029113557
title: AMI 실제 화자 분리 출력과 남은 정답 평가
status: active
created: 2026-10-29
updated: 2026-10-29
---
# AMI 실제 화자 분리 출력과 남은 정답 평가

후속 상태: [local-anchor 실험](diarization-local-anchor.md)에서 같은 입력의 오류 감소를 관측했고, [원본 DER 실제 대조](diarization-der-reference-actual.md)는 기존 control/개선 FP32·INT8의4개 exact 쌍에서 통과했다. 아래 초기 평가의 전체 scorer/JER 미검증 표시는 유지한다. 제품 품질 승인은 아니다.

2026-10-18. [실제 취득한 AMI Mix-Headset60초](diarization-corpus.md)에 기존 pyannote segmentation+WeSpeaker prototype을 FP32/INT8로 각1회 실행하고 Rust 평가기로 human-derived reference와 대조했다. **실행·실제 점수 계산은 완료했으며 현재 prototype 품질은 미달이다.** 공식 scorer 수치 parity는 별도 미검증이다. reference의 세 화자를 모델에 강제하지 않았다. 한국어·일본어·애니메이션·원거리 마이크 성능으로 일반화하지 않는다.

## 고정 실행 계약

[ADR-016](../../../adrs/player/adr-016-ami-diarization-observation.md)에 따라 source/EXE77파일을 선보존했다. worker/orchestrator SHA256은 `43bedaa70bb822f0e51f0a44336bdaa8dc326bd346ef7bde5e7b4846f2061b41`, manifest SHA256은 `d816de5102a0fd3758f33a73c754936a27f1d59fd9228468f9621fba4161bdc8`다.19tests/fmt/Clippy-Dwarnings/locked offline build를 통과했다. 두 실행은 순차 root 단일 worker였고 종료0·timeout=false다.

segmentation은 `onnx-community/pyannote-segmentation-3.0` revision `733a93b6473d019a773298e08cefa686894b1854`의 FP32/INT8, embedding은 `onnx-community/wespeaker-voxceleb-resnet34-LM` revision `6a61a1833ff2583aabeba044f5c8221f00b67ceb`다. 입력 crop SHA256 `54688b088c567a34d18901af07dba4606519e2edbec26d1f10d58e00ca0477a5`·embedding·모델을 gate에서 확인했다. 같은 Hamming fbank·cosine0.65 greedy clustering·입력60초를 유지하고 segmentation 정밀도만 바꿨다. 완전한 community-1/VBx 파이프라인이 아니다.

## 실제 관측

| 항목 | FP32 | INT8 |
|---|---:|---:|
| label |diarize-ami-fp32-r1 |diarize-ami-int8-r1 |
| 시스템 speaker cluster 수 |8 |6 |
| segments 원소 수, unknown 포함 |11 |8 |
| identity 미확정 overlap 후보 수 |1 |1 |
| 입력 로컬 source_offset_applied_us |0 |0 |
| preflight CPU % |29.19921875 |26.881720430107524 |
| preflight 여유 RAM bytes |35,022,811,136 |35,176,099,840 |
| 원 preflight.ready |false, player1개 |true, validation_apps없음 |
| wall seconds |39.350473 |0.9397178 |
| 표본 private peak bytes |111,804,416 |142,319,616 |
| 표본 working set peak bytes |110,374,912 |118,079,488 |

두 실행 모두 CPU4/interop1·BelowNormal·오프라인이며 purpose diarization_correctness_not_performance·performance_comparison_eligible=false다. 다른 UI·동시 작업·캐시 상태의1회씩 관측이므로 wall/메모리 차이를 양자화의 통제 성능 우열로 해석하지 않는다. FP32의 worker 내부 inference_seconds는2.090968, INT8은0.6218019였으며 process wall과 같은 지표가 아니다. 실행 후 시스템 CPU도 FP3285.26829268292683%/INT820.11661807580175%로 달랐다. admission은 사전 표본 조건이며 전체 호스트 hard cap이 아니다. root는 사용자 앱·OS·메모리 캐시를 변경하지 않았다.

reference상 crop 내 발화자는 세 명이다.8/6개 시스템 군집은 실제 화자 수를 맞혔다는 결과가 아니며, 군집이 줄었다고 정확도가 개선됐다고 판정하지 않는다. 예를 들어 FP32의 첫 known segment는10.814937초부터였고 reference에는0.37초 시작 turn이 있다. 전체 matching·unknown·overlap·UEM을 계산하기 전 원인이나 오류율을 단정하지 않는다.

새 출력은 source_offset_applied_us0·speaker_cluster_count·speaker_count_approvedfalse와 legacy speakers 필드의 군집 의미를 명시했다. 짧은 span은null, overlap은global_speakers=null로 보존했다. frame 경계는 아직 균등 투영 근사치이며 실제 모델 receptive-field calibration·다른 경계 정책과 같다고 표시하지 않는다.

## 평가 단계

별도 Rust 평가기는 RTTM/UEM의 crop·시간·화자와 hypothesis/audio hash를 검증하고 continuous DER collar0/overlap포함/UEM전체, 별도10ms JER와 최적 전역 화자 mapping을 구현했다. unknown/미귀속 overlap을 reference에서 제외하거나 임의 공통 speaker로 맞추지 않는다. reference상 silence도 평가에 포함한다. 준비 단계의 합성 검사와 아래 root 실제 실행을 구별한다.

[FP32 stdout](../../../evidence/ai/model-spikes-2026-10-18/results/diarize-ami-fp32-r1.stdout)·[실행](../../../evidence/ai/model-spikes-2026-10-18/results/diarize-ami-fp32-r1.json), [INT8 stdout](../../../evidence/ai/model-spikes-2026-10-18/results/diarize-ami-int8-r1.stdout)·[실행](../../../evidence/ai/model-spikes-2026-10-18/results/diarize-ami-int8-r1.json)을 보존한다. 평가가 없음을 숨기기 위해 임의 DER/JER를 만들거나0오류로 기록하지 않는다.

## 실제 human-reference 대조

[평가기 준비](diarization-evaluator-preparation.md)의 합성19tests/fmt/Clippy/build를 확인하고, root가 frozen EXE SHA256 `5202d56c5d91afece9a344a9b6d2572868707ecf46904b425ae5fa73a10aca6b`와 manifest SHA256 `74a79fdd2d27b08cfce78aadd8ce64d610d79dd9875ad9af446ab258136e3f38`를 재확인했다. frozen verify-build로23개 파일 hash를 확인한 뒤 실제 reference/UEM/WAV/hypothesis와 각 새 output을 지정해 두 번 평가했다. 모델을 다시 실행한 것이 아니다.

| Rust 평가 지표 | FP32 | INT8 |
|---|---:|---:|
| DER % |77.54037968441814 |77.88769723865877 |
| JER % |77.78615375785219 |82.88452876260921 |
| reference speaker-time us |20,280,000 |20,280,000 |
| MISS speaker-time us |8,295,813 |10,127,750 |
| FA speaker-time us |749,251 |442,125 |
| CONF speaker-time us |6,680,125 |5,225,750 |
| unresolved union us |1,392,125 |1,035,624 |
| unresolved overlap union us |475,375 |390,437 |

reference 화자는3명으로 파싱됐다. UEM은60초 전체이며 silence가 남아 false alarm을 계산한다. 미확정 시간은 reference에서 제거하지 않았고, 유효한 speaker hypothesis가 없는 시간은 MISS 등에 포함된다. 따라서 MISS는 단순 VAD 음성 검출 실패만을 뜻하지 않으며 identity abstention도 포함할 수 있다. 이 결과는 segmentation·embedding·greedy clustering·근사 시간 projection을 합친 prototype의 오류이며 개별 checkpoint 하나의 독립 오류율이 아니다.

INT8의 군집 수가 더 적어도 이 입력의 DER/JER는 좋아지지 않았다. 자동 화자 자막 적용을 승인하지 않는다. 각 실제 평가 JSON의 external_scorer_parity_verified=false를 유지한다. JER의10ms grid·decimal-us 정규화와 원본 scorer의 binary64 onset+duration/인쇄 precision 차이는 독립 대조 전 미해결이며 NIST 인증/전체 scorer 동일성을 주장하지 않는다.

[FP32 평가](../../../evidence/ai/model-spikes-2026-10-18/results/diarize-ami-fp32-r1-evaluation.json)와 [INT8 평가](../../../evidence/ai/model-spikes-2026-10-18/results/diarize-ami-int8-r1-evaluation.json)는 audio·RTTM/UEM·stdout·평가기 SHA 및 mapping/원시 합계를 기록했다. 다음은 공식 DER의 독립 수치 대조와 Hamming 전처리·군집 분절·miss/미확정 원인의 검토다. 파라미터를 reference3명에 맞춰 oracle로 조정하지 않는다.
