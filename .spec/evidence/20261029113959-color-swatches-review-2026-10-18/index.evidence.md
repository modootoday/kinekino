---
id: evidence:kinekino/20261029113959
title: CARD/FRAME 공용 색상표 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CARD/FRAME 공용 색상표 검토

2026-10-18. 기준 cards.png의 폰트·색상 영역1140,635,430,80을 Rust visual-review로2배 확대했다. 출력은 runtime/verification/design-crops/card-color-swatches-20261018-r23-2x.png이며 직접 열어 원형7색·선택 외곽 링을 확인했다. crop 좌표는 추출 영역이며 색상 pixel 샘플링이나 요소 정밀 bounds 측정이 아니다.

## 구현

color_swatches::unavailable을 CARD/FRAME이 공유한다. 20 논리 단위 원형, 간격6·테두리1·7색을 모두 design_tokens에 연결했다. 실제 선택값이 없으므로 임의 선택 링을 만들지 않고 선택값 연결 필요와 색 이름 tooltip을 제공한다. 클릭 가능한 편집기나 실제 문서 색 값으로 표현하지 않는다. 카드의 Pretendard 표시는 기존 정본을 유지한다.

## r23 관측

fmt/test/clippy/build 통과(72 passed, 외부 의존5 ignored) 후 소유 가상 모니터3500,100·1440×900에서 r23(PID66868, runtime/review-runs/ui-all-pages-20261018-r23/player.exe)을 실행했다. VM·DPI·모니터 구성 변경은 없다.

CARD에서 원형7색이 한 줄로 표시됐다. inspector를 아래로 내리면 선택값 연결 필요가 보이고 저장 footer는 유지됐다. FRAME 썸네일에서 같은 색상표가6+1로 줄바꿈되고 label이 옆에 세로 중앙 정렬되는 차이를 발견했다. Color 역할만 label 아래 전폭 배치하도록 공용 inspector_field 레이아웃을 수정했다. r23은 수정 전 관측이며 캡션 닫기로 종료했다.

최종 전폭 배치의 실제 재검증·좁은 창과 편집 가능한 선택 상태·ring/hover/focus·다국어 긴 label·실제 CARD 색/폰트 및 레이어 read model은 미검증/미연결이다. 시안 전체 일치 완료로 표시하지 않는다.

## r24 전폭 재검증

최종 수정 후 fmt/test/clippy/build 통과(72 passed, 외부 의존5 ignored). player0개 확인 후 같은 가상 모니터·1440×900에서 r24(PID66660, runtime/review-runs/ui-all-pages-20261018-r24/player.exe)를 실행했다. FRAME 썸네일 색상 label 아래 원형7색이 한 줄로 놓이고 선택값 연결 필요가 그 아래 표시됐다. 이전6+1 줄바꿈은 이 폭에서 해결됐다. 고정 footer와 outline/shadow 미상 표시도 유지됐다.

탭 진입에서 egui request_discard 4 frames 경고가 나타났고, 다음 직접 화면 조회에도 남아 있었다. 이번에는 일시 경고가 사라졌다고 판정하지 않는다. 후속 레이아웃 재계산/재도색 조사 대상으로 남긴다. 좁은 창·모든 상태의 실기 검증은 계속 남아 있다. r24는 캡션 닫기로 종료했다.
