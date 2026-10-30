---
id: evidence:kinekino/20261029113740
title: 구간·피사체·숏폼 확대 관측 — 2026-10-18
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 구간·피사체·숏폼 확대 관측 — 2026-10-18

Rust visual-review로 선택 원본의 아래 영역을2배 확대하고 직접 열었다. 원본은 `.spec/artifacts/design/v2/studio/`, 출력은 `runtime/verification/design-crops/` 기준이다. 좌표는 crop영역이며 내부 요소 정밀 bounds 측정값이 아니다.

| 원본 | x,y,w,h | 출력 |
|---|---|---|
| skip-settings.png | 275,575,895,280 | seg-range-detail-2x.png |
| subject.png | 785,400,300,240 | obj-selection-detail-2x.png |
| short.png | 270,615,870,290 | clip-track-detail-2x.png |
| short-recommend-refined.png | 260,585,880,300 | clip-candidates-detail-2x.png |
| short-batch.png | 260,220,875,640 | clip-batch-detail-2x.png |

SEG는 OP/ED 다른 보조색+label·범위handle·독립playhead·확인/후보pill·행별수정/삭제가 있다. OBJ는 mask contour+반투명fill과 white square handle이 있어 단순box와 다르다. CLIP은 filmstrip/subtitle/audio가 별도track이며 같은ruler/playhead, 추천행은 thumbnail/근거/link/3동작, 일괄은 회차표/계획요약의 별도 구조다. 현재 범용 editor의 일반preview/timeline/table은 이 구체적 구성을 대체하지 못한다.

컴포넌트 제작 책임은 시각 근거를 보유한 **현재 UI 세션**이다. SDK 세션은 typed read model/명령/정책을 조율한다. [SEG](../../player/presentation/studio/segments-visual.md), [OBJ](../../player/presentation/studio/subjects-visual.md), [CLIP](../../player/presentation/studio/shorts-visual.md)에 제작할 조합과 공유 기하·상태를 정했다. OBJ/A2D의 선택색 의미, CLIP 3하위 화면을 공통화로 지워버리지 않는다.

이번은 문서 선행과 참조 확대 단계이며 새 제품 코드 변경·player 실행·실제drag/keyboard/추론/저장은 하지 않았다. 기존 source-level 차이를 실제 렌더관측으로 과장하지 않는다. VM·DPI는 변경하지 않았다.
