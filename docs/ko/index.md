---
title: "kinekino: 스크립트로 다루는 비디오 플레이어"
description: "Rust로 만든 크로스 OS 비디오 플레이어·스튜디오. 앱, CLI, AI 에이전트가 같은 타입 명령으로 다룹니다."
ref: index
nav_order: 0
hero: { c: hero, visual: player-mock, scene: playback }
sections:
  - { c: features, cols: 3 }
  - { c: split, visual: player-mock, scene: timeline }
  - { c: split }
  - { c: features, cols: 3 }
  - { c: platform-list }
  - { c: split, visual: player-mock, scene: settings }
  - { c: timeline }
  - { c: cta }
---

# 스크립트로 다루는 비디오 플레이어.

kinekino는 영상을 재생하고, 자르고, 자막을 붙입니다. 모든 동작이 타입이 있는 명령이라 앱과 CLI, AI 에이전트가 같은 방식으로 다룹니다.

[다운로드](download.md) [CLI로 시작하기](cli.md)

- Rust 워크스페이스, 크레이트 20개 이상
- 타입 JSON 명령 스키마
- 로컬 우선, 클라우드 AI는 동의할 때만
- MIT 라이선스

## 할 수 있는 일

배지는 로드맵 계획과 CLI의 `capabilities` 출력에서 가져왔습니다. 약속이 아닙니다.

- **[재생](features/playback.md)**: 네이티브 mpv 호스트로 열기, 재생, 탐색, 회차 이동. `status:partial`
- **[자막](features/subtitles-ai.md)**: SRT/WebVTT 변환, 구간 내보내기, 큐와 화자 편집. `status:available`
- **[클립과 스냅샷](features/studio.md)**: 프레임 캡처, 스냅샷 내보내기, 쇼츠 렌더를 작업으로. `status:partial`
- **[리뷰](features/studio.md)**: AI 제안을 반영 전에 수락하거나 거절. `status:partial`
- **[AI 자막](features/subtitles-ai.md)**: 음성 인식, 화자, 번역을 하나의 추론 포트로. `status:planned`
- **[에이전트](agents.md)**: 지금은 CLI, MCP 서버는 계획에 있음. `status:partial`

## 타임라인에서 클립 만들기

보면서 구간을 표시하고 쇼츠, 카드, 스냅샷으로 만듭니다. 렌더는 지켜보고 취소하고 다시 시도할 수 있는 작업으로 돕니다.

- 구간 초안과 쇼츠 렌더 `status:partial`
- PNG 프레임 캡처(FFmpeg 필요) `status:partial`
- 스냅샷 내보내기: 지금은 Windows, Linux는 P4 `status:partial`

[스튜디오 기능](features/studio.md)

## 에이전트를 위한 설계

모든 명령은 요청 ID, 리비전, 멱등 키가 있는 JSON을 주고받습니다. 스크립트나 AI 에이전트가 안전하게 호출하고 다시 시도할 수 있습니다.

```sh
kinekino capabilities --json
kinekino schema subtitles.convert
kinekino --input request.json --json
```

[AI 네이티브 표면](agents.md)

## 이렇게 씁니다

- **시리즈 보기**: 멈춘 곳부터 이어 보고 인트로 건너뛰기. `status:partial`
- **강연 자막 만들기**: 받아쓰고 화자를 나눈 뒤 직접 다듬기. `status:planned`
- **쇼츠 만들기**: 구간 표시, 렌더, 리뷰. `status:partial`
- **라이브러리 자동화**: 스크립트로 자막 일괄 변환. `status:available`
- **에이전트와 함께**: 에이전트가 편집을 제안하고 사람이 리뷰. `status:partial`
- **오프라인 작업**: 업로드 없이 로컬 모델만. `status:planned`

[모든 활용 사례](use-cases.md)

## 하나의 코어, 모든 OS

공용 Rust 코어 아래에 OS 어댑터가 있습니다. 개발은 Linux를 먼저, 현재 앱은 Windows에서 돌고, 마지막(P8)에 모든 OS를 다시 확인합니다.

- **Windows**: 현재 앱과 네이티브 mpv 재생, 사람이 직접 확인. `status:partial`
- **Linux**: CLI와 로컬 IPC 동작, 재생(P2)과 GUI(P6) 진행 중. `status:partial`
- **macOS**: 아직 확인 전, P8에서 빌드 확인. `status:planned`

[플랫폼 상세](platforms.md)

## 미디어는 내 것

원하면 추론을 로컬에서 먼저 돌립니다. 클라우드 모델은 동의가 있어야 하고 비용 한도 안에서만 씁니다. 필요한 구간이나 프레임만 올리고 영상 전체는 올리지 않습니다.

> [!NOTE]
> 설정 화면과 추론 포트는 계획 단계입니다(P1b, P6). 기본값은 이미 정해져 있고 설정 페이지에 있습니다.

[설정과 개인정보](settings.md)

## 로드맵

2026-10-30 승인된 순서입니다. 크로스 OS 기반이 먼저입니다.

1. **P0**: 크로스 OS 기반
2. **P1**: 코어 컨텍스트
3. **P1b**: 추론 포트와 비용 보호
4. **P2**: Linux 재생
5. **P4**: 스튜디오 실행기
6. **P5**: 자막과 AI
7. **P6**: GUI, 설정, 마스코트
8. **P9**: MCP 서버
9. **P8**: Windows·macOS 검증

[전체 로드맵](roadmap.md)

## 다운로드

아직 공개 릴리스가 없습니다. 고양이가 아직 짐을 싸는 중이에요. 빌드가 나오면 OS별로 여기에 올립니다.

[빌드 상태 보기](download.md)
