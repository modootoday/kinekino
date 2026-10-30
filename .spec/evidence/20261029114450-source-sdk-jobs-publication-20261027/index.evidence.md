---
id: evidence:kinekino/20261029114450
title: 원 JobsCore 식별자·source 종료 관측·파일 발행 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 JobsCore 식별자·source 종료 관측·파일 발행 — 2026-10-27

## 실제 연결과 발견한 공백

[오프라인 body R5](source-sdk-offline-body-20261027.md)를 원 retaining JobsCore로 연결했다. 격리 mirror의 시험 preparation은 실제 AttemptScope에서 job/attempt/input/source/submission 식별자와 원 cutoff를 받는다. source admission은 실행 control의 동일 식별자와 cutoff를 검사한다. 브라우저 권한만 synthetic fixture이며 식별자 자체를 시험에서 생성하거나 JSON으로 역직렬화하지 않았다. fixture source cutoff는 원 core deadline 안에 cleanup3초를 예약하며 연장하지 않는다.

기존 download_registry/DownloadJobExecutor/RequestBoundPreparation/DownloadCoreSession·원 publication CAS·StagedDownloadOutput의 원 파일 guard/rename/verify를 그대로 소비했다. fixture BytesProbe는 원 VerificationLease의16byte만 검사하며 실제 미디어 decoding/track probe가 아니다. 외부 HTTP·모델·브라우저·GUI를 호출하지 않았다.

첫 실제 JobsCore 실행 R7은 source byte/원 child/receipt 정리에 성공했지만 `NeedsInput`/RevisionConflict `Settled publication requires the original successful attempt`로 끝났다. output=None/publication_started=false였으며 diagnostic blob54FC366D와 원 결과를 보존했다. core가 source 프로세스의 종료 관측을 받지 못해 NotStarted로 정산했기 때문이다. fake probe exit0이나 느슨한 publication gate로 우회하지 않았다.

격리 SourceOpener에 원 owner_observation을 추가했다. 기본값은 Pending으로, bool close=true나 worker JSON에서 종료를 추론하지 않는다. ProcessSourceOpener는 같은 원 host의 실제 process receipt와 원 물리 종료/transport receipt 완료 후에만 Exited를 전달한다. 원 host가 없고 실제 no-child 정리가 완료된 경우만 NotStarted다. source와 probe 관측을 합쳐 Pending·실패 코드·강제 종료·불완전 family/stream을 지우지 않는다. 기존 file-only 시험 opener4개는 source child를 만들지 않는 사실을 명시해 NotStarted를 구현했다. root 제품/원 R36은 변경하지 않았다.

## 첫 실패/선택 게이트

|로그/입력|실제 결과|
|---|---|
|1050|새 시험이 기존 media_download 마운트 문맥에서 인프라 경로를 찾지 못해 타입 실패. 원3leaf preimage-r6-typing 보존|
|1051/1052 R6|마운트 연결 수정 후 all-target strict/no-run 통과|
|1053 R6|jobs 디렉터리 미준비로 host 시작 전 NotFound,exact0pass/1fail·외부exit101/Empty/nonforced. runtime/verification/source-sdk-jobs-publish-os1-r6 원 증거 보존|
|1054/1055 R7|기존 디렉터리 계약을 맞춘 뒤 strict/no-run 통과|
|1056 R7|실제 source 이후 위 NeedsInput/정산 거절,exact0pass/1fail·외부exit101/Empty/nonforced. source child32852 exit0/Empty/nonforced. source-sdk-jobs-publish-os1-r7 원 증거·terminal-job/diagnostic 보존|
|1057 R8|종료 관측 보완의 첫 strict는 items_after_test_module 실패. preimage-r8-layout 및 원 stderr 보존|
|1058 R9|시험 모듈을 책임별 파일로 분리,제품 all-target/시험 feature strict 통과|
|1059 R9|제품 lib 및 media_download_owned test no-run 통과/75393ms. 후자는 실제 실행 아님|
|1060 R9|정확한 원 JobsCore/SDK/OS 종단1pass/906filtered/0.24s|
|1061 R9|종료 관측 불변식 순수3pass/904filtered|
|1062/1063|전체 fmt 통과/선택2284 content·identity 동일|

선택 input-r9 SOURCE.md SHA256 `DBCF23CE7A687129BBD1C47C319E6D9CA749224C5931C218D048EE57E31B08A0`,inventory `0A7A39B4E511C8ADF32B2DBEE213C4215ED5E133D2878B10ABD38BBF3B917A06`,2284files. 원 수정 전 core/adapter/기존 fixture4개 등은 preimage-r7-source-observation에 보존했다. R6/R7/R8 snapshot·producer·첫 실패를 덮어쓰지 않았다.

고정 R9 `native/source-sdk-integration-r1/fixed-r9/sdk-source-tests.exe`: SHA256 `91C6D7B5A137DD80DE3ECACA03389E3B121F0E9017E659294288CBDC5140D606`,108929024B. 같은 폴더 media_download_owned.exe53F6DACA38E0B4DD29664557C3ABD8B8883F4D296A3715BA8FEE57EC00CDA285/3575808B는 compile-only다.

실제 child는 원 R5 fixed fixture2895FD22/999424B이고 R9에서 재빌드했다고 계상하지 않는다. R9 operator index7ED34919C36B9FC2EE9A2B7681783109D670BF1BD276FBD95B315BD919B777BD/425B는 proof=false다. 원 C2 driver7E7AD258/45s+cleanup5s가 고정 test/child를 보유했다. 모든 live handle terminal 후 다음 heavy 작업을 시작했고 C-only/offline/locked/j1/conservative2를 유지했다.

## 성공의 실제 범위

정확한 case `infrastructure::download::process_source::tests::live::jobs::original_jobs_core_binding_drives_sdk_source_and_publishes_verified_fixture_bytes`.

원 job은 attempt1과 원 input SHA `9ff0268ff7d8f6b95ed52d065570265523f3be76b332e09d8274b610753f3551`를 유지해 Completed/publication_started=true/output16bytes/failure=null이 됐다. 공개된 fixture.bin의 SHA는 `541c1be35244beda30db9aa7f8c36dfabed97e21255a0aecdafb56f8df897219`이다. descriptor SHA는 request 선택 설명이며 미디어 내용 권한으로 바꾸지 않았다. source consume1/transport record1·원 retaining host joined/종료 receipt·마지막 guard1회 해제를 확인했다. guard 해제는 terminal record 관측과 경쟁시키지 않고 원 host 종료 뒤 검사했다.

원 source child PID35124/creation134375662260659066,exit0/Empty/nonforced; 원 raw EOF/final process receipt/IPC reap/local close 확인. IPC control/data EOF=false는 유지했다. 외부 test PID18364,exit0/Empty/nonforced,selected_one=true/ownership complete/raw EOF/first_error=null이다. 원 outcome/receipt/raw·inner-observed copy 및 원 tmp sdk-source-jobs-*를 `runtime/verification/source-sdk-jobs-publish-os1-r9`와 원 C tmp에 보존했다. 실행 후 고정 test hash와 root lock B2E3D540 동일도 확인했다.

순수3은 no-child를 process exit로 조작하지 않음,양쪽 실패/forced의 보존,Pending/불완전 EOF의 비완료를 확인한다. 실제 source permission·미디어 품질·전체 실패 행렬의 증거가 아니다.

## 남은 범위

이 연결은 private 후보이며 제품 canonical/CLI 등록은 하지 않았다. 실제 browser replay/current issuer·미디어 probe·정상 HTTP/제품 다운로드,CLI consumer 및 오류/취소/재시작 인수가 남는다. 실제 unassigned source close와 i32 범위를 넘는 Windows exit는 현재 owner observation이 Pending으로 남으므로 별도 typed 관측/실패 시험이 필요하다. probe 정리가 대기할 때 source도 원 예산 안에서 정리되는지 등 중복 owner의 실패/취소 행렬도 미검증이다. Compile-only 기존 media_download_owned4/CLI fixture의 실제 회귀는 미실행이다. 원 DFB0 190/bins·전체CLI/AI 사용자 품질0/9·GUI 시각 인수와 구별한다.

modoo fixture/mock 및 postcondition 지침을 재사용해 초기 준비 실패·실제 정산 거절·정상 원 binding/CAS/파일·synthetic permission/probe를 구별하고 첫 실패·원 소유권·자원 예산을 보존했다. 기존 WIP/소유권/source375 private overlay·추론/다운로드/수동GUI/WARP/push 보류를 유지한다.
