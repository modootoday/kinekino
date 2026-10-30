---
id: evidence:kinekino/20261029114710
title: Private launch-free·one-use parent custody — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Private launch-free·one-use parent custody — 2026-10-27

후속 R34~R36은 [부모 wire/host I/O 조립](source-parent-transfer-io-20261027.md)에 기록했다. 원 launcher/guard를 재사용해 bootstrap control·pending data read를 연결했고 actual OS3은 Open을 전송하지 않았다. 이 R33의 일회 launch gate를 새 입력 실행으로 바꾸지 않는다.

## 범위

private `parent_launch{.rs,/close.rs}`를 작성했다. ParentLaunch는 원 SpawnSpec/PreparedEvidence/PreparedSourceIpc/Deadline과 caller guard G를 infallible하게 보유하며 생성에서 launch/current callback/network를 실행하지 않는다. first try에서 검사 전에 spent를 설정하고 원 budget을 축소만 한다. spawn_source가 돌려준 original SourceFamily/SourceLaunchFailure를 ParentHost에 넣은 **뒤** 후속 검사를 실행한다. 검사 실패/후속 unwind 뒤에도 같은 host와 guard가 남는다. launch call 자체가 unwind해 반환하지 않으면 Unreturned로 남고 NoChild ACK를 만들지 않는다.

준비 상태의 close는 child 없이 원 evidence writer를 내려놓고 원 prepared IPC reap/local close 및 identity-checked raw EOF를 관측한다. launch 뒤 close는 [원 parent host](source-parent-host-custody-20261027.md)에 위임한다. guard는 원 scope가 object를 해제할 때까지 유지한다. holding G는 executable/ancestor pin 검증이 아니며 caller의 기존 trusted adapter가 실제 guard/path를 검증해야 한다. callback도 원 replay 권한을 발급하지 않는다. D issuer·SDK SourceOpener/DownloadSession 구현·registry 연결은 계속 미완료다.

## 입력·첫 실패

preimage lib/host 및 strict 전 OS test는 `runtime/verification/source-parent-once-preimage-r1`에 있다. R31 input72에서998 strict 통과했다. R32 inventoryC4DB5266/73files에서999 test strict err_expect 첫 실패를 보존했고1000 no-run은 미실행이다. err().expect를 같은 동작 expect_err로 바꾼 선택 R33 SOURCE.md SHA256 `8D7B622A49EA9F06114CB028C6C4BC3B5E9ADE25135D2431ACFDFA9DEDC85017`,inventory `914CB10F493A6B819D599D90D2DEA26DB37F81A4B7A0D55535FDCA7795BDE5F6`,73files다. lock BC3E5C43 유지.1001 strict·1002 producer no-run·1009 순수/메모리47(OS13ignored)·1010 fmt·1011 content/identity 동일73 통과했다.

고정 test `fixed-r33/parent_once_os.exe` SHA256 `A0E555758DB18276E261D7E69FEAA3FD18B9226C9DAB524EDB721ADDC05661E9`/1470976B. child는 R22 pin2BDCA67F/6648832B다. `source-parent-once-r33-index.json` SHA `FC037A15F73D613447591D3E5F1C6712F92D7EF3298346478D4EABB8CDA5E5DD`는 proof=false operator index다. 기존 독립 baseline driver7E7AD258/45s+cleanup5s를 사용했으며 각 원 outcome/receipt/raw와 inner-observed copy/tmp를 유지했다.

## 실제 OS4와 직렬 재검증

|선택 로그/root|actual exact 결과·관측|
|---|---|
|1003 source-parent-once-reject-os1-r33|1pass,pre-check1/Current/host없음/원 prepared IPC close·rawEOF/receipt0/재시도Spent|
|1007 source-parent-once-missing-serial-os1-r33|1pass,pre-check1/Launch/원 NoChild carrier 보존/NotCreated·NotApplicable/receipt0/재시도Spent|
|1008 source-parent-once-post-error-serial-os1-r33|1pass,check2/Current/같은 host closure·receipt1/재시도Spent|
|1006 source-parent-once-post-panic-os1-r33|1pass,check2/원 callback panic을 상위 retaining scope에 보존/같은 host closure·receipt1/재시도Spent|

post 두 경우 readiness 후 bootstrap/Open 없이 close했으며 원 worker exit23/Empty/nonforced를 확인했다. 이는 HTTP 이전 예상 실패이고 다운로드 성공이 아니다. 모든 경우 guard drop0이 object close 완료까지 유지되고 owner drop 후1을 in-process 확인했다. panic은 callback fixture의 원 Box payload이며 native worker panic/실제 SDK Mutex poison 검증이 아니다. ParentLaunch.first enum은 panic 경우 None일 수 있으며 상위 core가 원 panic을 별도로 보존해야 한다. readiness/ready_to_ack를 그 실패의 성공 판정으로 사용하지 않는다.

1004 missing의 exec handle이 live를 반환했는데 PM이 완료 확인 전에1005 post-error를 시작해 두 resource-runner의 실행 일부가 겹쳤다. 각 기능 시험은 통과했지만 중앙 직렬 배정 규칙 위반을 보존한다. 기존1004/1005 원 로그·copy/tmp는 삭제하지 않았다. handle99512 terminal을 확인한 뒤 같은 fixed producer로1007→handle62432 terminal 확인→1008을 실행했고 선택 직렬 근거를 이 두 실행으로 교체했다. 재빌드/입력 변경은 없었다.1007 receipt 마지막 쓰기07:48:21 UTC,1008 stdout 생성07:48:30 UTC도 순서를 확인한다. overlapped 실행을 CPU/메모리 성능 실측으로 계상하지 않는다.

## 남은 범위

launch call 내부 unwind의 실제 original carrier, 준비 중 현재성/취소·연결 지연, guard/ancestor 검증 실제 제품 adapter, worker bootstrap/Open/credit/Data/BodyEnd와 parent I/O 통합, D one-use replay/current/attempt, SDK/CLI와 GUI/AI 품질 인수가 남는다. root/factory 등록 없음, 기존 source375 private overlay·제품 DFB0 190/정상bins·소유권/WIP/원 실패·추론/다운로드/수동GUI/WARP/push 보류를 유지한다. 현재 OS4/순수47을 전체CLI 인수로 합산하지 않는다.

기존 modoo.today fixture/mock·postcondition/replay 지침을 재사용해 actual OS, callback/panic fixture, 원 process facts와 작업 성공을 구별했다. 타 언어 정책/검증 코드는 추가하지 않았다.
