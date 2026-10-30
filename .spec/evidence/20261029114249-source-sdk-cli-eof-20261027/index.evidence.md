---
id: evidence:kinekino/20261029114249
title: 실제 CLI의 길이 일치·BodyEnd 부재 거절 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 CLI의 길이 일치·BodyEnd 부재 거절 — 2026-10-27

## 검증 범위

[정상 CLI R14](source-sdk-normal-cli-20261027.md)의 실제 소비 경로에 EOF 사례를 추가했다. fixture worker는 Metadata length16과 고정16byte Data의 write completion 뒤 명시 BodyEnd 없이 원 IPC를 닫고 exit0으로 끝난다. declared length 일치·정상 process exit를 application body 종료로 오인하지 않는지 검사한다. parent가 읽은 중간 partial16byte를 별도 계측한 것은 아니며 고정 worker의16byte 송신과 실제 EOF/실패 경로를 입증한다.

제품 SDK/application 정책은 변경하지 않았다. private fixture의 URL 경로와 preparation flag 및 CLI 시험만 확장했다. source permission/body/BytesProbe는 synthetic이며 실제 HTTP/미디어 decoder/사용자 품질 승인을 뜻하지 않는다. 변경 전 worker/prepare/CLI/cancel leaf는 mirror runtime/verification/preimage-r14-eof에 보존했다.

## 선택 입력과 실제 게이트

input-r15 SOURCE.md SHA256 `9D579630B56CA230582EADE7A08DC674E3DD8A2AC426371C649B7D7C8D5172D4`,inventory `3B8168CE635891FBA4B3BEE83E952D7F0F946BF4ECC22BDE988397944695F018`,2286files.

|로그|실제 결과|
|---|---|
|1100|제품 all-target/시험 feature strict 통과|
|1101|정상 EOF fixture worker bin build 통과|
|1102|제품 lib test no-run 통과/30.12s|
|1103|BodyEnd 부재 CLI exact1pass/922filtered/0.41s|
|1104|정상 BodyEnd/Completed CLI 회귀 exact1pass/922filtered/0.40s|
|1105/1106|전체 workspace fmt 통과/선택2286 content·identity 동일|

첫 실행 실패는 없었고 기존 원 실패를 덮어쓰지 않았다. 이전 R14 CLI13회는 원 입력의 실행이며 이번에는 새 CLI10회만 계상한다.

고정 `native/source-sdk-integration-r1/fixed-r15/sdk-source-tests.exe`: SHA256 `15F2B7C3D16E02D25B58D24B3AE872B0167B475ADACC46AFBEA1099599C60D14`,109186048B. 같은 폴더 worker477ED83B9286BA0F556DD61F97C230FFD6F2C4A8770FFD45B6D161EA63E051F7/1000960B는1101 정상 producer다. 실제 CLI는 원 R14 fixed249DD5EC/36405248B·profile.test=false를 재사용했다. R15에서 CLI를 재빌드했다고 계상하지 않는다. 원 Rust deny-write/delete read guard와 동일 handle SHA/size 검사를 유지했다.

operator index D4D422101DFF7F3F4C4A643A01625B96DABF0A3B9357C1ED89E99AC94B1B3AA0/376B는 proof=false다. C2 fixed driver7E7AD258/45s+cleanup5s의 원 test/worker guard와 별도 CLI guard,원 request budget·C-only/locked/offline/j1/conservative2/heavy1을 사용했다. 각 live handle terminal 뒤 다음 heavy 작업을 시작했다.

## 실제 관측

정확한 negative case `infrastructure::download::process_source::tests::live::jobs::cli::normal_cli_rejects_matching_length_pipe_eof_without_body_end`.

capabilities/schema/jobs.submit/get/watch의 실제 CLI5회에서 source job은 Failed/EngineFailed,publication_started=false/output=null이었다. 원 부분 파일은 같은 정리 경로에서 제거됐고 원 source consume1/transport1/guard1 및 pipe/sdk Joined·retaining job host 종료를 확인했다. CLI jobs.get의 ok=true/exit0은 정상 조회이며 실패한 작업을 성공으로 계상하지 않았다.

원 source child19008/creation134375703583642195,exit0/Empty/nonforced. parent IPC는 data_eof=true/control_eof=false/io_reaped=true/local_endpoints_closed=true/had_error=false였고 원 raw EOF/final process receipt Recorded를 확인했다. 실제 peer EOF를 BodyEnd로 대체하지 않았다. 외부 test36288 및 정상 회귀37524는 각각 exit0/Empty/nonforced/raw EOF/selected_one=true/ownership complete/first_error=null이다.

정상 회귀도 같은 CLI5회로 Completed/실제16byte 발행을 유지했다. 각 원 request/response/receipt/raw/rpc-count/local-close 기록과 outcome·inner-observed copy를 `runtime/verification/source-sdk-cli-eof-os1-r15`, `source-sdk-cli-normal-os1-r15`에 보존하고 원 C tmp도 유지했다. 실행 후 test/worker/원 R14 CLI hash 및 root lock B2E3D540 동일을 확인했다.

## 남은 범위

부분 frame/중간 Data 단절·실제 CLI 취소/재시작·unassigned rollback·실제 OS probe 및 동시 오류/panic·browser issuer/실제 HTTP 인수는 남는다. 기본 CLI 지원·97개 명령 전체율·root 제품/canonical/source375 승격을 의미하지 않는다. 기존 WIP/소유권·DFB0 190/정상2bins·AI 사용자 품질0/9·GUI 시각 미인수와 추론/다운로드/수동GUI/WARP/push 보류를 유지했다.

modoo fixture/mock·postcondition 지침을 재사용해 length/process success와 명시 protocol 종료,CLI 조회 성공과 작업 실패,실제 EOF와 미계측 중간 바이트를 구별했다.
