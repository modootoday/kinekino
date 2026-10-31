---
id: plan:kinekino/20261031060318
title: "작품 탐색 남은 일: 시리즈·회차·재생 연결"
status: active
created: 2026-10-31T06:03:18Z
updated: 2026-10-31T06:03:45Z
rel:
  governs: [crates/kinekino-player-ui/src/catalog.rs]
---

# 작품 탐색 남은 일: 시리즈·회차·재생 연결

## Goal

작품 탐색에서 고른 작품을 회차 목록과 재생까지 이어, 로컬 폴더와 같은 시청 흐름을 소스 플러그인으로도 제공한다. 결정 배경은 [ADR](../adr/20261031060318-browse-plugins.adr.md).

## Steps

1. 항목 종류 구분: 목록 항목이 분류인지 작품(series)인지 프로토콜에 표시하고, 작품이면 `series`로 회차 목록을 받아 회차 패널(`View::episodes`)에 싣는다.
2. 회차 선택 → `resolve` → `StreamCandidate`를 엔진에 싣는다. `media_hosts` 밖의 URL과 헤더는 거부한다. 이전·다음화는 `episode` 인접 정보로 이동한다.
3. 플러그인 호출을 UI 스레드 밖으로 옮기고, 기다리는 동안 상태 문구를 보인다.
4. 검색(`search`) 입력과 결과 목록.
5. 12줄을 넘는 페이지의 스크롤 또는 화면 안 페이지 나눔.
6. 각 단계는 합성 플러그인 확장으로만 시험한다(실제 소스는 저장소 밖).

## Record

- 2026-10-31 1단계 전: 소스 목록 → 루트 목록 → 항목 목록, `더 보기`, 뒤로·Esc, 거절 안내까지 연결(`feat/browse-plugins`).
