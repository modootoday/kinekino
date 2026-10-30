---
id: adr:kinekino/20261030180000
title: "ADR 0008: 부모 단조 카운터 중앙 포트 — VAD 워커 제어 시계의 Cross-OS 이전"
status: active
created: 2026-10-30
updated: 2026-10-30
rel:
  governs: [crates]
---

# ADR 0008: 부모 단조 카운터 중앙 포트

## Context
- VAD 실행기(부모)와 워커(자식)가 QPC·`GetProcessTimes`·Toolhelp FFI를 직접 호출해 `cfg(windows)` 뒤에 있었다(ADR worker-monotonic-control 참고).
- 사용자 원칙: Cross-OS, 중앙 인터페이스 필수, OS별 분리.

## Decision
1. **포트**: `kinekino_core::platform::clock` — `CounterIdentity{kind, frequency_hz, parent_pid, parent_creation}`, `CounterSample{identity, ticks}`, `ParentCounter{kinds, sample_current, open_creator}`, `LiveCreator{pid, sample}`. `OsPolicy::counter()`로 얻는다.
2. **카운터 도메인 이름**: `windows-qpc/1`, `posix-monotonic/1`. 제어 파일 검증은 `COUNTER_KINDS`에 든 이름만 받는다. 부모와 자식이 다른 도메인이면 거부한다(같은 호스트 계약).
3. **Windows 어댑터**: 기존 코드 이동 — QPC, `GetProcessTimes` 생성 시각, Toolhelp로 생성자 조회, `OpenProcess`+`WaitForSingleObject(0)`로 생존 확인.
4. **Unix 어댑터**: `CLOCK_MONOTONIC`(POSIX 표준, NTP 단계 조정 없음) ns 단위(frequency 1e9). 부모 생성 스탬프는 `/proc/<pid>/stat` 22번째 필드(ADR 0007과 같은 근거). 생성자는 `getppid()==pid`이고 생성 스탬프가 그대로일 때 살아 있다고 본다(재부모화·PID 재사용 감지).
5. **`windows_loaded_runtime`은 `cfg(windows)` 유지**: 실제 Windows DLL 로드 승인은 OS 고유 어댑터의 일이며 다른 OS에 대응 개념이 없다.
6. **공유 모드 테스트 오라클**: 보유 가드에 대한 변경 시도는 `has_mandatory_share_modes()`가 참이면 실패해야 하고, 거짓(POSIX 권고 잠금)이면 성공하며 이후 신원·다이제스트 재검증이 가드 역할을 한다(`guarded_mutation`). 테스트를 건너뛰지 않는다.

## Consequences
- 미검증: Windows 실제 실행(어댑터는 이동 코드, Windows 타깃 clippy만), macOS/BSD(`/proc` 없음 → 생성 스탬프 조회 실패로 거부).
- `kinekino-vad-worker` 실행 파일 진입점은 `kinekino-app`의 bin(`src/bin/kinekino-vad-worker.rs`)으로 옮겼다(ADR p1-port-inversion과 같은 구성 루트 패턴). 워커 크레이트는 도메인으로 남고 `process_entry::main`만 제공하며, 어댑터 의존은 dev-dependency(테스트 설치)뿐이다. bin 이름·출력 경로는 그대로다.
