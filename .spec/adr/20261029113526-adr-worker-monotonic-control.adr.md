---
id: adr:kinekino/20261029113526
title: "ADR: 원 VAD operation cutoff의 같은 Windows 호스트 전달"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# ADR: 원 VAD operation cutoff의 같은 Windows 호스트 전달

상태: 2026-10-21 원 root57db58c/library9c1b6a3의 source-only 후보에서 출발해, root561ae926/library0b9ea031의 명시 합성 gate에서 일반129·경로 helper2·실제 child8의 고유139가 통과했다. [실제 검증 정본](../../sots/verification/ai/vad-worker-control-process-2026-10-21.md)의 최종 source39 E9C690·실패/수리·고정 BIN·보호·범위 제한을 따른다. SDK는 공통 계약·부모 adapter, Media는 child clock/cancel/entry 및 P0 fallible checkpoint를 소유한다. 원 source-only raw/entry8은 보존했고, 생산 native admission·추론·제품 전체 지원 완료를 뜻하지 않는다.

## 원 후보 기준의 관측과 근거

기존 RunControl과 AnalysisControl은 bool cancellation/opaque Instant를 제공하며, Deadline은 operation/end를 분리한다. NativeTransport는 cancel.signal 경로와 원 request 파일을 전달하지만 자식 cutoff를 전달하지 않는다. 부모 Supervisor의 강제 종료·정리와 자식의 협력 제어는 다르다. signal 읽기 실패를 bool로 감추지 않는다.

Microsoft의 [QPC 지침](https://learn.microsoft.com/en-us/windows/win32/sysinfo/acquiring-high-resolution-time-stamps)은 같은 시스템의 thread/process 간 counter 일관성과 ±1 tick 비교 불확실성을 설명한다. [QPF](https://learn.microsoft.com/en-us/windows/win32/api/profileapi/nf-profileapi-queryperformancefrequency)의 주파수는 부팅 시 고정이다. 원시 TSC·UTC deadline·다른 호스트/VM clock으로 대체하지 않는다. QPC와 FILETIME은 별개이며 [GetProcessTimes](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes)의 creation FILETIME은 살아 있는 원 parent 식별에만 사용한다.

기존 native cache의 rustc metadata는 1.99.0/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/x86_64-pc-windows-msvc다. 설치 std source와 해당 [Rust Windows Instant 원문](https://github.com/rust-lang/rust/blob/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/time/windows.rs)을 읽었다. QPC를 integer nanoseconds로 내림 변환하고 opaque epoch offset을 더하며, subtraction은 perf_counter epsilon을 처리한다. epoch/Instant 내부를 추출하지 않는다. 로컬 source SHA는 windows time `3E344B796C406F5446DC0D31BB7737302723F21B4C606ED2B30794731E9B53DE`, pal/windows/time `59BA46AF23BA2C12507BBB96D11BF76ABE53DF11397BD1BAAC8F864B378B6B1E`다. 실제 새 빌드의 toolchain/backend가 달라지면 이 오차 분석을 재검토한다. 이 문서는 Rust의 모든 미래 backend에 대한 보장이 아니다.

## 계약과 보수적 투영

1. 부모가 QPC-before → Instant sample → QPC-after를 같은 thread에서 읽는다. 기존 original.operation_remaining(sample)과 caller의 원 cutoff 중 작은 값만 사용한다. end/cleanup reserve는 자식에 넘기거나 재발급하지 않는다.
2. 원 operation cutoff를 `before + floor(remaining_ns × frequency / 10^9) - uncertainty`로 정한다. uncertainty는 cross-thread 비교/경계2 tick과 integer nanosecond 내림 오차를 위로 환산한 tick의 합이다. 이른 before 기준 선택은 관측 bracket 폭만큼 더 일찍 끝날 수 있다. after가 이미 cutoff에 닿거나 overflow/회귀/주파수 변화/음수 QPC이면 거절한다. scheduler 지연·counter 변환·파일 쓰기·child startup은 이 절대 cutoff를 소비하며 새 전체 timeout을 시작하지 않는다.
3. `WorkerOperationCutoff`는 schema/clock kind·frequency·parent PID/creation identity·before/after·uncertainty·absolute operation cutoff·원 request bytes SHA를 가진다. strictly typed metadata이며 authority token/admission credential이 아니다. 같은 freq/domain 문자열만으로 같은 boot·parent를 증명하지 않는다.
4. 자식 CounterClock은 원 trusted spawn/request와 결속된 실제 parent handle의 creation identity/liveness를 검증하고, 같은 호스트의 OS QPC를 읽어야 한다. DTO 값을 그대로 복사한 fake clock은 production 구현이 아니다. 부모가 종료됐거나 재부팅·다른 호스트·clock regression을 판별할 수 없으면 fail closed한다. trusted argv/원 sidecar pin과 live parent 검증은 child 후속의 필수 조건이다.
5. `WorkerControlView::checkpoint()`는 clock/request/만료를 먼저 검사하고, fallible exact-signal 관측 뒤 clock을 다시 읽는다. clock·permission·replacement·read 오류는 원 Result를 보존한다. 결과 Continue/CancellationRequested는 관측이며 Timeout은 오류다. 취소·만료·stdout flush·child exit는 process family/EOF/cleanup ACK가 아니다.
6. legacy AnalysisControl에 필요한 `project_worker_deadline`은 counter-before → Instant → counter-after에서 after 기준 잔여량을 내림·추가 margin으로 줄인 고정 Instant view다. 새 budget/Scope가 아니며 한 번 계산해 보유한다. 항상 fallible QPC checkpoint가 권위 있는 실제 제약이다. bool에서 IO 오류를 true/false로 바꾸거나 EngineFailed를 Cancelled로 바꾸지 않는다. Media가 기존 P0/engine 모든 checkpoint 지점에 최소 호환 fallible 전달을 연결해야 한다.

## 부모 sidecar와 취소 marker

SDK의 별도 native adapter는 worker-control.v1.json을 create_new/write/sync하고 원 read guard·전체 bytes SHA를 NativeTransport에 보유한다. 원 request JSON은 바꾸지 않으며 별도 `--worker-control-v1 PATH --worker-control-sha256 SHA --worker-control-bytes N`을 전달한다. 원 entry8은 이 argv를 지원하지 않으므로 Media의 명시 후속과 함께 채택해야 한다. 구버전 worker로 묵시 fallback하지 않는다.

취소 marker는 먼저 cancel.signal.pending의 완전한 bytes를 sync하고, [hard_link](https://doc.rust-lang.org/stable/std/fs/fn.hard_link.html)로 기존 목적지를 덮지 않고 공개한다. 목적지는 완성된 file을 가리키며 부모가 read guard를 유지한다. 파일시스템이 hard link를 지원하지 않거나 기존 파일·손상·교체가 있으면 오류로 닫는다. 부분 evidence는 삭제하지 않고 in-place marker fallback도 하지 않는다. child는 정확한 원 root/marker를 관측하고 오류를 보존하는 concrete WorkerCancellation을 Media 쪽에서 구현한다. 부모 signal_sent는 완전 publication 뒤에만 설정되며 ACK로 해석하지 않는다.

## 실제 검증과 남은 범위

보수 arithmetic·늦은 수신/만료·잘못된 request/parent/frequency·회귀/overflow·strict metadata·signal/clock 오류 보존·signal IO 중 만료 및 Windows sidecar/marker guard는 실제 SDK normal11에서 통과했다. 명시 합성 actual8은 같은 Windows 호스트의 원 live parent/QPC·원 marker·고정 cutoff·fallible engine checkpoint와 동일 owner/원 cleanup을 연결했다. 실제8 모두 family Empty/stdin EOF 요청/forcedfalse를 기록했고 original regular raw16을 보존했다. 이 관측은 named-pipe EOF나 OS pipe 단절 시험이 아니다.

고정 부모 operation 자연 만료의 NativeTransport 경합 관측과 별도 shortened caller-cap source 후보는 [후속 계획](../../plans/player/inference/worker-control-observation-followups.md)에 남는다. 동기 IO/engine을 이 계약으로 강제 선점한다고 주장하지 않는다. trusted release/profile/native module admission은 별도 경계이며 여기서 권한을 발급하지 않는다. production Unsupported는 최종 완료가 아니다.
