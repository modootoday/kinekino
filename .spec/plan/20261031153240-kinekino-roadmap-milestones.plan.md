---
id: plan:kinekino/20261031153240
title: kinekino 완성 로드맵과 마일스톤
status: draft
created: 2026-10-31T15:32:40Z
updated: 2026-11-06T11:35:00Z
rel:
  governs: [crates]
---

# kinekino 완성 로드맵과 마일스톤

## Goal

PotPlayer를 기준으로 한 Linux 우선 플레이어를 끝까지 완성하기 위한 마일스톤과 조각 순서를 정한다. 정기 작업(하루 두 번, 한 번에 한 조각)이 다음 조각을 고르는 기준이 이 문서다. 인벤토리 번호(#)는 [kinekino 로드맵](20261029151512-kinekino.plan.md), 탐색 후속은 [탐색 남은 일](20261031060318-browse-plugins-next.plan.md), 병렬 트랙은 [병렬 트랙](20261030013900-kinekino-parallel-tracks.plan.md)을 따른다. 항목 ID(PL-, UI-, PG- 등)는 구현 현황표의 행 ID다.

기준 시점: main `4ac53253`(2026-11-01). 현황 집계(72항목): 구현 37, 부분 20, 미구현 13, 보류 2, 미확인 0. 남은 일은 부분·미구현 33항목이다.

### 원칙

- 우선순위: 재생 핵심 → 플레이어 UI(PotPlayer 기준, 재생목록은 시안의 회차 목록 패턴) → 호스트·플러그인 경계 → 제작 기능과 구조 정리 → 마무리.
- 호스트 경계: 호스트 코드와 화면에는 사이트별 용어를 두지 않는다. 호스트가 아는 것은 하위 목록이 있는 항목과 재생 가능한 항목뿐이다(ADR 20261031101741). 해석은 플러그인 몫이다.
- 브랜드 이미지(앱·창·트레이 아이콘, favicon)는 마스코트 원본에서만 파생한다(ADR 20261031111316).
- 한 조각은 한 번의 실행으로 설계 → 테스트 먼저 → 구현 → 검증 → 로컬 main 병합까지 끝나는 크기다. 큰 항목은 첫 단계 조각만 잡는다.
- 병합 조건: 새 테스트와 전체 테스트 통과, `spec check` 오류 0, 변경 범위 fmt·clippy 통과, 공개 표면에 비공개 흔적 0.
- push·태그·릴리즈는 사용자 지시가 있을 때만 한다. 이 문서의 "릴리즈 후보"는 시점 표시일 뿐 실행 권한이 아니다.

## Steps

### 마일스톤 요약

|마일스톤|목표(사용자가 할 수 있게 되는 것)|항목 ID|조각 수(추정)|사용자 결정·실기 확인|
|---|---|---|---|---|
|M1 재생 핵심|Linux에서 로컬 파일을 PotPlayer처럼 이어보고, 다음 회차로 넘기고, 속도·트랙·자막을 바꾼다|PL-5, PL-6, PL-7, PL-8, PL-4, SB-2|10|없음(헤드리스 libmpv로 검증)|
|M2 플레이어 UI|PotPlayer 기준 단축키·우클릭 메뉴·재생목록 패널·설정 연결을 갖춘 창을 쓴다|UI-5, UI-6, PL-2 후속, UI-9 후속|13|시안 밖 화면 요소는 권고안으로 진행 후 보고. 기능 병합 뒤 시각 완료는 실화면 1:1 확인(M2-V*)|
|M3 호스트·플러그인 경계|외부 소스 플러그인의 재생 항목을 안전하게 재생한다(호스트 정책·자격증명)|PG-5, PG-8, PG-10, PG-2 후속, ST-2|8|외부 플러그인 실사이트 확인은 사용자(2026-11-06 원 님: M3-7 실사이트 확인 허용)|
|M4 제작 기능 기반|CLI로 캡처·스냅샷·작업 대기열·프로젝트·카드·전사를 Linux에서 쓴다|SB-3, SB-4, SB-5, SB-6, ST-3, ST-4, ST-5, ST-6|10|실제 추론 호출·과금(P7 승인 게이트)|
|M5 구조·플랫폼 정리|어댑터 밖 cfg 0에 가까워지고 Windows 전용 경로가 포트 뒤로 들어간다|OS-5, SC-2, SC-3, OS-4, DC-4, UI-8, PG-9|8|DC-4 범위 결정, GPU 경로·OS 격리(sandbox)는 설계 조각만|
|M6 마무리·배포 준비|여러 OS에서 검증된 설치형 빌드와 맞는 공개 문서를 갖춘다|OS-2, OS-3, UI-3 실기, CI-1, CI-2, DC-2, DC-3|6|Windows·Wayland·macOS 실기, CI 실행(push 필요), 서명|

합계 약 55조각. 릴리즈 후보 시점은 두 곳이다. **RC-A**: M1·M2 완료 후(로컬 재생 미리보기). **RC-B**: M6 완료 후(정식 후보). 릴리즈와 SDK 태그 실행은 사용자가 정한다.

### M1 재생 핵심

- 선행: 없음(libmpv 소프트웨어 렌더 경로, ADR 20261031020000).
- 완료 기준: (1) 재생 위치가 파일별로 저장되어 다시 열면 이어서 재생되는 것을 통합 테스트가 확인한다. (2) 끝에 도달하면 다음 항목이 자동 재생되고 설정으로 끌 수 있다. (3) 속도·오디오 트랙·자막 트랙·볼륨 명령이 controller 테스트로 검증된다. (4) ASS 자막이 스타일을 유지해 표시된다(libmpv 렌더 경로). (5) 수동 구간 건너뛰기가 UI에서 동작한다.
- 조각 순서:
  1. M1-1 자동 다음화: eof 시 다음 항목 재생, 끄기 설정(PL-6).
  2. M1-2 볼륨 증감·음소거 단축키와 표시(PL-5).
  3. M1-3 재생 속도 명령과 단축키(PL-5).
  4. M1-4 오디오·자막 트랙 순환 명령(PL-5).
  5. M1-5 이어보기 저장 형식과 읽기·쓰기(PL-7, 화면 연결 없음).
  6. M1-6 이어보기를 열기 경로에 연결(PL-7).
  7. M1-7 환경설정 영속(볼륨·속도·자동 다음화)(PL-7).
  8. M1-8 ASS·이미지 자막을 libmpv 렌더에 맡기고 텍스트 오버레이는 대체 경로로(PL-4, SB-2 첫 단계).
  9. M1-9 수동 구간 건너뛰기 UI(PL-8 첫 단계, 자동 감지는 M4 뒤).
  10. M1-10 일반 재생 명령의 CLI 경로를 libmpv 경로로 재판정·정리(PL-5).

### M2 플레이어 UI

- 선행: M1-2~M1-4(메뉴에 넣을 명령).
- 완료 기준: (1) PotPlayer 기본 단축키 표(문서화된 목록)의 항목이 keys 테스트로 검증된다. (2) 우클릭 메뉴에서 열기·재생·자막·오디오·화면 명령을 고를 수 있다. (3) 재생목록 패널이 시안의 회차 목록 패턴(현재 항목 표시·도킹)을 따른다. (4) 설정 창이 같은 제목 줄 규약을 쓴다. (5) 기능 병합된 UI 조각은 `.spec/artifacts/design/` 시안과 실화면을 1:1로 대조한 뒤에만 시각 완료로 표시한다(visual-review.md). 시안의 장식·오류는 복제하지 않고, 새 팔레트·목업 재생성은 하지 않는다(보류: UI 시안 작업 재개).
- 조각 순서(기능): M2-1 단축키 표 설계 ADR과 탐색·시크 단축키, M2-2 우클릭 메뉴 골격, M2-3 메뉴에 M1 명령 연결, M2-4 재생목록 패널 현재 항목·도킹 정합, M2-5 재생 정보(OSD) 표시, M2-6 설정 창 사용자 정의 제목 줄(UI-5), M2-7 남은 wry 의존 정리 판정(UI-6), M2-8 창·트레이 아이콘을 마스코트 파생 자산에 연결.
- 조각 순서(시각 검증, 기능 병합 후·화면별 1회 실행 크기): M2-V1 단축키·탐색 피드백 실화면(M2-1, 시크·탐색 키에 보이는 반응만·시안에 없는 장식 추가 금지), M2-V2 우클릭 메뉴 실화면 1:1(M2-2+M2-3, 배치·그룹·항목 대비), M2-V3 재생목록 패널 실화면 1:1(M2-4, 회차 목록 헤더·행·현재 강조·도킹), M2-V4 OSD 실화면 1:1(M2-5, 하단 토스트 시크·음량 문구·위치), M2-V5 설정 창 제목 줄 실화면 1:1(M2-6, aw-window-chrome 대비). M2-7은 UI 크롬 없음(판정만). M2-8은 사용자 보류(아이콘).

### M3 호스트·플러그인 경계

- 선행: 없음(M1과 독립, 순서상 뒤).
- 완료 기준: (1) 재생 스트림 호스트가 플러그인 선언 범위 밖이면 거절되는 테스트가 있다. (2) 비밀 값이 OS 자격증명 저장소 포트를 거쳐 저장된다(Linux Secret Service 어댑터, 실패 시 메모리 저장소로 명시적 대체). (3) 브라우저 캡처로 resolve를 위임하는 설계가 ADR로 정해지고 첫 경로가 계약 테스트를 통과한다. (4) 호스트 소스에 사이트별 용어가 없다(r-src lint).
- 조각 순서: M3-1 플레이어 도메인 정책 자리표시자 교체(PG-5), M3-2 자격증명 저장소 포트 계약 테스트, M3-3 Secret Service 어댑터(ST-2), M3-4 브라우저 resolve 위임 ADR(PG-8), M3-5 위임 첫 경로(목 엔진), M3-6 장기 실행 플러그인 워커 설계(PG-2 후속), M3-7 외부 플러그인을 재생목록 프로토콜에 맞추는 점검(PG-10, 저장소 밖; 2026-11-06 원 님 실사이트 확인 허용), M3-8 플러그인 SDK 문서 정합.

### M4 제작 기능 기반(CLI 우선)

- 선행: M1(재생 경로), 추론 실호출은 P7 승인.
- 완료 기준: 각 항목의 Linux CLI 경로가 Unsupported가 아니고 fixture 기반 테스트가 있다. 실제 클라우드 추론은 목 계약 테스트까지만.
- 조각 순서: M4-1 Jobs submit·cancel·retry CLI(ST-5), M4-2 프로젝트 export(ST-6), M4-3 스냅샷 Linux 경로(SB-3), M4-4 프레임 캡처 원격 입력 거절 정리(SB-3), M4-5 카드 부분 실패 정리(SB-4), M4-6 숏폼 회전·VFR 첫 단계(SB-4), M4-7 자산 재import(SB-4), M4-8 OCR 포트 목 계약(SB-6), M4-9 로컬 whisper 실측 준비(ST-4, 실측은 승인 후), M4-10 피사체 분할 첫 단계 설계(SB-5).

### M5 구조·플랫폼 정리

- 선행: 없음. 다른 마일스톤 사이의 빈 실행에 끼워 넣는다.
- 완료 기준: allowlist 줄 수와 어댑터 밖 cfg 수가 매 조각 줄어들고 신규 위반 0. GPU 렌더·OS 격리(sandbox)는 ADR로 범위가 정해진다.
- 조각 순서: M5-1~M5-4 플랫폼 cfg 정리 묶음(OS-5, SC-2), M5-5 allowlist 감축(SC-3), M5-6 GPU 렌더 경로 ADR(UI-8), M5-7 OS 격리(sandbox) 어댑터 ADR(PG-9), M5-8 Unix 브라우저 provider 판정(OS-4). DC-4(core 컨텍스트화·ID 이전)는 사용자 결정 후 별도 계획.

### M6 마무리·배포 준비

- 선행: M1~M3, CI 실행에는 push 지시 필요.
- 완료 기준: Windows 실제 창 스모크와 Wayland 리사이즈 실측 기록, macOS 빌드 판정, CI 녹색 실행 기록, README·홈페이지가 실제 기능과 일치.
- 조각 순서: M6-1 공개 표면 정합(DC-2), M6-2 CI 로컬 재현 점검(CI-1), M6-3 설치형 패키지 설계(CI-2), M6-4 마스코트 이벤트 포트 no-op(DC-3), M6-5 Windows·Wayland 실기 확인 체크리스트(OS-2, UI-3), M6-6 macOS 판정(OS-3).

### 보류(사용자 지시 전에는 손대지 않음)

- 릴리즈 실행과 SDK 태그 `plugin-sdk-v0.1.0`(CI-4, 릴리즈와 함께).
- allowlist 2~4단계 추가 범위.
- UI 시안 작업 재개.
- 미디어 다운로드(SB-7).
- 그 밖의 사용자 보류 결정.

### 예상 기간

- 근거: 남은 항목을 55조각 안팎으로 나눴다(항목당 평균 1.5조각). 하루 2회 실행, 검증 실패·막힘으로 병합까지 못 가는 실행을 30~40%로 보면 하루 1.2~1.4조각이다. 55 ÷ 1.2~1.4 ≈ 39~46일, 약 6~7주다.
- RC-A(M1·M2, 23조각)는 약 2.5~3주 뒤가 후보다.
- 불확실 요인: 빌드 락 공유와 느린 전체 테스트, 실기 확인(Windows·Wayland·macOS) 대기, P7 추론 승인, DC-4 결정, GPU·격리는 설계 이후 규모가 정해진다. 처음 1주 처리량을 보고 다시 추정한다.

## Record

- 2026-11-01: 초안. 미확인 18항목을 코드로 판정해 현황표 집계를 구현 37·부분 20·미구현 13·보류 2로 맞췄다.
- 2026-11-05: M1-7 viewing prefs (volume, mute, speed, auto-next) persist across runs (`09c159a9`).
- 2026-11-05: M1-8 styled (ASS/SSA) and picture subtitle tracks are drawn by the engine into the frame; text tracks stay on the overlay (`9fc048f8`).
- 2026-11-05: Manual skip ranges persist per local-file key and restore on reopen (UI criteria (5) through save); siblings are not copied (`46392c5d`). M1-10 still open — M1 not closed.
- 2026-11-05: M1-10 — product playback stays on `kinekino watch` → player-ui → libmpv; the old `kinekino-player` mpv-process IPC CLI stays experimental (ADR 20261105141813). M1 criteria (1)–(5) met on the product path — M1 closed.
- 2026-11-06: M2-1 — shortcut table ADR and seek/navigation keys (arrow seek steps, PageUp/Down, Backspace; skip keys confirmed). M2 not closed.
- 2026-11-06: M2-2 — right-click menu skeleton (pointer-anchored popup kept inside the window, five groups reusing existing commands, same close rules as the more menu). Visual check pending. M2 not closed.
- 2026-11-06: M2-3 — right-click menu wired to M1 commands (flat items under open/play/skip/subtitle/audio/screen; seek ±5/±30). Visual check pending. M2 not closed.
- 2026-11-06: M2-5 — seek and volume OSD on the existing bottom toast (`View.notice`); formats `{position} / {duration}`, `음량 N`, `음소거` (ADR 20261105182156). Transport-bar permanent speed label and centered OSD chrome deferred. Visual check pending. M2 not closed.
- 2026-11-06: M2-6 — settings window product caption (`aw-window-chrome`); undecorate OS frame; drag, double-click maximize, caption min/max/close (ADR 20261105191533). Edge resize deferred. Visual check pending. M2 not closed.
- 2026-11-06: M2-7 — remaining wry kept as the Windows WebView2 browser-engine adapter (not product UI); eframe/egui product-UI deviation closed; moving player browser modules into kinekino-browser deferred (ADR 20261105201855). M2 not closed.
- 2026-11-06: M2-4 — playlist panel follows mockup episode-list pattern (heading `회차 목록`, current-item amber bar/play/text, windowed dock, fullscreen hide keeping open state, Esc/P; ADR 20261106105936). Visual check pending. M2 not closed.
- 2026-11-06: M3-1 — player media/frame hosts admitted at runtime via AdmittedHosts (plugin media_hosts install; remaining hardcodes removed) (`ec74ce96`). M3 not closed.
- 2026-11-06: M3-2 — CredentialStore port contract expanded (key isolation, overwrite, delete-absent, Secret Debug redaction, empty keys InvalidInput); Session/File adapters pass the same harness (`e4e208e1`). OS Secret Service adapter still M3-3. M3 not closed.
- 2026-11-06: M3-3 — Linux Secret Service CredentialStore adapter (`SecretServiceCredentialStore`) meets `check_credential_store` on an unlocked collection (`d6c9b778`). App still uses Session store; product memory fallback not wired. Windows Credential Manager still out of scope. M3 not closed.
- 2026-11-06: M3-4 — browser-capture Resolve delegated as a host capability (`BrowserCaptureSession`); plugin never owns the browser; first path (mock engine + contract tests) is M3-5 (ADR 20261106032051). M3 not closed.
- 2026-11-06: M3-5 — first host BrowserCaptureSession path with mock engine and contract tests; plugin maps observations to StreamCandidate; NetworkInterceptor remains Allow/Block (ADR 20261106032051).
- 2026-11-06: M3-6 — long-running per-plugin media-source worker designed (OwnedFamily spawn, JSON-line serve loop, one in-flight request, idle/crash policy); implementation deferred (ADR 20261106051332).
- 2026-11-06: M3-6b — long-running per-plugin PluginWorker implemented (OwnedFamily spawn, SDK serve_loop, ProcessSource reuse, idle/crash/call-timeout; contract tests) (`1250a73b`). M3 not closed.
- 2026-11-06: M2 visual-verification slices added (M2-V1…M2-V5) after functional merges left 시각 미확인; M2 criteria (5) requires real-screen 1:1 before 시각 완료. M2 piece count 8→13; total ~55. Mockup regeneration still held.
- 2026-11-06: M3-7 real-site check allowed by 원 님 (2026-11-06); historical user-gate noted, not erased.
