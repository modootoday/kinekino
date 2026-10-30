---
id: evidence:kinekino/20261029114511
title: 원 grant와 immutable bootstrap의 요청 일치 — R23
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 grant와 immutable bootstrap의 요청 일치 — R23

## 구현과 경계

private SDK의 ProcessSourceOpener::open은 spent/current 검사 후, ParentTransfer의 원 immutable HttpBootstrap을 admission.check_request로 대조하고 나서만 start/launch/bootstrap을 수행한다. ReplayAdmission의 기본 request 검사 구현은 Unsupported다. ParentTransfer에는 immutable borrowed request getter만 추가했고 원 HTTP/launcher 정책은 넣지 않았다.

BrowserReplayAdmission은 원 grant의 response URL과 관측 Referer를 target/referer에 대조한다. URL은 url::Url의 canonical 표현으로 비교한다. URL/query 또는 Referer가 다르면 RevisionConflict다. 관측 Absent는 전송 None과만 일치하며, 원 Url은 해당 canonical URL과만 일치한다. 명시 Empty는 현 bootstrap의 Option<Url>로 표현할 수 없으므로 Unsupported이고 임의 omission으로 변환하지 않는다. method는 원 metadata의 GET admission과 고정 GET transport 경계에 남는다. 이 대조는 current/일회 소비를 대체하지 않는다. grant의 public check_request는 검사만 제공하며 constructor/Clone/Serialize 권한을 추가하지 않는다.

원 fixture는 요청 경로에 맞는 합성 network response/grant를 만들어 실제 오프라인 worker와 동일한 target을 대조한다. source-adapter-test-support feature에만 있는 factory이며 실 provider 정책·허가가 아니다. 독립 LiveAdmission도 명시 offline fixture 목적지만 허용한다. 제품 source 등록·HTTP는 수행하지 않았다.

## 변경·입력 핀

기존 private files: packages/media-source-host/src/parent_transfer.rs, src/infrastructure/download/{process_source.rs,browser_replay.rs,process_source/tests.rs,process_source/tests/live.rs,process_source/tests/live/jobs.rs}, src/application/browser_provider/{mod.rs,replay.rs,replay/fixture.rs,replay/tests.rs}. 신규 source 파일은 없다. 원 preimage는 후보 runtime/verification/preimage-r22-request-binding에 보존했다. 원 R36·donor·root/WIP/소유권은 변경하지 않았다.

input-r23 SOURCE.md SHA B81FC6CF3DB086B427AAF0A9733218C0CA78B1936CBD72D01331BFB3F6EF8C59;inventory5DD054620B4711AFE88F3E5D7EFA95E65793D273636BFB704F3E98F2CEB7FE3E;2291파일.

root lock B2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D/mirror lock955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5/AGENTS40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C 유지.

## 실제 검증과 첫 실패

1146 strict 첫 실패는 독립 integration harness에서 crate-private grant 대조 메서드를 접근하지 못한 것이다. 기존 source 경계에 맞춰 검사 메서드만 public으로 수정하고1147 support all-targets strict 통과했다. 1148 lib no-run terminal exit0 뒤 fixed-r23/sdk-source-tests.exe를 별도 copy/핀했다: SHA8BC85408053EEE3AC3E0E7DE56E166200B99AB68C87DA961289C1E614B835B39/109392896bytes.

재사용 worker는 R15/1101 SHA477ED83B9286BA0F556DD61F97C230FFD6F2C4A8770FFD45B6D161EA63E051F7/1000960bytes, 정상 CLI는 R14/1092 SHA249DD5ECEFFD0A476D3EC7B89B0F7003F9DB5DC13D285AE1126A4B9938DED32B/36405248bytes다. 새 빌드로 계상하지 않는다. operator index runtime/verification/source-sdk-integration-r23-index.json은 proof=false/SHA5FB07A72171BA12ACD130A2E7D5E0E3341B21982991AD7B3F223EC4A7CBFF907이다. 원 C2 독립 driver로 아래 exact3을 직렬 배정했고 각 outer outcome selected_one/ownership complete/physical_closed·exit0/Empty/nonforced·raw EOF·first_error null을 확인했다.

|gate|실제 결과|검증 한계|
|---|---|---|
|1149 request mismatch|1pass/941filtered. 현재 검사 통과 후 request validator 거절→child host 없음·consume0·재open 거절·원 prepared IPC 정리/receipt·guard drop1|validator는 명시 거절 fixture; Browser grant의 실제 URL 비교는1152 순수 시험으로 별도 확인. worker는 guard-only, 실행하지 않음|
|1150 정상 CLI|1pass/941filtered·실제5RPC,같은 target grant→16byte 검증/발행·Completed·정리|원 browser 관측/정책/probe/body 합성|
|1151 취소 CLI|1pass/941filtered·실제7RPC,probe pending 취소·partial 보유/정리·Cancelled/미발행|HTTP 전송·read중취소·일반Ctrl-C 아님|
|1152 browser pure|79pass/863filtered. 새2시험 URL/query/host/Referer 변경 거절과 Empty 미지원|실제 서비스 replay 허가 아님|
|1153 기본 strict|all-targets -D warnings 통과|support strict는1147|
|1154 fmt|receipt 재감사:exit1,서명 줄바꿈 불일치|당시 통과 보고 오류;1182에서 형식 수정 후 통과,원 실패 보존|
|custody|verify-current session78298 terminal exit0,2291 content/identity unchanged|private snapshot이며 canonical 적용 아님|

outer 근거는 runtime/verification/source-sdk-{request-mismatch,request-cli-normal,request-cli-cancel}-os1-r23/outer다. 원 inner 증거는 runtime/build-cache/tmp의 해당 tag 안에 보존했다. 정상 source31724/creation134375741282296417,취소 source25452/creation134375741344637827 모두 original receipt exit0/Empty/nonforced다. transport io_reaped/local_endpoints_closed/process Closed/receipt Recorded/raw EOF와 control/data peer EOF=false를 구별했다. 원 inner CLI rpc-count는 각각5/7이다. C-only native/source-sdk-integration-r1/tmp,stable MSVC,conservative2,heavy1,locked/offline/-j1 유지.

## 미완료

이 epoch은 URL/Referer 대조다. 원 attempt와 wire binding digest/byte budget의 추가 계약, original live document owner 이전/close ACK, concrete provider policy·HTTP worker/probe·제품 통합/전체CLI/GUI는 남는다. 원 grant에 다른 요청을 붙이는 공백을 줄였지만 실제 다운로드 지원으로 표시하지 않는다. 실제 브라우저/HTTP·추론·다운로드·수동GUI·WARP·push·설치를 실행하지 않았다. DFB0 190/정상bins와 AI0/9는 원 상태로 유지한다.

기독 modootoday/software-qa/test-fixture-and-mock-build 본문 SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 negative/positive 구분·실제 원 cleanup·fixture와서비스 사실 분리·첫 실패 보존을 적용했다.
