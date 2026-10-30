---
id: sot:kinekino/card
title: CARD 요소별 시각 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# CARD 요소별 시각 계약

[r133 toolbar strip 실행 검토](../../../verification/presentation/card-toolbar-strip-review-2026-10-19.md)에서1440/760 전체 폭 외곽선과 무테 동작,760 FRAME 기존 개별 버튼 유지를 확인했다. 원형 추가 glyph와 광학 간격·상태 등은 남는다.

CARD-01/05 면 후속: cards.png의(249,122,866,62)4배 확대는 전체 폭의 얇은 외곽 toolbar와 내부 무테 동작을 보여준다. 공용 toolbar의 명시적 Strip variant로 PANEL 면·THIN border·CONTROL radius·INLINE padding을 적용하고 내부 버튼 frame은 제거한다. 기존 개별 버튼 variant를 쓰는 FRAME/CLIP 등은 유지한다. 동작32/SVG20/간격8·그룹 구분선과 비활성 이유는 공유한다. 수정 후1440/760 및 기존 FRAME 소비자를 실제 확인한다.

[r132 긴 문구·중간 크기 검토](../../../verification/presentation/card-longlabels-frame-review-2026-10-19.md)에서1440/1180/980/760의 줄바꿈,980 여백 클릭·Down 선택,760 drawer 본문 스크롤·Escape 후 카드/스크롤 유지까지 확인했다. 좁은 창은 canvas 접근에 본문 스크롤이 필요하며 전체 시각 완성으로 판정하지 않는다.

[r131 선택 프레임 실행 검토](../../../verification/presentation/card-thumbnail-frame-review-2026-10-19.md):1440/760의 여백 클릭·키보드 선택·drawer/Escape·Empty를 확인했다. 전체 CARD 시각 일치는 남은 범위다.

CARD-04 선택 테두리 후속: cards.png 선택 thumbnail 확대에서 이미지 바깥 여백과 둥근 amber 외곽선을 확인했다. 공용 selectable thumbnail은132 footprint 안의 TIGHT4 여백에 이미지를 넣고 선택 외곽선2/radius5를 표시한다. 번호 badge는 실제 image rect에 기준을 두며 focus는 이미지 안쪽에 별도 얇은 외곽선이다. 모든 상태가 같은 footprint/image 크기를 사용해 선택 시 레이아웃이 움직이지 않게 한다. 후보 데이터·페이지 수·문구는 바꾸지 않는다. 실제1440/760 선택/키보드/Empty를 확인하기 전 완료로 표시하지 않는다.

CARD-01/05 후속: 공통 grouped toolbar32를 적용해 추가/복제/삭제와 undo/redo를 구분선으로 나눈다. 복제는 이미지 glyph 대신 직접 작성한 두 장 겹침 Copy SVG, 삭제/undo/redo는 공통 의미별 자산을 사용한다. 누락된 redo를 추가하되 실제 history 미연결의 비활성 상태를 유지한다. 카드 선택/문구/페이지 의미는 변경하지 않는다.

[r66 검토](../../../verification/presentation/card-frame-toolbar-review-2026-10-18.md)에서1440/760 도구와760 Right·3/3·설정/Escape를 확인했다. toolbar 면과 실제 history는 남은 범위다.

r55의 공용 문구 예약 영역을 기본/LongLabels1440·760으로 [실제 재검토](../../../verification/presentation/card-text-safe-area-review-2026-10-18.md)했다. 해당 표본에서 badge/문구/counter를 분리했다. 임의 길이·비정사각 문서·실제 출력 및 기존 선택 테두리 차이는 남는다.

CARD 긴 문구 예약 영역: page_marker::text_bounds가 번호 badge24+상하 여백4×2를 thumbnail 기준132에 대한 비율로 환산해 상하 영역을 예약한다. canvas와 thumbnail은 같은 비율/중앙을 유지하며 공용 artboard_text가 해당 bounds의 폭·높이에 맞춰 줄바꿈/최소 크기/넘침 안내를 적용한다. 원문은 삭제하거나 말줄임하지 않는다. FRAME 등 기본 Artwork의 기존 영역은 유지한다. 임의 길이 문구 모두가 맞는다는 보장은 하지 않는다.

r54에서1440/760·선택·방향키·Empty·LongLabels를 [실제 검토](../../../verification/presentation/card-pagination-review-2026-10-18.md)했다. 기본 번호/현재 페이지 동기화를 확인했으며 긴 문구와 badge 안전영역 및 선택 테두리 inset 차이는 남아 있다.

CARD-04/06 후속: thumbnail 번호를 별도 위쪽 텍스트에서 image 좌상단의 어두운 badge로 옮긴다. 공용 page_marker는 badge24·TEXT/CHROME·CONTROL radius를 사용하고 canvas 하단 중앙 current/total도 같은 페이지 의미로 표시한다. 선택 amber와 image 안쪽 keyboard focus를 구별하며 세로 rail은 위/아래, 좁은 가로 strip은 좌/우/Home/End로 접근한다. 기존 DesignSample 외 데이터를 만들지 않는다. 실제1440/760·Empty 확인 전 완료로 판정하지 않는다.

[기능 계약](cards.md), [공통 구성](../shared/screen-composition.md), [확대 근거](../../../verification/presentation/frame-card-projection-save-elements-2026-10-18.md). 기준 cards.png. 논리치수이며raster측정값이아니다.

| ID | 배치·형태 | 데이터·상태·컴포넌트 |
|---|---|---|
| CARD-01/05 | 제목/설명 → 추가/복제/삭제·구분선·undo/redo toolbar32 | 빈템플릿/frame/cue시작, 마지막삭제는빈초안. 실제history가없으면disabled이유 |
| CARD-04 | 내부왼쪽 세로thumbnailrail112~140 → 중앙cardcanvas | 번호badge좌상단·현재amberoutline/radius5, card_id/order선택, 위아래명명동작/drag대안 |
| CARD-02/03 | inspector 게시대상preset cards → ratioselector/빠른비율 | YouTube/Instagram은로컬출력목적, 공식현행제한보장이나게시동작아님. 브랜드SVG는필요시직접작성 |
| CARD-06/07 | 표지/본문/마지막template → 레이어canvas·맞춤선/safearea | 내용보존범위명시, current/total페이지표시, 화면전체preview와카드레이어편집분리 |
| CARD-08/09 | 제목/본문필드·자동맞춤안내 → font/색상swatches | Pretendard, 배경/outline/spacing, 초과텍스트삭제금지, 최소폰트/넘침경고. 시안50/100자는고정정책아님 |
| CARD-10/11 | 요약초안동작·source회차PTS, 피사체mask/crop | evidence연결, 제안/수락/기각·undo, 원본재사용 |
| CARD-12 | 비율별layout revision와미리보기 | 원본aspect왜곡금지, preset변경시변화범위표시 |
| CARD-13 | 카드별alt text multiline/초안표시 | sourceevidence와사용자수정, UI문구번역과문서본문분리 |
| CARD-14/15 | 선택카드저장 secondary/전체저장 primary40, 하단로컬파일저장안내 | 순번/개수/형식·font/넘침/픽셀/순서검사, OUT공유 |

canvas/thumbnail은같은문서에서렌더하며thumbnail을별도정적인가짜결과로유지하지않는다. thumbnail별텍스트도선택비율대로축소한다. inspector300/input32/body13/small11/padding12/group16. 980미만rail은가로thumbnailstrip으로전환가능하되번호/현재선택유지, inspectordrawer; canvas비율유지. LongLabels/긴한국어·일본어/alt/200카드내부scroll/Empty/입력필요를확인한다.

필요컴포넌트 CardThumbnailRail/CardCanvas/CardToolbar/CardTextInspector/OutputPreset/AltEditor. FRAME의layer/text/crop기하와공유하되CardDocument순서·preset파생정책은application. typed card/layers/copy/evidence/layout/alt read계약이필요하며현재genericpreview/table은대체물이아니다.

공용 artboard_text는 title/subtitle을 하나의 중앙 그룹으로 배치한다. 텍스트 폭은 artwork84%, 높이는70%의 논리 설계값이며 가운데 줄바꿈한다. 영역 초과 시 기본 글자 크기의50%까지 단계적으로 줄이고도 넘치면 검토 안내를 표시한다. 원문을 삭제·ellipsis 처리하지 않는다. 캔버스/thumbnail은 같은 함수를 사용하며, 문서의 실제 출력 레이아웃 검증·사용자 최소 크기 정책을 대신하지 않는다. hover에서 전체 원문을 확인할 수 있다.
