---
id: adr:kinekino/20261105182156
title: "재생 정보 OSD: 시크·음량 알림은 기존 하단 토스트"
status: active
created: 2026-11-05T18:21:56Z
updated: 2026-11-07T03:25:00+09:00
rel:
  governs: [crates/kinekino-player-ui]
---

# 재생 정보 OSD: 시크·음량 알림은 기존 하단 토스트

## Context

로드맵 plan:kinekino/20261031153240의 M2-5다. 속도·트랙·건너뛰기·이어보기는 이미 `View.notice`(하단 `kk-toast`)로 짧은 알림을 내고, 시크와 음량·음소거는 전송 막대만 바뀌고 알림이 없다. adr:kinekino/20261105151827는 시크 토스트를 M2-5로 미뤘다. 시안 studio 전송 막대에 고정 속도 표시가 있으나, 이번 조각은 키·명령 직후의 일시 알림만 다룬다.

## Decision

1. **채널.** 새 오버레이를 두지 않는다. 기존 `View.notice` + `NOTICE_FOR`(4초) + `#notice.kk-toast`를 OSD로 쓴다. 속도·트랙과 같은 자리·수명이다. 항목을 여는 중(`view.opening`)이면 시크·음량 OSD를 내지 않는다(여는 중 줄을 덮지 않음).
2. **시크.** `SeekBy`·`Seek`(슬라이더·Backspace 맨 앞)가 미디어에서 위치를 바꾼 뒤 `"{위치} / {길이}"`를 낸다. 시각은 `session::clock`(분:초, 1시간 이상이면 시:분:초). `Previous`가 현재 항목을 맨 앞으로 되감을 때도 같다. 미디어가 없거나 `duration ≤ 0`으로 no-op이면 알림 없음.
3. **음량·음소거.** `VolumeUp`·`VolumeDown`·`Volume` 뒤에는 `"음량 {0–100 정수}"`를 낸다(음소거 유지 중에도 단계가 바뀌면 같은 형식). `Mute`로 켜면 `"음소거"`, 끄면 현재 단계의 `"음량 N"`.
4. **이번 범위 밖(남은 일).** 전송 막대 고정 속도·상태 표시(시안 studio), 화면 중앙의 큰 OSD 크롬, 휠 음량. 하단 토스트 실화면(시크·음량 문구·자리·패널 토큰)은 2026-11-07 검증 완료 — 시안 반투명 필 대신 `.kk-menu`와 같은 `--aw-surface-panel`/`--aw-border`를 유지.

## Rationale

- 이미 검증된 토스트 경로를 쓰면 Meari prop·숨김·수명을 다시 설계하지 않아도 되고, 속도 알림과 자리도 같다.
- 시크는 막대만으로는 키 반복 중 새 위치를 읽기 어려워 위치/길이 한 줄이 필요하다. 음량은 막대가 있어도 전체화면 자동 숨김 중에 피드백이 비므로 같은 채널을 쓴다.
- 고정 전송 막대 표기는 시안 대조·레이아웃이 커서 별 조각으로 둔다.

## Consequences

- `tests/controller/seek_nav.rs`·`volume_speed.rs`가 시크·음량·음소거 알림 문구와 여는 중 억제를 검증한다. `osd_toast_css_uses_shared_panel_tokens`가 패널 토큰(메뉴와 동일 계열) 계약을 잠근다.
- SoT `player-window-controls`에 OSD 규칙을 한 블록 추가하고, adr:kinekino/20261105151827의 "시크 토스트·OSD는 내지 않는다(M2-5)"·남은 일 OSD 항목을 이 ADR로 갱신한다.
