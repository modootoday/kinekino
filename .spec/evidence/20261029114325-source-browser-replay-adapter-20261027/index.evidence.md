---
id: evidence:kinekino/20261029114325
title: 원 browser replay의 process source 소비 — R21/R22
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 browser replay의 process source 소비 — R21/R22

## 구현과 수명 경계

private SDK 후보에 infrastructure/download/browser_replay.rs를 분리했다. BrowserReplayAdmission은 원 provider Arc·non-Clone grant·원 AttemptScope에서 얻은 JobAttemptIdentityV1와 deadline·원 source owner guard를 소유한다. 생성은 launch-free/infallible이며 검사가 실패하기 전에 원 소유권을 보유한다. check는 원 실행 식별/deadline 동일성·원 cutoff·취소·provider current를 검사한다. consume은 local attempted를 먼저 남기고 원 provider의 일회 소비에 도달한다. 실패 뒤 같은 adapter로 재시도하지 않는다. publication check_origin은 consumed/current/원 deadline을 요구하되 publication control에 실행 식별이 없을 수 있음을 유지한다. 저장/CAS/작업 취소·정리는 기존 JobsCore/ProcessSource에 둔다.

기존 실제 JobsCore fixture는 이전 BoundAdmission의 합성 권한 대신 새 adapter를 사용한다. 원 AttemptScope는 실제 retaining coordinator에서만 얻으며 JSON에서 생성하지 않는다. missing execution identity·changed cutoff·소비 전 origin 거절을 source launch 전에 검사한다. fixture provider는 합성 BrowserWork/ledger/grant를 실제로 보유하지만 실제 브라우저나 서비스 허가는 아니다. 기존 source-adapter-test-support feature에만 별도 fixture factory를 공개하고 정상 제품 조립·provider policy는 추가하지 않았다. 실제 document_session Drop→cancel/release와 수명 이전의 제품 연결은 아직 남는다. fixture의 owner guard는 ()이며 원 synthetic work는 provider Arc에 보유한다; 실제 문서 owner 보유를 입증하지 않는다.

## 입력·원 실패 보존

기존 변경은 download/mod.rs·process_source.rs(control_check의 sibling 가시성만 변경), browser_provider/mod.rs·replay.rs·replay/tests.rs, process_source/tests/live.rs·live/jobs.rs다. 신규 browser_replay.rs와 replay/fixture.rs를 분리했다. preimage-r20-replay-adapter에 원본을 보존했다. process_source-reconstructed.rs는 단일 가시성 변경을 역적용해 복원한 content 사본이며 SHA D4BA69C3FB0F233542702BACB5AFCBABD066DB785A66F9E45F9362795C767FD8가 R20 inventory와 일치한다. 원 FileID 사본이라고 주장하지 않는다.

1134 첫 strict 실패는 shared jobs fixture의 integration alias와 library cfg(test) helper 접근이었다. 동일 fixture를 명시 test-support feature 경계에 두고 relative adapter 경로로 수정해1135 strict 통과했다. JobsCore 실제 fixture도 해당 feature 아래로 묶었다. 1142 기본 strict 첫 실패는 그 결과 남은 integration jobs import1 미사용이다. tests/media_download.rs의 import에 동일 feature 조건만 추가한 R22에서1143 기본 strict·1144 support strict 통과했다. 원 실패 로그는 보존하며 broad lint allow를 추가하지 않았다.

- R21 SOURCE.md DAE859271950325C6466FEE82A596FE9A71AA25B7319006070927F45674892CC;inventory667C96D81EE2CF32140D4318BB765EEE7C6CF35731F9101C4EA178DA525110FB;2291파일.
- R22 SOURCE.md B62815B57017A522FA149F8BC33A05680A4C717E711EDF111A6F95CB86B6A399;inventory2C44375D96B3C5F03663DC2780FE55954856EAD014E07823140C869565F2672E;2291파일. R21 대비 integration import cfg만 변경했다.
- root lock B2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D / mirror lock955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5 유지. 원 donor/root/WIP/소유권 유지.

## 실행 artifact·실제 결과

1136 lib no-run producer terminal exit0 후 fixed-r21/sdk-source-tests.exe를 복사·핀했다: SHA9A395FD061134E4734011706031C11410EE5EF998BAC7E1D907571F76DBDFEA9,109374464bytes. 실제 재사용 worker는 fixed-r15/477ED83B9286BA0F556DD61F97C230FFD6F2C4A8770FFD45B6D161EA63E051F7/1000960bytes(1101), 실제 정상 CLI는 fixed-r14/249DD5ECEFFD0A476D3EC7B89B0F7003F9DB5DC13D285AE1126A4B9938DED32B/36405248bytes(1092)다. 새 CLI/worker 빌드로 계상하지 않는다.

operator index runtime/verification/source-sdk-integration-r21-index.json은 proof=false, SHA0D44D2F73C8BF686EDE66F5970190295912AC5A9DDFB6F2767FA26096BFC764F/439bytes다. C2 고정 독립 driver7E7AD25832F177143A3D0D3376EEFC317AE1EF38356E1DC6606B5AECCD5F5F44로 exact3을 직렬 실행했다. 각 outer outcome은 selected_one/ownership complete/physical_closed true·exit0/Empty/nonforced·raw EOF·first_error null이다.

|실행|실제 근거|한계|
|---|---|---|
|1137 원 JobsCore 성공|exact1 pass/938filtered,16byte 검증·CAS 발행;source26244/creation134375735034531114/exit0 Empty nonforced|browser/policy/probe/body 합성|
|1139 정상 CLI|exact1 pass,실제CLI5회;원 grant 소비·Completed·발행·host 정리;source26636/creation134375735593995719/exit0 Empty nonforced|R14 CLI 재사용·로컬IPC|
|1140 취소 CLI|exact1 pass,실제CLI7회;BodyEnd 뒤 probe pending 취소·partial 보유/정리·Cancelled/미발행;source13844/creation134375735707368533/exit0 Empty nonforced|HTTP read 중 취소·일반Ctrl-C 아님|
|1141 순수회귀|fixed-r21 browser77pass/862filtered|실제 browser 실행 아님|
|1138 R21 strict|최종 R21 support all-targets 통과|default는1142 실패 보존|
|1143/1144 R22 strict|기본/support all-targets 통과|R21 actual3을R22 실행으로 승계하지 않음|
|1145 R22 fmt|--all --check 통과|전체190/정상bins 재실행 아님|

outer 근거는 runtime/verification/source-sdk-browser-grant-{jobs,cli-normal,cli-cancel}-os1-r21/outer, 원 inner 근거는 runtime/build-cache/tmp의 동일 tag 하위 sdk-source-jobs-33016-1793099903411690500 / sdk-source-cli-20804-1793099959248376700 / sdk-source-cli-33832-1793099970591942900에 보존했다. 원 source receipts의 IPC io_reaped/local_endpoints_closed·process Closed/receipt Recorded·stdout/stderr EOF를 확인했고 control/data peer EOF false와 구별했다.

R21 verify-current session99602와 R22 verify-current session9948은 각각 terminal exit0/2291 content/identity unchanged다. 모든 heavy 실행은 동일 original handle terminal 뒤 직렬 배정했고 C-only native/source-sdk-integration-r1/tmp·conservative2·locked/offline/-j1을 유지했다.

## 남은 범위

원 실제 document owner 이전·release/close ACK, concrete provider policy, HTTP worker/source request와 grant 바이트 binding·실제probe·전체 오류/취소 행렬·제품 source 등록·전체CLI/GUI는 미완료다. source guard 수명 인터페이스 존재를 실제 문서 보유 검증으로 표시하지 않는다. 실제 브라우저/HTTP·추론·다운로드·수동GUI·WARP·push는 실행하지 않았다. DFB0 190/정상2bins·AI0/9는 원 입력에 유지한다.

기독 modootoday/software-qa/test-fixture-and-mock-build 본문 SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 실제 port·setup/teardown·합성 권한/실제OS 분리와 첫 실패 보존을 적용했다.
