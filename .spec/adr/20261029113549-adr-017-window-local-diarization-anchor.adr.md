---
id: adr:kinekino/20261029113549
title: "ADR-017: 창 내부 로컬 화자의 anchor를 공유하는 실험"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# ADR-017: 창 내부 로컬 화자의 anchor를 공유하는 실험

상태: accepted for bounded experiment, 2026-10-18. 제품 기본값 또는 모델 품질 승인이 아니다. [AMI baseline](../../sots/verification/ai/diarization-quality.md)의 높은 혼동·과분할에 대해 모델 정보를 버리는 adapter 정책을 원인 후보로 분리한다.

baseline FP32의10초 window1에서 같은local_class2의 두 발화가 speaker_1과speaker_3으로 나뉘었다. 기존 구현은 모든 연속 run마다 독립 embedding/greedy assignment를 수행했다. [공식 pipeline source](https://github.com/pyannote/pyannote-audio/blob/main/src/pyannote/audio/pipelines/speaker_diarization.py)는(chunk,speaker)별 embedding 개념을 사용하지만, 우리의 longest clean-span 방식은 원본 mask pooling/전체 clustering 구현과 같다고 주장하지 않는다.

새 명시 role `diarize-local-anchor`는 각10초 창에서 단일 local_class1..3별로 가장 긴 비겹침 구간 하나를 선택한다. 최소8000samples/0.5초는 baseline과 같다. 동률은 먼저 나온 구간을 사용하고 anchor는 시간순으로 embedding한다. 같은 창/로컬 class의 다른 구간은 그 anchor의 global cluster ID를 공유한다. 충분한 anchor가 없으면null, overlap은global_speakers=null로 유지한다. 서로 다른 창의 class번호를 전역 화자로 간주하지 않는다.

단일실험에서 segmentation/WeSpeaker weights·입력·Hamming 전처리·전역 cosine0.65·시간 projection은 유지한다. 오디오를 splice/재표본화하거나 human reference의3명으로 군집 수를 강제하지 않는다. 모델 architecture의 로컬최대3클래스와 전체 reference3화자를 혼동하지 않는다. 기존 `diarize` role은 baseline 경로로 남긴다.

domain은 span 유효성·anchor 선택만 담당하고 infrastructure는 ORT/fbank/embedding을 수행한다. 실제 anchor의 window/local class/sample 좌표/global cluster를 출력에 보존한다. 관측 gate는 기존 고정 모델/crop/embedding SHA와 CPU<60%/RAM12GiB를 유지하고 두 명시role만 허용한다.

검증은 공유 구현 변경의 baseline control, 실험 FP32/INT8의 새label·frozen source/EXE, 동일 Rust DER/JER 평가로 분리한다. 개선되더라도 같은60초 입력에서 관측한 효과이며 held-out·다국어·공식 scorer parity·전체 diarization 또는 자동 자막 적용 승인으로 확대하지 않는다.
