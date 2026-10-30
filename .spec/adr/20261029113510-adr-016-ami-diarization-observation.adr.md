---
id: adr:kinekino/20261029113510
title: "ADR-016: AMI 고정 입력의 화자 분리 관측"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# ADR-016: AMI 고정 입력의 화자 분리 관측

상태: accepted, 2026-10-18. 기존 화자 prototype의 실행 성공과 실제 화자 품질 평가를 분리하기 위해 human-derived AMI only_words RTTM/UEM과 sample-exact60초 입력을 고정한다. [자료 취득 증거](../../sots/verification/ai/diarization-corpus.md)를 따른다.

`observe-diarization`은 선보존된 현재 model-spike 자기 EXE의 정확한 `infer diarize SEGMENTATION CROP EMBEDDING`만 허용한다. segmentation은 기존 공식 변환 FP32 SHA256 `057ee564753071c0b09b5b611648b50ac188d50846bff5f01e9f7bbf1591ea25` 또는 INT8 `465d0975bf70fbf14fb77c0589a5d346a9c07c2170345f529cf774678446db76`에 한정한다. WeSpeaker hash `3955447b0499dc9e0a4541a895df08b03c69098eba4e56c02b5603e9f7f4fcbb`와 실제 AMI crop `54688b088c567a34d18901af07dba4606519e2edbec26d1f10d58e00ca0477a5`를 실행 전에 확인한다.

CPU4/interop1·BelowNormal·오프라인·600초 제한·RAM≥12GiB/유한0≤CPU<60% admission·root 단일 worker를 유지한다. 원 preflight와 앱을 보존하며 purpose diarization_correctness_not_performance·performance_comparison_eligible=false로 기록한다. 실제 graph output의 batch1·양수frames·7클래스·유한값을 확인한다.

원본 영상107초 오프셋 설명을 범용 입력에 하드코딩하지 않는다. source_offset_applied_us=0은 입력 파일 로컬 시간이며 원본 복원 offset은 별도 corpus manifest가 담당한다. legacy speakers 필드는 cluster 수임을 명시하고 speaker_cluster_count·speaker_count_approved=false를 추가한다. 과거 원시 출력은 수정하지 않는다.

새 corpus의 human reference3화자를 clustering에 강제하지 않는다. unknown short span·global identity가 없는 overlap을 reference/UEM에서 제외하거나 임의 화자로 배정하지 않는다. 기존 frame 경계 균등 투영은 근사치로 남기며, RTTM 정답 경계를 그에 맞춰 바꾸지 않는다. DER/JER·원본 scorer parity·전체 diarization 채택은 별도의 Rust 평가를 통과하기 전 미검증이다.
