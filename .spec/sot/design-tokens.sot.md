---
id: sot:kinekino/design-tokens
title: 디자인 토큰 정본
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 디자인 토큰 정본

framed 표의 선택 행은 `TABLE_SELECTED`(AMBER_975) 면과 ACCENT·THIN 전체 윤곽/CONTROL radius를 사용한다. 상태 성공/경고와 별개의 선택 의미다. keyboard focus는 border FOCUS+THIN inset 안의 FOCUS2 외곽선으로 선택과 구별한다. 기본 표 선택 토큰은 유지한다.

읽기 전용 속성 행은 `METADATA_LABEL_WIDTH`144(아이콘20+FIELD8+라벨의 고정 영역),최소CONTROL32,FIELD8 열 간격을 사용한다. 값은 유동폭에서 줄바꿈한다. 아이콘/라벨/값은 같은 행의 수직 중앙을 공유하며 폼 입력과 구별한다.

OUT 공용 출력 요약 행의 썸네일은 `OUTPUT_THUMBNAIL_WIDTH`144 / `OUTPUT_THUMBNAIL_HEIGHT`81이다. source table의64와 책임을 분리한다. 썸네일·이름 사이 GROUP16, 이름/형식 사이 FIELD8, 이미지 CONTROL radius를 공유한다. source 이미지 contain과 출력 결과 이미지의 의미를 구별하며 임의 출력 실측을 생성하지 않는다.

MediaRail은 활성 track 채움과 원형 thumb를 같은 amber 의미 토큰으로 표시한다. 비활성 채움/손잡이는 중성 SECONDARY, 바탕 track은 BORDER 의미 토큰이다. CONTROL32 입력 높이와 별도 compact18 시각 높이, track3·기본 thumb 반경5·hover/drag 반경6·가로 inset6을 토큰으로 소유한다. focus는 별도 FOCUS 외곽선을 유지하며 색으로만 focus를 표현하지 않는다. 시안 근거는 playback-v2 볼륨 영역의4배 확대다. 변경 후 실제 inline/popup과 비활성 seek를 재검토한다.

JOB 표의 행 최소 높이는 QUEUE_ROW48, 상태 열 기준 폭은 QUEUE_STATUS112, 진행 열은 QUEUE_PROGRESS80이다. 작업/source/output 열은 유동폭, 고정 열 합계는 공용 TABLE_FIXED_MAX 비율을 넘지 않도록 축소한다. glyph20·가로/세로 padding8·선1/포커스2는 공용 범주를 재사용한다.

중성 metadata badge는 size24·pill radius12·small11·가로6/세로4·border1·최대 내용 폭240을 사용한다. Taffy FlexWrap 목록이 항목 단위 줄바꿈을 소유한다. MODEL filter 폭160은 검색 유동폭과 분리된 의미 토큰이다. badge는 자체 성공/경고 의미를 만들지 않고 원문과 별도 데이터 상태를 유지한다.

CLIP 후보 행의 넓은 화면 동작 영역은 `size::CANDIDATE_ACTIONS`240 논리 단위(60×4)로 소유한다. 패키지 tree와 숫자가 같아도 책임을 섞지 않는다. 980 미만은 폭 예약 대신 다음 행 배치하며 버튼32·glyph20·field gap8·card padding12를 공용 토큰에서 사용한다.

사용자 지시에 따라 Tailwind의 토큰 범주를 Rust UI에도 적용한다. Tailwind 패키지·테마를 도입하는 의미가 아니라 **값의 단일 정의 → 의미 토큰 → 컴포넌트 조합** 체계다. 시각 기준은 본 프로젝트 시안이며 참조 프로젝트의 색상은 가져오지 않는다.

정의는 `src/presentation/design_tokens.rs`, egui style 적용은 `player_theme.rs`, Flexbox/Grid 조합은 `layout/styles.rs`와 `leaves.rs`다. player_theme의 기존 색상 이름은 같은 토큰의 재노출이며 별도 팔레트가 아니다.

| 범주 | 소유하는 값·원칙 |
|---|---|
| color | 원시 neutral/amber/blue/red 팔레트와 의미 surface/text/border/accent/focus/selected/disabled/danger/state. RGB/RGBA 생성은 토큰 정의에만 둔다 |
| spacing | padding/margin/gap/inset, 제목→설명→탭→본문의 관계, 그룹/필드 간격. 현재4/6/8/12/16 논리 단위 |
| sizing | control32/primary40/title32/caption42/navigation180/rail48/inspector300/queue40, min/max와 icon/hit 영역 분리 |
| typography | Pretendard family/weight, heading23/body13/small11, line-height·tracking·말줄임/줄바꿈 책임 |
| border | 두께 thin1/emphasis1.5/focus2, 의미별 색상, style·inside/outside 정렬. 상태별 임의 선 굵기 금지 |
| radius | none0/small2/control5/window8, pill/full은 해당 형태가 필요한 컴포넌트에서 토큰으로 추가. 화면별 임의 모서리 금지 |
| effects | opacity/shadow/outline/ring·혼합 상태. 필요한 효과를 도입할 때 토큰을 먼저 정의하고 샘플 이미지의 glow를 무조건 복제하지 않음 |
| motion | duration/easing/transition/animation. 자동숨김 같은 application 정책 시간과 장식 transition 시간을 구분 |
| layout | breakpoint/container/grid/size constraints, z-order/layer/overflow. arbitrary spacer·화면별 마법 숫자로 별도 체계 생성 금지 |
| input | focus ring·disabled·hover/pressed/selected, pointer/hit/cursor와 접근성 표시. 시각 도형과 입력 영역 분리 |

범주별 사용하지 않는 값을 대량 선언하거나 dead-code 예외로 숨기지 않는다. 새 효과/그림자/모션 등을 실제 사용할 때 정의·상태·검증 근거를 함께 추가한다. 미디어 원본 좌표·PTS·도메인 검증 범위·정수0 같은 데이터는 디자인 토큰으로 바꾸지 않는다. 원본 pixel과 논리 UI 단위를 혼동하지 않는다. SVG stroke/viewBox는 공통 아이콘 원본 규격과 연결한다.

## 이번 적용과 후속 이관

모든 presentation의 직접 RGB/기본 흑백 선언을 color 토큰으로 이동했다. egui style, caption/control icon, media rail, selection frame, preview/canvas/layer, tab의 border/radius를 토큰에 연결했다. 제작 shell은 padding12, inspector padding12, 제목/설명4·탭 전16, 탭높이32, 탐색/검사기 breakpoint 토큰을 사용한다.

아직 남은 숫자 전체를 토큰화 완료했다고 선언하지 않는다. legacy toolbar의 일부 크기·간격, canvas/thumbnail/filmstrip의 보기 크기, 컴포넌트별 font override·glyph/hit 크기는 사용처별 의미 토큰으로 계속 이관한다. 새 코드는 이 체계를 선행 적용한다. 원시 값 grep만으로 화면 일치·접근성 인수를 대체하지 않는다.

색상이나 spacing 토큰을 변경하면 영향을 받는 공통 컴포넌트와 normal/hover/pressed/focus/selected/disabled를 함께 검토한다. 헤더 여백은 단일 screenshot의 빈 공간이 아니라 제목·설명·탭·본문의 정렬 관계로 판정한다. 전체 작업범위·불일치는 각 화면 시각 계약과 실제 관측에 남긴다.
# 정보 안내 색 후속

2026-10-19 queue-refined.png 현재 단계의 정보 banner를 기준으로 공용 notice::Kind::Info는 INFO_SURFACE/INFO_BORDER/INFO_FOREGROUND(파랑 계열)를 사용한다. Warning은 기존 NOTICE_SURFACE/NOTICE_BORDER/ACCENT(황색)를 유지한다. icon/제목/내용의 구조·border1/radius5/padding12는 공유한다. 상태를 색만으로 구분하지 않고 Info/Warning SVG와 안내 문구를 함께 유지한다. 값은 시안 계열에 맞춘 논리 팔레트이며 raster 정확 색 추출 결과는 아니다.

# 검토 문서 높이

`size::REVIEW_TEXT_HEIGHT = 256`은 공용 `text_document`의 원문 스크롤 최대 높이다. 본문은 전체 줄바꿈·선택·복사를 제공하며 글자 수로 자르지 않는다. 화면별 임의 높이 대신 이 토큰을 사용한다.

