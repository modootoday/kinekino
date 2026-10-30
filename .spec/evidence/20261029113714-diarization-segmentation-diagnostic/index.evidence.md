---
id: evidence:kinekino/20261029113714
title: 화자 누락의 segmentation 단계 진단
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 화자 누락의 segmentation 단계 진단

2026-10-18. [시간 분리 검증](diarization-temporal-holdout.md)의 FP32 출력은 입력 로컬0~20초에서 known/unknown/overlap span을 만들지 않고20초부터 speaker_1을 출력했다. reference의 MEO0154.6초 누락이 embedding 군집화 전 segmentation에서 시작하는지 확인한다. 정답을 모델 입력·threshold·speaker 수에 주입하지 않는다.

## 진단 계약

새 명시 role `diarize-pcm16-diagnostic`은 기존 pcm16-anchor의 입력·분류·anchor·cosine0.65·시간 변환·미귀속 overlap 정책을 바꾸지 않고 window별 관측만 추가한다. 순수 domain helper가 raw7-class frame scores·argmax class counts·exp row sum 범위와 실제 chunk의 PCM RMS/peak/zero sample 수를 보존한다. padding은 PCM 통계에서 제외하고 출력 shape·window/sample offset을 기록한다.

raw score는 threshold를 다시 고르기 위한 품질 정답이 아니다. exp row sum이1에 가까워도 calibration·확률의 정확도 검증이 아니다. RMS/peak 차이만으로 실제 발화/마이크 누락 원인을 단정하지 않는다. 기존 오류율이 진단 로깅 추가로 달라지면 그대로 개선으로 처리하지 않고 decision 경로 변경·재현 차이를 먼저 조사한다.

source·EXE 동결 및27tests/fmt/Clippy/build 후 기존60~120초 FP32/INT8 각각1회 새 label·자원 gate·순차1worker로 실행하고 같은5202 평가기로 점수 재현을 확인한다. 로그 추가 전 DER15.044017424975799/17.14106243949661, JER52.10018262457605/51.718722523920356을 baseline으로 유지한다. 실제 진단 결과·동결 identity는 아래에 추가한다.

## 실제 실행과 관측

27tests/fmt check/Clippy all-targets-Dwarnings/locked offline build 통과 후 EXE `20c91fc40f02872a6e6b007f79b016660125571990adaae435948f8ad725f9ee`의80파일을 선보존했다. manifest SHA256 `3e2945fa9a1b45ded79dc76bd1393c3be8ac104c89e8fb2b418b66a79209fe28`다. label `diarize-holdout-fp32-diagnostic-r1`은 CPU28.22265625%/여유35,502,874,624bytes, INT8 대응label은23.92578125%/35,336,949,760bytes에서 시작해 둘 다 success=true/exit0/timeoutfalse로 순차 종료했다. 기존 player1개와 사용자 ChatGPT/Terminal을 유지했다. ready=false를 자원 부족으로 혼동하거나 앱을 종료하지 않았다.

두 결과 모두6개10초창·각[1,589,7] 출력이며 모든 원시 frame scores를 보존했다. 같은5202 평가기를 실제 호출해 로그 추가 전 DER/JER를 각각 그대로 재현했다. 지표 재현은 tensor bit 동일성이나 성능 비교 검증이 아니다.

| window / 입력 로컬초 | PCM RMS | peak | FP32 argmax counts 0..6 | INT8 argmax counts 0..6 |
|---|---:|---:|---|---|
| 0 / 0~10 | 0.003543493917405164 | 0.04022216796875 | 589,0,0,0,0,0,0 | 589,0,0,0,0,0,0 |
| 1 / 10~20 | 0.0073010526865315145 | 0.13153076171875 | 589,0,0,0,0,0,0 | 559,0,0,30,0,0,0 |
| 2 / 20~30 | 0.005717904198211713 | 0.067291259765625 | 0,0,589,0,0,0,0 | 0,0,589,0,0,0,0 |
| 3 / 30~40 | 0.0068215546738869745 | 0.06964111328125 | 13,0,576,0,0,0,0 | 36,0,553,0,0,0,0 |
| 4 / 40~50 | 0.004288841371001674 | 0.035858154296875 | 0,0,589,0,0,0,0 | 0,0,589,0,0,0,0 |
| 5 / 50~60 | 0.0049441356392566135 | 0.033538818359375 | 85,0,504,0,0,0,0 | 56,0,533,0,0,0,0 |

실제 [보존 config](../../../evidence/ai/diarization-segmentation-diagnostic-2026-10-18/config.json)의 id2label에서 class0은 NO_SPEAKER, class2는 SPEAKER_2,4..6은 두 화자의 overlap이다. 고정 revision의 config SHA256은 `5a4c6bb4f49f1b47c9bd398aac78b8cd4a9f30a0ff91b5c2dc68d05e13e160cf`이며 기존 클래스 해석의 혼동은 확인되지 않았다. FP32 exp row sum은 약0.99999984~1.00000017이었다. 수치 관측이며 calibration 승인은 아니다.

## 원인 분리와 다음 게이트

reference의 MEO015는 입력 로컬3.81~6.33초와11.95~12.06초, 총2.63초를 단독 발화하고,20.50~21.66초와23.22~24.03초, 총1.97초를 FEE013과 겹쳐 발화한다. FP32는 앞의 두 창에서 모든 frame을 NO_SPEAKER로, 뒤의 창에서는 모든 frame을 단일 SPEAKER_2로 분류했다. 따라서 MEO0154.6초 누락은 적어도 이 argmax 출력 단계에서 이미 나타난다. embedding cosine threshold 변경만으로 이 누락을 회복한다고 주장할 수 없다.

단독 발화 쪽의 입력 RMS가 항상 검출된 창보다 작지는 않다. RMS만으로 음량 부족·마이크 불일치·실음성 누락을 단정하지 않는다. 원음 청취·source/reference alignment·짧은 문맥/overlap 집계·원본 전처리의 더 넓은 대조가 필요하다. 균등 frame 시간 투영은 근사치지만1178frame 전체의 NO_SPEAKER 판정을 단순한 시간 반올림만으로 설명하지 않는다. reference를 보고 수동 threshold를 맞추는 변경은 하지 않았다.

실제 stdout/.json/-evaluation.json은 각 label의 [보존 결과](../../../evidence/ai/model-spikes-2026-10-18/results/)에 둔다. 원본 JER 대조용 Rust adapter는 독립 연구 세션에서 준비 중이며 root의 실제 실행 전에 JER/global parity를 true로 표시하지 않는다. 진단 실행 성공은 모델 품질·전체 원인 해결·제품 기능 구현의 승인이 아니다.
