---
id: sot:kinekino/frame
title: FRAME 요소별 시각 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# FRAME 요소별 시각 계약

[r142 검사기 첫 제목 검토](../../../verification/presentation/inspector-first-heading-review-2026-10-19.md)에서1440 원본/편집과760 편집 drawer의 시작 정렬·Escape를 확인했다. 후속 그룹 구분은 유지한다.

FRAME 도구 후속: 원본/편집 하위 화면의 동일 FRAME_ACTIONS를 공용 grouped toolbar로 조합해 아이콘과 이름을 한 border/radius 버튼 안에 배치한다. 높이32·icon20·간격8을 공유하고 이전/고정/다음/원본 비교 의미와 인접 frame/PTS 미연결 상태는 유지한다. 실제 버튼 활성/프레임 이동 지원은 별도다.

[r66 검토](../../../verification/presentation/card-frame-toolbar-review-2026-10-18.md)에서1440/760 두 탭의 도구 순서/label/접근을 확인했다. 실제 frame 이동과 나머지 canvas 차이는 별도다.

2026-10-18 r51/r52 후속: 원본/편집 동작 순서 분리, thumbnail caption 중앙 정렬·선택 강조·image 내부 focus, Home/Left 선택을 [실제 검토](../../../verification/presentation/frame-neighbor-selection-review-2026-10-18.md)했다.1440/760/Empty를 확인했으며 인접 PTS·strip 간격·편집5frame·canvas 안전영역 등 남은 차이는 해당 기록에 둔다.

FRAME-02/03 후속 설계: 원본 preview→이전/현재/다음 thumbnail·중앙 정렬 caption→프레임 동작 순서를 따른다. 선택 caption은 amber, 키보드 focus는 별도 선으로 표시한다. 공용 FrameThumbnail과 keyboard_selection을 조합하고 실제 인접 frame PTS가 없는 sample에 임의 fps/시간값을 만들지 않는다. 현재 단계에서는 PTS 연결 필요를 표시한다. 실제 실행 후1440/760·선택·방향키·Empty를 확인한다.

[기능 계약](snapshots.md), [공통 구성](../shared/screen-composition.md), [확대 근거](../../../verification/presentation/frame-card-projection-save-elements-2026-10-18.md). 기준 raw-frame/snapshot-refined. 수치는 논리 설계값이다.

| ID | 배치·형태 | 데이터·상태·컴포넌트 |
|---|---|---|
| FRAME-01 | 제목/설명 → 원본프레임/썸네일편집 탭32 | stable subview, 원본과 편집문서 분리 |
| FRAME-02/03 | 원본: 큰frame → 이전/현재/다음3thumbnail·PTS → 뒤로/고정/앞으로 동작 | FrameStrip, 현재 amber테두리/문자, 실제decoder PTS·VFR 확인, 임의fps 환산 금지 |
| FRAME-02b | 검사기 원본파일정보 label-value: codec/해상도/fps/color/rotation/duration | 알려진/미상 구분, measured source revision |
| FRAME-04/05 | 자막포함 switch/설명, 편집모드 원본비교 동작 | burn-in제거 불가, 분리자막만 토글, 같은좌표 원본/편집 표시 |
| FRAME-06 | 편집canvas crop경계/handle·safearea, 아래frame/PTS·이동/고정/비교·filmstrip | CropCanvas, 원본px/출력px 구분, fit/fill·aspect 보존 |
| FRAME-07 | 텍스트layer선택경계 blue/whitehandle, inspector 제목·font/size·색/outline·배경/opacity | TextLayerEditor, UI와출력폰트 기본Pretendard, size는출력px, focus/selected 구분 |
| FRAME-08/09 | inspector 레이어 header+추가 → eye/type/name/lock/grip rows | LayerList/A2D공통표식, 안정layer_id·maskref·원본보존, mutation capability 분리 |
| FRAME-10/11 | fontfallback/overflow/color 경고 | 누락glyph·넘침·HDR확인, style변경은원본불변 |
| FRAME-12/13 | 원본PNG저장 secondary/편집본저장 primary40; 형식/이름/경로 | source frame과 output revision 분리, UI/OSD는출력제외, OUT preflight |
| FRAME-14 | 문구제안·cue근거·수락/되돌리기 | draft표시, 근거없는성공/대사 생성금지 |

일반입력32/body13/small11/group16/padding12. 980미만filmstrip은내부수평스크롤, 원본3frame은현재를우선하고나머지접근경로유지. 검사기drawer·탐색48은공통breakpoint. caption/큐는canvas스크롤밖. Empty·frame미준비·LongLabels·NeedsInput·font누락·넘침·저장실패 별도확인.

제작컴포넌트 FrameStrip/FrameInfo/CropCanvas/TextLayerEditor/LayerList/FrameExportInspector. typed neighborframes/PTS/sourceinfo/thumbnaildocument/textstyle/layer/outputcapability가필요하다. 현재일반preview/table은선택frame/편집canvas를대체하지못한다. 기존captureSDK 결과는실프레임근거로연결하되demoPNG를실측capture로표시하지않는다.
