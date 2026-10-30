---
id: evidence:kinekino/20261029113836
title: A2D 확대 관측 — 2026-10-18
status: active
created: 2026-10-29
updated: 2026-10-29
---
# A2D 확대 관측 — 2026-10-18

원본 `.spec/artifacts/design/v2/studio/asset2d.png`를 Rust visual-review로 crop하고 직접 열었다. 제품 GUI 캡처가 아니며 실제 실행 인수는 하지 않았다. [요소 계약](../../player/presentation/studio/assets-2d-visual.md)에 관측과 공통화 설계를 분리해 반영했다.

| 출력(runtime/verification/design-crops 기준) | 원본 x,y,w,h | 배율 | 관측 |
|---|---|---|---|
| a2d-tools-v2-2x.png | 285,115,180,530 | 2 | mode 버튼 radius, 도구 rail 선택 표시선/면, icon+label, 구분선·브러시/부드러움 slider+값 |
| a2d-layers-v2-2x.png | 285,645,890,265 | 2 | 선택 row와 비선택 row 중성 면 차이, eye/thumbnail/name/lock/grip, source 참조와 수직 구분선 |
| a2d-inspector-v2-2x.png | 1180,115,406,840 | 2 | source thumbnail/PTS, slider+numeric, pivot radio/XY, bounds XYWH, scale/color, 출력 체크와 하단 두 동작 |

crop 좌표는 검사 영역이다. 각 내부 요소의 실제 bounds·stroke·색을 자동 측정한 결과가 아니다. 공통 논리 치수는 요소 계약으로 정하고 실제 렌더와 비교해야 한다.

현재 소스를 확인한 차이:

- toolbar Fit은 zoom만 None으로 바꾸며 pan offset은 유지한다. 전체가 보이도록 복귀하는 계약과 다르다.
- Alpha mode는 선택 레이어의 컬러 RGBA를 표시한다. 실제 mask grayscale 표시와 동일한 것으로 검증하지 않았다.
- 마스크 tool 선택은 local view state이며 실제 브러시 편집이 아니다. brush/softness와 전용 inspector는 아직 미구현이다.
- layer visible/locked는 readonly 상태이며 false 상태의 독립 glyph·실제 toggle command는 없다. mask 전용 sample layer도 미제공이다.
- SDK bounds와 pivot은 canvas 원본px, pivot은 절대 좌표다. field-rules의 정규화 값과 모순되지 않도록 표현 단위를 명시해야 한다.

전체 원본 이미지에서 bounds8 handle·pivot·checker를 확인했으나 이번 crop은 도구/레이어/검사기에 한정했다. 현재 어두운 checker와 시안 밝은 checker, 선택 handle·pivot의 실제 크기·정렬은 새 player 렌더로 확인하지 않았다. 마지막 제품 검사는 64pass/5external ignored 및 fmt/clippy 통과였으며 시각 일치의 증거로 사용하지 않는다.
