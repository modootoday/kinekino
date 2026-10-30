---
title: "플랫폼"
description: "하나의 Rust 코어와 OS 어댑터. 로드맵 계획 기준 OS별 상태."
ref: platforms
nav_order: 5
hero: { c: prose }
sections:
  - { c: platform-list }
  - { c: prose }
---

코어, 명령, CLI는 어디서나 같습니다. OS마다 다른 것은 어댑터(mpv 호스트, IPC, 자격 증명 저장소, GUI 셸)뿐입니다.

## OS별 상태

- **Windows**: 현재 앱(메아리 UI), 직접 확인한 네이티브 mpv 재생, 네임드 파이프 IPC, 스냅샷 내보내기. 최종 확인은 P8. `status:partial`
- **Linux**: 개발 기준 OS. CLI와 Unix 소켓 IPC 동작, mpv 재생은 P2, 스냅샷은 P4, GUI는 P6. `status:partial`
- **macOS**: 아직 빌드·확인 전, P8에서 빌드 확인. `status:planned`

## 확인 방식

Linux는 개발 중에 먼저 확인합니다. Windows와 macOS는 로드맵 끝(P8)에서 다시 확인하고, 배지는 그 확인을 거친 뒤에만 사용 가능으로 바뀝니다.

CI는 손으로만 실행합니다. Linux(lint, 테스트, 사이트 빌드)가 통과 여부를 정하고, Windows(컴파일 확인)와 macOS(테스트)는 실행마다 고르는 참고용이라 실패해도 보고만 하고 실행 전체를 실패시키지 않습니다.

CI가 실제 기기 확인을 대신하지 못하는 부분:

- **창**: 네이티브 창 생성, 크기 조절, 전체 화면, DPI 배율은 데스크톱 세션이 필요한데 러너에는 없습니다.
- **mpv**: 실제 미디어 디코딩과 오디오·영상 출력, 하드웨어 디코딩은 확인되지 않습니다. mpv가 필요한 테스트는 `#[ignore]`입니다.
- **입력**: 키보드, 마우스, IME, 끌어다 놓기는 실제 사용자 세션에서 옵니다.
- **GL**: OpenGL 컨텍스트와 렌더링은 GPU와 디스플레이가 필요하고, 러너에서는 렌더링이 없거나 소프트웨어로 대체됩니다.

배지를 바꾸기 전에 이 부분은 각 OS에서 직접 확인합니다.
