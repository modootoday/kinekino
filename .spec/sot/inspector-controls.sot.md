---
id: sot:kinekino/inspector-controls
title: 검사기 컨트롤의 역할과 미연결 상태
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 검사기 컨트롤의 역할과 미연결 상태

[공용 컴포넌트](components.md), [토큰](design-tokens.md), [화면 구성](screen-composition.md)을 따른다. 전체 화면의 “연결 필요” 필드를 일반 TextEdit로만 대체한 상태는 시안 일치가 아니다. 이 계약은 다음 이관의 설계이며 구현 완료가 아니다.

## UI 인터페이스

UI projection은 stable field_id, 번역 가능한 label, ControlKind, typed optional value, optional choices/range, availability/reason을 가진다. ControlKind는 Text/Multiline/Select/Toggle/Scalar/Color/Path/Readout을 구분한다. label 문자열을 검사해서 입력 종류를 추측하지 않는다. application/domain 정책이나 파일 대화상자를 공용 컴포넌트 안에 넣지 않는다.

동작은 field_id와 typed intent로 caller에 반환한다. 실제 명령/저장/검증은 공통 application use case를 사용한다. 단순 디자인 검토 fixture의 임시 선택을 실제 SDK 설정이나 작업 성공으로 표현하지 않는다. 필드 값과 capabilities가 없으면 disabled와 사유를 유지한다.

## 시각 상태

| 역할 | Ready/연결된 값 | 미연결·알 수 없음 |
|---|---|---|
| Select | input32·trailing SVG chevron·선택값·option popup | disabled select 외형과 연결 필요/선택 필요. 실제 목록이 없으면 option을 발명하지 않음 |
| Toggle | checkbox/스위치와 번역 label, true/false | indeterminate/알 수 없음, false로 치환하지 않음 |
| Scalar | 얇은 round rail·숫자입력·단위·keyboard | 비활성 track·값 —, 실제0/minimum 값으로 표현하지 않음 |
| Color | token swatches·선택 ring·접근성 이름 | 미선택/비활성. 임의 기본색을 저장값으로 해석하지 않음 |
| Text/Multiline | input32 또는 내용 기반 높이·말줄임/줄바꿈 | placeholder/사유. 일반 Readout과 사용자 편집 필드 의미 분리 |
| Path | 표시·선택 명령·파일 충돌/검증 상태 | 경로 없음/선택 필요, 실제 경로/파일 존재를 발명하지 않음 |
| Readout | 시간/크기/sourcePTS/상태 등 표시 | —와 원인. 불필요한 편집 field 외형으로 표를 대체하지 않음 |

hover/pressed/focus/selected/disabled는 같은 토큰을 사용한다. 링·아이콘·문자 상태를 함께 사용하고 색만으로 의미를 전달하지 않는다. 공통 SVG chevron/Info/Warning과 field label/control 정렬을 공유한다. 고정 footer와 본문 scroll ID는 workspace/subview별로 격리하고 상이한 페이지의 scroll이 섞이지 않게 한다.

## 화면별 다음 이관

SEG 건너뛰기·자동재생 toggle/모드 select; SUB 언어·모델·출력 select/범위 scalar와 보존 toggle; OBJ 탐지/추적 segmented·mask scalar; CLIP 비율·모델 select/추천근거 toggle; FRAME 출력·텍스트/색·외곽선; CARD preset/ratio/template·font/color/alt; PROJ 좌표계/upaxis/단위/matrix/UV/output/target select; SAVE track toggles·저장 mode; MODEL 선택·정밀도/장치; JOB 단계/진행 readout; OUT 경로·충돌정책. 세부 필드/정책은 각 화면 계약과 SDK를 따른다.

이관 후 전체 하위탭×Empty/Populated/LongLabels/NeedsInput×좁은/넓은 실제 배치를 비교하고, 코드 구조 검사를 시각 인수로 대체하지 않는다.

## 첫 구현 경계

`components::inspector_field::{Field,Kind,unavailable}`는 stable ID·label·명시적 역할을 받는다. Select는 disabled ComboBox와 공통 chevron SVG, Toggle은 indeterminate와 알 수 없음 문자, Scalar는 채움/손잡이 없는 track과 —, Text는 연결 필요를 표시한다. Readout은 optional 문자열을 표시하며 값이 없을 때 —를 표시한다. FRAME은 snapshot의 소스 제목·프리뷰 시점을 연결하고, DesignSample의 해상도에는 등록된 데모 이미지 크기를 사용한다. 이를 실제 미디어 해상도나 색 공간 검증으로 해석하지 않는다. 선택 목록·boolean·numeric 값을 발명하지 않는다. PROJ/FRAME 2탭/CLIP 3탭의 시작점이며, 편집 가능한 typed value·ready 상태·edit intent·Color/Path/Multiline이 모두 구현됐다는 의미가 아니다. field ID는 번역 label과 독립적이다.

PROJ/FRAME/CLIP/SEG/SUB/OBJ/SAVE의본문scroll ID를workspace/subview별로분리했다. 값의문서상보존정책과별개로동일field ID를다른페이지에서재사용해scroll이전파되지않게한다.

SEG의 OP/ED/건너뛰기 동작은 Select, 자동재생/회차별 확인은 Toggle, 미리 알림은 Scalar로 이관했다. OBJ는 분석 방식/모델 Select, 선택 구간 Readout, 불투명도/경계 Scalar를 사용한다. SAVE는 형식 Select·저장 위치 Path·파일 이름 Text·크기/이어받기 Readout을 사용하며, 저장 방식 미연결 시 어느 radio도 선택하지 않는다. 저장 방식은 공용 described_choice로 제목과 설명을 조합한다. 미확인 트랙 checkbox는 indeterminate이며 공용 track_selection과 unavailable_select로 표시한다. OBJ 분석 segmented·Path·트랙 selector의 외형은 구현됐으나 실제 명령과 값 연결은 미구현이다.

SUB 5개 탭은 `screens::subtitle_inspector`가 stable ID와 역할 조합을 소유한다. 언어/모델/형식/범위는 Select, 옵션은 Toggle, 인원/표본 간격은 Scalar, 비교/근거/미상 ROI·구간은 Readout, 사전/경로/offset은 Text다. offset의 단위·numeric 편집과 사전/경로 picker는 후속이다. 실제 SDK 변환 폼과 demo projection을 혼동하지 않는다. 일반 문자열 목록을 모두 readonly 입력으로 그리던 `unavailable_fields`는 사용처 이관 후 제거했다.

MODEL/JOB/OUT는 `screens::system_inspector`가 조합한다. MODEL의 정밀도/장치는 Select, 미상의 repo/revision/license/크기는 Readout이며 선택된 모델명/설치 표시는 기존 row에서 가져온다. JOB의 진행/시간/입출력은 Readout이고 재시도·취소·출력 폴더는 고정 footer에 둔다. MODEL 다운로드도 같은 고정 footer를 사용한다. OUT의 summary는 SDK field_id를 그대로 사용하며 저장 동작을 검사기에 중복 생성하지 않는다. 실제 장치/모델 옵션·job 단계·다운로드/취소 명령 연결은 별도다.

Color의 첫 외형은 `color_swatches::unavailable`로 구현한다. CARD/FRAME이 원형20·간격6·테두리1과 공통 토큰 색상표를 공유하며 폭이 부족하면 줄바꿈한다. 각 색에 이름과 미연결 tooltip을 제공한다. 팔레트는 디자인 예시이며 실제 선택값이 없으므로 선택 링/기본색을 만들지 않고 클릭·저장 intent를 제공하지 않는다. 연결된 Color 값·편집·선택 링 검증은 후속이다.

Path의 첫 외형은 `input::unavailable_path`다. 유동 경로 입력32와 고정32 폴더 SVG 버튼을 Flex/Grid로 조합하고 경로 선택 필요를 표시한다. 실제 picker command가 없으므로 버튼은 비활성이며 사유를 제공한다. FRAME/CLIP/SUB/SAVE/OUT가 같은 외형을 공유한다. 실제 폴더 선택/취소 시 값 보존/경로 검증/충돌 처리는 application port 연결 후 검증한다.
