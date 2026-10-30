---
id: sot:kinekino/live-timeline-v2-reader
title: Live timeline v2 reader·권위 경계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Live timeline v2 reader·권위 경계

2026-10-27. 실제 publisher가 생성한 constructor-private PublishedTimelinePackage만 PublishedTimelineResolver의 입력이다. JSON/경로/호출자 manifest로 token을 만들지 않으며 token에는 실제 producer가 계산한 snapshot·원 source identity·tool/producer pin·명시 실행 계약을 보유한다. 임의 JSON의 SHA는 권위가 아니다. 재시작 후 임의 자료 자동 입장은 지원하지 않는다.

`live_2_1`은 mapping_revision·6역할 contract/root·3도구 pin·실제 Rust producer pin·budget·video stream index를 필수로 선언한다. contract에는 Default가 없으며 video interval과 capture source_us도 명시 필드다. 초기 live_2나 필드가 빠진 자료를 역사값으로 채우지 않는다. 새 immutable case의 initial mapping revision은 publisher가 명시적으로1을 발행하며 association CAS revision과 구별한다.

case/media/reference 불일치는 파일 조회 전에 거절한다. 동일 item에 선언된 asset ID/SHA/bytes를 기존 ReviewCaseEvidence로 읽고, 원 package를 포함한 새 package가 기술 검사 추가 외에 원 모델 조건/결과/기존 검사/known issues를 변경하지 않았음을 확인한다. 새 asset ID는 package와 case의 두 ID 규칙을 모두 충족한다. 원 파일명과 asset ID를 구별하며 기존 strict ID 규칙을 넓히지 않는다.

reader는 다음을 검사한다.

- 전체 artifact 역할/파일 집합의 누락·중복·추가 거절, 도구/producer 바이너리의 선언과 실제 SHA·size 보유.
- 원 INPUT source·tool path/FileID/SHA·budget와 전체 argv의 명시 계약 일치. producer/reader가 공통 인자 plan을 사용하며 기록 경로는 문자열 대조만 하고 실행/탐색하지 않는다.
- STARTED와 PROCESS의 동일 PID/creation, child exit0·Job empty·비강제·실제 stdin EOF 요청. v1 observer_accepted를 numeric exit로 바꾸지 않는다.
- raw FFprobe frame/window·16k audio 정수 clock, 원 PCM/재생 전체 exact byte·sample 수, 원본의 exact clip slice.
- decoded PNG 전체 검증과 원 showinfo PTS/timebase·origin metadata hash, source header/크기·선택 media SHA.
- 실제 publisher snapshot과 공통 TimelineObservations domain 규칙의 mapping 일치. BUNDLE/RETURN 요약의 true 필드는 권위나 원 관측을 대신하지 않는다.

lease가 새 case의 원 ReviewSession과 모든 선언 asset guard를 유지한다. 기존 application retain이 case/media/evidence lease와 snapshot을 함께 유지하고 변경을 재검사한다. 이 입장 성공은 아직 Verified 저장전이나 playback_ready·AI 사용자 품질 승인이 아니다. Reader는 새 child나 모델을 실행하지 않는다.

v1 archive reader와 기존 trusted nomination은 보존한다. 제품 startup·SDK/정상 CLI 저장/재열기/재생 조립, host 재시작 입장과 IO fault 전체·다른 offset/codec/source 전수는 남는다. actual9와 GUI 품질/시각 인수는 별도다. [실제 v2 검증](../../verification/application/timeline-live-reader-20261027.md).

## 동일 SDK 저장·재생 조립 후속

`LocalReviewRuntimeConfig`는 root/tool/profile/budget/deadline/output 제한만 소유하고 증거 형식을 소유하지 않는다. `assemble_sdk_with_evidence`는 trusted Rust에서 받은 같은 `Arc<dyn TimelineEvidenceResolver>`를 source 서비스와 playback launcher에 공급한다. worker는 원 Arc로 lease를 확보하고 원 case/media/proof를 물리 종료까지 보유한다. 새 v2 token을 v1 admission으로 변환하거나 저장 Verified에서 authority를 재구성하지 않는다. 기존 `LocalReviewSessionConfig`와 `assemble_sdk`/`assemble_local` archive 진입점은 유지하며 같은 공통 runtime 경로로 변환한다. 런타임 설정과 resolver는 SDK 요청의 JSON 입력이 아니다.

새 조립 구현의 실제 검증은 별도 후속 입력으로 실행하며 이전 SOURCE1E2 성공으로 승계하지 않는다. 제품 startup/재시작 입장·actual9·GUI/사용자 품질 인수는 계속 별도다.
