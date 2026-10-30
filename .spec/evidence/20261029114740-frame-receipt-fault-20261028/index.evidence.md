---
id: evidence:kinekino/20261029114740
title: 원 frame owner의 receipt 실패·명시 재기록 — R55
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원 frame owner의 receipt 실패·명시 재기록 — R55

2026-10-28 중앙 PM. 제품 로직 변경 없이 원 Session/Windows child/원 FileSink를 사용하는 ignored Rust 시험을 추가했다. 공개 SDK의 복구 API는 미구현이며 [복구 계약](../../contracts/production/frame-receipt-recovery.md)의 R0/R1/R2/R5/R6 기반 동작을 선택 검증했다. R3/R4/R7 및 실제 제품 연결을 완료로 표시하지 않는다.

입력2308 SOURCE SHA256 19E6CFA9C392F84F75235D7922B5238A6108B1089F8AE7D5A6A8971C1D47E097, inventory E20C96922E980E47522E101D75B4C4F29E6C0F6C74F2C724C0295A5996DE5BD1. source375/root canonical 미적용 private 후보. 원 owned/mod.rs와 첫 시험 실패 소스는 preimage-r53-frame-receipt-fault에 보존했다.

실제 child는 고정 시험 exe를 --exact frame_receipt_zero_case_fixture로 실행해 정상적으로 0개 시험/exit0로 끝난다. 이는 내부 기능 시험 통과 개수가 아니라 Windows 원 process family/receipt fault를 만드는 fixture다. 원 목적지 receipt-0000.json에 sentinel을 먼저 만들어 create_new 충돌을 유발했다. child exit0/Empty·physical_closed 후에도 receipt 미기록/settled=false, Session.poll 20회가 false·새 receipt 부재임을 확인했다. 시간 경과에 따른 자동 재시도를 구현하지 않았다.

명시적으로 같은 session.owner.record_cleanup과 같은 sink를 한 번 호출해 receipt-0001.json을 기록했다. 원 PID/creation 식별 유지, receipt의 child_exit0/familyEmpty/forcedfalse, 최종 settled=true를 확인했다. Session.complete는 최초 EngineFailed와 같은 메시지를 반환하면서 owned_process.physical_closed/receipt_recorded=true를 남겼다. 원 sentinel은 전후 동일하다. 재기록을 operation 성공으로 승격하지 않았다. 공개 API가 없으므로 이 직접 primitive 호출은 제품 사용자의 복구 요청을 증명하지 않는다.

|로그|결과|
|---|---|
|1439/1440 R54|fmt write0; no-run 첫 E0277 실패(LaunchFailure가 Debug 미구현이라 unwrap 불가), 원 로그/소스 보존|
|1441/1442 R55|fmt write0; 실패 owner를 인수하는 match로 수정 후 Native no-run0/88762ms|
|1443|2308 snapshot0|
|1444|선택1 pass/1556 filtered/0.05초, runner terminal0/15878ms|
|1445|Native all-target strict0/22826ms|
|1446|기본 all-target strict0/17665ms|
|1447|fmt check0/3707ms|

1448 최종 custody는 terminal0/42368ms, 입력2308 content/identity unchanged를 확인했다. 원 R52 전체190/정상bins는 해당 입력의 실제 결과로 유지하고 R55 재실행으로 계상하지 않는다. 이번 선택 시험의 외부 C2 familyEmpty/child0/nonforced/physical_closed=true다. 첫 compiler 실패를 새 성공으로 덮어쓰지 않는다.

고정 시험 runtime/build-cache/native/source-sdk-integration-r1/fixed-r55/frame-receipt-tests.exe SHA256 02E9DCFE0367C90D70D3AC1B580CA5E584230DDE62D47F3C1BB6754D63A2B823 /191995392bytes. raw/index: runtime/verification/source-sdk-r55-frame-receipt-index.json, source-sdk-r55-frame-receipt-r1/outer/*, source375-datalab-b0-r1-logs/1439~1448*. 원 collision·새 receipt·recovery-error.json 및 raw는 runtime/build-cache/tmp/source-sdk-r55-frame-receipt-r1/frame-owned-29352-1793124435100448500-0에 보존했다.

다음 실제 구현은 같은 opaque operation의 typed cleanup pending 관측·generation 바인딩 명시 재시도다. foreign/stale/duplicate/capacity/close·연속 저장 실패·부분쓰기·권한 오류를 별도 인수한다. 현재 전체 제품 CLI/GUI·AI품질0/9·receipt복구 미구현은 계속 미완료다. source/WIP/소유권·원실패·보류 유지, GUI/추론/다운로드/WARP/push 없음. C-only cache·heavy1/CPU2/BelowNormal·locked/offline/-j1, 컴파일 중 읽기 snapshot만 병행했다.

modootoday/software-qa/test-fixture-and-mock-build SHA256 A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및 code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B 적용: fault·원 소유권·명시 primitive와 공개 제품 API 지원을 분리했다.
