---
id: adr:kinekino/20261031020000
title: 플레이어 창을 Meari로 렌더하고 libmpv 프레임을 래스터 텍스처로 올림
status: active
created: 2026-10-31T02:00:00Z
updated: 2026-10-31T02:00:00Z
---

# 플레이어 창을 Meari로 렌더하고 libmpv 프레임을 래스터 텍스처로 올림

## Context

모든 UI는 Meari 기반이어야 한다. 기존 `kinekino-player`는 eframe·egui와 Windows WebView2 경로가 섞여 있었고, Linux에는 플레이어 창이 없었다(#29). 재생 엔진은 libmpv 소프트웨어 렌더 어댑터(`kinekino-libmpv`, `VideoSurface` 포트)로 마련됐다. 설정 창은 이미 Meari native([ADR](20261030040215-settings-window-meari.adr.md))다.

## Decision

- 새 crate `kinekino-player-ui`(Adapter)가 플레이어 창을 Meari로 렌더한다. TSX는 시안 정본표의 클래스(`aw-*`)와 토큰만 쓰고, 제품명은 `kinekino`와 `icons/brand.svg`로 표시한다. 시안 속 작품·서비스 이름은 예시 데이터로 보고 쓰지 않는다.
- 영상은 `<video id="video">` 요소의 레이아웃 크기로 `VideoSurface::render`를 호출해 RGBA를 얻고, Blitz 래스터 이미지(`SpecialElementData::Image`)로 올린다. 새 프레임(`frame_ready`) 또는 크기 변화가 있을 때만 올린다. GL 상호운용·WebView는 쓰지 않는다.
- 창은 Meari 준비형 드라이버(`NativeWindowPreparation` + `ChildWindowHost`)로 열어 창 동작 포트(최소화·최대화·전체화면)를 쓴다. 관찰된 최대화·전체화면 상태가 뷰로 돌아온다.
- 동작은 창과 분리된 `Controller<E: Engine>`에 있다. 같은 폴더의 동영상 파일이 회차 목록이다. 시험은 가짜 엔진으로 헤드리스 실행하고, 실제 libmpv·ffmpeg 시험은 도구가 없으면 실패한다(skip 없음).
- 진입점은 `kinekino watch [<file>] [--smoke] [--capture <png>]`이며 Linux·Windows가 같은 코드 경로다(`cfg` 없음).

## Consequences

- libmpv는 사용자가 설치한 라이브러리를 실행 중 읽는다. 바이너리 번들·정적 링크는 하지 않는다([설치 안내](../guide/mpv-install.guide.md)).
- 소프트웨어 렌더라 고해상도 전체화면은 CPU 비용이 크다. GPU 경로는 후속 과제다.
- 기존 `kinekino-player`의 eframe·egui·wry 화면 제거, 작품 탐색(소스 플러그인 연결), 단축키(Ctrl+O 등), 컨트롤 자동 숨김, 자막 오버레이는 후속 과제다.
