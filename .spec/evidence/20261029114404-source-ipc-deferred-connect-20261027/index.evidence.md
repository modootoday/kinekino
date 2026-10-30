---
id: evidence:kinekino/20261029114404
title: 원 IPC custody 이후 비동기 연결 — SDK R35/R36
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 IPC custody 이후 비동기 연결 — SDK R35/R36

## 변경과 범위

private source-sdk-integration-r1의 owned-process source_pipe만 변경했다. pair는 두 원 server/client handle을 만들지만 overlapped connect는 시작하지 않는다. PreparedSourceIpc가 두 lane을 소유해 반환된 뒤 poll_connected가 원 lane 슬롯에 Operation을 넣고 기존 PID 확인을 수행한다. connected 값은 실제 완료/PID 확인 뒤에만 true다. 연결 전 read/write와 close 이후 재연결을 거절한다. inherited child endpoint는 이미 연결된 원 핸들이므로 connected=true로 인수한다. 원 cutoff·handle 전달·launch whitelist·비상 Operation Drop는 유지한다.

이 변경은 두 번째 pair 실패 시 첫 pending connect를 Drop 비상 leak 경로에 보내는 구성 공백을 제거한다. core의 document owner 조기 인수나 모든 partial setup/panic/receipt 실패를 해결한 것은 아니다. poll_connected 호출자는 같은 원 PreparedSourceIpc를 계속 보유하고 오류 시 명시 poll_close해야 한다. 기존 fixture의 adopt 전 연결 대기는 아직 남는다.

## 원본과 입력

root/donor/원 HTTP bootstrap R36은 변경하지 않았다. 여기서 R35/R36은 SDK 후보의 별도 입력 번호다. preimage-r34-deferred-connect/source_pipe.rs SHA=CDA8707B0F67702E4F96308725B3CF8659E202BE65AC5B230EB95732DB1E1D86. 첫 lint 실패 입력은 preimage-r35-test-module-layout에 보존했다.

SDK R35 SOURCE.md=5C22840187C8FE2F423676F06FB251359905F89C221FF21E2A86AEC64A686ECE, inventory=EFEDEEC69D8072FFE53246794EAF2B977B97C25C87DEA7D1D24D2EC294090F49/2295. verify-current session21623 terminal exit0,2295 content/identity 동일. R36은 시험 모듈을 source_pipe/custody_tests.rs로 분리한 후계 입력이다.

SDK R36 SOURCE.md=D82E8C897B43AC424419F6E9CD123915394B91DA88C3D63A7AD7E7378DE9E4C1, inventory=470D73C86159D4283468E7FC254214559541901B048D8B6B33E3E9B5247C5E0D/2296. source_pipe.rs=FA3FE1F2451DF8917294F7EC0612769B4649ABCC709FE768F30B608B556701AE, custody_tests.rs=49C02333889919974444820EFA07BB21E73D7BB0408D570AC43761A235961EF7. AGENTS40C7F5A6은 원 핀과 동일하다.

## 실제 검사

최종 R36 verify-current session54746 terminal exit0으로2296파일 content/identity 동일을 확인했다. 모든 위 실행기/컴파일/선택 OS 시험 handle은 terminal 확인 후 다음 heavy 작업을 배정했다.

|실행|결과·경계|
|---|---|
|1208 R35 새 custody 시험|3/3; 두 번째 pair 오류 주입 후 첫 pipe Weak 회수, 미연결 IO 거절/명시 close, 원 양 lane 연결/close|
|1209 R35 제품 support all-targets strict|실제 receipt terminal exit0|
|1210 R35 owned-process lib|10/10; 새3 포함, 합산13으로 세지 않음|
|1211 R35 fmt|실제 receipt terminal exit0|
|1212 R35 root lib no-run|실제 receipt terminal exit0; compiler-artifact executable 확인 후 fixed-r35 보존|
|1213 R35 정상 CLI|exact1pass/951filtered; 원 SDK→source IPC→발행 흐름, 실제 정상CLI fixture5RPC|
|1214 R35 취소 CLI|exact1pass/951filtered; 원 partial/probe 대기 취소, 실제 취소CLI fixture7RPC|
|1215 R35 owned-process 자체 strict|첫 실패 exit101: items_after_test_module; 시험 위치 문제, 원 로그 보존|
|1216 R36 owned-process all-targets/test-support strict|시험 별도 파일 분리 후 실제 receipt terminal exit0; lint 예외 없음|
|1217 R36 owned-process lib|10/10|
|1218 R36 fmt|실제 receipt terminal exit0|
|1219 R36 제품 default all-targets strict|실제 receipt terminal exit0|

R35 고정 test=0E6DEBD2277405857C5E134A19D280CBF92CFC9DB89C59D61F746DE6C70F8713/109486080 bytes, producer1212. worker는 기존 R15/1101의477ED83B9286BA0F556DD61F97C230FFD6F2C4A8770FFD45B6D161EA63E051F7/1000960, 정상 CLI는 R14/1092의249DD5ECEFFD0A476D3EC7B89B0F7003F9DB5DC13D285AE1126A4B9938DED32B/36405248을 명시 재사용했다. 새 정상 bins로 계상하지 않는다. operator index r35 SHA3888C79211BC7FA2C64AAE8CD2DA62867531D505FE45CD7359A70DE3CAD9B8A5,proof=false.

outer 증거는 runtime/verification/source-sdk-deferred-ipc-cli-{normal,cancel}-os1-r35/outer다. 둘 다 selected_one=true/first_error=null/ownership complete/physical_closed=true/원 outer exit0·Empty·nonforced·raw EOF다. outer physical closure를 실제 browser/COM 종료로 확대하지 않는다. 본문/probe/권한은 offline fixture이며 실제 HTTP는 실행하지 않았다. R35 actual2를 시험 파일 분리 후 R36 재실행으로 표시하지 않는다.

## 남은 목표와 정책

원 document owner 조기 인수·adopt 전 부분 초기화·receipt/probe 독립정리·legacy file/probe 회귀·production provider policy/source 등록·실제 HTTP/probe·전체 CLI 인수·GUI/AI 비교/수정/판정과 시각 인수가 남는다. DFB0 190/정상bins와 기존 R34/browser86/core80은 원 입력의 근거다. source375 private overlay·WIP/소유권·AI 승인0/9·원 후보NeedsRepair 유지. 추론/다운로드/수동GUI/WARP/push0,설치0. C-only cache·heavy1·locked/offline/-j1·동일 handle terminal 후 직렬을 유지했다.

기독 modootoday/software-qa/test-fixture-and-mock-build SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 부분 setup 실패/원 teardown·실제와 fixture 분리, code-change-review-verification SHA9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B의 주장/근거 분리를 적용했다.
