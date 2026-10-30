---
id: sot:kinekino/a2d
title: A2D 요소별 시각 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# A2D 요소별 시각 계약

공용 보기32·실제 PNG alpha grayscale·원본/합성·760 레이어/설정/Escape·Empty의 r84/r85 결과는 [알파 보기 검토](../../../verification/presentation/a2d-alpha-view-review-2026-10-18.md)에 구분한다. 출력 알파와 실제 추출/정제의 완료 근거가 아니다.

A2D-02 후속: 세 보기 선택은 공용 segmented32·border/radius/selected/focus와 방향키를 사용한다. 원본/알파/합성 view state는 UI만 소유하고 원본 asset을 변경하지 않는다. Empty는 모두 미선택·비활성이다. 알파 보기는 선택 레이어의 허용된 내장 PNG alpha 채널(0..255)을 검정/회색/흰색으로 표시하고 원본 canvas 좌표와 layer bounds·pivot을 공유한다. 합성의 checker/RGB와 구분한다. 원본 불변 URI별 파생 alpha texture는 context 수명 캐시로 한 번 생성하며 외부 path를 열지 않는다. 연결/디코드 실패를 검정의 성공 mask로 표시하지 않고 안내한다. 실제 추출/정제/저장 결과로 표현하지 않는다.

A2D-04b 후속: 시안의 rail 구분선 아래 브러시 크기·부드러움 영역을 공용 brush_settings로 조합한다. 값 계약이 없으므로 중성 track·미상값 —·연결 필요를 표시하고 thumb/fill/기본값24·50을 만들지 않는다. 크기 label은 원본px, 부드러움 단위/범위는 계약 전이다.980 미만은 rail44를 유지하며 같은 컴포넌트를 설정 검사기 안으로 옮긴다. 중성 scalar 외형은 기존 inspector_field와 공통 input primitive를 사용한다. 실제 마스크 변경이나 브러시 값 편집 지원을 의미하지 않는다.

[r70 실제 검토](../../../verification/presentation/a2d-brush-settings-review-2026-10-18.md)에서1440 rail/760 검사기·내부 scroll/고정 하단·Escape를 확인했다. 실제 값/편집과 미검토 상태는 별도다.

A2D-05 후속: r64에서 인물 레이어 클릭 후 Down을 눌러도 선택·canvas bounds·검사기가 인물에 머무름을 실제 관측했다. layer_row의 상호작용 ID를 layer_id 기반으로 고정하고 공용 keyboard_selection의 Up/Down/Home/End를 표시 역순 목록에 적용한다. 최신 Taffy 배치의 response를 index별로 보존하며 같은 레이어의 중복 response를 누적하지 않는다. 선택 중성면+amber 좌측선과 키보드 focus 외곽선을 구분하고 선택된 stable ID를 canvas/검사기와 공유한다. eye/lock/grip은 읽기전용 상태를 유지한다.

[r65 실제 검토](../../../verification/presentation/a2d-layer-keyboard-review-2026-10-18.md)에서1440 Down/Home의 bounds·검사기,760 End/Up·자동 scroll·설정/Escape·Empty를 확인했다. 미연결 mutation과 나머지 시각 차이는 별도다.

[기능 계약](assets-2d.md)의 A2D ID를 세분화한다. 기준은 asset2d.png, 전체 골격은 [화면 구성](../shared/screen-composition.md), 관측은 [확대 기록](../../../verification/presentation/assets-2d-elements-2026-10-18.md)이다. 아래 치수는 논리 단위의 공통화 설계값이며 원본 raster 측정값이 아니다.

| 요소 ID | 배치·형태 | 데이터·상태 | 구현 책임/미비 계약 |
|---|---|---|---|
| A2D-01a | 검사기 상단 source thumbnail + PTS·좌표 보존 설명 | 원본 frame/PTS/revision, 없음은 재연결 안내 | FrameInfo, SourceSnapshot; 실제 PTS 없는 샘플을 실측으로 표시 금지 |
| A2D-02a | canvas 상단 왼쪽 원본/알파/합성 mode, 높이32·radius5 | 선택 mode·hover/focus 구별, 영상 원본과 마스크/합성 구분 | CanvasViewMode. 이미지의 합쳐진 '알파 합성' 문구는 기존 3모드 계약으로 정규화 |
| A2D-02b | checker canvas, 선택 레이어 blue bounds·8 white handle7, amber pivot cross | checker는 투명도 배경, 원본의 일부가 아님 | Checkerboard/SelectionFrame. 현재 어두운 checker와 시안 밝은 checker의 차이는 실제 렌더 비교 후 판정 |
| A2D-03a | 상단 오른쪽 minus/percent/plus/fit/hand, hit32·glyph20, gap4 | zoom100%=원본 pixel 배율, Fit은 전체가 보이는 배율; pan 별도 선택 상태 | CanvasToolbar, 확대/pan은 presentation state. Fit은 pan offset도 초기화하여 전체 복귀 |
| A2D-04a | canvas 왼쪽 내부 도구 rail112, 좁은 경우44; 포함/제외/지우기 icon+label | 선택 도구는 중성면+amber좌측선, 실행 불가면 이유 제공 | 공통 SVG BrushAdd/MaskExclude/Eraser; 실제 mask command 미연결은 편집 가능으로 표시 금지 |
| A2D-04b | rail 아래 구분선→브러시 크기/부드러움 label·slider·값 | brush 원본px, softness 단위·허용범위 명시; undo는 실제 문서 revision | BrushSettings. softness domain/read 계약은 미정, 임의 기본값 영속화 금지 |
| A2D-05a | canvas 아래 레이어 목록 `|` 원본 참조, gap12/구분선; 좁으면 세로 | source reference는 contain, 각 row 안정 layer_id, 앞 레이어를 위에 표시 | LayerList/SourceReference. SDK layers는 back→front이므로 표시만 역순 |
| A2D-05b | row: eye hit32·thumbnail40·label·lock hit32·grip hit32, 최소 높이48·radius5 | selected/visible/locked/reorder 구분. 숨김은 EyeOff, 해제는 Unlock 상태 필요 | layer read model은 존재. mutation command 없는 eye/lock/grip은 상태 표식·설명으로 표현하고 가짜 편집 불가 |
| A2D-06a | 검사기 알파 추출 group: label/slider/numeric 입력 3열, 필드 높이32 | threshold0~1, feather 원본px, 경계 정제는 별도 정의 필요 | AlphaSettings typed 계약 필요. 시안 threshold128은 8bit 예시이며 정본0~1과 혼용 금지 |
| A2D-07a | 중앙/하단/사용자 radio → X/Y 필드 동일 폭 | pivot px는 **canvas 절대 좌표**. 정규화 값/원본px 전환 기준을 label로 표시 | CanvasSnapshot.layer.pivot_px, 변환은 bounds와 동일 SourceTransform. layer-relative로 오해 금지 |
| A2D-08a | bounds X/Y/W/H와 단위px, 좁으면 2×2 | bounds는 원본 canvas 위치/배치 크기, trim offset 별도 | PixelBounds. trim과 배치 bounds의 의미를 혼합하지 않음 |
| A2D-09a | scale selector/값, 목표 output size/확대 경고 | 편집 view zoom과 export scale 별개 | ExportScale. 현재 canvas zoom_percent를 output scale로 사용 금지 |
| A2D-10a | color space와 alpha mode 정보/도움말 | sRGB·straight/premultiplied 처리 능력 확인 | typed output capability 필요, 지원 미확정 선택을 성공 상태로 표시 금지 |
| A2D-11a | source PTS/정적·시퀀스 구분, 시퀀스일 때 순서/누락 목록 | source frame 연결, 누락 결과 NeedsInput | sequence read contract 후속. 정적 시안에 없는 가짜 연속 frame 추가 금지 |
| A2D-12a | 검사기 하단 합성 검증 secondary, hit40 | 원본 배경 재합성과 차이·실패 정보. 시각 샘플을 검증 완료로 오인 금지 | 검증 command/result 포트 미연결은 이유와 비활성 |
| A2D-13a | RGBA PNG/mask PNG/manifest JSON 체크 목록 | 실제 출력 지원과 선택 상태 독립, manifest는 좌표·PTS·변환 포함 | typed output selection. 샘플에서는 실제 파일 생성 없음 |
| A2D-14a | 하단 패키지 저장 primary amber/hit40, 합성 검증 옆 | 저장은 OUT preflight·원본 보존·충돌 정책 공유 | 실제 command 가용성/검토 상태 확인 전 활성 금지 |

## 좌표·상태·반응형

원본 canvas 크기 → fit/zoom 배율 → pan → viewport clip 순서로 원본 이미지·레이어 bounds·pivot·선택 handle을 함께 변환한다. hit 좌표는 역변환한다. 화면 안의 checker 타일은 보기 요소이며 export에 포함하지 않는다. bounds 밖 handle은 viewport clip과 상호작용 가능 영역을 함께 검토한다. 편집 disabled 상태의 handle은 변형 가능 cursor를 제공하지 않는다.

Empty: canvas 구조·source 선택 안내와 비활성 도구를 유지한다. Populated: 원본+foreground+background+mask의 역할과 선택 상태를 보여준다. LongLabels: 레이어명은 truncate+tooltip, 수치/우측 동작은 고정한다. NeedsInput: 누락 asset을 명시하고 기존 source/frame과 정상 layer를 보존한다. 생성 중·실패·검증 전·검증 완료는 실제 application 상태를 따른다.

1440의 canvas+rail/검사기, 1280 경계 검사기 drawer, 1180 경계 탐색48, 980 미만 레이어/원본 세로 적층, 760에서 도구 rail44와 작은 canvas를 유지한다. toolbar가 넘치면 mode/zoom 두 행으로 나누고 아이콘을 화면 밖으로 밀지 않는다. 창 caption과 queue는 canvas 스크롤 밖에 둔다.

## 구현 전 남은 결정

알파 모드의 mask grayscale 표시, softness·경계 정제 단위, typed inspector, mask layer AssetId/역할, 실제 edit command, 숨김/잠금/순서 명령, 합성 검증 결과는 SDK와 계약을 정한다. 기존 readonly CanvasSnapshot 완료는 이 기능들의 완료가 아니다. WIP가 이 문서와 다른 부분은 수정 목록으로 유지하며 현 구현을 정본으로 올리지 않는다.
