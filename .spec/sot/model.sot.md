---
id: sot:kinekino/model
title: MODEL 요소별 시각 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# MODEL 요소별 시각 계약

[r153/r154 실제 표본 검토](../../../verification/presentation/model-review-cycle-2026-10-19.md): 기존 MODEL에 SDK 자료 비교/검토 방법/필수 메모·표본 판정·불변 이력을 연결했다.1440 병렬·760 적층, 실제 원문 읽기·복제 store 테스트 저장/재시작 이력·item 격리·없는 package 오류를 관측했다. 이 확장은 [별도 UI 계약](model-review.md)을 따른다.11frame 일시 레이아웃 경고의 원인과9개 전체 GUI 읽기 등은 미완료이며 기존 시안의 픽셀 일치 완료로 확대하지 않는다.

[r121 연속 그룹 검토](../../../verification/presentation/model-connected-groups-review-2026-10-19.md):1440 단일 외곽선·구분선·48높이 정렬/클릭/Space,760 후보 접기·음악 그룹 클릭/Space·하단 스크롤·설정/Escape 보존을 확인했다. 모서리 hover 확대와 다른 상태는 남아 있다.

MODEL-01b 연속 그룹 후속 설계: models-refined.png의 기능별 영역은 하나의 둥근 외곽선 안에서 수평 구분선으로 이어진다. 기능별 여섯 그룹은 공용 Accordion group 프레임·48높이 헤더·0행 간격을 사용하고 내부 헤더의 개별 radius/border는 제거한다. 후보 데이터는 기능별 분류 계약이 없으므로 별도 후보 영역을 유지한다. 카탈로그 개수·설치 checkbox를 임의 생성하지 않는다. 클릭·Space/Enter·Tab과 내부 focus 외곽선은 유지한다. 실제 검토 전 일치 완료로 표시하지 않는다.

[그룹 헤더 실제 검토](../../../verification/presentation/model-group-header-review-2026-10-19.md): r120의1440에서 SVG chevron·의미별 glyph, 오른쪽 빈 영역 클릭, Space 접기, Tab/Enter 다음 그룹 펼치기와 후보 선택/검사기 동기화를 확인했다. 다른 폭·상태 및 연속 행 외형은 별도다.

2026-10-19 MODEL-01b 그룹 헤더 후속: 공용 Accordion은 native CollapsingHeader의 상태/입력과 명시 ID를 유지하며 전체 폭·최소32높이·chevron과 기능 SVG20을 조합한다. 후보 그룹의 CollectionState 문구/실측 개수를 보존하고, 미연결 기능 그룹에는 임의0개를 추가하지 않는다. Mic/OCR/music/document는 직접 작성 SVG, 분할·추적/깊이는 기존 의미 SVG다. 기능별 실제 분류·카탈로그 내용과 헤더 외형은 구분한다.

[r117 선택 상세 검토](../../../verification/presentation/model-detail-review-2026-10-19.md):1440 선택/End·상세 배지/필드/다운로드,980 긴 문구 drawer·scroll/footer·Escape·Empty 선택 안내를 확인했다. 실제 catalog/설치와 나머지 상태는 별도다.

2026-10-19 MODEL-02/07 상세 후속: models-refined.png의 (1120,78,445,124)을4배 확대해 정보 heading→모델명/설치 상태 badge→설명을 비교한다. 선택 행의 원문은 공용 Badge로 구분하고 DesignSample과 실제 설치 상태를 구분하는 설명을 둔다. 선택 모델이 없으면 모델 선택 안내를 표시하며 임의 후보의 저장소/revision/license 필드를 생성하지 않는다. 하단 다운로드는 공용 Button40에 기존 아래 화살표 SVG를 결합하고 미가용 사유를 문자로 제공한다. 다운로드/기본 모델 설정 가용성은 변경하지 않는다.

[r100/r101 실제 목록 상태](../../../verification/presentation/collection-state-review-2026-10-19.md): 일반 미지원 확인 불가와 explicit Empty0/Populated3을 구분해 관측했다. 일반 Unknown/NeedsInput/Ready empty adapter는 미검증이다.

후보 heading의 개수는 공용 CollectionState를 따른다. 실제 Live Unsupported/Unknown/NeedsInput은 확인 불가/확인 필요/입력 확인 필요와 안내를 표시하고0개 후보로 해석하지 않는다. Ready 또는 명시적 DesignSample만 표시 행 수를 사용한다. [상태 검토](../../../verification/presentation/collection-state-review-2026-10-19.md)를 따른다.

[r89 실제 검토](../../../verification/presentation/model-metadata-review-2026-10-18.md):1440/760 검색 정렬·배지,760 긴 값 두 줄 배치·End 선택·설정/Escape 보존·Empty 전환을 확인했다.980/1180과 hover 원문은 미검증이다.

MODEL-01 후속: 검색은 공용 unavailable_search의 glyph20/입력32·얇은 border/radius5와160폭 미연결 filter selector로 배치한다. 실제 catalog/filter 능력이 없으면 비활성·연결 필요이며 검색0건/전체를 가짜 상태로 만들지 않는다. 후보 카드의 task/precision/install 원문은 공용 중성 badge24·pill radius12·padding6/4·small11로 구분한다. 임의 성공/허가 색을 부여하지 않는다. 긴 badge는 부모 폭에서 truncate+원문 tooltip, 항목 사이 wrap을 사용한다. 카드 선택·키보드·검사기 동기화와 sample 주석은 유지한다.

후보 카드 선택은 [공용 선택 카드 계약](../shared/selectable-cards.md)을 따른다. 기존 이름만 선택 표시하던 r67 차이를 카드 전체 면·좌측선/별도 focus로 수정한다. 실제 실행 검토 전 전체 일치를 표시하지 않는다.

[r68 실제 검토](../../../verification/presentation/model-card-selection-review-2026-10-18.md)에서1440/760 카드 본문 클릭·Down/End·검사기·설정/Escape·Empty 목록을 확인했다. 기능별 그룹/태그/설치와 나머지 상태는 후속이다.

[기능 계약](models.md)과 [공통 구성](../shared/screen-composition.md)을 적용한다. 기준 models-refined.png, [확대 근거](../../../verification/presentation/system-elements-2026-10-18.md). 치수는 논리 단위의 설계값이다.

| ID | 배치·형태 | 상태·SDK·컴포넌트 |
|---|---|---|
| MODEL-01a | 제목/설명 → 검색 icon+입력 `|` 기능/설치 filter → 기능별 accordion 목록 | 검색은 유동폭 minwidth0, filter 고정·높이32. FeatureGroup/ModelRow, repo+revision 안정 ID |
| MODEL-01b | 그룹 header glyph/기능명/개수/chevron; 후보 row 체크·이름·tag·설치 상태·상세 chevron | 검색0건과 미지원·미설정 구분, row 선택과 작업별 기본 선택 구분 |
| MODEL-02a | 우측 정보 heading → 모델명 badge `|` 설치 badge → 설명 → HF repo field+외부링크 → revision field+설명 | field32/radius5, 긴 commit monospace/tooltip, 가변 ref는 resolve 전 입력, 준비 시 immutable commit 표시 |
| MODEL-03a | 라이선스 label·검토 상태 pill·설명/링크 | amber 검토 필요, 허용/미확인/불가를 문자로 구별. 디자인 예시를 허가로 해석 금지 |
| MODEL-04a | 정밀도 selector → 호환성 경고 | Q4 등 이름만으로 지원 보장 금지, 실제 artifact+engine 능력으로 옵션 제공 |
| MODEL-05a | 패키지 크기 readout/bytes·확인 시점, cache/실행 RAM·VRAM 별도 | unknown은 확인 필요, 추정값 label, 설치 크기와 실행 메모리 혼합 금지 |
| MODEL-06a | 장치 selector → 지원 설명 | 자동/CPU/검증 GPU, runtime 측정과 추천 분리 |
| MODEL-07a | 다운로드 primary hit40와 미가용 이유, 진행 중 bytes/검증/취소 | 파일별 진행→해시검증→준비, 가짜 percent/성공 없음 |
| MODEL-08/09 | 약관/로그인 안내, 준비/해제 상태·사용 작업 링크 | credential 값 표시 금지, 준비 실패 이유·재시도 명명 |
| MODEL-10 | 제거 secondary·대상 경로/크기 요약 | 사용 중 차단/대기, 모델과 cache 제거 범위 명시 |
| MODEL-11/12 | 하단 자원 설정: 절약 mode·동시 수·재생 우선·작업 후 해제 | 공통 queue와 같은 label/상태/저장 책임. 실제 budget 미측정 표시 |
| MODEL-13/14/15 | 선택 후보의 품질·시간·메모리 근거, 진단/기본모델 설정 | 같은 입력·revision의 비교만, 비밀 없는 요약, 작업별 default는 startup loading 아님 |

검사기300/padding12/그룹16, 입력32/실행40, 본문13/보조11, glyph20/hit32. 좁은 창은 공통 drawer, 검색/filter wrap, 목록 내부 scroll. 길게 펼친 그룹이 하단 자원 설정과 caption을 밀지 않는다. Empty·검색0건·LongLabels·NeedsInput·다운로드·검증실패·준비·사용중을 별도로 확인한다. header/row는 keyboard focus와 펼침 상태를 제공한다.

현재 일반 table/InspectorField 문자열만으로 그룹·태그·설치 badge·revision 검토·다운로드 단계가 완성되지 않는다. typed model catalog/details/capability/resource read 계약이 필요하며 실제 모델 스파이크 기록을 임의 DesignSample 기본 선택으로 승격하지 않는다.

후보 목록은 row_id로 입력 ID를 격리한다. 클릭은 해당 행에 포커스를 주고, 공통 keyboard_selection을 사용해 Up/Down/Home/End로 선택·포커스·스크롤을 함께 이동한다. 기본 모델 설정이나 다운로드 실행으로 해석하지 않는다. 실제 키보드 관측은 검증 기록에서 별도 구분한다.
