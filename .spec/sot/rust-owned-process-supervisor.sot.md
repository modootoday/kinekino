---
id: sot:kinekino/rust-owned-process-supervisor
title: 중립 Rust owned-process supervisor 제안
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 중립 Rust owned-process supervisor 제안

현재 이 root 문서가 기능 정본이다. Meari checkout의 원 제안2문서는 이력으로 보존하며 renderer 계약과 분리한다. 2026-10-21 PM source-only 배정에 따라 독립 staging 구현을 작성했으며 아래 설계와 마지막 구현 절을 함께 적용한다. 지원 완료 또는 실행 검증 완료 상태가 아니다.

2026-10-21 PM 배정에 따른 읽기·설계 제안이다. 관측 root는 `d0d185faf09b56120ce5cf960a95fdcfa04f6029`다. 제품·라이브러리 코드 작성, Cargo, process/Job/미디어/브라우저/AI 실행은0이다. 이 공통 책임을 Meari renderer나 player domain에 넣지 않는다.

## 확인한 현재 경계

- tests/jobs/playback_owner.rs: ready의 blocking read_line, 종료 후 read_to_string, Drop의 wait/read_to_string은 deadline 밖에서 무기한 대기 가능하다. 부모 exit는 stdout을 상속한 자손의 EOF를 보장하지 않는다.
- tests/jobs/fixture.rs: call_root의 write_all/wait_with_output 및 Host ready/stop/Drop도 동기 pipe/wait에 의존한다. 현재 자막8 lease 파일이므로 편집하지 않는다.
- tests/support/snapshot_cli_evidence.rs: 원 request/raw evidence·prefix 한도·절대 timeout·worker 결과/미완료·직접 child kill의 정직한 기록은 재사용한다. 하지만 read worker의 read_to_end, unresolved_detached 및 Drop cleanup은 family 완료 계약이 아니다. 제품 Request/PNG/hash 정책은 공통 crate로 이동하지 않는다.
- tools/resource-runner/src/windows_budget.rs: 자신을 Job에 포함하고 affinity/priority만 설정한다. no kill-on-close이며 외부 예산 Job 종료를 소유 family 정리 수단으로 삼을 수 없다.
- tools/resource-runner/src/application/child_execution.rs: original error/cleanup/receipt error 구분은 재사용하되 blocking wait와 Drop cleanup은 새 bounded supervisor에 그대로 복사하지 않는다. resource-runner receipt의 terminal은 child 관측이지 family active0를 뜻하지 않는다.

## 정확한 패키지 경계

제안 패키지는 상위 저장소 `packages/owned-process`(crate owned-process, publish=false) 하나다. 독립 Git 라이브러리 Meari의 dependency로 넣지 않는다. 중립 cross-platform 계약과 Windows adapter를 한 패키지의 계층으로 분리하고 새 async runtime은 도입하지 않는다. 최초 구현의 family 보증은 Windows만 제공하며 다른 플랫폼은 명시 Unsupported다.

예상 신규 경로:

```text
packages/owned-process/Cargo.toml
packages/owned-process/src/lib.rs
packages/owned-process/src/domain/{mod,deadline,identity,observation}.rs
packages/owned-process/src/application/{mod,supervisor,raw_records}.rs
packages/owned-process/src/infrastructure/{mod,evidence_files}.rs
packages/owned-process/src/infrastructure/windows/{mod,launch,job,identity,handles}.rs
packages/owned-process/src/bin/owned-process-test-child.rs
packages/owned-process/tests/{state_machine,windows_family}.rs
```

domain은 deadline/상태/관측값만, application은 제한된 poll 상태 전이, infrastructure는 raw 파일과 Win32 handle만 담당한다. JSON event의 의미, mpv/native/provider/joined 성공 정책, CLI 요청 schema는 소비자에 남는다.

Cargo 변경은 추후 명시 배정한다. root workspace member와 root dev-dependency에 path crate를 추가하고 root lock에 새 local package를 등록한다. Windows adapter는 기존 고정 windows=0.62 계열을 우선 사용하며 필요한 Win32_System_JobObjects/Threading/IO/Security/Foundation API feature만 기존 cache와 대조해 고정한다. 새 버전·다운로드·runtime 도입은0이다. 별도 workspace인 tools/resource-runner는 공통화 단계에서만 path dependency 및 자체 lock을 갱신한다. 기존 EEA9 실행 파일/입력/실행 예산·receipt schema를 첫 supervisor 구현과 함께 임의 변경하지 않는다.

## launch와 소유권

1. 원 executable 절대 경로/identity와 cwd/env/argv, raw stdout/stderr 및 request stdin 파일을 spawn 전에 고정한다. 파일은 create_new이며 실패 시 child0이다. 프로그램은 shell/PATH 탐색으로 실행하지 않는다.
2. Windows adapter는 CreateProcessW의 명시 executable과 올바른 argv quoting, CREATE_SUSPENDED 및 숨김 창, STARTUPINFOEX handle allowlist를 사용한다. stdin/stdout/stderr만 상속하고 Job/process/thread/supervisor handle은 상속하지 않는다. std::process::Child는 suspended primary-thread 제어를 억지로 우회하지 않고 전용 OwnedProcess가 process+thread handle을 소유한다.
3. 전용 unnamed child-family Job을 만들고 KILL_ON_JOB_CLOSE·breakaway 비허용을 readback한다. supervisor 자신이나 외부 runner Job을 할당/종료하지 않는다. suspended child를 이 Job에 할당하고 IsProcessInJob(child,owned_job), GetProcessTimes creation identity 및 제약을 확인한 뒤 primary thread를 resume한다. nested Job을 구성할 수 없으면 fail closed하며 unsupervised 재시도0이다.
4. resume 전 할당 실패는 원 suspended child handle로만 terminate하고 bounded exit를 관측한다. resume 후 정리는 소유한 Job만 대상으로 한다. child/Job owner와 실패 stage/Win32 error/raw 기록을 함께 반환한다. pid/executable 검색 후 kill하는 API는 없다.

Job은 resource-runner의 예산을 상속받으며 CPU/메모리 정책을 다시 구현하지 않는다. inherited outer Job이 있다는 사실과 own inner Job membership은 각각 관측한다. nested Job/상속/실제 readback 성공 전에는 descendant containment를 주장하지 않는다. 프로세스가 기존 unrelated OS broker에 작업을 맡기는 경우까지 own Job의 자손이라고 주장하지 않는다.

## bounded I/O와 절대 deadline

정확한 보장 범위는 byte/memory 한도 및 poll/deadline 대기다. finite regular-file read/metadata/write와 동기 Win32 호출은 wall-clock bounded가 아니다. CloseHandle도 엄밀한 즉시 반환을 보장하지 않는다. 이번 범위에 hard filesystem deadline용 overlapped I/O나 새 runtime을 추가하지 않았다.

stdout/stderr는 처음부터 raw 파일에 연결한다. supervisor는 regular file의 유한 prefix를 offset 기준으로 읽으며 pipe EOF를 기다리지 않는다. ready는 bounded NDJSON line, terminal은 ready 이후 offset에서 bounded record를 찾는다. timeout/불완전 UTF8/길이초과/잘못된JSON은 raw bytes와 parse stage를 남긴다. 전체 raw와 보존 prefix·parse 성공을 구분하고 파일 크기 관측·설정 cap 초과는 own Job 종료 사유로만 사용한다. polling 사이의 disk 증가까지 강제 quota로 보장한다고 표현하지 않는다.

one-shot CLI는 이미 저장한 bounded request 파일을 stdin으로 사용해 blocking write worker를 없앤다. 장기 host는 supervisor가 쓰지 않는 stdin pipe의 write handle만 유지하고 종료 시 EOF를 전달한다. future interactive stdin이 필요하면 별도 bounded cancellable write adapter가 선행돼야 하며 write_all로 확대하지 않는다.

기준 Instant deadline은 최초 spawn 전 생성한다. ready/요청/종료/정리 단계가 시작될 때 시간을 리셋하지 않는다. 각 poll/endpoint request에는 남은 budget과 해당 단계 cap의 최솟값만 제공한다. cleanup 관측 시간을 전체 deadline 안에 예약하고 operation cutoff를 앞당긴다. deadline overflow/잔여0은 실패다. endpoint TransportOptions/RequestContext의 timeout 전달은 소비자 책임이고 새 네트워크 runtime은 만들지 않는다.

graceful EOF → 제한된 child/family poll → operation cutoff에서 own Job terminate → 전체 deadline까지 bounded poll 순서다. child process signaled/exit code와 own Job의 ActiveProcesses=0 readback을 독립적으로 기록한다. terminate 호출 성공·child exit·raw terminal JSON은 각각 family empty와 다르다. Job read 실패 또는 active>0은 Unknown/Incomplete이며 성공을 만들지 않는다. PID list query는 own Job diagnostics만, 종료 권한은 opaque owned Job handle뿐이다.

## 실패 반환과 Drop

UnassignedSuspended는 전용 Job의 membership이 확인되지 않은 별도 상태다. CreateProcess 성공 후 assign/terminate 실패에서는 kill-on-close로 그 child를 회수할 수 있다고 주장하지 않는다. 원 process/thread handle과 creation identity, 원 실패 및 종료 미확인을 반환한다. 이 상태의 Drop은 원 process handle에만 nonwaiting best-effort TerminateProcess를 한 번 요청하며 성공·종료를 발명하지 않는다. AssignedSuspended/Running은 소유 Job의 kill-on-close containment와 별도 terminal 관측을 구분한다.

launch 이전 실패는 child owner가 없다. spawn 이후 실패는 원 process/Job owner를 가진 Incomplete로 반환한다. wait/query/parse/receipt-write의 원 오류와 secondary cleanup/receipt 오류를 분리한다. 명시 finish는 completion receipt 또는 owner-retaining failure를 반환하고, timeout 뒤에도 살아 있는 owner를 자동 drop하여 성공으로 바꾸지 않는다.

Drop에서는 wait/read/join/sleep 루프를 실행하지 않는다. 남은 소유 Job handle의 kill-on-close는 마지막 containment 수단이며 종료 관측이나 정상 shutdown receipt가 아니다. 원 raw/이미 기록된 receipt는 삭제하지 않는다. cleanup 결과를 반드시 보존해야 하는 호출자는 must_use 결과와 명시 closeout을 소비한다. Drop 이후 완료 관측을 약속하지 않는다.

## source proposal의 최소 형태

```rust
// 제안 서명이며 아직 존재하거나 검증된 공개 API가 아니다.
pub enum FamilyObservation { Empty, Active(u32), Unknown }
pub struct Completion { pub child_exit: i32, pub family: FamilyObservation /* raw receipt */ }
pub struct Incomplete { /* original OwnedProcess, OwnedFamilyJob, cause, raw evidence */ }
pub enum FinishOutcome { Complete(Completion), Incomplete(Box<Incomplete>) }
// OwnedProcess::spawn_suspended_assigned(...) -> owner-retaining launch result
// OwnedProcess::poll_ready(deadline, prefix_limit) -> bounded raw record observation
// OwnedProcess::finish(deadline) -> FinishOutcome
// No public kill(pid), attach_existing(pid), or arbitrary Job handle constructor.
```

최종 구현에서 Complete의 family는 반드시 Empty여야 한다. 중립 supervisor는 raw record를 내보내고 playback_host_ready/stopped, engine hash, same endpoint, SDK/pipe/dispatch joined 판정은 소비자 helper가 수행한다. 서비스 semantic success와 forced family cleanup 성공은 서로 다른 필드다.

## 공식 근거

[Nested Jobs](https://learn.microsoft.com/en-us/windows/win32/procthread/nested-jobs)는 nested hierarchy와 상속/제약 실패 조건을 설명한다. [Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects)는 breakaway 및 Job 범위 종료를 설명한다. [CreateProcessW](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw)는 process/thread handle 및 명시 executable/argv 계약의 근거다. 이 문서 확인은 실제 adapter 동작 검증과 구분한다.

## source-only r1 구현 계약

staging은 `runtime/coordination/owned-process-r1/source/packages/owned-process`의 신규19파일이다. 실제 root packages/owned-process나 root Cargo/lock에 적용하지 않았다. 공통 core는 std만 사용하고 Windows target만 기존 고정 windows=0.62.2를 사용한다. test-support는 기본 비활성이며 Rust 자체 child와 fault seam만 제공한다. JSON 의미는 crate가 파싱하지 않는다.

`PreparedEvidence::create`는 raw 파일과 선택적인 finite request 파일을 spawn 전에 create_new한다. `RawFile`은 원 volume/file index(Unix pure fixture에서는 dev/inode)를 보관하고 독립 File::open 후 identity를 재검증한다. child stdout/stderr용 inheritable duplicate와 supervisor read offset은 공유하지 않는다. 동일 파일 안의 악의적인 외부 재작성이나 신뢰하지 않는 reparse root의 confinement까지 인증하지 않는다. caller가 소유한 trusted evidence directory가 입력 조건이다.

Windows spawn은 명시 absolute executable/cwd, UTF-16 NUL/길이 검증 및 writable command, cb/16-byte attribute backing lifetime/상속 handle allowlist를 구성한다. CREATE_SUSPENDED/NO_WINDOW와 bInheritHandles=true를 사용한다. control stdin write, 원 read, Job/process/thread handle은 상속하지 않는다. explicit environment는 현재 ASCII 변수명과 Unicode 값만 지원하며 None은 기존 환경 상속이다. 이미지 경로와 process creation time을 원 handle에서 readback한다. ResumeThread는 이전 suspend count1일 때만 Running으로 승격한다.

`WindowsFamily`에 spawn 전 Deadline을 보존하고 `Supervisor`가 최초 cutoff를 고정한다. 이후 wait/finish에 새 Deadline을 넘겨도 constrained_by로 단축만 허용해 cleanup 예약을 갱신하지 않는다. ready는 `RecordCursor`의 bounded raw line/offset이고 terminal의 개행 없는 마지막 record는 child/family 관측 뒤 caller가 요청한다. record는 JSON/UTF8 성공을 뜻하지 않는다.

`ProcessFamily`는 원 handle 관측 adapter 계약, `ReceiptSink`는 process 관측 persistence 포트다. `Supervisor::finish`의 Complete는 child exit와 family Empty 관측 및 sink 성공을 요구한다. forced 필드는 정상 service 종료와 구분한다. 오류·deadline·raw cap·sink 실패는 must_use Incomplete에 원 Supervisor/cause/termination_error/receipt_error를 반환한다. query 실패 시 family를 Unknown으로 남긴다. 남은 원 owner의 poll_state로 후속 cleanup을 관측할 수 있지만 새 receipt를 이전 deadline 내 semantic 성공으로 바꾸지 않는다. LaunchFailure는 생성 전 ownerNone과 생성 후 원 WindowsFamily 보존을 구분한다. 소비자는 반환된 원 오류와 raw evidence를 기존 wrapper의 실패 receipt에 남겨야 한다.

작성 회귀는 pure7 및 명시 실행 전용 Windows7이다. 후자는 EOF/비상속 writer, 부모 exit+상속 stdout+active family, own Job 밖 별도 소유 sentinel, 부분 attribute/create/assign/resume 실패, assign+terminate 미확인 후 Drop fallback 관측, query Unknown, 불완전 ready deadline, 실제 Unicode/quote argv 왕복과 NUL 사전 거부를 포함한다. 모든 Windows 회귀는 ignore/feature gate이며 실제 OS 실행은0이다. 직접 rustfmt/check 외 Cargo/typecheck/clippy/실행은 미수행이다.

## r2 감사 보완

r1 독립 읽기 감사에서 확정 UB/owner 손실/허위 family Empty 반례는 확인되지 않았으며 늦은 성공 판정2건을 보완했다. r2 source19 pin은 `3F31B613725CA797B722FAC54F9F68C4F603AD7F0C5C47C841A112873AE511A8`, 수정4경로다. sink Ok 뒤 현재 clock을 재검사해 전체 deadline 이후라면 이미 관측한 exit/Empty 및 sink 성공 사실과 원 owner를 보존하는 Deadline 실패로 반환한다. 정상 ResumeThread count1 뒤에도 operation cutoff를 재검사해 늦은 launch 성공을 거부한다. Running은 thread resume의 사실이며 launch admission 성공을 대체하지 않는다.

추가된 fake late-sink 및 post-resume fault 회귀로 작성 수는 pure8/명시 Windows8이다. source-only이며 실행 통과0이다. [r2 인계](../../../runtime/coordination/owned-process-r1/revision-r2/handoff.md)에 원 r1 보존·delta·검증 한계를 기록했다. 소비 배선 호환성 미결정은 그대로 유지하고 이번 판정 수리와 섞지 않았다.
