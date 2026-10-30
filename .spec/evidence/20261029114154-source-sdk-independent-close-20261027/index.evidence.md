---
id: evidence:kinekino/20261029114154
title: probe 대기와 독립적인 원 source 정리 — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# probe 대기와 독립적인 원 source 정리 — 2026-10-27

## 실제 결함과 수정

[원 JobsCore 발행 R9](source-sdk-jobs-publication-20261027.md)의 정리 경로를 확장했다. 기존 core는 probe.poll=Pending이면 즉시 반환해 source.poll_close를 호출하지 않았다. source가 부모 IPC 종료를 기다리면 probe 대기 중 source 정리가 진행되지 않았다.

Rust 시험 worker에 명시 offline hold fixture 경로를 추가했다. 고정16byte와 BodyEnd를 보낸 뒤 원 control pipe의 실제 EOF까지 대기한다. HTTP/DNS/브라우저/모델을 호출하지 않는다. probe는 OS 프로세스가 아닌 **원 sealed-output VerificationLease를 가진 synthetic Pending**이다. 원 JobsApi 취소 뒤 probe를 held로 두고 source가 먼저 정리되는지 관측한다. 실제 source child와 파일 lease/worker host는 원 소유자를 사용한다.

R10/1067에서 정확히 재현됐다: probe cancel 호출 뒤에도 source transport 완료 기록이2초 관측 구간에 생기지 않아0pass/1fail/2.18s였다. catch된 첫 실패를 유지하면서 probe를 release하고 원 retaining shutdown을 마쳤다. 이후 원 source child36144는 exit0/Empty/nonforced로 정리됐지만 **해당 시험 성공으로 바꾸지 않았다**. 외부 test36828 exit101/Empty/nonforced/raw EOF,원 failed verdict/outcome/inner evidence/tmp를 보존했다.

R11 core는 같은 poll에서 source host 정리와 probe cancel/poll을 각각 진행한다. 어느 한쪽 Pending/오류로 다른 쪽 정리를 생략하지 않는다. 다만 원 source session/guards·partial·probe lease는 probe 물리 종료 확인까지 보존하고,양쪽 종료 관측이 닫힌 후 session close/partial cleanup을 한다. 원 cutoff를 단축할 수만 있고 새 예산이나 재실행·재발행 gate를 만들지 않았다.

같은 poll의 최대4개 cleanup ApiError는 첫 run 오류를 유지하면서 별도 구조화된 병렬 오류로 반환한다. SourceOpener/원 host의 raw 오류 보유를 대체하지 않는다. 실제 동시 오류 주입은 아직 미실행이며 구조화된 ApiError 보존만 순수 시험으로 확인했다. panic 때문에 원 owner polling 자체가 불확실해지는 경우는 여전히 별도 미완료 범위다.

## 선택 입력과 실제 게이트

입력 R11 SOURCE.md SHA256 `3399C65E38B54B8D6A8720FB377EE7A1B4125DC84074B054F7742862C35BA4F3`,inventory `1A7B6973A3FD677EBED34960882C63C2E8F77E054EBB2DA271445F65E2F4443F`,2285files. mirror runtime/verification/input-r11에 있다. R9 정리 core/worker/prepare의 변경 전 leaf는 preimage-r9-independent-close,선택 R11 leaf도 preimage-r11로 보존했다. 편집 초기 patch context 불일치는 미적용 상태를 확인한 뒤 정확한 원문으로 재적용했다.

|로그|실제 결과|
|---|---|
|1064/1065/1066 R10|시험 추가 strict/정상 hold worker bin/제품 lib no-run 통과|
|1067 R10|위 probe 대기 결함 재현 실패;원 source snapshot83F706B8·고정 test7FEF08BB/108953600B 보존|
|1068/1069|정리 독립화 strict,구조화 오류 시험 추가 후 strict 통과;선택 전 단계|
|1070 R11|원 session/guard 해제 순서를 유지한 선택 제품 all-target/시험 feature strict 통과|
|1071|제품 lib 및 기존 media_download_owned no-run 통과;기존4 actual 시험 실행 아님|
|1072|원 취소+probe held+source 독립 종료 exact1pass/908filtered/0.15s|
|1073|정상 원 JobsCore 발행 Completed 회귀 exact1pass/908filtered/0.18s|
|1074|종료 관측/병렬 오류 보존 순수4pass/905filtered|
|1075/1076|전체 workspace fmt 통과/선택2285 content·identity 동일|

선택 `native/source-sdk-integration-r1/fixed-r11/sdk-source-tests.exe`: SHA256 `A45ED3B21C142E1749F30758EB714135FC451A2A64AEEC11890A0601E0EFBF4C`,108960768B. compile-only media_download_owned.exe2138A4C667DA294C51768B8E5EC740611524AF6C291E7DD9CF27F1B97C58E89D/3583488B도 보존했다.

실제 child `fixed-r10/source-sdk-fixture-worker.exe`: SHA256 `1E7A9B66CD8736B816B18F4D587624DF097C7527D842618C9F50862C047D8948`,1000448B. R10/1065 producer이며 R11 rebuild로 계상하지 않는다. operator index B3D8C5A52A55CF9A30577A6640CFBC45946175A6B5DB89DD77F887B31C6F4913/394B는 proof=false다. 독립 C2 fixed driver7E7AD258/45s+cleanup5s가 정확한 test/child를 guard했다. live terminal 확인 후 다음 heavy 작업을 시작했고 C-only/offline/locked/j1/conservative2/heavy1을 유지했다.

## 실제 취소와 guard 보존

case `infrastructure::download::process_source::tests::live::jobs::cancel::pending_probe_does_not_block_original_source_cleanup_or_release_partial_early`.

원 source transport receipt1이 probe release 전에 기록됐다. 그 시점 원 job은 Cancelling,최종 output 없음,원 partial 존재,guard drop0이었다. probe release 뒤 같은 opaque cooperative cancellation 경로로 Cancelled/cancel_requested=true/publication_started=false/output=null/failure=null을 기록했고 partial은 제거됐다. 원 host 종료 receipt/worker join 뒤 consume1/transport1/guard drop1을 확인했다.

원 source child36552/creation134375675120308662,exit0/Empty/nonforced. 원 IPC io_reaped/local_endpoints_closed=true/had_error=false,control/data EOF=false·원 raw EOF/final process receipt Recorded를 보존했다. 외부 test29656 exit0/Empty/nonforced/raw EOF/selected_one=true/ownership complete/first_error=null이다. 정상 회귀 외부 test33008도 같은 종료 후조건을 통과했다. raw/outcome/receipt와 inner-observed copy,원 tmp를 각각 `runtime/verification/source-sdk-pending-probe-os1-r11`, `source-sdk-jobs-publish-os1-r11`에 보존했다. 실행 후 고정 test/worker hash와 root lock B2E3D540 동일을 확인했다.

## 미완료/적용 경계

private mirror의 정리 정책 수정이며 root 제품·원 R36·기존 WIP/소유권/source375 private overlay를 유지했다. 실제 OS probe 대기/실패/panic·동시 cleanup 오류·unassigned/high-bit source exit·중간 read/EOF/Abort/timeout·CLI consumer의 실제 인수는 남는다. 위 Cancelled는 실제 coordinator와 원 source child의 관측이지만 synthetic source permission/body/probe를 사용했으며 실제 HTTP 취소/네트워크 ACK나 모델 품질 검증이 아니다. 원 DFB0 190/bins·전체CLI/AI 품질0/9·GUI 시각 인수와 구별한다. 추론/다운로드/수동GUI/WARP/push를 재개하지 않았다.

modoo fixture/mock·postcondition 지침을 재사용해 red→green의 동일 시험/원 프로세스·partial guard·원 취소 권한·첫 오류를 구별하고 incomplete를 성공 ACK로 바꾸지 않았다.
