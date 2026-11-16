---
id: sot:kinekino/project-status
title: kinekino 현재 상태와 우선순위
status: active
created: 2026-10-29
updated: 2026-11-16
verify: manual
---

# kinekino 현재 상태와 우선순위

## Rule

이 문서는 kinekino의 현재 상태를 한 곳에서만 적는 정본이다. 다른 문서는 상태를 복제하지 않고 이 문서를 링크한다. 기준 시점은 2026-11-16, `main` `46496b3`이다. 마일스톤 정의·완료 기준·조각별 기록은 [[plan:kinekino/20261031153240]]에, 기능 인벤토리(#)와 단계(P0~P9) 정의는 [[plan:kinekino/20261029151512]]에 있고 여기에는 상태만 적는다. 인벤토리 표의 행별 판정은 2026-10-29 시점 그대로라 현재 상태로 읽지 않는다.

- **우선순위:** 재생 핵심 → 플레이어 UI → 호스트·플러그인 경계 → 제작 기능과 구조 정리 → 마무리(로드맵 원칙). 리팩터링은 lint 예외 확대·허용치 완화·테스트 skip 없이 allowlist를 실제로 줄이는 방향으로만 한다.
- **구분:** 조각 병합, 실화면 확인, 마일스톤 닫힘은 서로 다른 판정이다. 아래에서 "완료"는 기록된 근거가 있는 것만, "미검증"은 실행 기록이 없는 것이다.

## 마일스톤 상태

|마일스톤|상태|남은 것|
|-|-|-|
|M1 재생 핵심|닫힘(2026-11-05). 실화면 보완 M1-V1~V3 완료(Linux X11)|엔진 자막 들어올림(`sub-pos`, 2026-11-16 코드·가짜 엔진 테스트) 실화면 확인|
|M2 플레이어 UI|닫힘(2026-11-08, Linux X11). 단축키·우클릭 메뉴·재생목록 패널·OSD·설정 창 제목 줄·창/트레이 아이콘|Windows·macOS 창·트레이 아이콘(플랫폼 어댑터 대기), Wayland 트레이 미확인, 버튼 `:focus-visible`, WM 최대화 동기화 실화면 확인(코드·테스트 2026-11-16)|
|M3 호스트·플러그인 경계|열림. M3-1~M3-8 조각 완료. 자격증명 저장소 제품 배선(OS 저장소 우선, 사용 불가 시 세션 저장소로 명시적 대체·보고) 완료(2026-11-16)|Windows Credential Manager·macOS Keychain 어댑터 없음(두 OS는 세션 대체로 동작), 실제 데스크톱 keyring에서 GUI 토큰 저장 확인 미실시|
|M4 제작 기능(CLI)|열림. M4-1~M4-10 조각이 각 범위 안에서 완료|완료 기준과 코드 대조 판정(2026-11-16: 카드·자산·클립 프로젝트 저장소, 제안 저장소 `--proposal-root`, 모델 목록 `--model-root`, 카탈로그 `--catalog-root`, 화자 문서, 리뷰 저장소 `--review-root`를 Linux CLI에서 열어 Unsupported 해소. 과거 증거 검증기·실행 호스트 `--serve-execution`·카드 리뷰 내보내기·결합 호스트 `--serve-playback/native/provider/review`는 Windows 전용 유지, 근거와 소유자 결정은 이슈 #30). 피사체 분할은 설계만(`segment_subject` unsupported), whisper는 실측 준비만(실측은 승인 후)|
|M5 구조·플랫폼 정리|열림. M5-1~M5-8 기록(M5-4~M5-8은 판정·ADR만)|allowlist 2~4단계(보류)와 ADR 후속 구현(GPU 렌더, 플러그인 OS 격리, Unix 브라우저 provider: 2026-11-16 Chrome 어댑터 프로토콜 세션과 자식 세션까지, provider 연결은 남음)|
|M6 마무리·배포 준비|가용 범위만 완료. 전체 미닫힘|Windows 재생, 실제 GPU·DPI, macOS 실기, 패키지 빌드. 아래 OS별 상태 참고|

## 게이트 상태

- **원격 CI:** `main` `46496b3`에서 CI·Window smoke 수동 실행 모두 성공(GitHub Actions, 2026-11-14). Linux x86_64·arm64 전체 테스트 녹색(CI-1 기록: 1865 통과, 0 실패, 53 무시).
- **로컬 재현(2026-11-16, Ubuntu 24.04 컨테이너):** fmt·clippy `-D warnings`·`lint-structure`(348 위반 전부 allowlist, 신규 0, 어댑터 밖 플랫폼 cfg 78파일 173건) 통과. `spec check`(specfold 0.5.1) 오류 0, 경고 4,701(대부분 옛 경로 링크), baseline 오류 122. 전체 테스트 1827 통과·39 실패·53 무시였고 실패는 모두 환경 요인으로 판정했다: 13건은 `XDG_RUNTIME_DIR` 미설정(설정 후 재실행 통과), 26건은 mpv/libmpv 0.37의 `loadfile` 인자 비호환(0.38 이상 필요, CI가 Ubuntu 26.04를 쓰는 이유). 코드 결함은 확인되지 않았다.
- **CLI 표면:** 호스트 없이 `kinekino capabilities`를 실행하면 120개 명령 중 8개(`version`, `capabilities`, `schema`, `subtitles.convert`·`export_range`·`from_transcript`, `production.capture_frame`·`export_snapshot`)만 supported다. 나머지는 job host·`--project-root`·`kinekino watch` 같은 호스트 맥락에서 판정된다.

## OS별 상태

- **Linux:** 주 개발·검증 OS. X11 실화면 확인 완료. Wayland는 hosted 러너(헤드리스 sway 1.11)에서 창 매핑·가장자리 드래그·리사이즈 30/30 확인. 리사이즈 지연 p50이 실행마다 39ms·235ms로 달라 원인 미규명. 실제 GPU·분수 배율·다른 컴포지터는 미확인.
- **Windows:** windows-2022 러너에서 플레이어 창 표시·바깥 리사이즈 30/30(p50 48ms, p95 70ms)·가장자리 드래그 위임 확인. libmpv DLL 폴더 탐색 결함 KB-015는 `9467dbd`에서 수정. **재생은 미검증**(GPU 없는 러너에서 `libmpv produced no frame`). 실제 GPU·DPI 배율도 미검증.
- **macOS:** 지원 판정(ADR)만 있다. 프로세스 시작 스탬프(IPC 선행 조건)는 2026-11-16 `proc_pidinfo`로 구현했고 `aarch64-apple-darwin` clippy만 통과했다(macOS 실행 미검증). `aarch64-apple-darwin` 타입 검사 통과, CI 권고 실행에서 테스트 약 284건 실패(프로세스 시작 스탬프 부재가 첫 원인). 실기 검증 없음.

## 보류(사용자 지시 전에는 손대지 않음)

릴리스·태그(`plugin-sdk-v0.1.0` 포함)·crates.io 게시, P7 실제 추론 호출·과금·실모델 실측, 미디어 다운로드, allowlist 2~4단계 추가 범위, UI 시안 작업 재개, DC-4(core 컨텍스트화·ID 이전) 구현 범위.

## 원격

`origin`은 `modootoday/kinekino`이다. 열린 이슈는 저장소 설정 단계(#8)와 남은 소유자 결정·환경 대기 목록(#30) 두 건이다. `needs-human` PR은 자격증명 저장소(#12)·릴리스 페이로드 목록(#13)·프로세스 샌드박스 정책(#14)과 dependabot(#1~#5, 그중 `meari`·`meari-native` 0.3.0과 `windows-core` 0.100은 고정 버전(`=`)을 넘는 업데이트)이다. 2026-11-16부터 `main` 반영은 PR로만 한다(토픽 브랜치 → PR → Linux 필수 체크 통과 후 병합, [[adr:kinekino/20261116062843]]). needs-human 경로를 건드리지 않는 에이전트 PR은 자동 병합을 켠다. `main` 규칙은 `.github/rulesets/main-protected.json`(필수 체크 `lint`, `Public surface`, `test (ubuntu-26.04)`)이다.

## 알려진 문서 불일치

- (2026-11-16 해소) README와 사이트가 MCP 서버·`kinekino settings`를 "planned"로 적던 것을 실제 범위(MCP는 기본 꺼짐·읽기 전용 기본·호스트 없이 되는 명령만 도구, partial)로 고쳤다.
- (2026-11-16 해소) 사이트 설정 페이지의 Pretendard 표기를 고정 Noto 글꼴로, 설정 페이지·Meari UI·추론 동의와 비용 보호를 실제 확인 범위(partial)로 고쳤다. 토큰을 "OS 자격증명 저장소에 둔다"던 표기도 고쳤다: 앱은 지금 `SessionCredentialStore`(세션 메모리)만 쓰고 OS 저장소 어댑터는 연결 전이다. (2026-11-16 이후) `kinekino settings`가 OS 저장소를 먼저 열고, 사용 불가일 때만 세션 저장소로 대체하며 `settings credentials`로 보고한다. 문서도 그에 맞췄다.
- 2026-11-14 이력 재작성 이전의 커밋 해시는 이 문서에서 인용하지 않는다.

## Why

상태가 여러 문서(HANDOFF, 계획, 대화)에 흩어지면 서로 어긋난다. 이 문서는 2026-10-30 이후 갱신되지 않아 M1·M2 닫힘과 원격 CI·OS 스모크 결과가 빠져 있었다. 한 문서에서만 갱신한다.

## Verification

사람이 검토한다(verify: manual). 브랜치·커밋 해시는 `git log`로, 마일스톤 근거는 [[plan:kinekino/20261031153240]]의 Record로, 게이트는 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo xtask lint-structure`, `cargo xtask spec-check`, `cargo test --workspace`로 다시 확인한다.

- 2026-11-16 skill: modootoday/spec-authoring/sot | 적용: 규칙·이유·검증 구조 유지, 사람 검토(`verify: manual`) 명시, 관측 사실과 미검증 분리 | 버림: `check_cmd` 신설(상태 문서는 명령으로 판정할 불변식이 아님).
