---
id: evidence:kinekino/20261029114802
title: 원 document session의 source owner 이전·정리 — R30/R31
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 document session의 source owner 이전·정리 — R30/R31

## 구현·공통 SDK 경계

private SDK의 BrowserDocumentSession::into_source_owner가 원 provider Arc·ticket·원 session control deadline을 non-Clone/non-Serialize BrowserDocumentSourceOwner로 이전한다. session 필드는 외부 구성 불가로 제한했고 source owner 생성은 원 session 경로에 한정했다. 이전이 끝나면 기존 session Drop은 cancel/release를 보내지 않는다. 일반 catalog/episode의 기존 bounded Drop 경로는 유지했다.

Source owner의 poll_release는 sleep/대기/새 browser 시작이 없는 한 번의 admission/snapshot 단계다. 원 ticket을 poll하고 snapshot ticket을 대조한다. 비terminal이면 취소를 요청하고, 요청이 승인된 뒤에는 재요청하지 않는다. 원 terminal 뒤에는 native Capture 해제 gate를 포함한 기존 provider.release의 성공을 기다린다. 알려진 원 state busy/queue full/native release pending만 pending으로 다루며 그 외 오류는 원 ApiError로 반환한다. NotFound/다른 ticket을 해제 성공으로 변환하지 않는다. 같은 object에 release 성공을 남겨 이후 poll은 무동작이다. shared BrowserProviderOwner.shutdown을 호출하지 않는다.

원 session deadline과 요청 cleanup deadline의 min만 유지한다. 만료된 live owner는 새 source context로 갱신할 수 없다. 만료 중 pending은Timeout이며 같은 owner로 실제 cleanup의 후속 관측은 가능하다. deadline을 늘리지 않고, 이후 job 성공 가능 여부는 원 core deadline gate가 판단한다.

BrowserReplayAdmission의 ReplayOriginOwner를 이 source owner에 연결했다. 원 provider Arc의 pointer identity·grant ticket을 검사하고 기존 provider.current 검사를 계속 수행한다. owner 정리가 시작되면 source current 재사용을 거절한다. 이전 () 시험 owner 구현은 제거했다. 실제 CLI fixture도 원 session에서 이전한 동일 source owner를 보유하며, provider/native completion은 여전히 합성이다. 실제 WebView·COM·공유 프로세스 종료 proof가 아니다.

## 정확 범위·보존

신규 application/browser_provider/document_owner.rs·document_owner/tests.rs를 책임별로 분리했다. 기존 document_session.rs·browser_provider/mod.rs·replay/fixture.rs·infrastructure/download/browser_replay.rs·process_source/tests/live/jobs.rs를 연결했다. 원 preimage는 후보 runtime/verification/preimage-r29-document-owner에 보존했다. 중간 R30의 deadline 보강 전 소스는 preimage-r30-document-deadline에 별도 보존했다. root/donor·원 WIP·담당 소유권과 source375 private overlay 경계 유지.

R30 inventory B1E5B2C9089F561804BCAE5737850A72608D5DE2EA14761C895DD84D8968559B/2294는 중간 구현이다.1185 strict·1186 no-run은 해당 입력의 실행이며 selected runtime 성공으로 계상하지 않는다. 최종 R31 SOURCE.md SHA EEC78FE0C92FC7C043EA6EEC7C6DF970C6972139195E7D3CE06B9D85169E7E51;inventory50066E713F0E88F1B7B31A4E9208B9AE2E6A2013145192AE85F4334F4633DF56;2294파일.

root lockB2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D/mirror lock955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5/AGENTS40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C 유지. C-only native/source-sdk-integration-r1/tmp·conservative2·heavy1·원 handle terminal 뒤 직렬·locked/offline/-j1 유지.

## 실제 검증

이번 compile/test gate 첫 실패는 없다. source 적용 중 문맥 불일치 patch2회는 atomic 거절되어 해당 호출에서 소스 변경이 없었고, 현재 문맥을 읽어 수정했다. 이전 fmt/시험 실패와 정정 기록은 보존한다. 최종 fmt는 실행기 receipt의 실제 child exit0을 확인했다.

- 1187 deadline 보강 strict 통과.1188 최종 lib no-run terminal exit0 후 fixed-r31/sdk-source-tests.exe를 별도 copy/핀했다:SHA1015E88D8FC46D638367EE515C04F3ADFE23A310E0AF52DD4E7E5DFAF088C30F/109469696bytes.
- 1189 application/browser86pass/864filtered. 새7시험은 handoff 시 cancel/release0,원 cancel 승인1회·native release pending/ticket 보존·완료 후 idempotence,queue full의 미승인 요청만 재시도,NotFound 비성공,다른snapshot 거절,deadline 불연장·만료 후 실제정리관측,원 Arc/ticket 일치와expired session 갱신 거절을 검증했다. provider/native 관측은 합성이며 WebView를 시작하지 않는다.
- 1190 정상 CLI exact1pass/949filtered,5RPC:원 grant·명시 document owner·실제 JobsCore/source child를 거쳐 검증/발행·owner release 이후Completed.
- 1191 취소 CLI exact1pass/949filtered,7RPC:BodyEnd 뒤probe pending 취소·partial 보유/제거·원 document owner 정리 경로 이후Cancelled/미발행.
- 1192 support all-targets strict,1193 기본 all-targets strict,1194 fmt --all --check 모두 실제 receipt terminal/exit0.
- custody verify-current session21853 terminal exit0,2294 content/identity unchanged.

actual2는 기존 독립 C2 고정 driver로 직렬 실행했고 두 outer outcome selected_one/ownership complete/physical_closed·first_error null·exit0/Empty/nonforced/raw EOF를 확인했다. 이 physical_closed는 driver/source child 정리의 사실이며 합성 browser의 OS 물리 종료로 확대하지 않는다. actual worker는 R15/1101 SHA477ED83B9286BA0F556DD61F97C230FFD6F2C4A8770FFD45B6D161EA63E051F7/1000960bytes,CLI는R14/1092 SHA249DD5ECEFFD0A476D3EC7B89B0F7003F9DB5DC13D285AE1126A4B9938DED32B/36405248bytes를 재사용했다. 새 정상 product bins 빌드로 계상하지 않는다.

operator index runtime/verification/source-sdk-integration-r31-index.json은proof=false/SHA0463A9861E1118E06E5381C58453DD1BF39F7D3994B466313A9543739AB84774다. outer는 runtime/verification/source-sdk-document-owner-cli-{normal,cancel}-os1-r31/outer에 보존했다. inner는tmp의 동일tag 하위 sdk-source-cli-12756-1793103877559013300 / sdk-source-cli-33832-1793103934717322900에 보존했다. 원 source3172/creation134375774777281245 및32492/creation134375775348946383 모두exit0/Empty/nonforced이며 io_reaped/local_endpoints_closed·Closed/Recorded/raw EOF다. peer control/data EOF=false는 local close와 구분했다. inner rpc-count는5/7이다.

## 남은 인수

실제 native browser/COM/callback 정리와 공유 host shutdown 경쟁,worker disconnect·hard release 오류의 전체 행렬,원 live owner를 넘긴 뒤 core custody 확정 전 fallible preparation/unwind의 원 owner 반환·보유 경로,실 provider policy·HTTP/probe·제품 등록/전체CLI/GUI는 남는다. 현재 production source factory는 등록하지 않았다. fixture의 명시 owner 구현을 actual native 인수나 사용자 품질 승인으로 계상하지 않는다.

R28 core80/지정DFB0 190/정상bins는 각 원 입력에 유지한다. R31에서 core80/전체190/bins를 재실행한 것으로 쓰지 않는다. AI사용자승인0/9·원 후보NeedsRepair·GUI 시각 미인수 유지. 실제브라우저/HTTP·추론·다운로드·수동GUI·WARP·push·설치는 수행하지 않았다.

기독 modootoday/software-qa/test-fixture-and-mock-build 본문 SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 원 owner 이전·setup/teardown·명시 관측·fixture/실제OS 분리 및 입력 고정을 적용했다.
