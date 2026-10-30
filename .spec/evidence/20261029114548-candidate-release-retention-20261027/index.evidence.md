---
id: evidence:kinekino/20261029114548
title: 원 candidate 정리 완료 전 terminal 보류 — R27/R28/R29
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 candidate 정리 완료 전 terminal 보류 — R27/R28/R29

## 구현

기존 media-jobs PendingOwnedAttempt의 Stage::Settled/Output에 원 candidate 정리 단계를 연결했다. 새 registry/thread/owner carrier는 없다. OwnedOutputCandidate::poll_release는 필수 typed port이며 정상 경로에서는 verify 후·Completed CAS 전에 호출한다. pending이면 같은 candidate·verified output·원 deadline을 보유한다. publish/verify 성공 상태를 유지하므로 정리 재조회가 재발행/재검증을 만들지 않는다.

늦은 취소도 unpublished candidate의 정리 완료 후에만 acknowledge_cancel한다. publication/verification/store 오류의 failure persistence도 원 candidate 정리 전에는 terminal을 기록하지 않는다. 원 첫 실패와 후속 정리 오류·diagnostic을 보존한다. release callback panic은 원 opaque payload와 candidate를 같은 slot에 보유하고 poisoned로 표시해 재조회 성공/closed를 조작하지 않는다. release 지연이 원 deadline을 연장하지 않는다. 이미 발행/검증한 출력은 정리 실패 진단에 유지하고 삭제하지 않는다.

DownloadCandidate는 원 source의 release를 poll한다. 실패/취소처럼 candidate가 반환되지 않은 DownloadCoreSession은 source child/probe 정리 후 origin release까지 관측해야 owner settlement를 반환한다. candidate가 반환됐으면 publication 직전 source.check_origin을 위해 origin을 보유하고 core의 후속 release 단계로 넘긴다. ProcessSource는 기존 child closed를 확인한 뒤 원 admission의 release를 호출한다. 일반 ReplayAdmission의 기본 release는Unsupported다. BrowserReplayAdmission을 source에 사용하려면 G가 명시 ReplayOriginOwner를 구현해야 하며 release가 시작되면 current 재사용을 거절한다. ()의 추가자원없음 구현은 source-adapter-test-support feature에만 있다. 실제 BrowserDocumentSession owner의 이전/비동기 cancel·native release 연결은 아직 작성하지 않았다.

VAD CompleteCandidate는 기존 프로세스 settlement 뒤 immutable output만 보유하므로 추가 resource release가 없음을 명시 구현했다. 새 추론/모델 실행은 하지 않았다. 기존 DownloadSession/SourceOpener는 이미 close된 기존 session을 닫는 호환 경로를 유지하고 ProcessSource는 명시 nonblocking origin hook으로 override했다.

## 소유권·입력 보존

private 변경은 media-jobs의 owned_attempt/{session,outcome}·worker/owned 및 tests/persistence, media-vad-executor candidate, download owned_session/ports와 ProcessSource/BrowserReplayAdmission 및 기존 fixture다. exact13 원 경로/복사 매핑은 후보 runtime/verification/preimage-r26-candidate-release/index.json에 보존했다. persistence 원 content는root와 R26 inventory SHA2C898A84D80F6063080C5395E37DB4EB90AAF85C5ACF6DFAA8AB786FC9FB71F8 일치 후 복사했다; 원 FileID 사본이라고 주장하지 않는다. 원 root/donor/WIP/소유권·source375 private overlay 유지.

|입력|SOURCE.md SHA|inventory SHA|범위|
|---|---|---|---|
|R27|원 snapshot 파일 참조|00C7A31375EE346C42286E5F1CF17EF8422456780D682B52066D333B3D1997F5|1170 시험 컴파일 첫 실패 보존|
|R28|E16F8408860B7D97CA8AD876451895942424674A2FB4DCAC55D9E691A0700279|D8BF43B5851EBE778A76D88DD6D67A88048C6FBCD2CF1E4475D915289162AAE9|2292파일,실제CLI/core검증 입력|
|R29|CE8BA14300407E67BC9BFE95786D7EB80B2E518C54E09E3F27A2BA5FAA3F4CF3|FEC695E24587FDAB53FF8175DD9B94B4C60714A6A196F93097FB86CC04915940|2292파일,R28 대비 replay.check_request 서명 형식만 수정|

root/mirror lock B2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D /955DEBA0338A3A32ED64D6B2A500C192CD55517336FE531D6EED374D0B2916A5,AGENTS40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C 유지.

## 검증과 첫 실패

- 1169 중간root support strict 통과. 1170 media-jobs 시험 첫 컴파일 실패는 persistence::verified 내부 helper의 sibling 가시성이다. 공개 SDK 권한을 늘리지 않고 pub(super)로 한정 조정했다. 원 stderr/receipt 보존.
- 1171 중간 selected14,1172 최종 selected15 통과. 새5시험은 release pending의candidate/verified보유·publish/verify1회,release오류 뒤 보유/정리 후Failed,늦은cancel의publish0,releasepanic payload/candidate보존·재poll0,publication첫실패와cleanup후속오류우선순위를 검증했다.
- 1179 최종 media-jobs lib 전체80/80 통과(새5 포함;앞 selected15와 합산하지 않음). 1180 원 download owner 관측·병렬 정리 오류 회귀4/4 통과.
- 1173 R28 root support all-targets strict,1177 media-jobs all-targets strict,1178 root 기본 strict 통과.
- 1174 lib no-run terminal exit0 뒤 fixed-r28/sdk-source-tests.exe를 별도 copy/핀했다:SHA8739BCFBE3DB32BD30F814B05738CFC1810447C5001BD04B38CB095029F64CCE/109419520bytes.
- 1175 정상 CLI exact1pass/942filtered,5RPC;1176 취소 CLI exact1pass/942filtered,7RPC. 원 child·SDK/jobs/CLI 정리 회귀를 통과했다. 실제 origin은 test-support ()이며 문서 해제를 입증하지 않는다.
- R28 verify-current session36498 terminal exit0,2292 content/identity unchanged.
- R29 1182 fmt exit0,1183 support strict exit0,1184 기본 strict exit0. verify-current session16458 terminal exit0,2292 content/identity unchanged. R28 actual/core 실행을 R29 실행으로 승계하지 않는다.

## fmt 보고 오류 정정

1181 fmt receipt는exit1이었다. replay.check_request의 pub(crate)→pub 변경 뒤 서명 줄바꿈이 맞지 않았다. 앞선 실행기 호출 뒤 다른 PowerShell 조회의 exit0만 읽고 일부 fmt를 통과로 잘못 보고했다. 원 receipt 재감사 결과1118/1124/1129/1133/1145는exit0,1154(R23)/1161(R24)/1168(R26)은exit1이다. 세 원 SOT를 정정했고 실패 receipt/stdout를 그대로 보존했다. 원 source375 DFB0의 검증과 private R23 이후 formatter 실패를 혼동하지 않는다. 원 R28 replay.rs preimage를 보존한 뒤 해당 leaf만 rustfmt하여1182에서 실제 exit0을 확인했다. 당초 실패가 소급 성공한 것은 아니며 형식 수정은 R29다.

## 실제 artifact·정리 증거

operator index runtime/verification/source-sdk-integration-r28-index.json은proof=false/SHA0F2A2E5263D22C8AAD1E947A5BBA777AE30A3F56D4A4E07880CE687D6D047F67다. 원 C2 독립 driver로 actual2를 직렬 실행했고 두 outer outcome 모두 selected_one/ownership complete/physical_closed·exit0/Empty/nonforced·raw EOF·first_error null이다. worker는 고정R15/1101(477ED83B9286BA0F556DD61F97C230FFD6F2C4A8770FFD45B6D161EA63E051F7/1000960bytes),CLI는고정R14/1092(249DD5ECEFFD0A476D3EC7B89B0F7003F9DB5DC13D285AE1126A4B9938DED32B/36405248bytes)를 실제 재사용했다.

outer는 runtime/verification/source-sdk-candidate-release-cli-{normal,cancel}-os1-r28/outer,원 inner는tmp의동일tag 하위 sdk-source-cli-30156-1793102426385309800 / sdk-source-cli-36160-1793102440393629300에 보존했다. source28256/creation134375760265235374 및35624/creation134375760405255652 모두exit0/Empty/nonforced,io_reaped/local_endpoints_closed·Closed/Recorded/raw EOF다. peer control/data EOF=false는 local close와 구분했다. C-only native/source-sdk-integration-r1/tmp·conservative2·heavy1·원 handle terminal 후 직렬·locked/offline/-j1 유지.

## 남은 작업

후속은 실제 BrowserDocumentSession의 소유권 이전과 명시 ReplayOriginOwner 구현·원 cancel admission/native Capture 해제/ticket release pending을 연결하는 것이다. 이번 구현은 이를 받을 core 단계이며 실제 문서/COM/공유 WebView 프로세스 해제 ACK가 아니다. provider별 실제 replay policy·HTTP/probe·제품 source 등록·전체CLI/GUI·AI사용자품질0/9는 남는다. 설치/실제브라우저/HTTP·추론·다운로드·수동GUI·WARP·push는 수행하지 않았다. DFB0 190/정상bins를 새 입력 실행으로 계상하지 않는다.

기독 modootoday/software-qa/test-fixture-and-mock-build 본문 SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 원 port/owner·pending/error/panic·합성/실제OS 분리·원 실패 보존을 적용했다.
