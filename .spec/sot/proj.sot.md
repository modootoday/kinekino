---
id: sot:kinekino/proj
title: PROJ 요소별 시각 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# PROJ 요소별 시각 계약

[r107 검사기 실제 검토](../../../verification/presentation/projection-inspector-review-2026-10-19.md): 공용 section으로 네 설정 그룹을 구분하고 1440 내부 스크롤·760 drawer 하단 도달/footer 고정·Escape 복귀를 확인했다. 도움말과 실제 값은 별도다.

2026-10-19 PROJ-08/09/10/15 검사기 후속: projection-package.png의 (1115,60,460,830)을2배 확대해 좌표계 및 단위 → 깊이 정보 → 행렬 및 UV → 출력 설정의 제목·separator·여백을 비교한다. 패키지 탭에서는 이 순서의 공용 inspector section을 사용하고 2.5D 탭은 깊이/투영부터 시작한다. 값 미연결 상태는 유지하며 시안의 RH/Y-up/미터 등을 실제 값으로 복사하지 않는다. 상대 깊이/기하 단위 설명은 해당 section에 둔다. footer 고정과 drawer 스크롤은 재검토한다.

[r105/r106 파일 행 실제 검토](../../../verification/presentation/projection-file-row-review-2026-10-19.md): 파일별 직접 작성 SVG, 행 전체 선택·focus,1440 End·760 Home과 폴더 접기를 확인했다. 실제 파일 검사/미리보기는 연결되지 않았다.

2026-10-19 PROJ-13a 후속: projection-package.png의 (860,194,235,203)을4배 확대해 폴더·image/mask/depth/JSON/cube glyph와 행 전체의 선택 면을 대조한다. 기존 텍스트 폭 selectable_label을 공용 FileRow로 교체한다. 행높이32, 아이콘20, 간격8, 전체 행 hitbox·중성 선택 면·별도 focus ring을 공유하고 stable file ID 기반 기존 키보드 선택을 유지한다. 파일 성격은 디자인 fixture에 명시된 종류이며 실제 확장자 검증/파일 읽기 정책이 아니다. 원본 참조 이미지는 선택 파일 결과로 바꾸지 않는다.

[r103/r104 원본 참조 카드 실제 검토](../../../verification/presentation/projection-source-reference-review-2026-10-19.md): 누락된 원본 카드를 공용 scene에 추가하고 1440 두 탭·760 패키지·설정/Escape·빈 상태를 확인했다. 전체 scene 시각 인수와 미검증 폭/상태는 별도다.

2026-10-19 원본 참조 카드 후속: r103 실제 1440 화면에서 PROJ-03의 좌상단 원본 inset이 누락된 것을 확인했다. 두 탭의 공용 ProjectionScene에 원본 비율을 보존하는 참조 카드를 제공한다. 폭이 좁으면 우상단 도구 아래로 내려 겹침을 피한다. DesignSample 원본만 표시하며 패키지 선택 파일이나 실제 깊이 결과로 표시하지 않는다. 실행 재검토 전 완료로 판정하지 않는다.

패키지 파일 선택 후속: r71에서 mask.png 클릭 후 Down이 선택을 바꾸지 않는 것을 확인했다. 샘플 파일 ID 기반 focus와 공용 Up/Down/Home/End를 연결하며 Taffy 재배치 response는 index별 최신값만 보존한다. 선택 파일명을 명시하고 같은 원본 이미지에 '원본 프레임(참고)'를 붙여 선택 파일 미리보기로 오인하지 않게 한다. 실제 file preview/type/revision read 계약은 후속이며 샘플 파일은 실제 파일이 아니다.

[r72 실제 검토](../../../verification/presentation/projection-file-navigation-review-2026-10-18.md)에서1440 Down/End·760 Home·설정/Escape·Empty 안내를 확인했다. 실제 파일 preview/검사와 전체 패키지 시각 인수는 별도다.

2026-10-18 PROJ-05/07 확대 후속: projection-package.png의 XYZ영역(275,489,102,95)과 toolbar(668,151,182,50)을4배 확대했다. 공용 projection_scene의 XYZ는 좌하단 inset64/44(space::SCENE_AXIS_LEFT/BOTTOM), 도구4개는 우상단 inset8·control32·gap4·padding6의 token 기반 canvas_controls로 조합한다. 이동/시점 이동/확대/맞춤 glyph를 사용하며 zoom-in/move-view SVG는 직접 작성한다. 실제 orbit 기능이 없으므로 미선택·비활성/사유를 유지한다. 평면 원본 비율은 asset의 width/height로 계산하고 두 방향 교차 격자는 viewport 하반부에 clip한다. 실제 mesh·depth·camera 결과로 승격하지 않는다. 두 탭과 좁은 창은 구현 후 실행 검토로 판정한다.

[기능 계약](projection.md), [공통 구성](../shared/screen-composition.md), [확대 근거](../../../verification/presentation/frame-card-projection-save-elements-2026-10-18.md). 기준 projection3d-refined/projection-package. 논리설계값이다.

| ID | 배치·형태 | 데이터·상태·컴포넌트 |
|---|---|---|
| PROJ-01/02 | heading/설명 → 2.5D깊이/투영패키지탭32 → sceneviewport | RGB/mask/sourcePTS/회전, 완전3D복원으로표시금지 |
| PROJ-03/04 | 깊이scene: 원본inset좌상단, RGB/depth/layer/projectionmode우상단 | DepthViewMode, 모델미선정안내, 상대depthgradient의min/max와단위없음표시 |
| PROJ-05/06 | grid위분리planes/texture/checker, XYZtriad → inspector범위/반전/평면수·plane/mesh | ScenePreview. 원본기하/depth/transform, actualmesh없으면illustrative DesignSample 표시 |
| PROJ-07/12 | scene아래카메라각slider/숫자, pan/orbit/zoom/fittoolbar | finiteFOV/near/far·원본위치복원, 제한각도·키보드수치대안 |
| PROJ-08/09 | inspector좌표계·upaxis·단위·matrix·UV그룹 | geometry단위와상대depth구분, column/row convention명시, 선택값이검증지원의증거아님 |
| PROJ-10/11 | alpha/color선택과holehatch | 미관측영역·생성채움origin분리, stretch로구멍감추지않음 |
| PROJ-13a | 패키지: scene `|` 레이어/파일tree → 유효성표 | tree folder/texture/mask/depth/manifest/scene glyph, 선택중성면, stableasset_id·filesize/dimensions |
| PROJ-13b | tree아래선택filethumbnail·이름·크기/형식 | contain/정보없음표시, 임의path실행없음, rasterallowlist/실제검증asset구분 |
| PROJ-14 | 유효성표 축/scale/UV/alpha/material/sourcePTS 상태·비고, target도구selector | 검증전/실패/통과·evidence별도, 모델/대상미검증amber경고 |
| PROJ-15 | 검증/패키지저장 primary40 또는하단actions | targetimport/roundtrip확인전내보내기불가, 수동plane지원별도표시, OUT공유 |

scene은viewportclip내에서비율/좌표축/카메라일관성을유지한다. 시안평면배열그림을실제추론/GLB호환증거로쓰지않는다. 980미만filetree는scene아래접이식목록, inspectordrawer, XYZ/모드toolbar겹침방지. 일반입력32/body13/small11/padding12/group16. Empty/depth없음/LongLabels/NeedsInput/검증실패/이전revision상태를확인한다.

패키지의980이상 구성은 유동 scene + 고정240 논리단위 filetree(`size::PACKAGE_TREE`, `layout::detail_sidebar`)로 나눈다. 동일폭2열로scene을과도하게줄이지않는다. 유효성영역은항목/상태/비고3열6행이며, 읽기전용입력필드를세로로나열해표를대체하지않는다. 실제raw이미지측정과구분한논리배치결정이며실제재검토근거를남긴다.

필요컴포넌트 ProjectionScene/DepthLegend/CameraControls/PackageTree/AssetDetails/ValidationTable/ProjectionInspector. typed scene/layers/depthrange/camera/convention/files/targetvalidation이필요하다. genericpreview/table로대체불가. 검증된RustGPU/scene구현이어려운범위는미구현으로표시하되다른UI기술로정책을우회하지않는다.
