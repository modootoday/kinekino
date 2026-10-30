---
id: sot:kinekino/job
title: JOB 요소별 시각 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# JOB 요소별 시각 계약

[r159 필터 실제 검토](../../../verification/presentation/job-filter-review-2026-10-19.md): 전체만 활성화하는 공용 필터를1440/760 실제 실행에서 확인했다. 비활성 실행 클릭·Right 제외,760 행/상세 동기·Escape·Empty를 확인했으며 실제 상태 조회와 줄바꿈 발생 폭은 미검증이다.

JOB-01 r159 후속 설계: r158 실제6개 상태탭은 선택만바뀌고같은목록을표시했다. typed 상태/filter capability 연결 전에는 ‘전체’만 선택 가능하게 하고 나머지는 비활성·사유를 표시한다. 상태/진행률 문자열로 작업 상태를 추정하지 않는다. 상태 필터는 페이지 탭과 분리해 JOB 본문 상단의 공용 `filter_bar` 버튼군으로 배치한다. 각 버튼은 TAB_MIN72/CONTROL32,THIN border/CONTROL radius·선택 amber 문자/윤곽·안쪽 focus를 사용하고 FlexWrap으로 줄바꿈한다. 키보드는 실제 enabled 항목만 이동하며 원래 항목 인덱스를 보존한다. 기존 JOB 탭 인덱스는 전체로 정규화하고 다른 화면 탭은 유지한다.1440/760에서 비활성 클릭/키보드·상태 미연결 안내와 행 선택/상세·Empty를 실제 확인한다.

JOB 카탈로그도 단일 ‘작업 대기열’ 화면으로 정정한다. 여섯 상태 선택은 그 화면 안의 필터이며 여섯 독립 페이지로 세지 않는다. r158의25개 진입 관측은 이전 카탈로그에 대한 역사적 증거로 보존한다.

[r157 선택 실제 검토](../../../verification/presentation/job-selection-review-2026-10-19.md):1440 선택만/포커스 상태·출력셀click/Down상세동기와760 End/Home/Down·drawer/Escape 선택/스크롤보존·Empty 윤곽제거를 확인했다. 복합셀/목록바닥면/상단배치 및 나머지 미검증은 남아 있다.

JOB-02a r157 선택 후속: queue-refined.png(295,289,809,77)3배 확대에서 선택 행은 얇은 amber 전체 테두리와 어두운 amber 계열 면을 사용한다. 기존 framed 표의 좌측선만 있는 선택 표시를 공용 DataTable의 framed 선택 면/THIN 전체 윤곽/CONTROL radius로 바꾼다. `TABLE_SELECTED`는 선택 의미 토큰이며 작업 성공·입력 필요 상태 색이 아니다. 별도 keyboard focus는 내부3 inset/FOCUS2로 분리하고 일반 표의 기존 외형은 유지한다. 데이터·행 전체 hit/키보드/상세 동기화는 변경하지 않는다.1440에서 선택만 있는 상태와 focus 상태를 따로 확인하고760에서 Down/Home/End·drawer/Escape 보존을 재검토한다.

[r149 프레임 실제 검토](../../../verification/presentation/job-table-frame-review-2026-10-19.md)에서1440 외곽/헤더/행선·출력셀click/Down/Home/End상세동기,1440/760긴문구·760drawer/Escape·Empty를확인했다. 복합셀/목록바닥면/상단배치차이는남는다.

후속 구현 상태: 위상/데이터 의미를 바꾸지 않고 DataTable::show_framed를 JOB에 연결했다. 셀 바탕이 외곽 radius를 덮지 않도록 첫/마지막 열과 첫/마지막 행의 모서리만 적용한다. header 구분선은 실제 셀 rect 안에서 FIELD만큼 세로 inset하며 전체 행 rect의 상단에 가로선을 그린다. 검사 파이프라인 session75245 진행 중으로, 이전r146 실제관측을 수정본 검증으로 사용하지 않는다.

JOB-02a 프레임 후속 설계: r146 실제 표는 행 배경만 있고 시안의 둥근 외곽 프레임/분리된 header 면이 없다. queue-refined.png(296,244,807,276)3배 확대 `runtime/review-runs/job-table-frame-r146-3x.png`에서 외곽 THIN/BORDER·CONTROL radius, CHROME header와 가는 header 열 구분을 재확인했다. 공용 DataTable의 명시적 framed variant로 구현하고 기존 기본 표의 외형을 일괄 변경하지 않는다. 선택 행은 기존 좌측 amber 표시와 별도 keyboard-focus 계약을 유지하며 시안의 amber 행 테두리를 focus와 혼동하지 않도록 상태별로 검토한다. 프레임은 표의 실제 content rect와 내부 clip을 따르고 빈 목록에서도5개 heading과 조회 상태 안내를 유지한다. 작업 kind/입력 보조/typed 상태가 제공되기 전 문자열로 아이콘이나 성공 상태를 추정하지 않는다. 이 항목은 설계이며 구현 전이다.1440/760·LongLabels·Empty 및 행 전체 클릭/Down/Home/End의 hit 영역을 재검토해야 한다.

[r146 순환 검토](../../../verification/presentation/job-cycle-review-2026-10-19.md)에서1440 출력 셀 클릭/Down의 전체 행 선택과 상세 소스 동기,760 내부 스크롤·진단 펼치기·Escape 선택 유지를 재확인했다. 복합 작업 셀/외곽 프레임/실행 영역 배치 차이는 남는다.

[r108–r110 단계·안내 검토](../../../verification/presentation/job-stage-notice-review-2026-10-19.md): 무선택/Live의 임의5단계 제거, DesignSample의 명시 배치 예시,1440 Info 파랑/PROJ Warning 황색·760 drawer/Escape를 확인했다. 실제 stage 계약과 모든 공용 안내 소비자의 인수는 별도다.

2026-10-19 JOB-05 단계 범위 후속: 선택 작업이 없는 상태에5단계·진행 readout·작업 입력 section을 생성하지 않는다. 작업 선택 안내를 제공하며 고정 footer의 비활성 사유는 유지한다. typed 단계가 없는 Live에는 순서/현재 단계를 추측하지 않는다. DesignSample의 선택 행에서만5단계의 시각 배치 예시와 첫 단계 강조를 명시적인 예시 caption으로 검토한다. 진행률/작업명으로 실제 단계를 추정하지 않는다. SDK 세션에 ordered stage ID/label/current/status와 attempt/input_revision 경계를 조율 요청했다.

[r100–r102 실제 목록 상태](../../../verification/presentation/collection-state-review-2026-10-19.md): 일반 미지원 안내·다섯 heading과 explicit Empty의 등록 없음 표시를 구분해 확인했다. 다른 상태/좁은 폭은 미검증이다.

read adapter가 열 metadata도 제공하지 않을 때는 UI의 작업/소스/상태/진행/출력 heading 골격을 유지하고 값이나 행을 생성하지 않는다. 단독 상태 header만 남기지 않는다. 미확인 안내는 표 아래 별도로 표시한다.

빈 행 배열만으로 ‘등록된 작업 없음’을 표시하지 않는다. 공용 CollectionState가 Live read 상태의 미지원/미확인/입력 필요와 확인된 빈 목록을 구분한다. [상태 검토](../../../verification/presentation/collection-state-review-2026-10-19.md)를 따른다.

[r92 실제 행 검토](../../../verification/presentation/job-row-review-2026-10-18.md):1440/760의48행·glyph·상태/진행 분리, 출력 셀 클릭의 전체 행 focus, Down/End·설정/Escape,760 긴 값/Empty/NeedsInput을 확인했다. 작업별 glyph/입력 보조 설명과 typed 상태·단계는 여전히 미완료다.

JOB-02 후속 범위: 공용 DataTable의 행 외형 옵션으로 최소48높이·첫 열의 중성 List SVG20·행 전체 선택/포커스 hit 영역을 제공한다. 작업 종류가 typed 값으로 공급되기 전 glyph를 작업명 문자열로 추측하지 않는다. 표시용 표 투영에서 기존 task/source/progress/output 원문은 유지하고 status 열을 분리하되 값이 없으면 `—`로 표시하며 상태 연결 필요 안내를 둔다. Missing 상태를 대기/실행/완료로 보충하지 않는다. 새 옵션은 JOB에 적용하고 기존 화면의 기본 표 밀도는 유지한다. 행 전체 클릭과 Down/Home/End·검사기 동기화, 긴 문구·좁은 폭을 실제 실행으로 검토한다.

[r90/r91 동작 버튼 검토](../../../verification/presentation/job-actions-review-2026-10-18.md)에서1440 아이콘·label 조합,760 설정의 버튼 단위 줄바꿈·긴 제목·Down 선택/상세 동기·Escape 복귀를 확인했다. 목록/실제 단계/능력과 나머지 상태는 미완료다.

JOB-07/08/09 후속 구현 범위: 공용 Button의 icon 조합으로 재시도 Undo·취소 Close·출력 폴더 Folder SVG20과 label을 같은 hit 영역에 배치한다. 높이는 주요 동작40, border1/radius5를 공유한다.300폭 검사기에서는 세 동작을 3열로 압축하지 않고 각각 온전한 버튼 단위로 Taffy FlexWrap한다. 전체 시작은 Play SVG20을 같은 공용 Button에 조합한다. 실제 작업 capability 미연결 상태는 비활성과 사유를 유지한다. 기준 queue-refined.png의 actions 영역(1140,698,418,54)4× 및 rows(297,245,805,270)2×. 목록 행 높이·작업별 glyph·상태/입력 분리는 이번 버튼 변경과 별도 미완료 범위다.

[기능 계약](jobs.md)과 [공통 구성](../shared/screen-composition.md)을 적용한다. 기준 queue-refined.png, [확대 근거](../../../verification/presentation/system-elements-2026-10-18.md). 치수는 논리 설계값이다.

| ID | 배치·형태 | 상태·SDK·컴포넌트 |
|---|---|---|
| JOB-01a | heading/설명 → 전체/실행/입력필요/실패/완료 filter `|` 동시 수/전체 시작 | 공통 tab/filter32, 선택과 실행 primary 구분. 대기 상태도 필터 접근 가능 |
| JOB-02a | 작업·입력·상태·진행·출력 header28, row 최소48: glyph/작업명+소스·보조 설명 | job_id/attempt/input_revision 선택. current row 중성면+amber선, 상태 icon+문자, focus 별도 |
| JOB-03/04 | 시작 primary40·시작 가능 이유, 대기 순서 동작 | readiness/dependencies 기반, 준비0건 disabled. 순서 변경은 실행 취소 아님 |
| JOB-05a | 우측 선택 job heading/icon/source → 단계 stepper → 현재 단계 banner | 원형 단계와 연결선, 현재 amber+label, 완료/대기/실패 문자. 시안5단계를 모든 job의 고정 정책으로 만들지 않음 |
| JOB-06a | 현재 단계 아래 진행률/경과/남은 시간 label-value 행 | unknown `—`, 실측 progress unit·total·ETA confidence 별도, 불확정 spinner와 percent 구분 |
| JOB-07/08/09 | 입력 summary → 재시도/취소/출력 폴더 row | hit32/주요40, running 취소요청·취소완료 별도, pause는 지원 때만, 재시도는 동일 입력 revision |
| JOB-10 | 입력 부족 필드 amber 문자와 해당 편집기로 이동 | model/path/track 해결 경로, 오류 해결과 재시도 분리 |
| JOB-11/12 | 검증된 output 링크/폴더, 기록 제거 secondary | 이력과 파일 삭제 별도, 임의 shell 실행 금지 |
| JOB-13/14 | 오래된 결과/복구 확인 badge·설명 | 현재 초안 덮어쓰기 금지, 자동 중복 실행 금지 |
| JOB-05b | inspector 하단 접이식 진단 → 오류/해결 설명 | NeedsInput은 안내/경고, Failed는 오류, Cancelled는 중성. 시안처럼 입력필요를 무조건 적색실패로 표현하지 않음 |

검사기300, group16/padding12, body13/small11, glyph20. 좁은 창 상세는 drawer, stepper는 줄바꿈/세로 단계로 전환, label을 지우지 않는다. 목록은 내부 scroll, empty는 작업 없음+진입 안내, long source는 truncate/tooltip. 선택 변경 시 다른 job의 진단·출력·진행값이 섞이지 않아야 한다.

하단 공통 queue40은 전체 job 상세 페이지와 별도다. 접힘 bar와 펼친 목록은 같은 read state를 사용한다. 현재 sample queue 텍스트 나열/일반 table만으로 stepper·진단·의존성·작업별 actions를 대신하지 않는다. typed job detail/stage/progress/action capability 계약이 필요하다.
