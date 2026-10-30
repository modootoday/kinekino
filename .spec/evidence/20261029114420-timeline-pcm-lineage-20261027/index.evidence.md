---
id: evidence:kinekino/20261029114420
title: 실제 원본 PCM 재생·클립 계보
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 원본 PCM 재생·클립 계보

2026-10-27 중앙 PM. `production/pcm_lineage.rs`가 원본 ProducedAudioPcm을 보유한 채 실제 FFmpeg raw 입력 재생 디코드를 실행하고 원 PCM 전체와 직접 바이트 비교한다. 클립은 원본 1/16000 clock의 half-open 범위를 정확한 바이트 offset으로 변환해 create-new 파일로 기록·해시·보유한다. 재생 PTS를 원본 PTS로 간주하지 않는다. 전체 timeline Verified 권위나 AI 품질 승인이 아니다.

## 입력과 검증

SOURCE `B1B46C92308C404C7756E67B45F76ABEDCA92C36780AA99FAA66F6738D1353E2`/2208파일, inventory `5F4DE664EFD598635FB882E18243F7D32B8DB296385DA5CEA61515EFDC4ECAF6`. 입력 소유 디렉터리는 `runtime/verification/source375-timeline-lineage-input-r1`다. 원375 private overlay·기존 canonical/각 이전 epoch와 구분하며 이번 기록은 canonical 승격이 아니다. preimage와 SHA 목록은 timeline-pcm-lineage-preimage-r1에 보존했다.

| 로그 | 실제 근거 |
|---|---|
|461|native-proposal-window/all-targets strict 통과|
|462|정책21통과/5ignored: 음수 origin·half-open 범위·초과 거절·단일 바이트 변경 거절 포함|
|464|실제 시험 no-run 통과|
|465|실제1/9.28초 통과: 영상/clock/원PCM 회귀·정상 재생/클립 계보·생성 후 취소|

후속 [동일 입력 통합](source375-timeline-lineage-regression-20261027.md)의 선택190·정상2bins·저장CLI3·fmt 및 전후2208파일 content/identity 동일도 통과했다. 정책21/실제1을 전체 제품 인수로 확대하지 않는다.

원본/재생 PCM은 10078204 samples의 전체 바이트가 일치했고, 양쪽 SHA256은 `74F30535A9EF581F3A048917941DE6F41DAD1C40101149C1EBE84352A1300D8B`다. source clock272000~408000(17~25.5초)의 clip_start_sample272000/clip_samples136000, clip SHA256 `A2F8019D45FEB1109EF9A16ACCC3A6650B2F0F2DC068F2EFD76392F8EC39F33B`를 보유한다. 실제 재생 childPID14316/creation134375277735543630/exit0/비강제다. 범위 밖 입력은 child 생성 전 InvalidArgument/owner 없음으로 거절했다.

재생 생성 후 취소는 최초 Cancelled, 원 PID37488/creation134375277745462274/exit124/forced=true/Job empty/cleanup_error=null, observation_published=false를 보존했다. pending owner는 파생 입력 PCM도 borrow해 원 파일 guard가 물리 종료까지 유지된다. 이를 정상 child 성공으로 계상하지 않는다.

원 증거는 `runtime/verification/timeline-lineage-production-actual-datalab-r1/RETURN.json`, REPLAY-CANCEL.json 및 component별 INPUT/DERIVED-INPUT/STARTED/PROCESS/RETURN, 로그 source375-datalab-b0-r1-logs/461 이후다. SOURCE/producer/도구/nomination guard와 슬롯 독점·재예약도 유지했다. 외부 프로그램은 기존 SHA 고정 FFmpeg6.1/FFprobe를 사용했다.

## 미완료와 적용 판단

decoded origin·범위 끝점 capture, 모든 관측을 결합한 opaque proof/custody 등록, 제품 trusted admission/CLI startup, 전체CLI 인수와 실제9 AI/GUI는 남는다. archive reader·저장 Verified·부분 producer만으로 새 권위를 공급하지 않는다. 수동GUI·추론·모델 다운로드·WARP·push는 재개하지 않았다. 새 의존성 설치도 없다.

modoo.today `modootoday-free/ai-quality-plus/agent-run-postcondition-and-replay-check` 본문 SHA61843099b7d54c3d7ac226218a5ee1b6ccdbe36ef84268762dbc33a68fbb58b5/worked-example SHA80c9fe8a8a79dd8e08d84b36483d7742f273e55fb2769d2e6156a1111fdd5c11의 주장·원 실행·후조건 분리를 적용했다. 기존 QA fixture/code-change 검토 지침에 따라 합성 경계 시험과 실제 source-bound 실행을 구분했다. 기존 소유권·WIP·원 실패·보류를 유지한다.
