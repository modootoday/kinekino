---
id: plan:kinekino/20261029113509
title: owned-process r2 생산자 gate 실행 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# owned-process r2 생산자 gate 실행 준비

최신 상태: 실제 gate에서 r2 pure8/Windows8이 통과했고, 최소 lint 수리 r3/r4를 분리 보존한 뒤 r4 default/support strict·fmt 및 영향 Windows exact1을 재검증했다. exact1은 기존8과 중복이므로 고유16이다. 등록 root default/native strict도 추가 인수하고 최종 보호 차이0 후 heavy/shared를 PM에 반환했다. [실행 정본](../../sots/verification/runtime/owned-process-producer-2026-10-21.md)을 참조한다. 아래 준비 문장은 당시 입력이며 현재 root 채택이나 helper 이관 완료를 뜻하지 않는다.

현재는 준비만 완료했다. source19 r2 pin `3F31B613725CA797B722FAC54F9F68C4F603AD7F0C5C47C841A112873AE511A8`, pure8/명시 Windows8 작성·실행0이다. root `35d257c7d9db1d055afd6f2341b84df74127c956`와 미적용 registration before를 대조했다. root Cargo SHA `BF81B5BED5E8BA1DF2A39CE9B2ECF1CDEE4C0B20CACDF72149A90E17CCD2E473`, lock SHA `7CC98A9406B50CD03BA69B4FE7B95914DE1F5A1771985A4831EA24918C567B38`는 원 등록 제안 당시와 같다. git apply --check만 수행해 patch 적용 가능성을 확인했고 실제 적용·Cargo·shared·Job 실행0이다.

## 입력과 lock

SDK heavy 반환 후 PM이 격리 snapshot/최종 base를 배정한다. 최신 root Cargo/lock, source19/신규19 부재, 보호 목록과 runner EEA9/기존 target을 다시 고정한다. 현재 patch는 workspace members/default-members 및 root dev-dependency만 제안하며 source19 manifest를 변경하지 않는다. root live/index에 적용하지 않는다.

원 lock을 보존한 뒤 등록된 격리 snapshot에서 첫 default compile 한 번만 --locked 없이 `cargo check --offline -j 1 -p owned-process --all-targets`로 수행한다. 기존 lock을 우선 사용해 local owned-process 및 root dev-dependency edge를 추가하도록 하고 before/after diff를 인수한다. 전체 generate-lockfile로 registry 버전을 다시 선택하지 않는다. registry package/version/source/checksum의 뜻밖 변경이나 새 다운로드 필요가 나오면 원 입력/로그를 보존하고 중단한다. 승인된 lock 이후 모든 명령은 --locked다. offline lock 생성 자체가 아직 실행됐다고 기록하지 않는다.

## 고정 명령과 실제 선택 기준

모든 실행은 EEA9/conservative2/offline/-j1/RUST_TEST_THREADS1 및 배정된 기존 cache를 사용한다. 이전 명령의 원 handle terminal을 회수한 뒤 다음 명령으로 진행하며 최초 실패에서 멈춘다.

```text
cargo check --offline --locked -j 1 -p owned-process --all-targets --features test-support
cargo test --offline --locked -j 1 -p owned-process --test state_machine -- --test-threads=1
cargo test --offline --locked -j 1 -p owned-process --test windows_family --features test-support -- --list
cargo test --offline --locked -j 1 -p owned-process --test windows_family --features test-support -- --ignored --test-threads=1
cargo clippy --offline --locked -j 1 -p owned-process --all-targets -- -D warnings
cargo clippy --offline --locked -j 1 -p owned-process --all-targets --features test-support -- -D warnings
cargo fmt --check -p owned-process
```

pure 회귀는 integration target `state_machine`의8개다. --lib로 바꿔0selected를 성공 처리하지 않는다. Windows 목록 명령은 시험 실행0이며 인벤토리만 확인한다. Windows 실제 --ignored 실행에서 아래8개 이름·통과/실패/무시/제외를 원 로그로 확인한다. 기본 실행에서 ignored8 표시는 pass8이 아니다.

1. stdin_writer_is_not_inherited_and_eof_completes_without_force
2. parent_exit_and_inherited_stdout_do_not_hide_live_family_or_kill_outside_sentinel
3. partial_startup_failures_keep_original_handles_and_never_run_child_code
4. failed_assignment_and_failed_termination_remain_unconfirmed_until_drop_fallback_is_observed
5. family_query_failure_returns_original_owner_with_unknown_receipt
6. incomplete_ready_line_times_out_without_waiting_for_pipe_eof
7. arguments_roundtrip_without_shell_and_nul_is_rejected_before_create
8. successful_resume_returning_at_cutoff_is_an_owned_launch_failure

pure 목록은 deadline 비리셋/request 예약, parent exit와 family 구분, 원/secondary 오류와 owner, raw 독립 offset/identity, record budget 만료, cleanup reserve 소진, terminal receipt 오류, late sink Ok의8개다. phase별 실제 pass를 합산하고 목록 조회·중복 실행은 더하지 않는다.

## nested Job와 outside sentinel의 실제 소유

Windows parent-exit 회귀는 동일 Rust test runner 안에서 sentinel과 대상 family를 각각 `spawn`해 별개의 private FamilyJob/원 process+thread handle/creation identity를 보유한다. 외부 EEA9 budget Job을 상속받은 사실을 own Job 할당 전 IsProcessInJob로 관측한 값이 true인지 단언한다. 부모 exit0 뒤에도 own family Active임을 실제 query하고 대상 Job만 종료한 후 Empty를 단언한다. 그 뒤 sentinel의 같은 identity와 미종료를 확인한다. sentinel도 자신의 Supervisor::finish로 정리한다. 외부 Job 종료나 PID 검색/재오픈/사용자 앱 정리는 없다.

unassigned 실패 회귀의 ExitProbe는 원 process handle의 private non-inheritable duplicate이며 임의 PID를 받아 만들지 않는다. 원 실패 owner Drop 전 종료 미확인과 Drop fallback 후 원 handle signaled를 구분한다. 부분 startup·query·late admission의 원 owner/error는 각각 별도 단언이다. 이 동작은 실제 gate 전에는 검증됐다고 하지 않는다.

현재 시험은 성공 시 temp raw directory를 제거하고 assertion 결과를 test log에 남긴다. 따라서 실제8 통과를 인수해도 모든 내부 child PID/creation/개별 raw stream의 영구 원시 이력을 저장했다고 확대하지 않는다. 외부 runner receipt/PID와 시험 내부의 실제 handle·family assertion을 구분한다. 더 상세한 지속 evidence가 필요하면 별도 명시 범위로 검토하며 이번 준비에서 source19를 변경하지 않는다.

## 마감과 다음 단계

source19/원 r1, registration·lock delta, actual fixture binary/runner SHA·bytes, 원 handle/receipt·기록PID, 보호 범위를 대조한다. 실패 원 로그와 미완료 owner 결과를 보존하고 수정은 새 pin으로만 한다. 이번 gate에는 helper/jobs lease/동일 endpoint 실제3category/미디어/브라우저/AI가 포함되지 않는다. 생산자 인수 뒤 [소비 배선](owned-process-consumer-wiring.md)을 별도 배정한다.
