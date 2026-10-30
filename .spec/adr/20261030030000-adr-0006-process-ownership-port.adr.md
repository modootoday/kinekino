---
id: adr:kinekino/20261030030000
title: "ADR 0006: 프로세스 패밀리 소유 — core 중앙 포트, Windows Job Object·Unix 프로세스 그룹 어댑터, 능력표"
status: active
created: 2026-10-30
updated: 2026-10-30
rel:
  governs: [crates]
---

# ADR 0006: 프로세스 패밀리 소유 중앙 포트

## Context
- `kinekino-process`는 Windows 전용 `infrastructure::windows`(Job Object, 명명 파이프, overlapped I/O)에만 구현이 있었다. 비Windows 빌드는 lib 전체에 `allow(dead_code, ...)`를 걸어야 했고, 상위 크레이트(player, source-host, vad-executor)가 OS 게이트에 묶여 있었다.
- 사용자 원칙: Cross-OS, 중앙 인터페이스 필수, OS별 분리. 기술 선택은 업계 표준과 리서치 근거로 한다.

## Decision
1. **중앙 포트**: `kinekino_core::platform::process`에 둔다.
   - 포트: `ProcessOwnership`(OsPolicy::process()), `OwnedFamily`, `PreparedFamily`, `ExitProbe`, `SourcePipe`/`PipeOperation`/`SourceChildEndpoint`.
   - 값 타입: `Deadline`, `ProcessIdentity`, `FamilyObservation`, `SourceCutoffs`, `FamilyBudget`, `MemoryPeaks`, `LaunchState`, `FaultPoint`.
   - 구현은 `kinekino-platform-windows::process::ownership`과 `kinekino-platform-unix::process::ownership`에 둔다. `kinekino-platform::CurrentOs`가 단일 cfg로 고른다.
   - `kinekino-process`의 `infrastructure::family`(스폰 파사드)와 `infrastructure::source_channel`(레인 상태기계)은 OS 중립이다. `kinekino-process`에는 OS cfg와 `windows` 의존이 없다. 예외는 Windows 계약 테스트용 dev-dependency로, 6번째 묶음에서 어댑터로 옮긴다.
2. **Windows**: 기존 동작을 그대로 유지한다.
   - `CREATE_SUSPENDED` 후 Job Object에 할당하고, `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`와 메모리·친화도 제한을 건다. 할당·검증이 끝난 뒤 재개한다.
   - 핸들은 `PROC_THREAD_ATTRIBUTE_HANDLE_LIST` 허용목록으로만 상속한다.
   - 근거: Microsoft Learn "Job Objects"는 프로세스 집합을 하나의 단위로 관리하고 종료하는 표준 메커니즘이다. watchexec의 `process-wrap`(`JobObject` 래퍼)도 같은 방식을 쓴다.
3. **Unix**:
   - `setsid()`로 자식을 새 세션·프로세스 그룹의 리더로 만든다.
   - 종료는 `kill(-pgid, SIGTERM)` 후 유예 500ms가 지나면 `SIGKILL`을 보낸다. 두 번째 terminate 호출이나 poll에서 승격한다.
   - Linux는 `PR_SET_PDEATHSIG(SIGKILL)`을 건 뒤 `getppid()`를 다시 확인해, fork와 prctl 사이에 부모가 죽은 경우를 막는다.
   - 예산:
     - 메모리는 `RLIMIT_AS`로 건다. 프로세스 단위이고 자손에게 상속되지만 패밀리 합계는 아니다.
     - 친화도는 Linux `sched_setaffinity`로 건다.
   - 소스 레인은 `socketpair(AF_UNIX, SOCK_STREAM)`을 쓴다.
     - 미리 연결되어 있으므로 `needs_connect=false`다. `SO_PEERCRED`/`getpeereid`로 uid를 확인한다.
     - 자식 fd만 `pre_exec`에서 `FD_CLOEXEC`를 해제한다.
   - 근거: `process-wrap`의 `ProcessGroup`/`ProcessSession` 래퍼, POSIX `setsid(2)`/`kill(2)` 음수 pid 규약, systemd·tokio 생태계의 TERM→KILL 관례.
4. **능력표 `ProcessCapabilities`**: 각 항목은 `Enforced`/`Advisory`/`Unsupported`로 표시한다.

   | 항목 | Windows | Linux | 기타 Unix |
   |---|---|---|---|
   | family_containment | Enforced | Advisory(setsid로 이탈 가능) | Advisory |
   | family_termination | Enforced | Advisory | Advisory |
   | cleanup_on_owner_death | Enforced(KILL_ON_JOB_CLOSE) | Advisory(PDEATHSIG, 아래 주의) | Unsupported |
   | memory_budget | Enforced(패밀리 commit) | Advisory(RLIMIT_AS, 프로세스 단위) | Advisory |
   | affinity_budget | Enforced | Advisory | Unsupported |
   | family_memory_peaks | Enforced(Job 피크) | Advisory(직계 자식 maxrss) | Advisory |
   | exact_family_count | true | false(/proc pgid 근사) | false |

   테스트는 건너뛰지 않고 능력값과 비교해 검증한다.
5. **PDEATHSIG 주의**: 이 신호는 부모 *프로세스*가 아니라 fork를 호출한 *스레드*가 끝날 때 발생한다(prctl(2)). 스폰한 스레드가 먼저 끝나면 자식이 죽는다. 반대로 부모가 SIGKILL로 죽어도 손자에게는 전달되지 않는다. 그래서 Advisory로 둔다.
6. **cgroup v2는 보류**:
   - 위임된 cgroup이 있을 때만 Enforced 격리·합계 메모리를 줄 수 있다. 감지 함수 `cgroup_v2_delegated()`만 둔다.
   - 데스크톱 세션이 systemd 위임 없이 실행되는 경우가 흔하므로 기본값은 RLIMIT와 관측이다.
   - 위임 환경용 어댑터는 같은 포트 뒤에 나중에 추가한다. 이때 능력표 값만 바뀐다.
7. **입력 리더 `read_only_duplicate`**:
   - Windows는 `DuplicateHandle(FILE_GENERIC_READ)`를 쓴다.
   - Linux는 쓰기 전용 fd를 거부한 뒤 `/proc/self/fd/N`을 읽기 전용으로 다시 열고 dev·ino 동일성을 확인한다.
   - 기타 Unix는 `try_clone`을 쓴다. 권한은 원래 열린 모드를 따르므로 Advisory다.

## Consequences
- `kinekino-process`의 lib 전역 `cfg_attr(not(windows), allow(...))`를 없앴다. `test-support` cfg와 `windows` 일반 의존도 없앴다.
- 공개 경로가 `infrastructure::windows`에서 `infrastructure::family`로, `WindowsFamily`가 `OwnedProcessFamily`로 바뀌었다. `ExitProbe`는 `Box<dyn ExitProbe>`가 되었다.
- 미검증: Windows 어댑터는 Linux에서 `x86_64-pc-windows-msvc` clippy로 컴파일만 확인했다. 실제 Windows 실행 테스트는 하지 못했다.

## 후속: fork 창과 flock

Unix 어댑터가 `pre_exec`(setsid·PDEATHSIG·RLIMIT·affinity) 때문에 fork 경로를 쓰므로, exec 전 자식이 부모의 잠금 fd(open file description)를 잠깐 공유한다. 그 사이 해제된 `flock`이 여전히 점유된 것처럼 보일 수 있어 `kinekino-platform-unix::fs::try_lock_exclusive`는 `WouldBlock`을 최대 250ms 재시도한다(CLOEXEC로 exec 시 자동 해제). 실제 writer 충돌은 여전히 `RevisionConflict`로 보고된다.

## Follow-up (batch 3, 2026-10-30): frame_capture / clip_* / review_* / timeline_evidence / mpv ungated

- `frame_capture::process` always runs through the owned-family port; the POSIX-only plain
  `Command` fallback is removed (one code path, capability-dependent guarantees).
- `ProcessOwnership::current_affinity` (Windows `GetProcessAffinityMask`, Linux
  `sched_getaffinity`, `None` elsewhere) replaces direct Win32 calls in player tests.
- `OsPolicy::try_lock_shared` (Windows no-op under mandatory share modes; POSIX `flock` shared)
  makes user-review readers and writers mutually exclusive on advisory-lock OSes.
- mpv uses `hide_console_window` and `fresh_guid` instead of `CommandExt`/`CoCreateGuid`.
  `LegacyAudioHandoffPorts` moved into `infrastructure::browser` (it couples browser and mpv).
- Tests that asserted Windows share-mode denial now compare against
  `has_mandatory_share_modes()` and assert the POSIX branch (retained-handle authority, advisory
  lock `WouldBlock`) instead of being skipped.
- Still gated pending batch 5 (browser port): `playback_workspace` (needs
  `browser_provider_jobs`), plus `browser*`, `video_surface`, `webview`, `window_*`.
