---
id: evidence:kinekino/20261029114555
title: 요청 준비 전 원 core source 인수 — SDK R37
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 요청 준비 전 원 core source 인수 — SDK R37

## 구현과 한계

ProcessSourceOpener::adopt_in_session은 기존 SourceCustody 슬롯에 동일 opener를 먼저 넣은 뒤 요청 생성 callback을 실행한다. 같은 기존 Arc/Mutex State 안에서 원 admission/document owner·sink·deadline·ParentLaunch를 보유한다. callback은 admission을 빌릴 수만 있고 원 owner/launch를 반환받지 않는다. 요청 생성/원 binding 검사 성공 뒤에만 원 pending_launch를 ParentTransfer로 옮긴다. 새 public pending owner carrier/registry/thread는 없다.

transfer가 아직 없으면 원 ParentLaunch로 명시 close와 준비 transport receipt를 수행한다. 새 record_prepared_transport 기본값은 Unsupported이며 미구현 sink를 cleanup 성공으로 바꾸지 않는다. callback panic으로 같은 mutex가 poisoned 되어도 정상 사용은 거절하고 cleanup/release는 원 state를 회수해 계속한다. request error는 원 ApiError를 보존한다.

fixture의 준비 단계는 IPC를 연결하지 않는다. open에서 원 admission/요청을 검사한 뒤 ParentTransfer→ParentLaunch::poll_prepared로 기존 lane 연결을 진행하고, 기존 one-use start/bootstrap/consume/Open 흐름으로 이어진다. 원 cutoff/currentness를 유지하며 준비 callback에서 child/HTTP를 시작하지 않는다. 기존 이미 준비된 ProcessSourceOpener::new 소비자도 유지한다.

이번 인수 경계는 이미 만들어진 unstarted ParentLaunch 및 원 admission을 인수한 시점이다. 원 provider session을 최초 획득하는 production factory, IPC/evidence를 만들기 전의 모든 호출자 소유권까지 해결한 것은 아니다. production factory는 아직 등록하지 않았다. fixture 권한/본문/probe를 실제 browser/HTTP 성공으로 계상하지 않는다.

## 보존·핀

preimage는 후보 runtime/verification/preimage-r36-early-source-custody의 process_source.rs,jobs.rs,live.rs,parent_launch.rs,parent_transfer.rs,cli.rs,cancel.rs,tests.rs다. root/donor/원 HTTP bootstrap R36·source375 원본은 변경하지 않았다. AGENTS SHA40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C 확인.

R37 SOURCE.md=A00BDA38F2D285593E12F0A5D339629F9847FFCAD2C83832353862F262C10C0D,inventory97A425FA3C624B898E6C4C29A286C1CEC0CB4A56457A9923683AD6CAC3660C0C/2296. fixed-r37/sdk-source-tests.exe=6244F416A4CFC7EA28532A179628D705484A5E94EB99F3BCF764BA9E5F9F5440/109516288 bytes,producer1222. compiler-artifact executable를 확인한 후 terminal/copy/핀했다. operator index r37 SHA38526B62DAD2411297089B56E66A9D5EEE83C8C462C8F030901D3BFDEF16F494,proof=false.

worker는 기존 R15/1101의477ED83B9286BA0F556DD61F97C230FFD6F2C4A8770FFD45B6D161EA63E051F7/1000960,정상 CLI는 R14/1092의249DD5ECEFFD0A476D3EC7B89B0F7003F9DB5DC13D285AE1126A4B9938DED32B/36405248을 재사용했다. 새 정상bins 빌드가 아니다. 요청 오류/panic 두 case에서 worker는 guard-only이며 실행하지 않았다.

## 실제 실행

최종 verify-current session10655 terminal exit0,2296파일 content/identity 동일 확인.

|gate|결과|
|---|---|
|1220 첫 support strict|exit101;기존 cancel fixture의 새 필드 누락·기존 시험의 Option transfer 접근 누락. 원 로그 보존 후 mount/initializer만 보완|
|1221 support strict/1222 no-run|각 실제receipt terminal exit0|
|1223 요청 생성 오류|exact1pass/953filtered;원 NeedsInput·consume0·child0·미발행·출력없음·준비IPC receipt1/guard drop1·retaining shutdown|
|1224 요청 생성 panic|exact1pass/953filtered;Failed/Internal·poisoned 원state cleanup/release·child0/consume0/미발행·준비IPC receipt1/guard drop1|
|1225 정상CLI/1226 취소CLI|각 exact1pass/953filtered;원 발행 Completed/원 probe 대기 취소 Cancelled;정상5RPC/취소7RPC|
|1227 source 모듈 기본 실행|8pass/18ignored/928filtered;ignored 원Windows 시험을 실행했다고 표시하지 않음|
|1228 parent lib|0tests;성공 receipt는 있지만 기능 회귀 근거로 사용하지 않음|
|1229 제품 default strict|실제receipt terminal exit0|
|1232 codec 직렬/1233 parent strict 직렬|12pass 및 parent-host-test-support all-targets -D warnings;각 실제receipt terminal exit0|
|1234 fmt|실제receipt terminal exit0|

배정 오류:1230 codec의 handle21185 terminal 확인 전1231 strict를 배정해 build-directory Cargo lock 대기가 발생했다. 둘 다 exit0이지만 heavy1 직렬 배정 준수로 표시하지 않는다. 원 stdout/stderr/receipt를 보존하고 같은handle terminal 확인 뒤1232 종료→1233 종료를 별도 직렬 실행했다. 전체 실행이 처음부터 직렬이었다는 주장은 하지 않는다.

actual4는 고정 C2 driver로 실행했다. outer는 source-sdk-request-{error,panic}-os1-r37 및 source-sdk-early-custody-cli-{normal,cancel}-os1-r37/outer다. 각각 selected_one=true/first_error=null/ownership complete/physical_closed=true/outer exit0·Empty·nonforced·raw EOF. 외부 driver의 physical_closed를 browser/COM 종료로 확대하지 않는다.

요청 error inner sdk-source-jobs-37156-1793108741976637000 및 panic inner sdk-source-jobs-35688-1793108759448968600은 각 tmp tag 아래 보존했다. 둘 다 transport-prepared-no-child의 host=None/prepared_ipc Some(io_reaped/local_endpoints_closed=true,had_error=false)/raw_eof=[true,true]/launch_unreturned=false를 원 파일에서 확인했다. peer EOF=false이며 fake process exit0을 만들지 않는다. 오류 진단은 needs_input/Original fixture request preparation failed/reported_error_request_id=original-request-preparation/original_deadline_exceeded=false/publication_permit_retained=false/verified_output=null을 보존했다.

## 다음 목표

legacy file/실제 probe 회귀, receipt 실패 시 origin 정리의 독립 진행, production provider 정책/원 factory 소유권·등록, 실제 HTTP/probe와 전체 CLI 인수 행렬을 닫은 뒤 GUI/AI 비교·수정·판정·이력과 시각 인수를 진행한다. 원 DFB0 지정190/정상bins·R34/R36/browser86/core80은 각 원 입력의 근거다. source375 private overlay·WIP/소유권·원 실패·AI 사용자0/9·NeedsRepair를 유지한다. C-only cache·locked/offline/-j1 유지. 설치/실제browser/HTTP·추론·다운로드·수동GUI·WARP·push0.

기독 modootoday/software-qa/test-fixture-and-mock-build SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 원 setup/teardown·부분오류/panic 및 fixture/실제 분리와 code-change-review-verification SHA9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B의 주장/근거·첫 실패 보존을 적용했다.
