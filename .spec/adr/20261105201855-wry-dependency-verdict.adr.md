---
id: adr:kinekino/20261105201855
title: 남은 wry는 웹 재생 엔진 어댑터로 유지하고 제품 UI eframe 편차는 닫는다
status: active
created: 2026-11-05T20:18:55Z
updated: 2026-11-05T20:22:00Z
rel:
  governs:
    - crates/kinekino-player/Cargo.toml
    - crates/kinekino-browser/Cargo.toml
    - crates/kinekino-player/src/infrastructure/browser
    - crates/kinekino-player/src/infrastructure/browser_provider
  depends_on: [adr:kinekino/20261030050042, adr:kinekino/20261031020000]
---

# 남은 wry는 웹 재생 엔진 어댑터로 유지하고 제품 UI eframe 편차는 닫는다

## Context

로드맵 plan:kinekino/20261031153240의 M2-7(UI-6)이다. ADR 20261030050042는 제품 UI를 Meari로, WebView2(Wry)를 웹 재생 엔진 어댑터로만 쓰기로 했고, 이관이 끝날 때까지 `kinekino-player`가 eframe을 쓰는 것을 알려진 편차로 적었다. 그 뒤 일반 플레이어 창은 `kinekino-player-ui`(Meari·libmpv, ADR 20261031020000)로 옮겼고, `kinekino-player/Cargo.toml`에는 eframe·egui가 없다. 한편 Windows 전용 `wry`·`webview2-com`은 여전히 `kinekino-player`의 `infrastructure/browser`·`browser_provider`·일부 raw-window 경로에 직접 있고, `kinekino-browser`의 `webview2` 피처 어댑터는 스텁 수준(eval Null, interceptor unsupported)이다. M2-7에서 남은 wry를 지금 지울지, 엔진 어댑터로 유지할지, player→browser 이관을 이번 조각에서 끝낼지 판정한다.

## Decision

1. **wry는 웹 재생 엔진 어댑터로 유지.** Windows WebView2를 쓰는 호환 웹 플레이어·플러그인 브라우저 경로의 어댑터 의존으로 `wry`(및 필요 시 `webview2-com`)를 남긴다. 제품 UI 렌더러가 아니다. ADR 20261030050042의 엔진 한정 결정을 유지한다.
2. **제품 UI eframe 편차는 닫는다.** 일반 플레이어·설정 창의 제품 UI는 Meari(`kinekino-player-ui`, `kinekino-settings-ui`)가 정본이다. `kinekino-player` Cargo 의존에서 eframe·egui가 빠진 상태를 이관 완료로 본다. `presentation/web_ui`의 Meari TSX(스튜디오 등)는 wry UI가 아니며, 이 조각의 제거 대상이 아니다.
3. **player → `kinekino-browser` 이관은 이번 조각에서 하지 않는다.** `browser`·`browser_provider`의 관측·가로채기·오디오 경로는 아직 player에 있다. browser 크레이트 webview2 어댑터가 그 계약을 받지 못한 채 옮기면 동작이 빠진다. 이관은 별도 후속 조각(범위·테스트 먼저)으로 둔다.
4. **정리(이번 조각).** 이 ADR과 현황표·큐·로드맵 Record만 갱신한다. wry 의존 삭제, browser 모듈 이동, Cargo feature 재구성은 하지 않는다.

## Rationale

- 로드맵 M2-7 문구는 “남은 wry 의존 **정리 판정**”이며, 즉시 삭제가 완료 조건이 아니다.
- wry를 지금 제거하면 Windows 웹 엔진 경로가 깨진다. 유지가 기존 ADR·코드 관측과 맞다.
- 이관을 이 조각에 넣으면 한 실행 크기를 넘고, browser 어댑터 공백을 메우는 구현이 선행돼야 한다.
- skill: 조회 결과 무관(아키텍처 판정; 도메인 스킬 미적용).

## Consequences

- UI-6의 “eframe·egui 제품 UI 제거”는 이 판정으로 닫는다. wry가 player Cargo에 남는 것은 엔진 어댑터로 허용된 상태다.
- UI-6 남은 일: `kinekino-player`의 browser·browser_provider를 `kinekino-browser` 포트 뒤로 모으는 이관(후속). 그 전까지 player의 Windows wry 직접 의존은 유지한다.
- `local_file_picker`·mpv parent preflight가 `wry::raw_window_handle`을 쓰는 것은 후속에서 `raw-window-handle` 직접 의존으로 바꿀 후보이며, 이번 조각에서는 손대지 않는다.
- wry·WebView2를 제품 UI(Meari 창·캡션·메뉴)에 다시 쓰지 않는다.
