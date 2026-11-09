---
id: plan:kinekino/20261031153240
title: kinekino 완성 로드맵과 마일스톤
status: draft
created: 2026-10-31T15:32:40Z
updated: 2026-11-08T17:27:47Z
rel:
  governs: [crates]
---

# kinekino 완성 로드맵과 마일스톤

## Goal

PotPlayer를 기준으로 한 Linux 우선 플레이어를 끝까지 완성하기 위한 마일스톤과 조각 순서를 정한다. 정기 작업(Asia/Seoul 기준 매일 매시, 10시·15시 제외, 최대 10분 지연 시작, 한 번에 한 조각)이 다음 조각을 고르는 기준이 이 문서다. 인벤토리 번호(#)는 [kinekino 로드맵](20261029151512-kinekino.plan.md), 탐색 후속은 [탐색 남은 일](20261031060318-browse-plugins-next.plan.md), 병렬 트랙은 [병렬 트랙](20261030013900-kinekino-parallel-tracks.plan.md)을 따른다. 항목 ID(PL-, UI-, PG- 등)는 구현 현황표의 행 ID다.

최초 기준 시점: main `4ac53253`(2026-11-01), 당시 72항목(구현 37·부분 20·미구현 13·보류 2). 2026-11-07 재정리는 main `4d98d94a`와 아래 Record를 기준으로 한다. 조각의 병합·시각 검증·마일스톤 닫힘은 서로 다른 판정이며, 이번 문서 정리를 기능 구현이나 실기 검증으로 세지 않는다.

### 원칙

- 우선순위: 재생 핵심 → 플레이어 UI(PotPlayer 기준, 재생목록은 시안의 회차 목록 패턴) → 호스트·플러그인 경계 → 제작 기능과 구조 정리 → 마무리.
- 호스트 경계: 호스트 코드와 화면에는 사이트별 용어를 두지 않는다. 호스트가 아는 것은 하위 목록이 있는 항목과 재생 가능한 항목뿐이다(ADR 20261031101741). 해석은 플러그인 몫이다.
- 브랜드 이미지(앱·창·트레이 아이콘, favicon)는 마스코트 원본에서만 파생한다(ADR 20261031111316).
- 한 조각은 한 번의 실행으로 설계 → 테스트 먼저 → 구현 → 검증 → 로컬 main 병합까지 끝나는 크기다. 큰 항목은 첫 단계 조각만 잡는다.
- 병합 조건: 새 테스트와 전체 테스트 통과, `spec check` 오류 0, 변경 범위 fmt·clippy 통과, 공개 표면에 비공개 흔적 0.
- 승인된 정기 작업은 게이트 통과·로컬 main 병합 직후, 작업 트리가 깨끗하고 공개 표면 검사 적중 0이며 origin/main이 main의 조상일 때만 `git push origin main`을 일반 push한다. 조건 불충족·거절이면 중단하고 원인을 기록한다. 강제 push·이력 재작성·브랜치 push는 하지 않는다. 태그·릴리즈·crates.io 게시는 별도 사용자 지시 전까지 금지한다. 이 문서의 "릴리즈 후보"는 시점 표시일 뿐 실행 권한이 아니다.

## Steps

### 마일스톤 요약

|마일스톤|목표(사용자가 할 수 있게 되는 것)|항목 ID|실행 단위 수|사용자 결정·실기 확인|
|---|---|---|---|---|
|M1 재생 핵심|Linux에서 로컬 파일을 PotPlayer처럼 이어보고, 다음 회차로 넘기고, 속도·트랙·자막을 바꾼다|PL-5, PL-6, PL-7, PL-8, PL-4, SB-2|16|M1 닫힘 유지. M1-V1~V3는 GUI 확인 보완이며 기능 완료를 다시 열지 않는다.|
|M2 플레이어 UI|PotPlayer 기준 단축키·우클릭 메뉴·재생목록 패널·설정 연결을 갖춘 창을 쓴다|UI-5, UI-6, PL-2 후속, UI-9 후속|14|시안 밖 화면 요소는 권고안으로 진행 후 보고. 기능 병합 뒤 시각 완료는 실화면 확인과 개선(M2-V*). 시안 일관성이 낮을 수 있으므로 시안 간 불일치는 공용 토큰·컴포넌트·화면 규칙을 우선한다(2026-11-06).|
|M3 호스트·플러그인 경계|외부 소스 플러그인의 재생 항목을 안전하게 재생한다(호스트 정책·자격증명)|PG-5, PG-8, PG-10, PG-2 후속, ST-2|11|SDK 문서 정합은 태그 없이 진행. 외부 플러그인 실사이트 확인은 사용자(2026-11-06 원 님: M3-7 실사이트 확인 허용)|
|M4 제작 기능 기반|CLI로 캡처·스냅샷·작업 대기열·프로젝트·카드·전사를 Linux에서 쓴다|SB-3, SB-4, SB-5, SB-6, ST-3, ST-4, ST-5, ST-6|10|목 계약·Linux CLI·Unsupported 해소만 진행. P7 실제 추론 호출·과금·실모델 실측은 별도 승인 전 금지.|
|M5 구조·플랫폼 정리|어댑터 밖 cfg 0에 가까워지고 Windows 전용 경로가 포트 뒤로 들어간다|OS-5, SC-2, SC-3, OS-4, DC-4, UI-8, PG-9|8|기존 범위 cfg·allowlist 감축만. allowlist 2~4단계 추가 범위와 DC-4 구현은 보류. GPU·OS 격리(sandbox)는 ADR만.|
|M6 마무리·배포 준비|Linux에서 할 수 있는 문서·게이트·패키지 설계를 마치고 OS별 실기 대기를 분리한다|OS-2, OS-3, UI-3 실기, CI-1, CI-2, DC-2, DC-3|7|Windows·Wayland 실기 미가용은 환경 대기. macOS는 지원 판정까지. 서명·스토어·릴리즈 금지.|

실행 단위 합계 **66조각**(M1 16 + M2 14 + M3 11 + M4 10 + M5 8 + M6 7). 종전 55는 초기 추정으로, 이미 수행한 M1-9·M3-1 분할과 M3-6b·파일 열기 수정 조각을 포함하면 기존 61이었다. 이번에 M2-V5b와 M1-V1~V3 4조각을 더하고, M6-5를 Windows·Wayland 2조각으로 나눠 61→66으로 맞췄다. 2026-11-07 문서 정리 시점에 완료 이력 35, 후속 31이며 후속에는 환경 대기가 포함된다. **RC-A**: M1·M2 기능 완료와 GUI 보완 완료 후(30조각, 로컬 재생 미리보기; M1 자체는 이미 닫힘). **RC-B**: M6 완료 기준을 실제 증거로 채운 뒤(정식 후보). 릴리즈와 SDK 태그 실행은 별도 사용자 지시로만 한다.

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
- 실행 단위 보정: M1-9는 9a(컨트롤러)·9b(단추·단축키)·9c(저장)로 분할했고, 파일 열기 비동기화·로딩 표시 수정(KB-003)을 포함해 기존 기능·수정 13조각이다. M1은 2026-11-05 닫힘을 유지한다.
- GUI 확인 보완(기능 재개 아님, 각 1실행): M1-V1 이어보기 토스트의 실제 표시·입력 방해 여부(선행 M1-6), M1-V2 ASS 또는 텍스트 자막과 컨트롤 겹침 실화면(선행 M1-8), M1-V3 수동 건너뛰기 단추·단축키·구간 지우기 실화면(선행 M1-9c). 실제 플레이어 조작·시안 대조·스크린샷과 차이 판단을 남겨야 완료다. 목업 재생성은 하지 않는다.

### M2 플레이어 UI

- 선행: M1-2~M1-4(메뉴에 넣을 명령).
- 완료 기준: (1) PotPlayer 기본 단축키 표(문서화된 목록)의 항목이 keys 테스트로 검증된다. (2) 우클릭 메뉴에서 열기·재생·자막·오디오·화면 명령을 고를 수 있다. (3) 재생목록 패널이 시안의 회차 목록 패턴(현재 항목 표시·도킹)을 따른다. (4) 설정 창이 같은 제목 줄 규약을 쓴다. (5) 기능 병합된 UI 조각은 `.spec/artifacts/design/` 시안과 실화면을 대조·개선한 뒤에만 시각 완료로 표시한다(visual-review.md). 시안 묶음의 일관성이 낮을 수 있다(2026-11-06 원 님). 1:1 대조는 하되 시안 간 불일치·장식·오류는 복제하지 않고, 이미 정한 토큰·공용 컴포넌트·화면 규칙을 우선해 개선한다. 새 팔레트·목업 재생성은 하지 않는다(보류: UI 시안 작업 재개). M2 닫힘에는 (1)~(5)에 더해 M2-8 창·트레이 아이콘 연결·실화면 확인과 M2-V5b 설정 창 실화면 확인이 필요하다. DOM/CSS·마운트 확인만으로 (5)를 충족하지 않는다.
- 조각 순서(기능): M2-1 단축키 표 설계 ADR과 탐색·시크 단축키, M2-2 우클릭 메뉴 골격, M2-3 메뉴에 M1 명령 연결, M2-4 재생목록 패널 현재 항목·도킹 정합, M2-5 재생 정보(OSD) 표시, M2-6 설정 창 사용자 정의 제목 줄(UI-5), M2-7 남은 wry 의존 정리 판정(UI-6), M2-8 창·트레이 아이콘을 마스코트 파생 자산에 연결.
- 조각 순서(시각 검증·개선, 기능 병합 후·화면별 1회 실행 크기): 각 조각은 실화면 대조와 함께, 시안 간 불일치가 있으면 공용 규칙 쪽으로 개선한다(목업 재생성 없음). M2-V1 단축키·탐색 피드백(M2-1, 시크·탐색 키에 보이는 반응만·시안에 없는 장식 추가 금지), M2-V2 우클릭 메뉴(M2-2+M2-3, 배치·그룹·항목을 공용 메뉴 규칙과 대조), M2-V3 재생목록 패널(M2-4, 회차 목록 헤더·행·현재 강조·도킹), M2-V4 OSD(M2-5, 하단 토스트 시크·음량 문구·위치), M2-V5 설정 창 제목 줄(M2-6, aw-window-chrome 대비). M2-7은 UI 크롬 없음(판정만). M2-V5(`00c9df38`)는 CSS·마운트 정합까지만 완료한 이력으로 유지하고, M2-V5b에서 실제 설정 창의 제목 줄·드래그·최대화/복원·min/max/close와 본문·포커스를 확인한다(선행 M2-6·M2-V5). 실화면을 볼 수 없으면 시각 완료로 표시하지 않는다. M2-8은 ADR 20261031111316에 따라 기존 마스코트 파생 자산만 창·트레이에 연결하고 실제 표시를 확인한다(새 시안·목업 재생성 없음).
- 후속 후보(이번 조각에 포함하지 않음): 전송 막대 고정 속도 표시, 중앙 OSD, 설정 창 가장자리 리사이즈, 메뉴 키보드 탐색.

### M3 호스트·플러그인 경계

- 선행: 없음(M1과 독립, 순서상 뒤).
- 완료 기준: (1) 재생 스트림 호스트가 플러그인 선언 범위 밖이면 거절되는 테스트가 있다. (2) 비밀 값이 OS 자격증명 저장소 포트를 거쳐 저장된다(Linux Secret Service 어댑터, 실패 시 메모리 저장소로 명시적 대체). (3) 브라우저 캡처로 resolve를 위임하는 설계가 ADR로 정해지고 첫 경로가 계약 테스트를 통과한다. (4) 호스트 소스에 사이트별 용어가 없다(r-src lint).
- 조각 순서: M3-1 플레이어 도메인 정책 자리표시자 교체(PG-5), M3-2 자격증명 저장소 포트 계약 테스트, M3-3 Secret Service 어댑터(ST-2), M3-4 브라우저 resolve 위임 ADR(PG-8), M3-5 위임 첫 경로(목 엔진), M3-6 장기 실행 플러그인 워커 설계(PG-2 후속), M3-7 외부 플러그인을 재생목록 프로토콜에 맞추는 점검(PG-10, 저장소 밖; 2026-11-06 원 님 실사이트 확인 허용), M3-8 플러그인 SDK 문서 정합(태그 생성·릴리즈·게시 없이 현재 계약·예제·문서만 맞춘다).

### M4 제작 기능 기반(CLI 우선)

- 선행: M1(재생 경로), 추론 실호출은 P7 승인.
- 완료 기준: 구현 조각은 기존 항목의 Linux CLI 경로가 Unsupported가 아니고 fixture 기반 테스트가 있다. OCR는 목 계약, whisper는 실측 준비, 피사체 분할은 첫 설계까지만 완료 범위다. M4-1~M4-10은 개별 실행 가능하며 P7을 전체 선행 게이트로 삼지 않는다. 실제 클라우드 호출·과금·실모델 추론·모델 다운로드는 승인 전 금지한다. M4-9는 재현 절차·fixture·측정 항목 준비까지만 하며 실측하지 않는다. 자동 검사는 AI 품질에 대한 사용자 GUI 승인을 대신하지 않는다.
- 조각 순서: M4-1 Jobs submit·cancel·retry CLI(ST-5), M4-2 프로젝트 export(ST-6), M4-3 스냅샷 Linux 경로(SB-3), M4-4 프레임 캡처 원격 입력 거절 정리(SB-3), M4-5 카드 부분 실패 정리(SB-4), M4-6 숏폼 회전·VFR 첫 단계(SB-4), M4-7 자산 재import(SB-4), M4-8 OCR 포트 목 계약(SB-6), M4-9 로컬 whisper 실측 준비(ST-4, 실측은 승인 후), M4-10 피사체 분할 첫 단계 설계(SB-5).

### M5 구조·플랫폼 정리

- 선행: 없음. 다른 마일스톤 사이의 빈 실행에 끼워 넣는다.
- 완료 기준: 구현 조각은 allowlist 줄 수·어댑터 밖 cfg 수의 전후 수치와 동작 불변 검증을 기록하고 신규 위반 0을 지킨다. 설계·판정 조각은 수치 감소를 가장하지 않고 GPU 렌더·OS 격리(sandbox)·provider의 범위와 미지원 경계를 ADR로 정한다. 기존 범위 감축은 진행하되 allowlist 2~4단계 추가 범위 확장은 보류한다.
- 조각 순서(각 1실행, 기존 플랫폼 정리 범위만): M5-1 경로 선택 cfg 첫 조각(OS-5), M5-2 프로세스 선택 cfg 첫 조각(OS-5, 선행 M5-1), M5-3 IPC 호출부 플랫폼 경계 첫 조각(OS-5·SC-2, 선행 M5-2), M5-4 남은 Win32 surface·desktop host 경계 판정(OS-5·SC-2, 선행 M5-3; 실기 없는 ungate 금지), M5-5 기존 ADR 20261031110642 범위 allowlist 감축(SC-3, 추가 범위 금지), M5-6 GPU 렌더 경로 ADR(UI-8), M5-7 OS 격리(sandbox) 어댑터 ADR(PG-9), M5-8 Unix 브라우저 provider 판정(OS-4). DC-4(core 컨텍스트화·ID 이전) 구현은 사용자 결정 대기에 남긴다. 설계 범위도 확인하지 못했으므로 이번 정리에 임의 구현 조각을 더하지 않는다.

### M6 마무리·배포 준비

- 선행: M1 닫힘, M2 닫힘(기존 기준 (1)~(5)·M2-8·M2-V5b), M3-1~M3-8 완료. M3-8은 태그 없이 문서 정합만 한다. 승인된 일반 push로 CI가 실행되면 그 실제 결과를 읽어 기록하되, push를 CI 녹색으로 간주하지 않는다.
- 완료 기준: (1) Linux에서 가능한 공개 표면 정합·CI 로컬 재현·설치형 패키지 설계·마스코트 이벤트 포트 no-op·macOS 지원 판정은 각각 근거를 남긴다. (2) Windows 실제 창 스모크와 Wayland 리사이즈 실측, 원격 CI 녹색은 실제 실행 기록이 있을 때만 완료다. 환경이 없으면 환경 대기로 분리하고 건너뛰되 실패나 완료로 가장하지 않는다. (3) macOS 지원 판정 완료와 macOS 실기 검증 완료를 구별한다. (4) 환경 대기가 남으면 가용 범위 완료라고만 쓰며 M6 전체·다중 OS 검증·RC-B를 완료로 닫지 않는다. 서명·스토어 제출·태그·릴리즈·게시 권한은 포함하지 않는다.
- 조각 순서: M6-1 공개 표면 정합(DC-2), M6-2 CI 로컬 재현 점검(CI-1), M6-3 설치형 패키지 설계(CI-2, 게시 아님), M6-4 마스코트 이벤트 포트 no-op(DC-3), M6-5a Windows 실제 창 스모크(OS-2·UI-3), M6-5b Wayland 리사이즈 실측(UI-3), M6-6 macOS 지원·빌드 가능성 판정(OS-3, 미지원 결론도 근거와 한계로 기록). M6-5는 OS별 환경 대기가 다른 조각을 막지 않도록 5a·5b로 분리했다.

### 보류(사용자 지시 전에는 손대지 않음)

- 릴리즈 실행과 SDK 태그 `plugin-sdk-v0.1.0`(CI-4, 릴리즈와 함께).
- allowlist 2~4단계 추가 범위.
- UI 시안 작업 재개.
- 미디어 다운로드(SB-7).
- P7 실제 추론 호출·과금·실모델 실측(승인 전 준비·목 계약만).
- DC-4 구현 범위 결정과 그 밖의 사용자 보류 결정. M2-8 기존 마스코트 파생 아이콘 연결은 보류에서 제외한다.

### 실행 순서·예상 기간

- 우선 실행: M2-V5b → M1-V1 → M1-V2 → M1-V3 → M2-8 → M3-8 → M4 개별 조각. M5 개별 조각은 선행을 지켜 빈 슬롯에 진행하고, M6는 위 선행을 갖춘 후 OS별 환경 대기를 건너뛴다. 승인·환경 장벽은 해제하거나 완료로 가장하지 않는다.
- 정기 슬롯은 매일 22회(Asia/Seoul 10시·15시 제외)이며 처리량은 보장하지 않는다. 현재 후속 31조각 중 환경 대기 2조각, 가용 범위 후보 29조각이다. 모든 후보가 1회에 끝난다고 가정하면 29 ÷ 22 ≈ 1.3일이지만, 이는 테스트·빌드 락·분할·실화면 검증 시간을 무시한 산술 하한이지 납기 예측이 아니다.
- RC-A는 누적 30조각 중 완료 30(M1-V2·M1-V3·M2-8 포함, 2026-11-08)이다. 실제 검증 전 완료 날짜를 약속하지 않는다.
- 불확실 요인: 공유 빌드 락·전체 테스트·GUI 실행 가능 여부·미검증 CLI 상태·Windows/Wayland/macOS 환경·CI 결과·P7 승인·DC-4 결정. 실제 처리량을 관측해 다시 추정하며 환경 대기는 무인 완료를 보장하지 않는다.

## Record

- 2026-11-01: 초안. 미확인 18항목을 코드로 판정해 현황표 집계를 구현 37·부분 20·미구현 13·보류 2로 맞췄다.
- 2026-11-05: M1-7 viewing prefs (volume, mute, speed, auto-next) persist across runs (`09c159a9`).
- 2026-11-05: M1-8 styled (ASS/SSA) and picture subtitle tracks are drawn by the engine into the frame; text tracks stay on the overlay (`9fc048f8`).
- 2026-11-05: Manual skip ranges persist per local-file key and restore on reopen (UI criteria (5) through save); siblings are not copied (`46392c5d`). M1-10 still open — M1 not closed.
- 2026-11-05: M1-10 — product playback stays on `kinekino watch` → player-ui → libmpv; the old `kinekino-player` mpv-process IPC CLI stays experimental (ADR 20261105141813). M1 criteria (1)–(5) met on the product path — M1 closed.
- 2026-11-06: M2-1 — shortcut table ADR and seek/navigation keys (arrow seek steps, PageUp/Down, Backspace; skip keys confirmed). M2 not closed.
- 2026-11-06: M2-2 — right-click menu skeleton (pointer-anchored popup kept inside the window, five groups reusing existing commands, same close rules as the more menu). Visual verification completed by M2-V2 (`944e16c3`). M2 not closed.
- 2026-11-06: M2-3 — right-click menu wired to M1 commands (flat items under open/play/skip/subtitle/audio/screen; seek ±5/±30). Visual verification completed by M2-V2 (`944e16c3`). M2 not closed.
- 2026-11-06: M2-5 — seek and volume OSD on the existing bottom toast (`View.notice`); formats `{position} / {duration}`, `음량 N`, `음소거` (ADR 20261105182156). Transport-bar permanent speed label and centered OSD chrome deferred. Visual verification completed by M2-V4 (`8603658d`). M2 not closed.
- 2026-11-06: M2-6 — settings window product caption (`aw-window-chrome`); undecorate OS frame; drag, double-click maximize, caption min/max/close (ADR 20261105191533). Edge resize deferred. DOM/CSS consistency checked by M2-V5 (`00c9df38`); real settings-window verification completed by M2-V5b (`4cea001f`). M2 not closed.
- 2026-11-06: M2-7 — remaining wry kept as the Windows WebView2 browser-engine adapter (not product UI); eframe/egui product-UI deviation closed; moving player browser modules into kinekino-browser deferred (ADR 20261105201855). M2 not closed.
- 2026-11-06: M2-4 — playlist panel follows mockup episode-list pattern (heading `회차 목록`, current-item amber bar/play/text, windowed dock, fullscreen hide keeping open state, Esc/P; ADR 20261106105936). Visual verification completed by M2-V3 (`acfe8a61`). M2 not closed.
- 2026-11-06: M3-1 — player media/frame hosts admitted at runtime via AdmittedHosts (plugin media_hosts install; remaining hardcodes removed) (`ec74ce96`). M3 not closed.
- 2026-11-06: M3-2 — CredentialStore port contract expanded (key isolation, overwrite, delete-absent, Secret Debug redaction, empty keys InvalidInput); Session/File adapters pass the same harness (`e4e208e1`). OS Secret Service adapter still M3-3. M3 not closed.
- 2026-11-06: M3-3 — Linux Secret Service CredentialStore adapter (`SecretServiceCredentialStore`) meets `check_credential_store` on an unlocked collection (`d6c9b778`). App still uses Session store; product memory fallback not wired. Windows Credential Manager still out of scope. M3 not closed.
- 2026-11-06: M3-4 — browser-capture Resolve delegated as a host capability (`BrowserCaptureSession`); plugin never owns the browser; first path (mock engine + contract tests) is M3-5 (ADR 20261106032051). M3 not closed.
- 2026-11-06: M3-5 — first host BrowserCaptureSession path with mock engine and contract tests; plugin maps observations to StreamCandidate; NetworkInterceptor remains Allow/Block (ADR 20261106032051).
- 2026-11-06: M3-6 — long-running per-plugin media-source worker designed (OwnedFamily spawn, JSON-line serve loop, one in-flight request, idle/crash policy); implementation deferred (ADR 20261106051332).
- 2026-11-06: M3-6b — long-running per-plugin PluginWorker implemented (OwnedFamily spawn, SDK serve_loop, ProcessSource reuse, idle/crash/call-timeout; contract tests) (`1250a73b`). M3 not closed.
- 2026-11-06: M2 visual-verification slices added (M2-V1…M2-V5) after functional merges left 시각 미확인; M2 criteria (5) requires real-screen 1:1 before 시각 완료. M2 piece count 8→13; total ~55. Mockup regeneration still held.
- 2026-11-06: M3-7 real-site check allowed by 원 님 (2026-11-06); historical user-gate noted, not erased.
- 2026-11-06: M3-7 — external source plugin playlist protocol aligned (browse nesting + Item.kind container/playable; live check). Resolve media-host rejection left open. Plugin commit outside host repo. M3 not closed.
- 2026-11-06: M2-V* visual review treats mockup-to-mockup inconsistency as expected (원 님). Verification includes consistency-first improvements; do not copy conflicting mockup details or regenerate mockups.
- 2026-11-07: M2-V1 — seek/nav visual verify done (real player + mockups); text cue yields to bottom toast while notice is set; keep session clock and shared seek-thumb tokens (do not copy mockup zero-pad or missing thumb). M2 not closed.
- 2026-11-07: M2-V3 — playlist panel visual verify done (real player + mockups); panel/border/selected colors use product tokens instead of mockup hex; keep episode-list structure (heading, dock, amber current row). M2 not closed.
- 2026-11-07: M2-V4 — OSD toast visual verify done (real player seek/volume/mute); keep shared panel tokens with menu (no mockup translucent pill). M2 not closed.
- 2026-11-07: M2-V5 — DOM/CSS consistency and mounted markup checked (`00c9df38`); shared aw-window-chrome/--aw-surface-chrome and brand/context flex lifted into components.css. Correction: this is not real-window visual completion. M2-V5b remains required for settings-window interaction and screenshots. M2 not closed.
- 2026-11-07: M2-V2 — right-click menu real-player visual review completed (`944e16c3`): reservation 280×720, scrolling and 14px icons; keep shared kk-menu tokens instead of copying inconsistent mockup details. M2 not closed.
- 2026-11-07: Reorganized milestones for unattended continuation: proceed with existing icon assets, tag-free SDK documentation, mock/CLI production work and bounded structural cleanup; add M2-V5b and M1-V1~V3 real-window checks, separate individual production/structure/finishing slices and OS-specific environment waits. M1 remains closed; M2 remains open. Keep releases/SDK tags/publication, P7 live inference, allowlist stages 2–4 additional scope, mockup regeneration and media download held. Execution count corrected from the initial 55 estimate (61 existing units) to 66, RC-A 23→30.
- 2026-11-08: M2-V5b — real settings-window caption verify done (X11 with window manager): drag, double-click maximize/restore, min/max/close, body and input focus. Full prop commits on maximize (Meari rejects partial updates); shared `input:focus` uses `--aw-focus`. Keep product name, shared brand icon, and caption icon token; rounded OS chrome is mockup decoration. Button `:focus-visible` and WM-driven maximize sync remain open. M2 not closed.
- 2026-11-08: M1-V1 — resume notice real-window verify done (X11 with window manager): reopening a local file with a saved point shows `이어서 재생 m:ss` on the shared bottom toast, playback continues from the point, pause and seek work during the notice (seek replaces it), and app or window-manager close saves. Fix: the clock and seek bar kept the stale engine time until the restore seek landed; the restored time is now held for up to one second. M1 stays closed.
- 2026-11-07: skill: modootoday-free/product-strategy-plus/roadmap-gating | 적용: 선행 증거·실측 완료 기준·진행/보류 이유 분리 | 버림: 분기별 점수화·신규 후보 확장(기존 범위 재정리만).
- 2026-11-08: M1-V2 — subtitle and controls real-window verify done (X11 with window manager): with the transport shown the text cue sat flush on the seek chrome; the root now carries `kk-controls-up` and lifts `#subtitle` / `#notice` by `--aw-transport-height`, then returns them to the stage bottom when controls auto-hide. ASS engine path was exercised; syncing engine `sub-margin-y` to control visibility remains open. M1 stays closed.
- 2026-11-08: M1-V3 — manual skip real-window verify done (X11 with window manager): mark start/end, skip offer buttons, S shortcut, and clear worked; the text cue stayed under skip/seek toasts because `.kk-subtitle { display:flex }` beat `[hidden]`; added `.kk-subtitle[hidden]` and a regression test that scans overlays with author `display:`. Visual complete; M1 stays closed.
- 2026-11-08: M2-8 (window part) — the player window now carries the mascot-derived app icon on Linux/X11 (16–64 px set on `_NET_WM_ICON`); real-window check showed it in the panel task list and pager. Windows/macOS wait for a Meari window-icon option, and the tray icon needs a tray port design first, so M2 stays open.
- 2026-11-08: M2-8b — tray icon port and platform adapters designed (ADR 20261107231529): host `TrayPresence`, Linux StatusNotifier first, same mascot-derived app-icon assets, no implementation; Windows/macOS Unsupported; wiring and xfce4-panel verify deferred. M2 stays open.
- 2026-11-08: M2-8c — the player shows the mascot-derived app icon in the Linux system tray (StatusNotifierItem, tooltip `kinekino`, activating it raises the window; registration off the window loop, a missing tray host is reported once). Real-window check on xfce4-panel (X11): tray icon shown, minimize then activate restored the window. With M2-1…M2-8 and M2-V1…M2-V5b done, M2 criteria (1)–(5) and the window/tray icon check are met on Linux — M2 closed. Windows/macOS window and tray icons wait for platform adapters (M6 platform checks); Wayland tray is unverified.
- 2026-11-08: M3-8 — plugin SDK docs aligned with the current contract without a tag: crate docs and a doctest show `serve_loop` as the plugin entry (host keeps one long-running process per plugin; one-shot `serve` still restarts but is not recommended), `Item.kind` container/playable, and path or git-rev dependency until a tag exists; extraction SoT records S0–S3 done and S5 held (`4d888796`). M3 not closed: product memory fallback for the credential store is not wired.
- 2026-11-08: M4-2 — project storage opens on Linux (`--project-root`) and `projects.export` writes one fixed stored revision as a JSON document to a new absolute path; existing files are never replaced (create-only, `revision_conflict`), with CLI fixture tests for errors and collisions (ADR 20261108112026, `484a3dd1`). Explicit overwrite/rename policy and multi-item export remain. M4 not closed.
- 2026-11-08: M4-3 — `production.export_snapshot` is already wired on Linux (fixed-SHA local PNG, explicit pixel crop, create-only output, no FFmpeg). CLI fixture tests now lock schema, capabilities, invalid payloads, unsupported context options, remote or relative path rejection, overwrite and stale-hash conflicts, and CRC-damaged sources (`233e15a4`). Frame-capture remote-input rejection remains (M4-4). M4 not closed.
- 2026-11-08: M4-4 — `production.capture_frame` keeps refusing remote input on Linux: URLs and protocol prefixes, UNC and relative paths, playlists, playlists disguised as MP4/MKV/WebM (including symlinks) and remote output paths fail with nothing published, and a local listener confirms no connection is opened. Locked with CLI fixture tests, no product code change (`f410430b`). M4 not closed.
- 2026-11-09: M4-5 — card bundle partial failure, cancellation and explicit retry stay on the shared fixture path: a mid-bundle render error keeps its original code and publishes nothing, cancel is acknowledged without writing the in-flight card, verification and existing-output failures stop before publication, and retry uses a fresh staging under the original Jobs attempt identity. The Linux CLI refuses card host and export commands in this build without writing files or claiming quality approval. Locked with application fixture tests and CLI contract tests, no product code change (`43196174`). The native bundle renderer remains unavailable here. M4 not closed.
- 2026-11-09: M4-7 — published asset source archives can be imported through the Linux CLI as a new draft. `asset_sources.import` is wired on the existing archive service, schema, capabilities, and SDK. Export then import creates a new document identity at revision 1 from original archive bytes, without review admission or quality approval; reusing the original id, omitting mask acknowledgement, and a timeout field are refused, and a second import conflicts. Locked with CLI fixture tests (`45e994d6`). Windows project-root unblocking remains out of scope. M4 not closed.
- 2026-11-09: M4-6 — rotation and variable frame rate first step is locked on Linux with locally generated fixtures. Frame capture through the CLI reports a display-matrix rotation (90, 270 stored as -90, 180 stored as -180) without applying it, keeping stored dimensions and the unrotated pixels, and on a variable frame rate source returns the first original frame at or after the request from source timestamps. Clip source inspection reports the rotation and the candidate encoder refuses rotated sources as unsupported before encoding; an unrotated variable frame rate source keeps original timestamp membership and encodes to the preset's constant rate. Test-only, no product code change (`4c49e234`). Applying rotation and HDR remain out of scope. M4 not closed.
- 2026-11-09: M4-8 — the OCR executor contract is locked against the in-process mock: each sampled frame is sent once with only its image bytes and equal texts merge into one timed span; a backend failure or cancellation stops at that frame, keeps the finished spans and charges nothing for the failed call; a missing OCR capability is unsupported and a wrong output task is a protocol failure; consent and the cost confirmation gate stop before any upload. Frame times must now be finite and non-negative and the sampling interval finite, refused before any call (`66e6b75b`). No model or network call; real OCR backends remain behind the inference approval gate. M4 not closed.
- 2026-11-09: M4-9 — local whisper.cpp measurement preparation is in place without running a model: a generated 16 kHz mono PCM16 format fixture, the required measurement items (character error rate marked as needing a later licensed speech fixture), and a reproduction procedure that does not download weights. `assess_prep` reports artifact readiness without setting `measured`, and `start_measurement` stays unsupported until explicit approval (`81a43259`). Prep is not a completed measurement. M4 not closed.
- 2026-11-09: M4-10 — subject segmentation first step is designed, not implemented (ADR 20261108211212): one frame and one mask in, a same-size single-channel or alpha mask out with larger values meaning subject, damaged or resized responses are protocol failures and uncharged, coverage is a plain pixel ratio and never a quality score, and results stay unreviewed until a person approves them in the GUI. `production.segment_subject` remains unsupported; tracking, brush corrections, a real adapter behind the inference approval gate and the projection package are follow-ups. All ten M4 slices are now done within their stated scope, but M4 is not marked closed until its criteria are checked against the code; no model call was made (`a568af37`).
- 2026-11-09: M5-1 — the first path-selection platform cfg slice removes the Windows-only gate from the local video selection rule test in the player domain: the rule (absolute path, known video extension, no programs, playlists, relative paths or URLs) is now checked on every target using the OS temp directory as the absolute root, and the matching structure allowlist entry is dropped. Platform cfg outside adapter crates went from 176 in 81 files to 175 in 80 files; behaviour is unchanged (`36700c83`). Native picker dialogs and the remaining host cfg stay for later slices. M5 not closed.
- 2026-11-09: M5-2 — the first process-selection platform cfg slice removes the Windows-only gate from the compile-time check that the process supervisor holding an owned process family is `Send`; the owner type is platform-neutral, so the check now runs on every target and the matching structure allowlist entry is dropped. Platform cfg outside adapter crates went from 175 in 80 files to 174 in 79 files; behaviour is unchanged (`bd945c67`). Windows-only process family contract tests and real Windows runs stay for later. M5 not closed.
- 2026-11-09: M5-3 — the first slice at the local-host playback control call site removes the Windows-only gate from the test that checks the native playback control provider's capability and schema entries agree without claiming engine readiness; the provider channel is platform-neutral, so the test now runs on every target and the matching structure allowlist entry is dropped. Platform cfg outside adapter crates went from 174 in 79 files to 173 in 78 files; behaviour is unchanged (`3bf78a50`). The named-pipe host transport tests and real Windows runs stay for later. M5 not closed.
- 2026-11-09: M5-4 — the remaining Win32 surface and desktop host gates were assessed without code changes (ADR 20261109021423, `091e2e5a`). The mpv Win32 surface, the player native window assembly and presentation, and the Windows branch of the desktop host body stay gated until a Windows desktop check; the playback and provider hosts depend on the WebView2 browser engine adapters and move with OS-4; the job read assembly and the execution host were found portable and are left as separate test-first follow-ups. The structure allowlist is unchanged. M5 not closed.
- 2026-11-09: M5-5 — the structure allowlist was checked against the scope already allowed by ADR 20261031110642 without code changes. That scope (stage 1) only drops entries whose violations are gone; `cargo xtask lint-structure` reports 348 violations, all allowlisted, 0 new and 0 fixed, so no entry can be dropped within it. The allowlist stays at 348 entries (207 file-and-dir, 78 r-cfg, 37 file-lines, 26 flat-dir); further reduction needs the held stages 2 to 4. M5 not closed.
- 2026-11-09: M5-6 — the GPU render path for the player window was recorded as a decision without code changes (ADR 20261109041206, `914e6f17`). The first stage only selects Meari's GPU window renderer, through a published Meari release with `window-gpu`, as its own dependency upgrade; video keeps the RGBA `VideoSurface` port, and a GPU video texture is a later port with its own record. The choice (CPU by default, GPU, or GPU with a software adapter for diagnosis) is a presentation option with no automatic CPU fallback, the window loop never waits without bound for GPU completion, and the path counts as better only when the delay from a resize to the first frame at the new size goes down. No GPU timing exists yet, since the dev host has only a software adapter. M5 not closed.
- 2026-11-09: M5-7 — the OS sandbox for plugin processes was recorded as a decision without code changes (ADR 20261109051610, `a72cdae6`). A platform-neutral sandbox policy rides on the existing process ownership spawn, each OS adapter reports file, network and system-call guarantees as enforced, advisory or unsupported, and a plugin spawn fails visibly when a requested guarantee is not enforced unless the user grants that plugin an unsandboxed run; there is no automatic fallback. Document calls run without network, source workers keep network in the first stage, both launch paths move onto the process port, and the signature and hash check before spawn is its own slice. Linux (landlock and seccomp) and Windows (AppContainer inside the Job Object) are in scope; macOS reports unsupported. M5 not closed.
