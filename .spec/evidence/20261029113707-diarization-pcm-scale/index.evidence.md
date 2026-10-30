---
id: evidence:kinekino/20261029113707
title: WeSpeaker PCM 배율 단일 요인 실험
status: active
created: 2026-10-29
updated: 2026-10-29
---
# WeSpeaker PCM 배율 단일 요인 실험

2026-10-18. 실행 전 가설과 범위를 기록한다. [고정 upstream infer_onnx.py](https://github.com/wenet-e2e/wespeaker/blob/45941e7cba2c3ea99e232d02bedf617fc71b0dad/wespeaker/bin/infer_onnx.py)의 compute_fbank는 정규화 waveform에32768을 곱한 후 Hamming/80mel/25ms/10ms/dither0/use_energyfalse·utterance별 평균 제거를 적용한다. 기존 Rust 경로는 normalized waveform을 그대로 사용했다.

평균 제거가 모든 배율 차이를 없앤다고 가정하지 않는다. log 에너지 floor가 개입할 수 있다. 합성 작은 진폭 입력에서 기존/배율 적용의 평균 제거 후 feature 차이를 확인했지만 이는 실제 embedding 품질 증명이 아니다. Torchaudio 전체 수치 parity도 미검증이다.

새 명시 role `diarize-pcm16-anchor`는 기존 local-anchor와 같은 segmentation·10초창·근사시간·anchor 선택·cosine0.65·overlap 미귀속 정책을 사용한다. embedding fbank 입력 배율만32768로 바꾸며 segmentation waveform은 기존 normalized PCM 그대로다. 이전 role과 결과는 유지하고 제품 기본값은 바꾸지 않는다. reference3명에 맞춰 threshold를 조정하지 않는다.

## 검증 계획

24tests/fmt/Clippy/locked offline build와 EXE/source 선보존 후 기존 AMI ES2004a0~60초·모델 FP32/INT8 각1회, 새 label·CPU<60%/여유12GiB admission·4threads/1interop/BelowNormal로 순차 관측한다. 결과는 같은5202 Rust evaluator로 DER/JER·MISS/FA/CONF를 대조한다. 기존 local-anchor FP32 DER69.24773/JER71.72540, INT8 DER67.97645/JER75.88526을 baseline으로 유지한다. 성능 순위·다른 입력·품질 승인으로 일반화하지 않는다.

초기 상태는 코드/합성 검사 준비이며 실제 배율 적용 모델 실행·개선 판정은 아직0회다. 실제 결과와 frozen identity를 아래에 추가한다.

## 실제 결과

24tests·fmt check·Clippy all-targets-Dwarnings·locked/offline build가 통과했다. EXE `9ef54944aa203d5b08dc5945baf0ac15a3e5e731060f7accfd10ccc203db623e`의78파일을 실제 추론 전에 보존했다. manifest SHA256은 `c75e8440fee8a5b83c1d420c112c9efa455a260f90cbc6b5bf1ce5b6d7b3fca6`다. 실행 경로는 gate가 허용하는 tools/model-spike/target/debug이며 worker와 orchestrator의 SHA가 이 동결 EXE와 일치한다.

첫 FP32 pcm16-r1은 동결 보존 디렉터리의 EXE를 직접 worker 경로로 지정해 허용 위치 gate에서 거부됐다. 모델 실행0회/preflight만 보존했다. 허용 경계를 넓히지 않고 새 label r2에서 기존 허용 경로·동일 SHA로 실행했다.

| 실행 label | admission CPU % | 여유RAM bytes | DER % | JER % |
|---|---:|---:|---:|---:|
| diarize-ami-fp32-pcm16-r2 | 13.320274240940256 | 38,228,369,408 | 69.24772682445759 | 71.72540450264229 |
| diarize-ami-int8-pcm16-r1 | 9.9609375 | 38,252,924,928 | 67.97645463510848 | 75.88525753651477 |

두 worker success=true/exit0/timeoutfalse로 순차 종료했다. 기존 player1개와 ChatGPT/Terminal을 유지했고 ready=false를 자원 gate와 혼동하지 않았다. 동일5202 평가기를 실제 호출해 새 출력 각각을 평가했다. 결과는 배율 미적용 local-anchor와 DER/JER가 같다. tensor의 bit 일치나 모든 입력에서 영향이 없음을 주장하지 않는다. 작은 진폭 합성 feature 차이는 실제 cluster 품질 개선으로 이어졌다고 볼 수 없다.

전처리 계약 차이는 확인·수정했지만 이 입력에서 품질 향상은 관측되지 않았다. 새 경로는 명시 실험 role로 유지하고 제품 기본값 채택은 하지 않는다. 새로운 출력에 embedding_pcm_scale32768을 기록했다. segmentation normalized 입력·시간 경계·unknown/overlap 정책은 유지한다. 따라서 현재 큰 MISS와 CONF가 배율 누락만으로 설명된다는 가설은 이 결과로 지지되지 않는다.

실행·stdout·평가는 각 label의 runtime/ai-spike/results 원본과 [보존 결과 디렉터리](../../../evidence/ai/model-spikes-2026-10-18/results/)에 둔다. 참조 전처리 전체 수치 parity·다른 입력·창 간 군집화·시간 receptive-field 보정·JER 원본 대조는 남아 있다. [공식DER4쌍대조](diarization-der-reference-actual.md)는 이전 local/control 출력 범위이며 새 pcm16 출력의 별도 원본 실행으로 소급 표시하지 않는다.
