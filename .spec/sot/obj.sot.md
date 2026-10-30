---
id: sot:kinekino/obj
title: OBJ 요소별 시각 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# OBJ 요소별 시각 계약

[r136 실제 track 행 검토](../../../verification/presentation/subject-row-review-2026-10-19.md)에서1440/760 thumbnail·검토 상태·track ID·menu slot과 Empty 전환 뒤 행 제거를 확인했다. 추가/이름 편집/다중선택/메뉴 intent는 남는다.

OBJ-03/04 첫 목록 이관은 mask.subjects의 실재 track만 표시한다. 공용 SubjectRow는 원본 영역 thumbnail44×40·두 줄 식별/검토 정보·미연결 menu slot32를 둔 PANEL/radius5/여백6 행이다. 유일한 track이 현재 canvas에 표시된 경우 amber 외곽선으로 문맥을 나타낸다. 다중 track 선택/편집은 아직 연결하지 않으며 행 클릭으로 성공을 꾸미지 않는다. 사용자 별명이 없는 track에는 ‘이름 미지정’과 실제 track ID를 표시한다. thumbnail은 원본 polygon bounds의 표시용 crop으로, 분리된 alpha 산출물이나 모델 정확도를 뜻하지 않는다. 원본 allowlist·크기·polygon 범위 검증이 안 되면 thumbnail을 비운다. Empty와 Live 미연결은 별도 안내한다.

2026-10-19 SDK 소유 세션 확인: SubjectMask는 track_id/polygon_px/alpha_asset/review_state를 제공하지만 label·잘라낸 thumbnail·개별 source-frame ref는 없다. MaskSnapshot의 공통 source_asset/PTS/원본 크기는 사용할 수 있다. 일반 table row에는 track FK가 없다. 후속 목록 투영은 mask.subjects의 실제 track_id를 기준으로 하며, optional 사용자 label을 없는 상태로 보존하고 원본+polygon/alpha의 crop preview는 표시용 파생물로 구분한다. alpha_asset 자체는 full source-canvas alpha이므로 subject thumbnail로 간주하지 않는다. live mask=None/Unsupported를 빈 목록0으로 바꾸지 않는다.

[r134 보기·패널 검토](../../../verification/presentation/subject-modes-panels-review-2026-10-19.md)에서1440 원본/Right 마스크/End 합성,760 마스크와 drawer/Escape 보존을 확인했다. 당시 남은 segmented focus는 [r135](../../../verification/presentation/segmented-focus-review-2026-10-19.md), thumbnail/실제 track 목록은 위 r136 근거로 보완했다. 다중 선택·canvas 동기와 전체 시각 인수는 남는다. 일반 table3행을 mask1track과 임의 대응시키지 않는다.

보기 버튼의 클릭은 해당 항목에 focus를 주고 공용 segmented가 Left/Right/Home/End 이동을 처리한다. 보기 상태는 UI 임시 상태이며 source/mask revision·검토 상태를 변경하지 않는다. 공용 overlay는 toolbar와 같은 Frame을 사용한다. 실제 실행 결과는 [보기 전환 검토](../../../verification/presentation/subject-view-modes-review-2026-10-18.md)에 구분한다.

OBJ-07 후속: 원본/마스크/합성은 UI 보기 상태로 영상 contain rect 우상단8에 공용 overlay Frame·segmented32를 배치한다. 기본 합성은 기존 청록28%/contour/bounds를 유지하고, 원본은 오버레이를 숨기며 마스크는 같은 원본 크기·PTS의 허용된 alpha 자산을 흰색/검정 배경으로 표시한다. 샘플의 원본·PTS·origin 일치 검사를 공유한다. Empty/미연결은 모두 미선택·비활성으로 전용 구조를 유지한다. 소스/마스크 값·리비전·추론/편집 정책을 변경하지 않는다.

OBJ-04b 후속 구현 범위: workspace_read::MaskSnapshot이 source_asset·원본width/height·source_pts_us·subjects(track_id, polygon_px, alpha_asset, review_state)·origin/state를 소유한다. 준비 상태와 검토 확정을 구분한다. DesignSample은 forest1672×941 프레임에 사람이 직접 그린 SVG silhouette와 동일 pixel polygon을 공급한다. 실제 모델 결과·정확한 머리카락 경계가 아니다. mask_overlay는 같은 SourceTransform으로 alpha28%·청록 contour·bounds를 clip하며 원본asset/PTS가 다른 mask는 표시하지 않는다. Ready와 실제 추론 성공을 동일시하지 않는다. Empty/Live 미연결은mask없음이며 데이터 계약·시각 확대·실행 후 증거로 별도 판정한다.

2026-10-18 OBJ-05/06 도구 rail 후속: subject.png 원본 crop(261,158,70,270)을4배 확대해 세로 pointer/plus/minus/brush/eraser와 어두운 바탕을 재확인한다. 논리 hit32·glyph20·gap4·padding6·border1·radius5로 공통화하고 영상 contain rect의 좌상단 안쪽8에 고정한다. toolbar::canvas_unavailable가 소유하며 Taffy 세로 조합을 사용한다. pointer는 직접 작성한 selection-pointer.svg다. 실제 명령 미연결에서는 모두 비활성·미선택이고 이름/사유 tooltip을 유지한다.1440/760 실제 재검토 전에는 완료로 판정하지 않는다.

[기능 계약](subjects.md), [공통 구성](../shared/screen-composition.md), [확대 관측](../../../verification/presentation/segment-subject-clip-elements-2026-10-18.md)을 적용한다. 기준 subject.png, 수치는 논리 설계값이다.

| ID | 배치·형태 | 데이터·상태·책임 |
|---|---|---|
| OBJ-01a | 제목/설명 → 영상 canvas/transport → thumbnail 추적 시간축 | source PTS/transform·track revision. 원본 contain·mask·선택 bounds가 같은 좌표 변환 |
| OBJ-02a | 검사기 탐지/분할/추적 세 mode32 | 서로 다른 작업능력·모델을 구분, 탐색 자체는 실행 아님 |
| OBJ-03/04 | 검사기 피사체 목록 header+추가 → thumbnail/사용자label/메뉴 row, 선택 amber선 | stable track_id, model class와 별명 분리. 영상 선택/목록 선택 동기화 |
| OBJ-04b | 영상 위 cyan mask contour·반투명 채움·box·white square handles7 | track 선택 표시, 피사체 외형과 box를 구분. clip/crop나 A2D pivot 표시로 대체 금지 |
| OBJ-05/06 | 영상 왼쪽 도구 rail: 선택/포함/제외/brush/erase, glyph20/hit32 | 원본 좌표 point/brush preview, undo와 삭제 접근 가능. A2D와 도구 외형/기하 공유, 정책 분리 |
| OBJ-07 | 영상 우상단 원본/마스크/합성 mode, inspector opacity slider+값 | view state와 mask 원본 값 별도. checker는 투명 보기 배경 |
| OBJ-08 | 경계 세부 slider/numeric·원본px label | 원본 비교·지원 범위, 알 수 없는 모델옵션을 가짜 조작으로 만들지 않음 |
| OBJ-09/10 | 분석 범위 in/out → 모델 → 주요 인식시작40/취소 | source frame·revision 고정, 현재 view zoom과 분석 좌표 분리 |
| OBJ-11 | filmstrip의 컷/소실/가림 구간·경고 glyph+검토필요 문자 | 실패 interval read model, source evidence로 이동, 무조건 새개체 교체 금지 |
| OBJ-12/13/14 | 마스크 검토 secondary/저장, 활용 대상 진입 | 구간별 review, 미검토경고, 선택 track/source refs를 CLIP/FRAME/CARD/A2D/PROJ에 전달 |

1280 검사기drawer·1180 탐색48·980 미만 도구/목록 보조행 전환, canvas 비율 보존. toolbar를 clip하지 않고 두 행 배치한다. 모든 선택 색은 label/shape와 함께, focus는 별도. Empty/track0개/asset미연결/추적일부실패를 구분한다. 새 command가 없으면 mask 도구는 비활성과 이유 표시, passive 선택경계는 resize cursor를 주지 않는다.

필요 컴포넌트: SubjectCanvas/MaskOverlay/ToolRail/SubjectList/TrackingFilmstrip/SubjectInspector. SourceTransform·selection primitives·tool SVG는 A2D와 공유하되 cyan track selection과 blue asset bounds 의미를 분리한다. 현재 generic preview/timeline/table만으로 mask contour·피사체 목록·실패 interval을 재현하지 못한다. typed track/frame-mask/points/review 계약이 필요하다.

분석 방식의 첫 외형은 공용 segmented의 탐지/분할/추적3칸이다. 실제 값·capability가 없으면 모두 미선택·비활성이고 연결 필요를 표시한다. preview bounds는 selection_frame::paint_subject_bounds와 SUBJECT_SELECTION 청록색 토큰, 공용7단위 white square handle을 사용한다. A2D transform은 CANVAS_SELECTION 파란색, OCR은 황색4원형 corner를 유지한다. bounds는 DesignSample 장식이며 실제 contour/alpha mask/추적 성공을 의미하지 않는다.
