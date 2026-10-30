---
id: evidence:kinekino/20261029113958
title: CARD 도구 모음 외곽 면 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CARD 도구 모음 외곽 면 검토

2026-10-19. CARD-01/05. 기준 `v2/studio/cards.png`의(249,122,866,62)를4배 확대했다. 파생물 `runtime/review-runs/card-toolbar-r133-4x.png`에서 하나의 둥근 외곽선 안에 무테 추가·복제·삭제·구분선·undo/redo가 놓인 것을 확인했다. r132의 개별 버튼 외곽선은 기준과 달랐다.

## 공용 변경

toolbar에 `unavailable_strip` 변형을 추가했다. PANEL 면·THIN 선·CONTROL radius·INLINE padding 안에 기존 Taffy 그룹을 넣는다. 내부 버튼은 frame(false)이며32높이/SVG20/간격8과 비활성·이유를 공유한다. CARD가 이를 사용한다. 기존 `unavailable_grouped` 소비자인 FRAME/CLIP 등은 버튼별 frame을 유지한다. 새 명령·가짜 활성 상태는 만들지 않았다.

## 실제 관측

r133 `runtime/review-runs/ui-card-toolbar-20261019-r133/player.exe`, 소유 PID99432를 기존 가상 모니터3500,100에서 실행했다. VM·DPI·모니터 구성 변경 없음.

- 1440×900 CARD: toolbar가 본문 유동폭 전체로 펼쳐지고 하나의 외곽선/면 안에 다섯 무테 동작과 구분선이 표시됐다. 하단 thumbnail과 canvas는 toolbar padding 증가에 따라 함께 아래로 이동했다. canvas 비율은 유지됐다.
- 760×480 CARD: 다섯 동작과 구분선이 한 줄에 들어가고 strip의 우측 끝까지 면/외곽선이 이어졌다. thumbnail strip과 겹치지 않았다. canvas 일부는 본문 스크롤 아래에 있으며 최초 viewport에 전체 canvas가 들어간다는 판정은 하지 않는다.
- 760 FRAME 원본 프레임: 본문을 스크롤해 이전 프레임/프레임 고정/다음 프레임/원본 비교 네 개별 테두리 버튼이 유지되는 것을 확인했다. FRAME 편집 탭·CLIP의 실제 재검토까지 확대하지 않는다.
- caption으로 소유 검토 창을 닫고 후속 CIM에서 player.exe/mpv.exe가 없었다.

## 검사·미검증

전체 cargo test 통과(lib81pass/1ignore 및 기본 통합·doc test), player 빌드 통과. 최초 fmt는 병행 local_peer/peer_token 편집의 포맷 차이로 실패했고 clippy는 peer_token/tests.rs의 미사용 import/mut 및 E0596으로 실패했다. 해당 소스를 수정하지 않고 소유 세션에 전달했다. 소유 변경 뒤 최종 `cargo fmt --check`와 `cargo clippy --all-targets --target-dir runtime/parallel-build/ui -j 1 -- -D warnings`가 통과했다. 테스트 후 UI Rust 변경은 없다.

기준의 원형 Plus·동작 간 상대 폭과 광학 간격·gradient·활성/hover/pressed·긴 번역 wrap은 남는다. toolbar 외곽 구조 수정이며 CARD 전체 시각 일치나 편집 명령 완료가 아니다.
