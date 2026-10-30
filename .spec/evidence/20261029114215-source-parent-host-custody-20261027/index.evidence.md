---
id: evidence:kinekino/20261029114215
title: Private 부모 source host custody — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Private 부모 source host custody — 2026-10-27

후속 R31~R33의 launch-free/one-use 준비 객체·post-check 오류/unwind custody와 actual OS4는 [일회 launch 전용 기록](source-parent-launch-once-20261027.md)에 분리했다. root SDK/D issuer 연결은 아직 미완료이며 이 문서의 원 R30 host 인수를 새 입력 실행으로 바꾸지 않는다.

## 선택 후계 R30 — 미할당 child의 독립 close 범위

R29에 parent-host-test-support feature와 original launch gate3를 추가했다. 983 strict/984 no-run 통과,985 no-child·987 AssignedSuspended는 각각 actual1pass였다. **986 UnassignedSuspended는8초 원 예산 내 ready가 되지 않아 actual0pass/1fail**이며 원 carrier를 in-process 보존했다. B WindowsFamily::poll_family는 UnassignedSuspended에서 의도적으로 Unknown을 반환하므로 기존 generic Supervisor의 family=Empty close 조건에 넣으면 안 된다. 원 B 정책 결함으로 바꾸거나 이 실패를 성공으로 치환하지 않는다. 원 R29 input SOURCE.md `AB2F9E09584E6A797ACEF54C9FC616CFF3F0C26A6EB029972DFFE40C5F0952B2`/inventoryD2832883와 fixed-r29 launch test8A7F5FE3/1435648B·원985~987 로그/inner copy/tmp를 유지했다.

R30 parent host는 **B를 변경하지 않고** 미할당 owner를 원 LaunchFailure.owner 안에 그대로 보유한다. 원 child handle exit와 원 미할당 Job의 empty를 각각 poll한 뒤 별도 UnassignedCloseReceipt(identity/child_exit/original_job/launch_state)를 기록한다. child가 그 Job의 멤버였다고 주장하는 ProcessReceipt family=Empty는 만들지 않는다. ProcessClosure::UnassignedClosed와 ParentReceiptSink::record_unassigned를 추가했고 NoChild/Assigned 범위와 구별한다. 원 launch rollback이 이미 한 termination을 자동 재시도하지 않는다. 생성 전 실패는 NotCreated/NotApplicable/sink0, 할당 후 owner는 기존 Supervisor/final receipt 경로를 사용한다. from_launch_failure의 Supervisor 전환 설명은 assigned owner에 해당하며 unassigned는 원 carrier에 남는다.

Cargo/host/close/tests preimage는 `runtime/verification/source-parent-launch-preimage-r1`에 있다. 선택 input-r30 SOURCE.md SHA256 `C8F6FD8C59AF64B5329DC848D223BAE304C4FDCE03C3FDBAAA54306D16927C3E`,inventory `2DD43E146A595579A90A63296A57DCDC9B9CC5FAE86F872D788965A2B127CAA3`,70files,lock BC3E5C43 유지.988 strict·989 exact producer no-run·990~993 actual OS4·994 순수/메모리47(OS9ignored)·995 fmt·996 content/identity 동일70·997 원 B32 동일 통과했다. 새 B source/원 clone은 편집하지 않았다. test-support 기능은 명시 private 검증용이며 production factory 등록이 아니다.

선택 fixed-r30 launch test SHA `D88C26C4CB4C5CFCB00031739E4CF36B848CF4BB375E4EB7E0C1A97FEA40CA0E`/1442816B,host test SHA `983D0FD3BE3BC5218C9FB4277011A4ED0A93403F4A29285DF5A8C425B897630F`/1444864B.989의 Cargo JSON이 가리키는 새 parent_host_os-8d51c4cbba3a62b0.exe를 선택했으며 남은 구 e1c5a97b alias를 선택하지 않았다. worker는 fixed-r22/2BDCA67F/6648832B이다. operator index `source-parent-scope-r30-index.json` SHA `AA6EEE156C4EA042D0D72A823365085ED0E9906DEC80D7EA29530B91F11633EA`는 proof=false다. 각 exact는 독립 baseline driver7E7AD258/45s+cleanup5s에서 실행했다.

|로그/root|actual 결과와 범위|
|---|---|
|990 source-parent-unassigned-os1-r30|1pass,원 PID37484/exit124·원 미할당 JobEmpty·state UnassignedSuspended를 별도 scoped receipt1회,ProcessReceipt 없음|
|991 source-parent-no-child-os1-r30|1pass,stage prepare_or_create/NotFound 보존,identity없음/NotCreated/NotApplicable/sink0|
|992 source-parent-assigned-os1-r30|1pass,원 PID27736/exit124·AssignedSuspended·원 familyEmpty/process receipt1회|
|993 source-parent-partial-receipt-os1-r30|1pass,과거 partial 저장 뒤 final sink 실패에서 final=false/Unrecorded/ready=false/calls1/원 오류 보존|

990~992는 primary thread를 resume하지 않았다. 생성 후 failure의 exit124는 launch rollback이 원 process/Job termination을 이미 요청한 결과다. 그 뒤 Supervisor forced=false가 보여도 전체 경로에서 종료 요청이 없었다는 뜻이 아니다. worker HTTP/GUI/모델 실행은 없다. 모든 gate는 원 IPC local reap/close·raw EOF를 따로 확인했고 parent control/data EOF=false는 유지했다. scoped receipt는 실제 원 handle 관측을 별도 사실로 기록한 것이며 B의 Unknown을 Empty로 덮어쓰지 않는다. 각 root outer/inner-observed/tmp 원본을 보존했다.

신규 scoped receipt의 sink 실패/panic/late-write·termination 실패/지연·예산 소진 및 완전한 SDK ACK/SourceOpener/D replay/정상 HTTP 인수는 계속 남는다. 실제 OS4를 전체CLI·제품190/bins·AI/GUI 완료로 계상하지 않는다. root 미채택·소유권/WIP·원 실패·추론/다운로드/수동GUI/WARP/push 보류 유지.

## 선택 후계 R28 — 부분 receipt와 최종 receipt 구별 수정

**R26/R27 private host는 과거 부분 진단 저장이 있는 조합에서 false ACK readiness 결함이 있었으며 후계 선택은 R28이다.** R26의 두 OS 성공은 그 원 제한 시나리오로 보존하고 이 추가 조합까지 통과한 것으로 계상하지 않는다. 제품 root에는 이 host를 채택하지 않았다.

원 Supervisor의 receipt_recorded()는 한 번이라도 sink가 성공한 사실이고 final receipt의 보장이 아니다. 새 OS 시험에서 원 child가 살아 있을 때 별도 `prior-partial-receipt.txt`에 child_exit=None/family=Unknown snapshot을 실제 기록했다. 이후 worker 종료/IPC/raw close 뒤 최종 sink의 부분 쓰기+BrokenPipe를 주입했다. R27 input SOURCE.md `3B7EC4FFC2055DA566648C9E03EEE97093917F02255EACD3F70C5C25525EF7FA`/inventory874FF459에서972 strict·973 no-run은 통과했으나974 actual exact1은 **0pass/1fail**이었다. 원 facts는 any_receipt=true/final_receipt=false/first Receipt인데 receipt Recorded/ready=true였다. in-process 원 receipt-incomplete carrier를 보존한 채 시험이 실패했고 outer test3492는 exit101/Empty/nonforced/rawEOF로 정리됐다. 원 worker의 과거 부분 snapshot PID25648을 최종 종료 receipt로 바꾸지 않는다.

판정을 원 cleanup_progress.final_receipt_recorded()로 바꿨다. preimage close/tests는 `runtime/verification/source-parent-final-receipt-preimage-r1`에 있고 원 red test `fixed-r27/parent_host_os.exe` SHA `16DCD30982FC7CF5AF35D62D4C52B22AB2D46B602C730CF9A3F2B94D68AAC03E`/1439232B를 유지한다. red index47C166D5/374B는 proof=false다. code/원 오류/sink재시도 금지 이외의 B 정책은 바꾸지 않았다.

선택 input-r28 SOURCE.md SHA256 `2E2AB1AB87CC064B8DC5FA2F40CB06E849E7A20A0C9EFBDCFA50668070BE0308`, inventory `30A46825770E7C92B5AEA03909E1E4FCD2080083CF8F56FBC6F63DB5F12E1CB7`,68files,lock BC3E5C43 유지. parent test 고정 `fixed-r28/parent_host_os.exe` SHA `69445F30A7F77EB20BA6A225360CF5DDA02BBBB19E2C5021E4121657580F58B8`/1439232B. 소비 worker는 R22 pin2BDCA67F/6648832B 그대로다. operator index `source-parent-host-r28-index.json` SHA `6A430FEF88255E812A908D438B497A549023029169B066E773FE48832DDFFDE2`는 proof=false다.

975 strict·976 no-run·977~979 actual OS3·980 순수/메모리47(OS6ignored)·981 fmt·982 content/identity 동일68 통과. exact OS는 같은 독립 baseline driver7E7AD258/45s+cleanup5s로 수행했다.

|로그/root|actual exact 결과와 facts|
|---|---|
|977 source-parent-partial-receipt-fixed-os1-r28|1pass,any=true/final=false,Receipt Unrecorded/ready=false/sink calls1/원 Receipt 오류 보존|
|978 source-parent-host-normal-os1-r28|1pass,any=true/final=true,Receipt Recorded/ready=true/sink calls1|
|979 source-parent-host-sink-failure-os1-r28|1pass,any=false/final=false,Receipt Unrecorded/ready=false/sink calls1|

red974와 green977~979 모두 outer 원 outcome/raw/receipt·inner-observed copy를 보존하고 원 tmp도 유지했다. green은 원 worker exit23/Empty/nonforced를 in-process 확인하며 물리/IPC/raw closure와 receipt 상태를 별개로 판정한다. receipt 실패는 실제 디스크 고장이 아닌 부분 쓰기 후 Rust sink 주입이다. 전송할 bootstrap/Open 없이 HTTP 이전 종료를 사용했으며 정상 다운로드 성공은 아니다. actual partial-launch/no-child·sink panic·close 예산 소진·SourceOpener/D replay/SDK 및 AI/GUI는 여전히 미완료다. 기존 WIP/소유권/원 실패·보류 유지.

## 범위와 선택 입력

`runtime/development/source-http-bootstrap-r1/packages/media-source-host/src/parent_host{.rs,/close.rs}`를 작성했다. root 제품/SDK/factory를 변경하지 않았다. 기존 원 SourceFamily/SourceLaunchFailure만 인수하며 다른 launcher나 PID reopen을 만들지 않는다. worker/input/ancestor guard·원 replay/current·attempt의 발급과 보유는 후속 원 제품 adapter 책임이다. [worker/protocol 원 입력과 증거](source-http-bootstrap-private-20261027.md)를 승계 성공으로 바꾸지 않는다.

R25 input67에서963 strict 통과. 선택 R26 SOURCE.md SHA256 `442D816549AEB47F2BB9870A592F5B136875E92AE2A4F869458290C62F78E0D9`, inventory `D15B20B431D05DC45047D7493C65DA043F589E548A6BF0615D6B147856D8D48D`,68files. lock `BC3E5C43CFD1B4198C3782957FF82B5132B92F4A627A53CD6F0A8768579A9C1E` 유지. Cargo/lib preimage는 `runtime/verification/source-parent-host-preimage-r1`이다. C-only native/source-http-bootstrap-r1 및 tmp, stable MSVC1.99.1/locked/offline/j1/heavy1을 유지했다.

## 소유와 close 계약

from_running은 원 Supervisor와 IPC를 함께 보유한다. from_launch_failure는 원 carrier의 WindowsFamily를 정확1회 옮겨 Supervisor로 감싸고 launch 오류/termination 오류/evidence를 계속 보유한다. 새 process owner를 PID로 만들지 않는다. 후자는 실제 partial-launch gate를 아직 실행하지 않았으므로 타입 검사만 입증한다.

poll_close는 원 Deadline을 축소만 하며 원 process cleanup과 IPC cancel/reap을 따로 poll한다. physical close 후 원 identity-checked raw record cursor로 채널당 최대4096byte record 하나씩 읽고 EOF를 확인한다. EOF/receipt/Job closure를 counter로 추정하지 않는다. receipt sink는 모든 물리·raw 사실을 확인한 뒤 딱1회 시도하며 호출 전 attempted를 기록해 실패·unwind 뒤 재시도하지 않는다. 원 error는 Host 또는 원 Supervisor/LaunchFailure에 보존한다. Host secondary 최대8과 overflow, 원 Supervisor의 상세 cleanup 오류도 별도로 남는다.

HostCloseFacts는 NotCreated/Pending/Closed 및 NotApplicable/Unrecorded/Recorded를 구별한다. ready_to_ack는 IPC reap/local close+원 process closure+raw EOF+해당 receipt 상태의 조합이다. caller가 transport facts를 저장하고 원 오류를 확인해야 하며 그 값만으로 SDK ACK/다운로드 성공/출판을 발급하지 않는다. no-child launch는 receipt를 만들어내지 않는다. raw/receipt 오류 후 원 carrier는 계속 남는다. blocking finish/Drop loop를 도입하지 않았다.

## 실제 OS2

parent test 고정 copy `runtime/build-cache/native/source-http-bootstrap-r1/fixed-r26/parent_host_os.exe`, SHA256 `C32363D8260C2E22E3CBBEED7F6F707A1C95C955B2FA7F780BFF4069D4257148`,1436672B. worker는 이미 고정한 **R22** `fixed-r22/media-source-worker.exe`, SHA `2BDCA67FB13578D0328C69BC6A6C901100A467C0854F645864D2290BCBCAFCC9`,6648832B를 사용했다. R26 normal worker 실행으로 계상하지 않는다.

독립 baseline driver7E7AD258/924672B의 exact1/ignored/45s+cleanup5s로 각각 실행했다. operator index `source-parent-host-r26-index.json` SHA `CA3916EFBF48B71A8401DCFC92D02AA137EAF5F2F55AFA05B94502D75217AC1D`,571B,proof=false. 원 Cargo/source와 receipt를 대신하지 않는다.

|로그/root|결과|확인한 사실|
|---|---|---|
|966 source-parent-host-normal-os1-r26|exact1pass,outerPID28348/exit0/Empty/nonforced/raw EOF|worker9576/exit23/Empty/nonforced,IPC local reap/close+raw EOF+receipt Recorded 뒤 ready=true|
|967 source-parent-host-sink-failure-os1-r26|exact1pass,outerPID33172/exit0/Empty/nonforced/raw EOF|원 worker exit23/Empty/nonforced를 in-process 확인,물리/raw 완료이나 receipt Unrecorded/first Receipt/ready=false,sink calls1 유지|

두 경우 모두 readiness 관측 뒤 bootstrap/Open 없이 원 parent endpoint를 닫았다. HTTP preparation에 도달하지 않는다. worker 작업은 예상 실패23이며 host 정리 완료와 작업 성공을 구별한다. parent control/data EOF=false는 그대로 보존한다. receipt 실패는 실제 worker/IPC와 **Rust sink의 부분 쓰기 후 BrokenPipe 주입** 조합이며 실제 디스크 고장을 관측한 것은 아니다. 실패 carrier는 같은 host를 in-process 보존했고 성공 receipt를 만들어내지 않았다. fault worker PID를 별도 성공 receipt로 기록했다고 주장하지 않는다.

각 root에 outer outcome/receipt/raw와 inner-observed 원 facts/raw/receipt 복사본을 두고 원 tmp도 유지했다. 정상 facts 파일은 ready=true/receipt calls1/first 없음, fault facts는 process Closed/rawEOF true이지만 ready=false/receipt calls1/first Receipt다. fault receipt 파일은 partial 데이터뿐이며 정상 receipt가 아니다. 파일·Job·IPC 재획득 전수/누수 없음은 입증하지 않았다.

## 게이트와 남은 범위

964 strict,965 no-run producer,966/967 실제 OS2,968 순수/메모리47(OS5ignored),970 fmt,971 전후68 content/identity 동일을 통과했다.969 parent-host normal tree는 owned-process/Windows/url을 포함하며 reqwest/tokio는 없다. fixed parent/worker 해시는 실행 후 동일하다. 이번 새 첫 실패는 없다.

partial-launch/no-child·raw 오류·sink panic·close 예산 소진/Incomplete·강제 종료/receipt 실패의 추가 실제 행렬은 남는다. 원 SourceOpener의 launch-free/one-use open과 브라우저 D replay/current/attempt·SDK registry 연결, 실제 정상 HTTP/Data/BodyEnd·AI/GUI도 남는다. 기존 root DFB0 190/정상bins·원 B/worker 성공을 이 새 범위로 승계하지 않는다. source375 private overlay,원 소유권/WIP/실패/추론·다운로드·수동GUI·WARP·push 보류 유지.

기존에 읽은 modoo.today fixture/mock 본문 A393F15F…F100 및 postcondition/replay 본문61843099…8B5의 원 owner·주입 경계·postcondition 분리를 적용했다. 실제 OS closure와 sink 주입·제품 성공을 구별했으며 타 언어 검증 코드를 도입하지 않았다.
