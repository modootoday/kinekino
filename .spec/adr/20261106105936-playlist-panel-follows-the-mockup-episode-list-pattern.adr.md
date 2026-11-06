---
id: adr:kinekino/20261106105936
title: Playlist panel follows the mockup episode-list pattern
status: active
created: 2026-11-06T10:59:36Z
updated: 2026-11-06T17:20:00Z
rel:
  governs: [crates/kinekino-player-ui]
---

# Playlist panel follows the mockup episode-list pattern

## Context

로드맵 plan:kinekino/20261031153240의 M2-4다. 완료 기준은 재생목록 패널이 시안의 회차 목록 패턴(현재 항목 표시·도킹)을 따르는 것이다. 정본은 design-asset-usage의 playback 시안 행과 visual-elements VIEW-08a/b, potplayer-ux의 P 토글이다. 기존 구현은 우측 도킹·amber 현재 행·play 표식을 이미 갖고 있으나, 헤더·aria·메뉴 문구가 `재생목록`이고, 전체화면에서 패널이 남을 수 있으며, Esc가 패널을 닫지 않고 P 단축키가 없다. 원 님(2026-11-06 t524s1972)이 시안 기준 권고안 진행을 승인했다.

## Decision

1. **라벨.** 패널 헤더·`aside` aria-label·닫기 단추·전송 막대 목록 단추·⋮ 메뉴 열기/닫기 문구를 시안의 `회차 목록`으로 통일한다. 호스트 명령 이름 `TogglePlaylist`와 영문 식별자는 그대로 둔다.
2. **현재 항목.** 시안·VIEW-08b대로 현재 행만 amber 좌측선(2px)·play glyph·amber 글자·중성 선택면(`--aw-selected-surface`)을 쓴다. 패널 배경·구분선도 시안 hex가 아니라 `--aw-surface-panel`·`--aw-border`를 쓴다(M2-V3). 번호와 제목은 데이터에 있는 값만 표시하고 제목에서 번호를 추측하지 않는다. 긴 제목은 기존 truncate 행 폭 계약을 유지한다.
3. **도킹.** 창 모드에서 패널이 열리면 stage에 `kk-docked`를 붙여 영상·시작 영역을 `--aw-episode-panel`(260)만큼 줄인다. 전송 막대는 전폭을 유지한다(기존 CSS). 전체화면에서는 시안대로 패널을 숨기되 `panel_open` 상태는 보존해 복귀 후 다시 보인다.
4. **열기·닫기.** P와 목록 단추·패널 닫기·⋮의 회차 목록 항목은 `TogglePlaylist`. Esc는 메뉴(기존) → 열린 패널 → 전체화면 순이다. 창 모드에서 패널만 열려 있으면 Esc는 패널만 닫는다.
5. **빈 상태.** 작품명(`work`)이 비면 소제목 행을 숨긴다(VIEW-08 후속). 목록이 비면(마운트 시 행 0) 본문에 `회차 정보가 없습니다`만 둔다. 로컬/원격을 가리는 문구는 호스트에 넣지 않는다. PotPlayer식 검색창·도구줄·새 팔레트는 시안에 없어 추가하지 않는다.

## Consequences

- `tests/controller/playlist_panel.rs`가 헤더 문구·현재 행 props·도킹 class·전체화면 숨김·Esc/P·빈 작품명 숨김을 검증한다.
- 시각 일치는 실화면 확인(M2-V3) 뒤에만 완료로 적는다. 색은 제품 토큰을 우선한다.
