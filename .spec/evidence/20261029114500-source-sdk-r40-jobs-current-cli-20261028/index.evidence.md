---
id: evidence:kinekino/20261029114500
title: R40 현재 CLI의 durable jobs 재검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R40 현재 CLI의 durable jobs 재검증

## 후속 범위 출력 인수

후속 `source375-custody verify-current`는 동일 input-r40 SOURCE 핀으로 terminal exit0, `current source content/identity unchanged files=2297`을 반환했다. 직접 실행 handle93277의 결과이며 별도 resource-runner receipt가 있는 것으로 주장하지 않는다.

1306 `range_export::fresh_cli_range_job_publishes_verified_bytes_and_operation_specific_idempotency`: exact1 통과/65 filtered, terminal exit0/14035ms. 실제 범위 VTT byte/SHA·원본 보존·operation별 idempotency를 검사했다. 증거 `runtime/verification/source-sdk-r40-range-normal-os1`.

1307 `range_export::fresh_cli_range_job_reports_digest_losses_empty_range_and_no_overwrite`: exact1 통과/65 filtered, terminal exit0/2638ms. 원본 해시 불일치 SourceExpired, 정밀도 손실 NeedsInput 및 명시 allow_loss 성공, 기존 출력 보존 RevisionConflict, 빈 범위·필수 range 누락 거절을 검사했다. 증거 `runtime/verification/source-sdk-r40-range-errors-os1`.

이 두 시험도 동일 R40 jobs producer/CLI와 독립 증거 경로를 사용했고 직렬 실행했다. 총 선택4시험 통과이며 원1303 실패는 별도 보존한다. `jobs.get/submit/list/watch/cancel/retry`의 명시 worker 근거와 `subtitles.convert/export_range` 작업 경로의 근거로 연결한다. 미실행 timeout/IO fault 전체 및 기본 standalone 직접 호출 전수 인수는 남는다.

2026-10-28 중앙 PM. 제품 소스 변경 없이 고정 R40 jobs producer와 정상 CLI를 사용했다. 기존 source375 private overlay·WIP·소유권 및 보류를 유지한다.

- 시험 producer: `runtime/build-cache/native/source-sdk-integration-r1/fixed-r40/exact190-product/jobs-d75793112fc410a6.exe`, SHA256 F5979426A3468C0EA04DA77731D94C51F26BE56E7C67E36647D7290B071981C2.
- CLI: `runtime/build-cache/native/source-sdk-integration-r1/fixed-r40/player-cli.exe`, SHA256 ED1D4899A4825626F25B058CF47BB74856ABB2D54C082021EB0ABAC11DEBBE94. fixture가 동일 해시를 확인하여 별도 증거 경로에 복사해 실행한다.
- 입력: input-r40 SOURCE DB7C9807F9891A80BDF105992E760796D0DA1F71179E63DCE8F296767218E97B. 새 소스 변경·canonical 적용 없음.
- 1303 첫 실패: JOB_CLI_SHA256의 대문자 표기와 fixture의 소문자 문자열 비교 불일치. 기능 실행 이전 assertion 실패이며 원 raw/receipt를 보존했다. 환경값만 소문자로 수정했다.
- 1304: `actual_cli_explicit_host_and_sdk_share_durable_jobs_without_gui` 정확1 통과/65 filtered, runner terminal exit0. 실제 변환·중복/충돌·watch/list·손실/재시도·readonly·중단 복구를 기존 시험으로 재검증했다. 증거 `runtime/verification/source-sdk-r40-jobs-lifecycle-r2-os1`.
- 1305: `cancel::actual_cli_running_cancel_ack_and_publication_winner_keep_real_file_receipts` 정확1 통과/65 filtered, runner terminal exit0. convert/range 각각 발행 예약 전·후 취소 네 경우를 검사한다. 예약 후를 파일 저장 후로 해석하지 않는다. 증거 `runtime/verification/source-sdk-r40-jobs-cancel-os1`.

C2 total60000/cleanup5000, conservative2, CPU2, BelowNormal로 직렬 실행했다. 명시 subtitle worker가 있는 host의 실제 로컬 기능 근거이며 기본 CLI 전체 지원·모델 추론·실제 HTTP·GUI 인수가 아니다. 새 제품 빌드나 전체190 재실행으로 계상하지 않는다. fixture/mock skill의 통제 gate와 실제 파일 검증 구분을 적용했다(출처 `modootoday/software-qa/test-fixture-and-mock-build`, SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100).
