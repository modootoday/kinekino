---
id: evidence:kinekino/20261029113817
title: OBJ Web 전용 컴포넌트 제작 전 대조
status: active
created: 2026-10-29
updated: 2026-10-29
---
# OBJ Web 전용 컴포넌트 제작 전 대조

2026-10-20 승인 v2/studio/subject.png(1586×992)를 열고 도구 rail과 피사체 목록을 확대 검토했다. 원 시안/SDK 소유 파일은 변경하지 않았다. 증거는 `runtime/quality-review/web-subject-source-r1/`의 crop이며 제품 렌더가 아니다. [시각 계약](../../player/presentation/studio/subjects-visual.md)과 [구성 정본](../../player/presentation/shared/screen-composition.md)이 규격을 소유한다.

| 요소 | 원본 crop x/y/w/h·확대 | 관측 → 구현 책임 |
|---|---|---|
| OBJ-05/06 tool rail |264/160/64/264·4×| 어두운 세로 면/얇은 테두리/작은 radius, 선택·포함·제외·brush·erase5개. active만 amber glyph/outline. 공용 SVG selection-pointer/plus/mask-exclude/brush-add/eraser를 소비하고 glyph/hitbox를 분리 |
| OBJ-03/04 list |1198/158/374/162·2×| 제목/추가 같은 행→thumbnail·사용자label·세로menu→label 의미 안내. 선택 외곽선과 경고·검토 의미를 구분. source-canvas alpha를 thumbnail로 쓰지 않음 |
| OBJ-01/07 source canvas |전체 이미지 직접 관측,별도crop 미작성| 영상 contain 위 좌측rail·우상단3보기, 하단transport. cyan contour와 box/white handles는 동일 원본 좌표. arbitrary CSS polygon으로 실제 결과를 꾸미지 않음 |
| OBJ-09/11 tracking |전체 이미지 직접 관측,별도crop 미작성| filmstrip의 썸네일·독립 playhead와 실패 interval의 붉은면/경고glyph/문자. 일반 표나 무조건 성공 진행률로 대체 금지 |

원 이미지에 선택 고양이 thumbnail과 사용자 별명은 있지만 현재 SDK `workspace_read/mask.rs`의 SubjectMask는 track_id/polygon_px/alpha_asset/review_state만 제공한다. MaskSnapshot에는 source_asset/source_pts_us/width_px/height_px/origin/state가 있다. 별명/분리thumbnail/개별frame_ref/실패interval은 없으므로 없는 값을 생성해 실제 데이터로 표시하지 않는다. 이름 미지정·실제 track ID를 유지하고 thumbnail은 검증된 source crop 파생물만 허용한다. 마스크 없음/Unsupported와 정상0track을 같은 Empty로 바꾸지 않는다.

신규 source 제안은 `studio/objects/`에 workspace/도구/보기/목록/검사기를 분리하고 styles/objects.css를 독립 소유한다. StudioFrame/InspectorDock/원P1a10/CSS token/SDK는 복제·수정하지 않는다. 실제 media/mask/filmstrip은 검증된 typed child slot으로 조합하고 각 unknown/capability 이유를 원자료와 분리한다. view mode/tool 선택은 presentation이고 탐지/분할/추적/편집/검토/저장은 SDK command 계약 인수 전 실행 완료로 세지 않는다.

PM 후속 배정으로 `studio/objects/`에 workspace/tool-rail/view-modes/subject-row/subject-list/inspector의 TSX6개와 styles/objects.css를 작성했다. source-inventory.json에 정확7경로/해시를 보존한다. 원P1a10 직접 재해시 차이0, 공통style/tokens/SDK/route등록 수정0이다. 새 Cargo/GUI/브라우저·추론0, parser/타입/빌드/fmt/test/clippy/asset/runtime·실제 입력·시각 인수는 미완료다. rootindex는SDK소유이므로 UI stage/commit0.

후속 Rust 초안 `web_ui_adapter/studio/subjects_projection.rs`는 기존 WorkspaceSnapshot/MaskSnapshot를 borrow한다. origin/state는 workspace와 mask 양쪽을 보존하고, mask=None을 Missing으로 유지하며 실제 0개 목록과 구별한다. 실제 track와 review를 그대로 참조하고 사용자 label/검증된 crop thumbnail은 None으로 둔다. 선택은 명시 track ID와 일치한 경우뿐이며 사라진 ID를 첫 행으로 자동 대체하지 않는다. 다른 workspace 입력은 WrongWorkspace로 거부한다. 이 초안은 studio/mod.rs에 등록하지 않았으며 TSX/Native/SDK command와 연결되지 않았다. 테스트2개는 작성만 했고 미실행이다. 별도 projection-draft-pin.json SHA `EEBD70B282A1C2900A461C88F13162C1859E39770DB3746F4491606C8D09575D`로 구분한다.

PM은 신규 화면의 무제한 source 확대보다 Inspector→P2→동일 SDK native 영상 연결을 우선 배정했다. 따라서 OBJ7과 투영 초안은 동결하며, source7-freeze-recheck.json=[]로 확인했다. 이 초안을 SDK 공개 compile 분리 작업에 자동 등록하거나 검증됐다고 표시하지 않는다. 중앙 인수 후 동일 원본 graph와 typed readmodel 계약으로 재개한다.

workspace는 원본/mask canvas·overlay tools/modes·transport·tracking filmstrip을 각 child slot으로 나눈다. 실제 canvas/원본좌표/검증된 mask/실패interval 생성은 아직 없고 기존 generic table로 대체하지 않았다. row는 실제track ID/optional사용자label의명시Boolean/review문구/검증된thumbnail slot과 선택·메뉴 capability를 받는다. 목록 상태문구는 source adapter가 제공해 Unsupported/NeedsInput/정상0개를 보존한다. tool rail5개·보기3개·분석3개는 각각 명시callback을 받으며 원 source semantics를 삭제하거나 arbitrary JS로 실행하지 않는다.

Inspector는 피사체목록→작업3선택→시간구간→불투명도/원본px경계→모델slot/검토→고정footer를 선언했다. 현재SDK에없는opacity/feather설정은hasMaskSettings=false로slider/입력을숨기고unknown문구를보존한다. capability는권한이아니며실제SDK검증을대체하지않는다. slider0~100은표시범위이며문자열입력을명시callback으로전달하고원본px/모델/분할/시간검증정책은SDK에유보한다. 범위직접편집은후속typed editor intent이며현재문구/수정요청만있다. 기본browser range의형태가승인thumb/track과같다는주장도하지않는다.

CSS색/border/radius/여백은기존공통변수만소비한다. OBJ전용minimum320과rowthumbnail44×40은기존native규격의명시localtoken projection이며공통token자동동기화는아니다. 기존SVG6개를직접참조하고기존자산수정/신규이미지생성은하지않았다. 주요미인수표현은ReactNode다중slot,Boolean/문자열비교및hidden/pressed属性、useId、controlledrange/input、callbackprops다. 같은원본parser진단을확인한후Meari생산자와조율하며입력단순화로완료시키지않는다.
