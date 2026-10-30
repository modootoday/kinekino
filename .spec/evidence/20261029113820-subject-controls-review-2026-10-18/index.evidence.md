---
id: evidence:kinekino/20261029113820
title: 피사체 선택 의미·분석 방식 후속
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 피사체 선택 의미·분석 방식 후속

2026-10-18. [OBJ 계약](../../player/presentation/studio/subjects-visual.md)과 기존 확대 obj-selection-detail-2x.png를 다시 직접 확인했다. 원본 subject.png의785,400,300,240 영역2배 확대다. 청록색 범위/윤곽과 흰 사각 handle, 반투명 mask를 관측했다. 정확한 pixel 색 sampling은 미수행이며 토큰 색은 논리 설계값이다.

기존 OBJ preview는 A2D용 파란 bounds를 재사용했고 분석 방식은 일반 Select였다. 공용 segmented 탐지/분할/추적3칸과 미선택·비활성·연결 필요 표현으로 수정했다. 공통 square bounds 기하를 공유하되 SUBJECT_SELECTION 청록색 토큰으로 의미를 분리했다. 사각 handle7도 공용 sizing token으로 옮겼다.

실제 피사체 외형 contour·반투명 fill·track별 실패 interval·선택 목록 동기화·분석 명령은 미완료다. 장식 bounds를 mask 완성이나 SDK 분석 성공으로 표시하지 않는다. fmt/test/clippy/build와 실제 실행 결과는 후속 기록으로 구분한다.

## r30 실제 재검증

fmt/test/clippy/build 통과(77 passed,8 ignored). 공통 token 직접 사용으로 불필요해진 player_theme의 CANVAS_SELECTION 재노출 import는 제거했다.

player0개 확인 후 r30(PID114020, runtime/review-runs/ui-all-pages-20261018-r30/player.exe)을 소유 가상 모니터3500,100·1440×900에서 실행했다. OBJ에서 청록색 bounds·흰 사각 handle·미선택 탐지/분할/추적3칸·분석 방식 연결 필요·고정 footer를 확인했다. A2D로 이동해 파란 transform bounds·pivot이 유지됨을 확인했고, SUB OCR로 이동해 황색 테두리와 흰 원형 네 corner가 유지됨을 확인했다.

모든 장식은 명시된 DesignSample이다. 실제 mask·ROI·transform 편집 및 지원 옵션은 검증하지 않았다. 좁은 OBJ 창과 모든 선택/focus 상태도 남아 있다. VM·DPI·모니터 구성은 변경하지 않았고, 검토 후 캡션 닫기로 종료했다.
