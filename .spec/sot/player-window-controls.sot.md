---
id: sot:kinekino/player-window-controls
title: Meari 플레이어 창의 현재 조작 동작
status: active
created: 2026-10-31T05:51:10Z
updated: 2026-10-31T05:51:40Z
verify: [cargo test -p kinekino-player-ui]
rel:
  governs: [crates/kinekino-player-ui]
---

# Meari 플레이어 창의 현재 조작 동작

## Rule

- 창은 OS 장식 없이 열린다. 앱 제목 줄 배경을 끌면 창이 이동하고, 400ms 안에 두 번 누르면 최대화·복원한다. 가장자리·모서리의 보이지 않는 띠를 끌면 크기가 바뀐다(X11, `window/edges.rs`). 가장자리 띠 판정이 제목 줄보다 먼저다.
- 전송 막대 오른쪽 ⋮(더 보기) 버튼은 막대 위로 메뉴를 연다. 항목은 `파일 열기…`(Ctrl+O), 회차 목록 열기/닫기, 전체화면(F)이다. 다른 명령이나 다른 곳 클릭은 메뉴를 닫는다.
- Esc: ⋮ 메뉴가 열려 있으면 메뉴만 닫고 전체화면은 유지한다. 메뉴가 닫혀 있고 전체화면이면 전체화면을 끝낸다. 창 모드에서는 아무 일도 하지 않는다.
- 전체화면에서 재생 중이고 패널·메뉴·상태 문구가 없으면 입력 없이 2.5초 뒤 컨트롤을 숨기고, 포인터 이동·클릭·키·휠로 다시 보인다.
- 파일 열기에서 대화상자가 400ms보다 빨리 아무 결과 없이 끝나면(포털·zenity 없음) 설치 또는 `kinekino watch <파일>` 안내를 보인다. 시작 화면은 상태 줄에, 재생 중에는 영상 하단 토스트(`#notice`)로 4초 보이고 사라진다.
- 회차 패널은 짧은 회차 이름을 줄임 없이 보인다.
- 작품 탐색: 시작 화면의 `작품 탐색`은 설치된 소스 플러그인(`browse` 선언) 목록을 시작 화면 자리에 보인다. 소스가 없으면 상태 줄에 설치 안내만 보인다. 소스를 고르면 그 첫 목록, 항목을 고르면 그 항목의 목록을 한 화면 12줄로 보이고, `더 보기`는 다음 페이지, 뒤로 버튼과 창 모드 Esc는 한 단계 뒤로(첫 단계에서는 탐색 화면을 닫는다) 간다. 소스가 거절하면 목록은 그대로 두고 이유를 보인다. 파일을 열면 탐색 화면은 닫힌다. 결정은 [ADR](../adr/20261031060318-browse-plugins.adr.md).

## Why

시안의 플레이어 조작(메뉴·전체화면·자동 숨김)을 Meari 창에서 그대로 제공하고, 사용자가 대화상자가 없는 데스크탑에서도 다음 행동을 알 수 있게 한다. 결정 배경은 [ADR](../adr/20261031055110-player-window-controls.adr.md).

## Verification

- `cargo test -p kinekino-player-ui` (`tests/controller.rs`: 메뉴, Esc 순서, 대화상자 없음 안내·토스트 만료, 자동 숨김; `tests/episode_rows.rs`; `window/edges.rs` 단위 시험)
- `cargo test -p kinekino-app --test plugins_e2e` (합성 플러그인으로 `PluginCatalog` 목록·페이지)
- 데스크탑 확인: `launch.sh`로 띄운 앱에서 xdotool로 전체화면·⋮·Esc·포인터 이동.
