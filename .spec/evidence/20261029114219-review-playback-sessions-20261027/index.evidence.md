---
id: evidence:kinekino/20261029114219
title: Review 세션 유스케이스·고정 adapter 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Review 세션 유스케이스·고정 adapter 검증

2026-10-27 중앙 PM. [공통 세션 계약](../../player/application/review-playback-sessions.md)을 구현한다. 기존 modoo.today code-change-review-verification/test-fixture-and-mock-build의 합성/실제·주장/근거·실패 보존 기준을 적용한다. 현재 진행 기록이며 transport 명령·GUI 인수 완료가 아니다.

## 공통 정책

315/318/321 세션 fixture5시험 각각 통과. 최신321은 실제 case lease 보유·동시 same-key open의 owner1개·capacity/취소/권한·물리 종료 전 재입장 거절·닫기 재시도·post-start 관측 실패의 원 owner 보존/동일키 복구·foreign host/selection/owner 거절을 확인했다. fixture는 플랫폼·실제 proof 근거가 아니다. 316/317/320/322 strict는 각각 당시 변경에 통과했다. 323 actual target no-run과 fmt check 통과 후 아래 r1입력을 고정했다.

## 첫 실제 실패 보존

r1입력 runtime/verification/source375-review-session-input-r1/SOURCE.md SHA256=6E82CD4849E1FD6537347AFD65BA4CB8BB3EF5BE0CC8FC0660F8EE8E4D496487, inventory4617DF4C04605707229F30C9C577F5EB50D02558F4976A231ABC1924C759191B/2180파일.

324 actual 시험은 outer101/9537ms, 시험0pass/1fail/8.94초다. 실제 Verified SDK 저장·조회 후 LocalReviewSessionLauncher::new에서 PermissionDenied로 실패했다. immutable_storage_paths::root는 Prefix::Disk만 받는데 constructor가 내부 비교용 canonical VerbatimDisk 경로를 config에 덮어쓴 뒤 root 입장 함수에 다시 넘겼다. 새 runtime worker/child를 만들기 전 constructor 오류이며 실제 runtime 디렉터리는 비어 있었다. 실제 OS 권한 부족이나 의존성 누락으로 해석하지 않는다.

원 출력 runtime/verification/review-session-actual-datalab-r1의 CREATE/VERIFY/REPLAY/INSPECT/UNCONFIGURED/WITNESS/RUNTIME-WITNESS와 빈 runtime/user-review를 보존한다. RETURN.json은 없으며 전체 성공으로 계상하지 않는다. 원 WITNESS의 new_decode_execution=true는 실행 전 의도 표기였고 실제 실행 근거가 아니다. r2부터 requested_decode_execution과 실행 전 new_decode_execution=false를 분리한다. 후속 성공 RETURN에서만 실제 실행을 기록한다.

실패 소스·시험 parent/helper와 원 test EXE를 runtime/verification/review-session-first-actual-failure-r1에 보존했다. 사본 EXE SHA256=027CBF0CA3DA9E768FE5C16BDE2FD96338CEE70AD41EEED285D61C539ACA588C는 원 WITNESS producer SHA와 일치한다. 사본의 FileID는 원 FileID가 아니다. 325는 전후2180파일 content/identity 동일로 종료0을 확인한 뒤 소스를 수정했다.

## 경로 수리와 후속

canonical 경로는 disjoint 비교에만 사용하고, root 입장은 guard로 보유한 원 ordinary Disk 경로를 다시 사용한다. 공통 root 보안 경계를 완화하지 않는다. 준비 오류의 원 ErrorCode는 worker first_error_code와 SessionFailure로 보존하며 일반 기존 runtime 문자열 오류는 EngineFailed로 남긴다. 이 후계의 strict/no-run·새 r2입력·actual 결과는 실제 종료 확인 후 추가한다.

## r2 실제 성공

326 수정후 all-targets/native-proposal-window strict·fmt check 및327 actual no-run 통과. 입력 runtime/verification/source375-review-session-input-r2/SOURCE.md SHA256=4312298343004ED550CC897C18AC50977992894704A3391A2751515F48B2CA6A, inventory6EC12AE9E26F475875148DD85E6C870FC0FDE6F331B2E82915843364E9FC0949/2180파일을 새로 고정했다.

328 정확 actual1 통과/outer0/17933ms(시험17.19초). 실제 봉인 Sprite 근거로 SDK Verified revision2를 저장한 뒤 새 typed ReviewPlaybackSessions와 실제 고정 LocalReviewSessionLauncher에서 open→connected→close→물리 종료를 확인했다. 같은 key는 취소 control이 있어도 같은 session을 반환했고, 다른 host scope는 거절했다. 종료 뒤 같은 key 조회도 원 terminal session을 반환했다. revision1 Unverified는 worker 준비에서 NeedsInput을 보존하고 child 미생성/physical_closed/owner_finished로 종료했다. 이 실패 관측의 family_empty=false를 Job Empty로 승격하지 않는다.

runtime/verification/review-session-actual-datalab-r2에 원 OPEN/CONNECTED/CLOSE/CLOSED/UNVERIFIED-NO-CHILD와 각 runtime INPUT/PROCESS·user envelope를 보존한다. 실제 PID6700/creation134375139830460190, exit0/비강제/원 Job Empty/IPC 종료/first_error 없음. PROCESS.json은 run 반환 직전의 owner_thread_ended=false이고 CLOSED.json은 그 뒤 실제 worker.is_finished=true를 관측한 별도 근거다.

- RETURN.json SHA256=7BF9B04CF67FA7231AD8AC3FE19435B8CF952F064F6D611023592474520ADEA0.
- WITNESS.json SHA256=8E6BC11731A2217152E81D744CA7A642C24FEEF442ADE4CCAFB3EC24B6F6CD3D.
- RUNTIME-WITNESS.json SHA256=09B1E369AD3F034986B585451D53F14DA4DF91A28F5028FAEC21480528F5CBA8.

329 전체 입력 후속 감사 종료0으로2180파일 content/identity 동일을 확인했다. 코드 편집은 freeze 이후 중단했다. source375 원 manifest/private overlay와 앞선6E82 실패·원 EXE/로그를 보존하며 성공 r2로 덮어쓰지 않는다.

이번 성공은 headless CPU local runtime의 typed Rust 유스케이스다. 실제 mpv decode가 있었지만 새 FFmpeg capture/model inference/다운로드/GUI는0이다. SDK transport 명령은 아직 등록하지 않았고 transport_commands_registered=false·playback_ready=false·actual9_binding=false·quality_approved=false를 유지한다. fixture host scope는 이 시험의 명시 scope이며 제품 host 재시작에 재사용할 값이 아니다.

기존 SOURCE6381DE6F의190/명시bins와 EBFFE의scoped worker 실제 시험을 이 새 구현의 전체 검증으로 승계하지 않는다. 공개 command 등록/schema/CLI·새 host scope 공급·command별 제어·재시작/취소 행렬·frame 실제 관측·AI/GUI는 미완료다. 모델/다운로드/수동GUI/WARP/push 보류·heavy1/CPU2/BelowNormal/C 문맥별 cache·기존 소유권/WIP를 유지한다.
