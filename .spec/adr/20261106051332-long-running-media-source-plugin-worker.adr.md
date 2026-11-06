---
id: adr:kinekino/20261106051332
title: 장기 실행 미디어 소스 플러그인 워커
status: active
created: 2026-11-06T05:13:32Z
updated: 2026-11-06T05:14:00Z
rel:
  governs:
    - crates/kinekino-source-host
    - crates/kinekino-plugin-sdk
  depends_on:
    - adr:kinekino/20261030200000
    - adr:kinekino/20261030030000
    - adr:kinekino/20261031102400
---

# 장기 실행 미디어 소스 플러그인 워커

## Context

로드맵 plan:kinekino/20261031153240의 M3-6(PG-2 후속)이다. ADR 0011 Decision §2는 미디어 소스를 **별도 프로세스 + IPC**로 두고, 프로세스 수명·버전 협상 비용을 받아들인다. 현황표 PG-2는 manifest·discover·`ProcessSource`가 **구현**이지만 남은 일로 **장기 실행 워커 미구현**을 적고 있다.

코드 현황(2026-11-06):

- `ProcessSource::call`은 호출마다 `Command::spawn` → stdin에 JSON 한 줄 → stdout 한 줄 읽기 → `kill`/`wait`다(`plugin_host/process.rs`). 소유 포트(ADR 0006)를 쓰지 않는다.
- SDK `serve` / `serve_io`도 **한 요청·한 응답 후 종료**다(`kinekino-plugin-sdk` `serve.rs`, `wire` 주석).
- 플레이어 UI는 browse/resolve를 **호출당 짧은 워커 스레드**에서 돌리고, 동시 진행은 하나뿐이며 늦은 응답은 버린다(ADR 20261031102400). 프로세스 수명이 아니라 UI 스레드 보호다.
- `source-host`의 `worker_session` / `http-worker` / Windows `worker_ipc`는 **미디어 전송(HTTP 바디) 레인**용이며, `ProcessSource` 카탈로그 IPC와 다른 계층이다. 이 조각에서 합치지 않는다.
- M3-4/M3-5의 호스트 `BrowserCaptureSession`은 플러그인 Resolve가 호스트 능력을 쓰는 경로다. 장기 워커 채널 위에서 호스트→플러그인 능력 메시지를 실을 여지는 남기되, 이번 설계의 구현 범위는 아니다.

권고안(루틴 진행):

1. **활성화된 플러그인마다 장기 워커 프로세스 하나.** 호출마다 spawn/kill하지 않는다.
2. 스폰은 **ADR 0006 `ProcessOwnership` / `OwnedFamily`**로 한다(Windows Job Object, Unix 세션·프로세스 그룹). 원시 `Command`만 쓰는 경로는 후속에서 걷는다.
3. 와이어는 기존 **줄 단위 JSON(`source/1`)** 을 유지하되, 프로세스가 stdin EOF·호스트 shutdown·유휴 만료까지 **요청을 반복**한다. 첫 구현은 **동시 진행 요청 1개**(UI 규칙과 맞춤). 상관 id·다중 진행은 후속.
4. 유휴 타임아웃 후 graceful 종료; 다음 호출이 다시 띄운다. 크래시·프로토콜 깨짐은 해당 호출 `Unavailable`, 재시도는 호출자 몫(자동 재시도 없음).
5. 구현·계약 테스트는 **M3-6b**. 이번 조각은 ADR만.

## Decision

1. **수명 모델.** 호스트는 플러그인 id당 `PluginWorker` 하나를 둔다. 상태: `Stopped` → (첫 호출 또는 예열) `Starting` → `Ready` → (유휴·shutdown·고장) `Stopping`/`Stopped`. `SourceProvider` 구현은 워커에 한 줄 요청을 보내고 한 줄 응답을 기다린다. 호출마다 새 프로세스를 만들지 않는다.
2. **프로세스 소유.** 스폰·종료·예산은 ADR 0006 포트(`reserve_family` / `spawn` / terminate TERM→KILL)를 쓴다. stderr는 기본 null(현행과 동일); 진단 로그는 후속. 자식이 만든 손자도 Job/세션 규칙으로 묶인다.
3. **와이어·SDK.**
   - 요청·응답 JSON 스키마(`Request`/`Response`, `document/1`)는 유지한다. `api: source/1` 버전을 올리지 않는다.
   - SDK에 `serve_loop` (가칭)을 추가한다: stdin에서 줄을 읽는 동안 `handle_line`을 반복하고, EOF면 정상 종료한다. 기존 `serve` / `serve_io`(단발)는 테스트·구형 바이너리용으로 남긴다.
   - 호스트는 장기 모드만 쓴다. 단발 전용 구형 플러그인은 한 응답 뒤 EOF로 끝나므로 호스트가 `Unavailable` 후 다음 호출에서 재시작하며, 호환 표기는 SDK 문서(M3-8)에서 다룬다.
4. **동시성.** 워커당 **진행 중 요청 최대 1개**. 두 번째 호출은 첫 응답(또는 실패) 뒤에만 보낸다. 플레이어 UI의 “한 번에 하나·교체 시 늦은 응답 폐기” 규칙과 맞춘다. 다중 진행·상관 id는 필요해지면 별도 ADR.
5. **타임아웃·유휴.**
   - **호출 타임아웃:** 호스트가 요청을 보낸 뒤 응답 한 줄(또는 프로세스 종료)을 기다린다. 만료 시 워커 패밀리를 terminate하고 호출은 `Unavailable`. 기본값은 구현 조각에서 고르고 테스트로 고정한다(현행 kill-after-read와 같은 상한 정신, 구체 초는 M3-6b).
   - **유휴 타임아웃:** Ready 상태에서 마지막 응답 이후 일정 시간 요청이 없으면 graceful shutdown(stdin close → 짧은 대기 → terminate). 다음 `SourceProvider` 호출이 다시 띄운다.
6. **고장.** 비정상 종료, 잘린 줄, JSON 파손, 선언 밖의 스트림 호스트(현행 resolve 검사)는 그 호출을 실패로 돌리고 워커를 `Stopped`로 내린다. **같은 논리적 호출을 호스트가 자동 재시도하지 않는다.**
7. **호스트 능력(브라우저 캡처)과의 관계.** M3-4/M3-5의 `BrowserCaptureSession`은 호스트 소유이다. 장기 워커가 열린 뒤, 플러그인이 Resolve 중에 호스트 능력을 요청하는 **역방향 메시지**는 이 채널에 얹을 수 있으나 스키마·핸드셰이크는 **후속 조각**이다. 이번 결정는 카탈로그 요청/응답 루프와 수명만 닫는다.
8. **제외.** `http-worker` 미디어 전송 레인과 통합하지 않는다. OS 샌드박스(seccomp 등)는 ADR 0011 후속 그대로. 설정 페이지·플러그인 enable UI는 범위 밖. 실사이트 플러그인 점검(M3-7)·SDK 문서 정합(M3-8)은 별도.
9. **M3-6b 범위(구현).** `PluginWorker` + OwnedFamily 스폰, SDK `serve_loop`, `ProcessSource`(또는 후속 타입)가 워커를 재사용, 계약 테스트: 연속 browse 두 번이 프로세스 한 번, 유휴 후 재스폰, 크래시 시 Unavailable·다음 호출 재시작, 호출 타임아웃 terminate. 단발 `serve` fixture 경로 회귀 유지.

## Rationale

- 호출마다 spawn은 cold start·TLS·쿠키 Jar·로그인을 매번 버리게 해 browse→series→resolve 연속 사용에 비용이 크다. ADR 0011이 이미 프로세스 분리 비용을 받아들였으므로, 그 비용을 **플러그인당 한 번**으로 줄이는 것이 자연스럽다.
- ADR 0006을 쓰지 않는 원시 `Command`는 Job Object/PDEATHSIG·예산이 없어 손자 누수·고아 위험이 있다. 장기 수명일수록 소유 포트가 필요하다.
- 와이어를 유지하면 기존 fixture·golden·외부 플러그인 스키마를 깨지 않는다. 루프는 SDK 진입점 추가로 충분하고, `source/2` 버전 bump는 다중 진행·역방향 능력이 생길 때까지 미룬다.
- UI는 이미 동시 1호출이다. 워커도 1로 맞추면 큐잉·상관 id 없이 M3-6b를 작게 끝낼 수 있다.
- http-worker와 섞지 않는 이유: 전송 레인은 credit·부분 write·EOF 계약이 다르고 Windows IPC feature 게이트가 있다. 카탈로그 JSON 루프와 수명만 먼저 닫는다.

## Alternatives rejected

| 대안 | 이유 |
| --- | --- |
| 호출마다 spawn 유지(현행) | PG-2 남은 일·로그인/세션 재사용 요구를 못 닫음 |
| 플러그인당 워커 풀(N>1) | UI·호스트가 동시 1요청; 복잡도만 증가 |
| 첫 조각에서 상관 id 다중 진행 | M3-6b 비대; UI가 아직 1호출 |
| `source/2`로 스키마 교체 | 불필요; 루프만으로 수명 목표 달성 |
| http-worker 레인과 통합 | 계약·플랫폼 게이트가 다름 |
| 크래시 자동 재시도 | 중복 부작용(결제·조회 수) 위험; 호출자 명시 재시도만 |
| 이 조각에서 OwnedFamily 코드까지 | M3-4와 같이 설계만; 구현은 M3-6b |

## Consequences

- PG-2는 설계 완료로 **부분**이 된다(장기 워커 코드는 M3-6b까지 미구현).
- M3-6b가 `PluginWorker`·`serve_loop`·재사용 `ProcessSource`와 계약 테스트를 넣는다. 큐에 M3-6b 행을 둔다.
- 기존 단발 e2e(`plugins_e2e`, fixture `serve`)는 M3-6b에서 회귀로 유지한다.
- M3-5 브라우저 캡처 IPC를 워커 채널에 싣는 일은 별도 후속이며, 이 ADR만으로 PG-8을 닫지 않는다.
- 외부 플러그인은 장기 루프를 쓰도록 SDK 예시·문서(M3-8)에서 안내한다. 단발 바이너리는 한동안 재시작으로 동작할 수 있으나 권장하지 않는다.

## Record

- 2026-11-06: M3-6 — long-running per-plugin worker designed (OwnedFamily spawn, JSON-line loop, one in-flight request, idle/crash policy); implementation deferred to M3-6b.
