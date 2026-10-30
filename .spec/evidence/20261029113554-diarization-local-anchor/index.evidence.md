---
id: evidence:kinekino/20261029113554
title: 창 내부 로컬 화자 일관성 실험
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 창 내부 로컬 화자 일관성 실험

2026-10-18. [baseline의 실제 품질 미달](diarization-quality.md)에 대해 모델이 출력한 창 내부 local_class를 버리는 adapter 정책을 분리한다. [ADR-017](../../../adrs/player/adr-017-window-local-diarization-anchor.md)의 새 명시 role이며 제품 기본값/자동 화자 승인으로 취급하지 않는다.

## 관측과 변경

기존 FP32 window1의local_class2는10.814937~14.719812초에서speaker_1,18.828500~20.000000초에서speaker_3으로 나뉘었다. 기존 방식은 같은 창의 동일 로컬 화자라도 매 연속 구간의 embedding을 독립 군집화했다. 새로운 `diarize-local-anchor`는 각 창/local_class별 가장 긴 단일 비겹침 구간≥8000samples만 embedding하고 그 ID를 같은 창/class의 다른 구간과 공유한다.

reference의3명이나 단어 경계를 읽어 clustering을 맞추지 않는다. 로컬1..3은 모델 architecture의 클래스이며 전역 화자 수와 다르다. 창마다 캐시를 초기화하고 창 사이에는 기존 cosine0.65의 greedy clustering을 유지한다. weight·입력·Hamming·frame projection·overlap 미귀속 정책은 유지한다. 오디오를 splice하거나 새 전역 speaker ID를 reference에서 주입하지 않는다. 충분한 anchor가 없는 class는null이다.

domain helper는 class/좌표/구간 중복과 최소 길이·가장 긴 anchor 선택을 담당한다. inference adapter는 실제 embedding과 local/global ID 연결 및 anchor의 원본 window sample 좌표를 보존한다. Python pipeline의 mask pooling이나 전체 community-1/VBx와 동등하다고 주장하지 않는다.

## 실행 전 검사와 control

22tests/fmt/Clippy-Dwarnings/locked offline build를 통과한 EXE SHA256은 `54d2bd3c7613aca41dd3413b4c48f059f857724ab08e454e0244255f38b61127`다.78파일 선보존 manifest SHA256은 `ca8ac211ad24e71556e8ed75a2060cca00d4fe4c1809231d2dc08764e73e9e24`다. 기존 baseline role도 같은 공유 경로에서 유지한다.

`diarize-ami-fp32-control-r2`/r3는 preflight CPU69.53125%/99.70703125%로 거부됐으며 모델 worker0회다. 기록을 새 실행으로 덮어쓰지 않았다. 이후r4는 CPU25.915221579961468%·여유36,156,334,080bytes에서 baseline 실제 실행을 완료했다. 원ready=false/player1개도 유지했고 앱을 종료하지 않았다.

같은5202d56c Rust 평가기로 control-r4를 실제 평가한 결과 DER77.54037968441814%/JER77.78615375785219%로 최초 baseline 지표와 같았다. 이 검사는 파생 지표의 재현이며 모든 tensor/bit/segment JSON의 완전 일치를 증명한 것은 아니다.

새 local-anchor의 첫label `diarize-ami-fp32-local-r1`은 preflight CPU100%로 거부됐다. 이 label의 모델 실행은0회다. 이후 아래 새 label로 실제 실행했다. 사용자 앱/작업을 종료하거나 admission을 완화하지 않았다.

[control stdout](../../../evidence/ai/model-spikes-2026-10-18/results/diarize-ami-fp32-control-r4.stdout), [실행](../../../evidence/ai/model-spikes-2026-10-18/results/diarize-ami-fp32-control-r4.json), [평가](../../../evidence/ai/model-spikes-2026-10-18/results/diarize-ami-fp32-control-r4-evaluation.json)를 보존한다.

## 실제 실험 결과

동일 AMI ES2004a 첫60초, 동일 모델 revision·입력·5202 평가기에서 다음을 관측했다. DER/JER는 낮을수록 좋다. reference 화자 수를 추론에 제공하지 않았다.

| 정책·정밀도 | 실제 label | DER % | JER % |
|---|---|---:|---:|
| 기존 FP32 control | diarize-ami-fp32-control-r4 | 77.54038 | 77.78615 |
| local-anchor FP32 | diarize-ami-fp32-local-r2 | 69.24773 | 71.72540 |
| 기존 INT8 control | diarize-ami-int8-control-r2 | 77.88770 | 82.88453 |
| local-anchor INT8 | diarize-ami-int8-local-r1 | 67.97645 | 75.88526 |

FP32 실험은 admission CPU57.71484375%/여유30,151,573,504bytes, INT8 실험은16.69921875%/33,838,792,704bytes에서 시작해 success=true로 종료했다. INT8 control도 CPU10.25390625%에서 완료했고 원 baseline의 DER/JER를 재현했다. 모든 관측은 성능 순위용이 아니며 기존 사용자 앱과 작업을 유지했다. 종료 후 다른 세션의 cargo 보류를 해제했다.

같은 창의 동일 로컬 화자 일관성 수정으로 이 입력의 오류가 줄었지만 여전히 DER 약68~69%로 높다. 이 결과는 개별 checkpoint 품질이나 일반화, INT8 우위, 제품 기본값 채택의 근거가 아니다. 전처리·창 간 clustering·overlap·miss 개선, held-out 입력, 공식 DER 대조와 JER parity는 남아 있다. 실제 결과는 runtime/ai-spike/results의 각 label.json/.stdout/-evaluation.json에 원본을 유지하고 report로 보존한다.
