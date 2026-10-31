---
id: plan:kinekino/20261031060318
title: "탐색 남은 일: 플러그인 재생목록·재생 연결"
status: active
created: 2026-10-31T06:03:18Z
updated: 2026-10-31T11:40:00Z
rel:
  governs: [crates/kinekino-player-ui/src/catalog.rs]
---

# 탐색 남은 일: 플러그인 재생목록·재생 연결

## Goal

작품 탐색에서 고른 작품을 회차 목록과 재생까지 이어, 로컬 폴더와 같은 시청 흐름을 소스 플러그인으로도 제공한다. 결정 배경은 [ADR](../adr/20261031060318-browse-plugins.adr.md).

## Steps

1. 항목 종류는 범용 두 가지뿐이다: `container`(하위 목록)와 `playable`(재생 항목). 시리즈·회차 해석은 플러그인 몫이며 호스트에는 그런 개념이 없다([ADR](../adr/20261031101741-plugin-playlists-generic-container-and-playable-items.adr.md)).
2. 재생 항목 선택 → 그 목록의 재생 항목을 그대로 큐(재생목록)로 → `resolve` → URL·헤더를 mpv에 전달. `media_hosts`·https(루프백만 http)·헤더 허용 목록 밖은 거부한다. 이전·다음은 큐 안에서 이동한다.
3. 플러그인 호출을 UI 스레드 밖으로 옮기고, 기다리는 동안 상태 문구를 보인다.
4. 검색(`search`) 입력과 결과 목록.
5. 12줄을 넘는 페이지의 스크롤 또는 화면 안 페이지 나눔.
6. 각 단계는 합성 플러그인 확장으로만 시험한다(실제 소스는 저장소 밖).

## Record

- 2026-10-31 1단계 전: 소스 목록 → 루트 목록 → 항목 목록, `더 보기`, 뒤로·Esc, 거절 안내까지 연결(`feat/browse-plugins`).
- 2026-10-31 1·2단계 완료(`feat/plugin-playlist-resolve`): 범용 container/playable, resolve와 허용 목록, 재생목록 변경 시 재마운트, END_FILE 사유 확인. 데스크톱에서 합성 플러그인으로 재생·다음 항목 확인.
- 2026-10-31 3단계 완료(`feat/async-plugin-calls`): browse·resolve를 작업 스레드로, "불러오는 중…" 표시, 늦은 응답 폐기. 데스크톱에서 3초 지연 합성 플러그인으로 창이 계속 반응함을 확인.
- 2026-10-31 4·5단계 완료(`feat/browse-search-scroll`, [ADR](../adr/20261031103033-browse-search-and-scrolling-listings.adr.md)): 열린 소스 안 검색(`Catalog::search`, 기본은 미지원 거절), 더 보기는 이어 붙이고 목록은 스크롤(최대 60줄), 입력 중에는 단축키 대신 글자 입력·Enter 검색·Esc 뒤로, 새 목록은 맨 위부터. 데스크톱에서 검색·Esc·더 보기·휠 스크롤 확인. 한글 IME 입력은 미확인.
