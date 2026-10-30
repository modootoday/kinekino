---
id: evidence:kinekino/20261029114226
title: durable jobs 실제 SDK/CLI — DATALAB 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# durable jobs 실제 SDK/CLI — DATALAB 2026-10-27

기존190은 jobs의 관측/정리 정책6개만 실행한다. 등록 목록에 있다는 이유로 실제 종단 성공을 계상했던 추정을 정정하고, 고정 SOURCE28B8A1E9/2244에서 별도 실제 시험2개를 실행했다. 소스 변경·새 기능 구현·다운로드/추론/수동GUI/WARP/push 없음. 원 WIP·소유권·private overlay·원 증거 보존.

## producer와 입력

- 입력 `runtime/verification/source375-store-discovery-input-r1/SOURCE.md` SHA256=28B8A1E922C867B25BB5ED08E489A057B13F23306131866BB6BEA04F668E7DBC, inventory=C2E18E1B936CC2EC9CBF651BC7FCB08A93C990326A02B6C7C52EA39F5F08D522,2244개.
- 773 `cargo test --locked --offline -j1 -p kinekino-player --features native-proposal-window --test jobs --no-run --message-format=json` exit0. 원 Cargo JSON의 jobs producer `runtime/build-cache/native/debug/deps/jobs-9cc89d02f699ca54.exe`를 사용했다.
- 명시 JOB_CLI_EXE는 `runtime/build-cache/native/debug/player-cli.exe`, JOB_CLI_SHA256=1DF8ADD0F6210B4E7C38CEC46943D0EA6647530086902DCB752752D4C735378D. fixture는 원본/복사 CLI hash를 대조해 `cli-sha256.txt`에 보존했다. JOB_EVIDENCE_DIR는 `runtime/verification/durable-jobs-actual-r1`이다.
- 로그는 `runtime/verification/source375-datalab-b0-r1-logs/773~776-durable-jobs-*`. source/toolchain/cache는 기존 C-only 격리 정책을 유지하고 resource-runner conservative2/BelowNormal/무거운 작업1로 실행했다. 제품 코드 변경이 없으므로 전체strict/fmt/190/정상bins를 불필요하게 반복하지 않고 각 기존 입력의 실제 근거로 유지한다.

## 실제 결과

취소 경계 표현 정정: cancel-after의 gate는 실제 파일 저장 완료 뒤가 아니라 reserve_publication 성공 뒤·파일 생성 전이다. 요청 응답은 publication_in_progress/running/output=null이며 gate 해제 후 Completed와 실제 출력 SHA를 확인한다. cancel-before는 예약 전 requested/cancelling 뒤 Cancelled다. 아래 ‘저장 전/후’는 부정확한 초기 표현이며 실제 관측 범위는 이 정정을 따른다. 실제 완료 뒤 취소는 lifecycle의 cancel-completed=already_terminal이라는 별도 근거다.

776 전후 동일 검증은 exit0/42381ms로 종료했다. 전체2244 입력 content/identity 동일을 확인했으며 아래 진행 중 문장은 종료 전 이력이다.

774 `actual_cli_explicit_host_and_sdk_share_durable_jobs_without_gui`는 1/1·실제CLI28회 exit0/3287ms. lifecycle 디렉터리에 source/output·durable stores·각 요청/stdout/stderr/exit와 host ready/stop 기록을 보존했다.

| 검증 범위 | 실제 결과 |
|---|---|
| 기본 vs read-only vs 실행 host | 기본 jobs Unsupported, rooted reads 가능, host의6 schema supported; rooted submit 거절 |
| 변환·중복 | 실제 SRT→VTT 완료, cue/한국어·일본어/원 source 보존, 동일 key 중복은 같은 job/created=false, 변경 요청은 idempotency conflict |
| 취소·이벤트·목록 | 완료 job cancel=already_terminal, 잘못된 attempt 충돌, watch2개→next_sequence로 나머지3/gap=false, list 성공 |
| 손실 변환·재시도 | VTT→SRT는 기본 NeedsInput/출력 없음, 명시 retry attempt2도 원 입력 동일/NeedsInput, stale retry 거절, allow_loss 명시 요청만 완료 |
| 저장·재시작 | SDK/read-only fresh CLI가 같은 output 확인, 강제 종료 원 host를 재시작하면 attempt1 Interrupted·자동 재실행 없음·기존 출력 존재 여부 유지 |
| 정상 host 종료 | normal/restart stop 모두 dispatch completed·pipe joined·sdk joined. normal completed_total2, restart interrupted_total1 |

775 `cancel::actual_cli_running_cancel_ack_and_publication_winner_keep_real_file_receipts`는1/1·실제CLI6회 exit0/2754ms. cancel-before/cancel-after에 각각3회 요청을 보존했다. 원 subtitle executor와 실제 파일/실제 IPC를 사용하며 제어된 Rust gate로 저장 전/후 취소 순서를 고정했다. 저장 전 Cancelled/output없음·원 source 그대로, 저장 후 Completed/원 output SHA일치·원 source 그대로를 확인했다. 실제 네트워크·임의 executor의 취소·모델/GUI 동작으로 확대하지 않는다.

776 전체 입력 전후 content/identity 검증은 진행 중이다. 이번 두 시험에서 첫 실패는 없으며 이전 조합 discovery의 첫 실패765와190 배정 첫 실패743은 원 기록으로 보존한다. modoo.today test-fixture-and-mock-build 본문 A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 실제/합성·통제 gate/실제 파일 경계를 적용했다. 전체 job operation·모든 오류/IO fault·실제 inference/download executor·전체CLI/AI 품질/GUI 인수는 남는다.
