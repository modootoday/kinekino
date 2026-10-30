---
title: "재생"
description: "회차, 이어 보기, 건너뛰기를 갖춘 네이티브 mpv 재생. 모두 타입 명령으로 다룹니다."
ref: playback
nav_order: 10
hero: { c: hero, visual: player-mock, scene: playback }
sections:
  - { c: features, cols: 2 }
  - { c: prose }
---

kinekino는 mpv를 네이티브로 띄우고 모든 조작을 `playback.open`, `playback.seek`, `playback.set_volume` 같은 타입 명령으로 감쌉니다. GUI 버튼, CLI, 에이전트가 같은 명령을 보냅니다.

## 되는 것

- **열기, 재생, 일시정지, 탐색, 볼륨, 음소거**: mpv 호스트로. `status:partial`
- **회차**: 이전·다음 회차, 자동 재생 카운트다운. `status:partial`
- **인트로 건너뛰기**: 수동 건너뛰기 표시. `status:partial`
- **이어 보기와 환경설정**: 위치, 오디오·자막 트랙 기억. `status:planned`
- **브라우저 소스**: 카탈로그와 회차 페이지. `status:partial`

## OS별

재생은 Windows에서 직접 확인했습니다. Linux mpv 호스트는 로드맵 P2, macOS는 P8에서 확인합니다.

> [!IMPORTANT]
> 아직 공개 빌드가 없어 개발 트리 밖에서는 재생을 써 볼 수 없습니다.
