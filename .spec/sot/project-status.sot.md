---
id: sot:kinekino/project-status
title: kinekino 현재 상태와 우선순위
status: active
created: 2026-10-29
updated: 2026-10-30
verify: manual
---

# kinekino 현재 상태와 우선순위

## Rule

이 문서는 kinekino의 현재 상태를 한 곳에서만 적는 정본이다. 다른 문서는 상태를 복제하지 않고 이 문서를 링크한다. 기준 시점은 2026-10-30 23:30 KST, 로컬 `main` `6a6d503`이다. 단계 정의와 완료 기준은 [[plan:kinekino/20261029151512]]에 있고 여기에는 상태만 적는다.

- **우선순위(사용자 지시 2026-10-30):** Cross-OS 기반(P0) 위에서 단계별 완성. 리팩터링은 lint 예외 확대·허용치 완화·테스트 skip 없이 allowlist를 실제로 줄이는 방향으로만 한다.
- **병합 완료(로컬 main):** 구조 재편 M1a(`c698f40`)·M1b(`116d5a4`, `259e3a0`)·M2(`ec00ae5`)·M3 플러그인 호스트(`ec08d47`)·M4 카탈로그 플러그인(`47ddd71`)·M5 문서 분리(`c6a97da`); P1b 설정 CLI(`cd6e2e1`); P3 adblock 명령(`32b78bc`); P4 Linux snapshot(`afb815a`)·Windows 실측(`2542829`); P5 ASS 출력(`6b69409`)·Linux job host(`3be1df6`)·AI 실행기(`a723381`)·ASR 큐(`a1a7ea1`)·전사→큐 Jobs(`f7ff389`); P6 설정 페이지(`a9a495b`)·Windows 설정 창(`a820298`)·Meari 설정(`4d9fdf0`); P9 MCP 서버(`198ae78`); 문서·라이선스·CI 권고 작업(`e21efe8`~`80ece14`).
- **플러그인 SDK([[sot:kinekino/plugin-sdk-extraction]]):** `crates/kinekino-plugin-sdk` 0.1.0(serde·serde_json만, publish=false). S1 wire 추출·S2 `plugin.json` 매니페스트·합성 골든 fixture(core·host 양쪽 테스트가 읽음)·S3 `SourcePlugin`/`serve` 병합. S4는 비공개 저장소 대신 kinekino 밖 `/workspace/kk-sdk-consumer-check` 합성 소비자를 path·git(file://, main `6a6d503`) 의존 양쪽으로 빌드·실행해 검증(git 확인용 폴더는 정리). S5(push·`plugin-sdk-v0.1.0` 태그)는 사용자 승인 대기.
- **Meari:** `vendor/meari`를 Meari main `dab9c74`(crates/ 레이아웃)로 재벤더링(`9ba32e2`). Pretendard 제거, 글꼴은 Noto Sans KR/JP(OFL)만, 리비전 고정·SHA-256 검증 다운로드 캐시(`kinekino fonts fetch`), 저장소에 글꼴 미포함.

## 로드맵 단계별 상태

|단계|상태|남은 것|
|-|-|-|
|P0 기반 아키텍처|구현|lint-structure 372건 allowlist 고정·신규 0, Windows 컴파일 게이팅 완료|
|P1 core 컨텍스트화|부분|#32 완료(ae7c252): `kinekino-core::playback`(PlaybackControl 이전, `PlaybackBackend` 포트·FakeBackend·`check_backend` 계약, 재개 정책 #9). playback file-and-dir 11건 해소로 allowlist 372→361. 다음 컨텍스트 분리는 인벤토리 순서대로|
|P1b 추론 포트·HF 어댑터|부분|포트·HF 어댑터·목 서버·비용 원장·설정 CLI 구현. Unix 영구 토큰 저장소 `FileCredentialStore`(0600, `932493b`). OS 키체인 연동·실 HF 호출 미검증|
|P2 Linux 재생|부분|8e518bf: `kinekino-platform-unix::mpv::MpvBackend`(JSON IPC 소켓, `PlaybackBackend` 구현, `check_backend` 통과). 헤드리스 `--vo=null --ao=null` 시험 4건, ffmpeg 합성 4초 영상. mpv 탐색: 명시 경로→`$KINEKINO_MPV`→PATH, 바이너리 미커밋. 설치 절차 `.spec/guide/mpv-install.guide.md`. 이어보기 저장 `89e3e03`: `ResumeStore` 포트·`FileResumeStore`(JSON 원자 저장)·`ResumingPlayback`(열 때 복원, checkpoint/stop 저장·완료 시 삭제), mpv 왕복 시험. VideoSurface 포트 `108880c`, libmpv 렌더 어댑터 `132c0e8`, Meari 텍스처 `202c104`, 플레이어 창 병합. 남은 것: GPU 렌더 경로|
|P3 BrowserProviderPort|부분|adblock 명령·포트 병합, 실사이트 검증 보류(네트워크)|
|P4 제작 executor|부분|Linux snapshot·Windows 실측 병합, 품질 판정은 사용자|
|P5 자막·AI executor|부분|ASS 입출력·ASR·전사→큐·번역·OCR·화자분리 executor(목 계약 테스트) 병합(`6a6d503`). 실 HF 호출 미검증|
|P6 설정·GUI|부분|설정 페이지·설정 창(Meari) 병합. 플레이어 창(Meari, `kinekino-player-ui`): 4b 텍스처 `202c104`, 5 창·컨트롤러·`kinekino watch`(Linux·Windows 같은 경로), Ctrl+O(macOS Cmd+O) 파일 열기 `c3f31a0`, 전체 화면 재생 중 컨트롤 자동 숨김(2.5초) `3620096`, libmpv `sub-text` 자막 오버레이 `4dca584`. 남은 것: 기존 `kinekino-player` eframe·egui·wry 제거, 작품 탐색 연결, Windows 실제 창 스모크([ADR](../adr/20261031020000-player-window-meari-libmpv.adr.md))|
|P7 실 HF 평가|미착수|키·한도·동의 승인 필요|
|P8 Windows·macOS 최종 검증|부분|아래 Windows 경과. macOS는 CI 권고 설정만, 실측 없음|
|P9 MCP 서버|구현|`kinekino mcp serve` 병합|

## Windows

2026-10-29 보류(`kinekino-platform-unix`가 Windows에서도 빌드되어 check 실패) → 2026-10-30 사용자 지시로 재개. Windows 게이트 범위 확장(`3e77636`), 테스트 정리 병합(`1b029f5`): build-host에서 18개 크레이트 clippy `-D warnings` 통과, 테스트는 symlink 권한 오류 1314(개발자 모드/권한 없음, 환경 요인) 4건 외 통과.

## 원격

원격 없음. push·remote add·릴리스·퍼블릭 전환은 사용자 승인 필요.

## Why

상태가 여러 문서(HANDOFF, 계획, 대화)에 흩어지면 서로 어긋난다. 한 문서에서만 갱신한다.

## Verification

사람이 검토한다(verify: manual). 브랜치·커밋 해시는 `git log`로 확인한다.
