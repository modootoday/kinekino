---
id: evidence:kinekino/20261029114501
title: Fresh live v2의 동일 SDK 저장·재생 실제 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Fresh live v2의 동일 SDK 저장·재생 실제 검증

2026-10-27 중앙 PM. SOURCE `49C38F206BD897032E0FEE6887F2E8AC181CD74F77719A1AE208E3BB10B74FC4`, inventory `C7238CCAC1C477151B45910C809B66E25C994C5B2FA89D2F02563C52FAAE2424`, 2221파일을 `runtime/verification/source375-live-sdk-input-r1`에 고정했다. 원375/private overlay·기존 archive의 성공과 구분한다. 편집 전 원본은 `runtime/verification/source375-live-sdk-r1-preimage`에 보존했다.

`LocalReviewRuntimeConfig`가 실행 설정만 소유하고 동일 Arc resolver를 SDK source 서비스와 worker에 공급한다. 기존 archive public struct/진입점은 유지한다. fresh token을 archive admission으로 가장하거나 저장 Verified로 권위를 재구성하지 않는다. [계약](../../player/application/live-timeline-reader.md).

| 로그 | 실제 결과 |
|---|---|
|524|native-proposal-window lib/actual target strict exit0, 기존 의존성 blitz-paint deprecated 경고 별도|
|525|archive SDK 경계2/2: overlap은 writable 초기화 전 거절·readonly 무생성|
|526|새 입력 snapshot2221파일|
|527|실제 시험 no-run exit0/70265ms|
|528|fresh generation/publication/reader→동일 SDK/정상 CLI40→headless 재생/종료 actual1 통과, 시험57.80초/runner58499ms|

528 원본은 `runtime/verification/timeline-live-sdk-actual-datalab-r1`이며 `live-sdk-cli` 아래 요청/응답/stdout/stderr/각 process receipt와 WITNESS·TRANSPORT-SHUTDOWN·RETURN을 보존했다. actual producer SHA `3C5D5B65D4AD290DEBDF9906A28E569E6E692FFA29501533C5987124999DBD82`, CLI SHA `ECD3BFCE05B127A4E2EE2A116F73CD15783361142529A3A8C26893293958BB37`. 이 CLI는 actual no-run이 생성한 구성 시험 대상이며 후속 정상 bins와 별도 식별한다.

새 package/case SHA `36DBB96447AC0CA062E40E2B05C2CB0FB96DE4ABE73C904575A78536B96CDE8D`, manifest SHA `4D6501FE5F43FC499E3B9B87C97C77DCBD94C396C959FD42199C5EB638A0A475`. Sprite 기술 표본의 원본 SHA7CFC8B8C…와17~25.5초 mapping을 사용했다. 새 빈 user root에서 CLI create revision1→증거 revise Verified revision2→동일 요청 replay→foreign proof 거절→inspect 동일 저장 association→같은 selection 재생을 실행했다. schema/open replay·오염 path 거절·외부 scope/instance/selection 거절·범위밖 seek·play/seek/음량/mute/pause·경계 관측·close/replay·Unverified child 미생성도 포함한다.

TRANSPORT-SHUTDOWN은 pipe/SDK Joined·dispatch Completed·3 owners/3 physically_finished·first_error null·complete true다. 정상 미디어 owner의 원 PROCESS receipt를 보존했다. 경계 pause overshoot 관측0us는 이 실행의 결과이며 프레임 단위 정지 보장은 아니다. GUI를 실행하지 않았다.

상위 RETURN의 `verified_timeline=false`는 기존 producer/publication만의 비권위 기록을 유지한 필드이며, 새 `live_sdk_cli.configured_source.verified`에 저장된 Verified revision2 성공과 구별한다. `playback_ready=false`와 quality_approved=false를 유지하며 실제9 표본 또는 사용자 AI 품질 승인으로 계상하지 않는다.

후속 통합 검증은 아래 별도 로그로 같은 입력에서 완료했다. 이전 입력의 결과로 승계하지 않았다. 제품 기본 startup·재시작 후 fresh authority 재입장·crash/IO 전수·전체CLI/GUI/AI 인수는 남는다. WIP/소유권·첫 실패·원본 증거와 수동GUI/추론/다운로드/WARP/push 보류를 유지한다.

| 후속 로그 | 결과 |
|---|---|
|529/530|Product no-run·고정145/145|
|531/532/533|Core no-run·532 잘못된 group명으로 시험 전 거절·533 정확한 meari-core18/18|
|534/535/536|Native no-run·meari-native26/26·자동meari-os1/1|
|537|정상 player/player-cli build exit0, 두 artifact profile.test=false/fresh=true|
|538|native-proposal-window clippy --all-targets -- -D warnings exit0|
|539|기존 저장/replay/CAS·combined·readonly 실제CLI3/3,24.21초|
|540|cargo fmt --check exit0|
|541|verify-current2221파일 content/identity 동일 exit0, stage 없음|

네 exact group-return.txt는 같은 SOURCE49C38F20…와 execution_result/producer_integrity=Ok(())를 보유한다. core 재시도는 별도 `source375-live-sdk-core-exact-r2`, 나머지는 `source375-live-sdk-{product,native,os}-exact-r1`이다. 고정190 계획 SHA082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464와 원 등록/EXE/PDB를 대조했다. 532 실패 로그를 삭제하지 않았다.

537 정상 player SHA `79C6AEB379160C37A538440860E3AC950BAEFB64F0867A9C9318A83D93AFA07F`, CLI `ECD3BFCE05B127A4E2EE2A116F73CD15783361142529A3A8C26893293958BB37`. root HEADd13cbfca140e41863b4d0bd27930be3f57a1123f/Meari0844f6c24caf224617ca8abbb2dd75695a971b99·두 lock·Cargo/rustc/custody SHA는 이전 핀과 일치한다. 전역 toolchain은 변경하지 않았다. C 문맥별 cache/tmp·locked/offline/jobs1·CPU2/BelowNormal·무거운 producer1을 유지했다.

기존에 읽고 적용한 modoo.today `software-qa/code-change-review-verification`(주장/근거 분리), `software-qa/test-fixture-and-mock-build`(실제/fixture 범위), `ai-quality-plus/agent-run-postcondition-and-replay-check`(원 실행/후조건/replay, body SHA61843099B7D54C3D7AC226218A5EE1B6CCDBE36EF84268762DBC33A68FBB58B5)을 적용했다. exact 시각/PID/nonce를 고정값으로 만들지 않고 각 실행 receipt를 소유한다. Codex runtime 인증을 주장하지 않는다.
