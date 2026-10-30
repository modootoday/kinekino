---
id: evidence:kinekino/20261029114658
title: 원 세션 인수 후 준비 callback·source custody — R32/R33/R34
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 세션 인수 후 준비 callback·source custody — R32/R33/R34

## 구현과 소유권

DownloadOwnedExecutor::prepare_owned는 원 DownloadCoreSession을 생성만 하고 반환한다. host preparation은 core가 이미 같은 세션을 보유하는 run 안에서 실행한다. callback 전후 원 ScopeControl checkpoint와 input/identity/deadline 검사를 유지한다.

application/download/preparation.rs에 기존 SourceOpener/PreparedDownload/DownloadPreparation 계약과 새 PreparedDownloadResources·SourceCustody를 책임별로 분리했다. SourceCustody는 원 세션의 source 슬롯에 대한 non-Clone/non-Serialize borrowed context이며 새 public pending owner Box/registry가 아니다. 빈 슬롯의 단일 borrowed context를 만들고 adopt가 context를 소비해 source를 infallible하게 원 슬롯에 넣는다. 이후 callback 오류/unwind에서도 local source가 아니라 원 core session이 동일 opener/owner를 보유한다.

구체 process fixture는 prepare_in_session을 구현하고 source를 먼저 adopt한 뒤 fallible output 준비를 한다. receipt 파일 열기는 문서 owner 인수 전으로 옮겼다. legacy prepare는 launch-free/file-only 호환이며 live owner를 받을 경로로 사용하지 않는다. 두 preparation 경로 모두 새 process/HTTP launch를 허용하지 않는다. launch는 인수된 opener.open의 기존 gate에 남는다.

실패 중에는 prepared resources/source session이 없어도 원 adopted opener를 계속 poll_close/poll_release한다. 실제 opener의 원 owner_observation을 사용한다. source 슬롯이 없는 경로는 launch-free preparation에서 인수·launch 자체가 없었던 경우다; worker JSON/누락PID를NoChild로 바꾸지 않는다. 원 오류·panic과 원 document owner/준비된IPC를 동일 세션에서 보유해 정리한 후 원 terminal 정책을 적용한다.

## 변경·입력 보존

신규 preparation.rs,기존 download/{mod.rs,owned_session.rs,job_executor.rs},process_source/tests/live.rs·live/jobs.rs와 독립 mount tests/support/download_owned_module.rs·tests/media_download_owned.rs를 변경했다. 원 preimage는 후보 runtime/verification/preimage-r31-preparation-custody에 보존했다. receipt 준비 순서 변경 전은preimage-r32-receipt-order,기대 상태 수정 전은preimage-r33-expected-state에 별도 보존했다. root/donor/WIP/소유권과 source375 private overlay 유지.

|입력|핀·범위|
|---|---|
|R32|inventory8532AB0B1AF9CA57EE3F46F82A01A97896E343272DA587B6BA73AEF235A0BF39/2295;1197 중간no-run,선택 runtime 실행 아님|
|R33|inventory681DA886932F0381336E7173C6416AB11FA9639A2D02A9D5180AD249E40FFECC/2295;1199 첫 runtime red 보존|
|R34|SOURCE.md3F526868FD948186B7CF291F029B2C2B2C48C6DE6F6526BCBFA6C92FB221605A;inventoryBBAED9C83681C9E856B3580A824A0DD64523841F5120D7F765F592137944BED6/2295;기대 상태만 수정한 최종입력|

root lockB2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D/mirror lock955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5/AGENTS40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C 유지.

## 실제 실패·검증

1195 strict 첫 실패는 기존 독립 owned download mount가 이동 전 owned_session 내부 계약을 직접 import한 것이다. 동일 preparation 모듈을 mount하고 공개 계약 경로를 연결해1196 strict 통과했다. 시험 제거·lint 완화는 없다.

1199 R33 첫 actual 실패는 expected state 오류다. callback의 원 NeedsInput을 core가NeedsInput으로 보존했는데 fixture가Failed를 기대했다. outer test exit101/selected_one=false와 원 terminal-job.json/진단을 보존했다. R34에서 기대를 error→NeedsInput,panic→Failed로 맞췄으며 제품 정책은 변경하지 않았다.

1200 최종 lib no-run terminal exit0 뒤 fixed-r34/sdk-source-tests.exe를 copy/핀했다:SHA19FCA0961C6B9330F63FACD56B7D821501A804BEC5D45219AA9387EB7ACBDD26/109484544bytes. 원 R33 red 실행파일7B8445737750DF7F5514AB93C443CEF2B385C7A08E08FE135743EED3F87980BB/109487104bytes도fixed-r33에 유지했다.

|gate|실제 결과|경계|
|---|---|---|
|1201 setup error|exact1pass/951filtered;NeedsInput·grant consume0·source child0·publication0·출력없음·transport1·원guard drop1|callback after adoption에 주입한 오류|
|1202 setup panic|exact1pass/951filtered;Failed/Internal·source child0/consume0/publication0·원guard drop1|원panic·retained source cleanup;실제 browser아님|
|1203 정상 CLI|exact1pass/951filtered,5RPC;같은SDK 원source 검증/발행/Completed·정리|오프라인body/probe와 합성browser|
|1204 취소 CLI|exact1pass/951filtered,7RPC;probe pending 취소·partial보유/제거·Cancelled/미발행·정리|HTTPread중취소·일반Ctrl-C 아님|
|1205/1206 strict|support/기본 all-targets -D warnings 실제receipt terminal exit0|독립 legacy mount 컴파일 포함|
|1207 fmt|--all --check 실제receipt terminal exit0|과거fmt실패 정정 유지|
|custody|verify-current session59888 terminal exit0,2295 content/identity unchanged|private 입력이며canonical 적용 아님|

actual4는 고정 독립 C2 driver로 직렬 실행했고 최종 각 outer는 selected_one/ownership complete/physical_closed·first_error null·exit0/Empty/nonforced/raw EOF다. setup2의 worker artifact는guard-only로 실행하지 않았다. 원 준비된IPC의 receipt는 host=None,prepared_ipc Some(io_reaped/local_endpoints_closed=true,had_error=false),raw_eof=[true,true],launch_unreturned=false이며 fake source process exit0을 만들지 않는다. peer EOF=false와 local close를 구별했다.

error inner 진단 job-sources/02924a07f7c3dceeb4fd6170d391d092970a988490b56b1eefd3310410bdf5ee.utf8의 첫 code/message와 reported_error_request_id=original-setup-after-adoption 보존을 확인했다. panic은 원 Internal/Original owned session run panicked와 retained opaque payload 경로를 사용했다. 두 경우 출력은null/미발행이다.

## artifact·원 증거

operator index runtime/verification/source-sdk-integration-r34-index.json은proof=false/SHA6850A858FDF0FA469EB27F15A704F199B5E931B9D6762BA7D5F4FCB46D4D1876이다. 실제 재사용 worker R15/1101은477ED83B9286BA0F556DD61F97C230FFD6F2C4A8770FFD45B6D161EA63E051F7/1000960bytes,CLI R14/1092는249DD5ECEFFD0A476D3EC7B89B0F7003F9DB5DC13D285AE1126A4B9938DED32B/36405248bytes다. 새 정상bins 빌드로 계상하지 않는다.

outer는runtime/verification/source-sdk-custody-{setup-error,setup-panic,cli-normal,cli-cancel}-os1-r34/outer에 보존했다. 원 inner는tmp 동일tag 하위 sdk-source-jobs-26860-1793106308544298100 / sdk-source-jobs-33888-1793106374927368200 / sdk-source-cli-31432-1793106509875046700 / sdk-source-cli-21624-1793106640095572400이다. 정상 source23296/creation134375801100329093 및취소26708/creation134375802402366117 모두exit0/Empty/nonforced·원 transport Closed/Recorded/raw EOF다. browser/native 해제 관측은 합성이며 공유 WebView 프로세스 물리 종료 proof가 아니다.

C-only native/source-sdk-integration-r1/tmp·conservative2·heavy1·동일handle terminal 뒤 직렬·locked/offline/-j1 유지. 설치/실제브라우저/HTTP·추론·다운로드·수동GUI·WARP·push0이다.

## 남은 범위

### 후속 소스 대조 — 부분 IPC 생성 경계

현재 후보의 packages/owned-process/src/infrastructure/windows/source_pipe.rs에서 pair는 server/client handle을 만든 뒤 Operation::connect를 시작한다. PreparedSourceIpc::new는 control pair 성공 후 data pair를 생성한다. 따라서 data pair 오류 시 이미 시작한 control connect는 반환된 PreparedSourceIpc를 통한 poll_close 소유권을 얻지 못한다. operation.rs의 Operation::drop은 cancel/poll 한 번 뒤 pending이면 원 allocation/event/pipe를 forget하고 emergency counter를 올린다. 이는 명시된 메모리 안전 비상 보존이며 정상 reclamation/cleanup ACK가 아니다. 이 경로가 실제 OS에서 발생했다는 관측은 아직 없고, 코드상 실패 경계를 확인한 것이다.

또한 live/jobs.rs Preparation은 IPC 생성·연결 대기를 원 SourceCustody.adopt 이전에 수행한다. 이후 원 document owner 획득 뒤에도 wire_binding 검사·fixture assertion·HttpBootstrap 생성 등이 adopt 전에 남는다. R34의 after-adoption 오류/panic 시험은 이 경계를 포함하지 않는다.

다음 변경의 완료 조건은 (1) 원 document owner를 fallible 초기화 전에 원 core 슬롯에 보유, (2) 부분 IPC handle/overlapped operation을 같은 보유 경로에서 명시적으로 정리, (3) 두 번째 lane 생성/연결 실패와 panic에서도 원 오류·deadline·정리 관측 보존이다. 연결 지연만 적용하고 미연결 상태를 connected로 처리하거나 비상 leak를 정상 종료로 계상하는 수정은 허용하지 않는다. 기존 donor/R36/root는 유지하고 private 후보의 preimage 고정 뒤 변경한다. 이번 대조는 실행 시험·새 producer·새 전체190/정상bins 성공이 아니다.

증명 범위는 adopt 이후 callback error/panic이다. adopt 이전 실제 source 획득·부분IPC 초기화/원owner 반환,IPC builder의 모든 부분 실패,receipt 실패·pending probe와 origin 정리의 독립 진행,legacy file/probe 전체 실제 회귀·추가 저장/취소 행렬은 남는다. 실제 provider policy·HTTP/probe·native browser/COM/host종료경쟁·제품source등록/전체CLI/GUI도미완료다. 새 borrowed context 존재만으로 이 행들을 완료로 표시하지 않는다.

R28core80/R31browser86/DFB0지정190·정상bins는각원입력에 유지하고 R34재실행으로 계상하지 않는다. AI사용자승인0/9·원 후보NeedsRepair·시각미인수·원WIP/보류 유지.

기독 modootoday/software-qa/test-fixture-and-mock-build 본문 SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 원setup/teardown·오류/panic·실제OS와합성권한 분리·원실패보존을 적용했다.
