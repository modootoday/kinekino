---
id: evidence:kinekino/20261029114232
title: 정상 CLI의 검증 단계 취소·원 부분 파일 보존 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 정상 CLI의 검증 단계 취소·원 부분 파일 보존 — 2026-10-27

## 검증 범위

[정상 CLI/EOF R15](source-sdk-cli-eof-20261027.md)를 실제 `jobs.cancel` 소비로 확장했다. source는16byte/BodyEnd 뒤 원 control EOF까지 기다리고,synthetic probe는 원 sealed VerificationLease를 보유한 Pending 상태다. 취소는 BodyEnd 이후 **검증 단계**에서 시작한다. HTTP 전송 중 취소·Ctrl-C·실제 OS probe의 종료 시험은 아니다.

기존 R11의 독립 정리·opaque cooperative cancellation·원 JobsCore/CAS/파일 정책을 그대로 사용했다. 변경은 private 시험 helper와 Gate의 시험 모듈 가시성뿐이다. 변경 전 CLI/Gate leaf는 mirror runtime/verification/preimage-r15-cli-cancel에 보존했다.

## 선택 입력과 게이트

input-r16 SOURCE.md SHA256 `89E8C5E06F7CA14EBF60AA96EF7D2E6B203AC1A7830B2DBA1D03C7DA7228991B`,inventory `D408364C83D61C44FAA056BEE52AD311D3A4377A4EFF24052C4028DCC622E8F3`,2286files.

|로그|실제 결과|
|---|---|
|1107/1108|제품 all-target/시험 feature strict 및 lib no-run 통과|
|1109|실제 CLI 검증 단계 취소 exact1pass/923filtered/0.43s|
|1110|정상 CLI Completed 회귀 exact1pass/923filtered/0.40s|
|1111/1112|전체 workspace fmt 통과/선택2286 content·identity 동일|

첫 실행 실패는 없었다. 원 R14/R15 등의 CLI 회수는 해당 입력의 근거로 유지하며 이번에 새로 실행한12회만 계상한다.

고정 `native/source-sdk-integration-r1/fixed-r16/sdk-source-tests.exe`: SHA256 `ABFF8205015A1EE7D02C296E1AD43700CC522724A6A42A64A5BEB4380F03291B`,109197824B. source worker는 원 R15 fixed477ED83B/1000960B,정상 CLI는 원 R14 fixed249DD5EC/36405248B·profile.test=false다. 두 BIN을 R16 재빌드로 계상하지 않는다. 원 C2 fixed driver7E7AD258/45s+cleanup5s가 test/worker를 보유하고 CLI는 원 deny-write/delete read handle의 SHA/size로 고정했다. operator index372844FDD281ACD072B925AD14D989A011BB93E032C57443FA724BE045298B51/415B는 proof=false다.

C-only/locked/offline/j1/conservative2/heavy1과 원30초/cleanup5초 요청 예산을 유지했다. live terminal 뒤에 다음 heavy 작업을 시작했다. 실행 후 test/worker/CLI hash와 root lock B2E3D540 동일도 확인했다.

## 실제 CLI12회

정확한 취소 case `infrastructure::download::process_source::tests::live::jobs::cli::normal_cli_cancel_retains_partial_until_probe_close_and_records_terminal_cancelled`.

capabilities/schema/jobs.submit/jobs.cancel/대기 jobs.get/최종 jobs.get/jobs.watch의7회 CLI를 실행했다. cancel 응답 후 probe는 held이고 원 source는 이미 실제 종료/transport receipt1이었다. 대기 조회는 Cancelling,최종 파일 없음,원 partial1개/길이16,외부 write open은 원 sealed lease 때문에 Windows sharing violation32,guard drop0을 확인했다. 취소 요청 성공을 정리 완료로 취급하지 않았다.

probe release 뒤 최종 조회는 Cancelled/cancel_requested=true/publication_started=false/output=null/failure=null이었다. jobs.watch에는 Cancelling/Cancelled가 있고 Completed는 없었다. partial은 원 정리 경로에서 제거됐다. 원 local host pipe/sdk Joined 및 retaining job host 종료 뒤 consume1/transport1/guard drop1을 확인했다. 정상 회귀도 별도5회 CLI로 Completed/실제16byte 발행을 유지했다.

취소 source child760/creation134375709290944546,exit0/Empty/nonforced,원 final receipt/raw EOF/IPC reap/local close·had_error=false였다. parent control/data EOF=false는 그대로 보존했다. 외부 취소 test14364와 정상 회귀31116은 모두 exit0/Empty/nonforced/raw EOF/selected_one=true/ownership complete/first_error=null이었다. 각 CLI 응답 request ID/예상 exit/raw complete/원 deadline도 검사했다.

원 request/response/rpc-count/local-close/source receipt 및 outer outcome/raw를 `runtime/verification/source-sdk-cli-cancel-os1-r16`, `source-sdk-cli-normal-os1-r16`에 보존했다. inner-observed copy와 원 C tmp도 유지했다. 대기 partial의 크기/쓰기 거절은 고정 Rust 시험의 실제 assertion이며 완료 뒤 존재하지 않는 파일을 과거 관측으로 대신하지 않았다.

## 인수 경계

source permission/body/대기 probe는 fixture이고 실제 browser issuer/HTTP·OS probe·사용자 품질 승인을 뜻하지 않는다. 실제 CLI 데이터 전송 중 취소·강제 종료/재시작·unassigned rollback·동시 오류/panic 및 전체 명령 인수는 남는다. 원 제품/canonical/source375 overlay·WIP/소유권·DFB0 190/정상2bins·AI 사용자 품질0/9·GUI 시각 미인수와 기존 추론/다운로드/수동GUI/WARP/push 보류를 유지했다.

modoo fixture/mock·postcondition 지침을 재사용해 요청 수락/Cancelling/물리 source 정리/lease 보존/최종 Cancelled 각각의 실제 증거를 분리했다.
