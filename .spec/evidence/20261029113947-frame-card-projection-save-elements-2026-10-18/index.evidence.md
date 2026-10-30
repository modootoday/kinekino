---
id: evidence:kinekino/20261029113947
title: 프레임·카드·투영·영상저장 확대 관측 — 2026-10-18
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 프레임·카드·투영·영상저장 확대 관측 — 2026-10-18

선택원본을Rust visual-review로2배확대해직접열었다. 원본은 `.spec/artifacts/design/v2/studio/`, 출력은 `runtime/verification/design-crops/` 기준. 좌표는crop영역이며내부요소정밀측정값이아니다.

| 원본 | x,y,w,h | 출력 |
|---|---|---|
| raw-frame.png | 415,665,600,190 | frame-selection-detail-2x.png |
| snapshot-refined.png | 1155,285,430,630 | thumbnail-layers-detail-2x.png |
| cards.png | 235,120,195,790 | card-rail-detail-2x.png |
| projection3d-refined.png | 815,150,320,240 | projection-mode-detail-2x.png |
| projection-package.png | 860,150,240,445 | projection-files-detail-2x.png |
| download-refined.png | 285,640,815,265 | save-tracks-detail-2x.png |

FRAME현재thumbnail은amberoutline/문자·PTS와이동/고정동작, 편집inspector에는font/color/outline/background/layer eye·lock·grip/두저장동작이있다. CARD는번호badge가있는세로thumbnailrail·선택둥근amber외곽선이다. PROJ는mode/XYZ/grid·plane와파일tree/선택assetmetadata가있다. SAVE는checkbox/유형glyph/selector/정보/언어의트랙표이며미확인과없음표현이분리된다.

현재범용editor의preview/table은이전용구조를재현하지못한다. [FRAME](../../player/presentation/studio/snapshots-visual.md), [CARD](../../player/presentation/studio/cards-visual.md), [PROJ](../../player/presentation/studio/projection-visual.md), [SAVE](../../player/presentation/studio/downloads-visual.md)에컴포넌트/상태/SDK책임을분리했다. 생성시안의systemfont/72px·파일용량·회차PTS는샘플이며실측/기능상한으로복사하지않는다.

이번은설계선행/참조확대작업이다. 새player실행·frame이동/레이어편집/카드저장/3Dimport/다운로드를실제로검증하지않았다. 전체요소좌표·색sampling과실제렌더1:1확대대조도미완료다. VM·DPI는변경하지않았다. 사용자확인용player종료는사용자가담당한다.
