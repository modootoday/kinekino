---
id: evidence:kinekino/20261029114001
title: JOB 선택 면·윤곽 r157
status: active
created: 2026-10-29
updated: 2026-10-29
---
# JOB 선택 면·윤곽 r157

2026-10-19. [JOB 시각 계약](../../player/presentation/system/jobs-visual.md)을 따른다. queue-refined.png의(295,289,809,77)3배 확대 `runtime/review-runs/job-selection-r157-3x.png`에서 전체 amber 윤곽·어두운 선택 면을 확인했다. 기존 framed 표의 좌측 표시선만 있는 선택을 공용 DataTable의 THIN/CONTROL radius 전체 윤곽과 TABLE_SELECTED(AMBER_975) 면으로 바꿨다. keyboard focus는3 inset/FOCUS2로 별도 표시한다. 일반 표와 데이터·정책은 변경하지 않았다.

session13350 fmt--check·전체test139pass9ignored(lib94pass1ignored)·clippy all-targets-Dwarnings14.32s·player build0.82s terminalexit0. test compile49.78s. 기존 클릭/키보드 회귀와 화면 렌더 테스트를 포함하지만 실제 인수는 아래 관측과 구별한다.

실행본 `runtime/review-runs/ui-job-selection-20261019-r157/player.exe`, PID51468, SHA256 `de4163dccfe07922294384488601062c5daa984bd9e3813c811bb2b9a34aa515`. fresh sky 대상 없음 확인 뒤 기존 가상 모니터3500,100에 하나 실행했다. VM·DPI 변경 없음.

-1440 Populated: JOB 탐색에 포커스가 있고 첫 행이 선택된 상태에서는 얇은 전체 윤곽/어두운 면만 표시됐다. 두 번째 행의 출력 셀을 클릭하면 안쪽 포커스 윤곽이 추가되고 우측 상세가 프레임 저장 예시/디자인 소스2로 동기화됐다. Down은 세 번째 행의 선택/포커스와 자산 내보내기 예시/디자인 소스3으로 이동했다.
-첫 출력 셀 입력은 user input detected로 수행되지 않았다. 즉시 새 상태를 관측한 후 같은 작업을 수행해 두 번째 행 선택을 확인했다. 실패한 첫 입력을 성공 관측으로 쓰지 않는다.
-760×480 Populated: 크기 변경 뒤 선택한 세 번째 행이 유지됐다. 첫 행 출력 셀 클릭으로 포커스,End로 마지막 행 선택·전체 윤곽이 보이도록 자동 스크롤,Home으로 첫 행,Down으로 두 번째 행 이동을 확인했다.
-760 drawer에서 프레임 저장 예시가 표시돼 선택과 동기화됐다. Escape로 닫으면 두 번째 행 선택의 얇은 윤곽/면과 스크롤이 유지됐고 포커스는 설정 버튼에 있어 안쪽 윤곽은 없었다.
-760 Empty에서 이전 행·선택 면/윤곽이 제거되고5개 heading과 등록된 작업이 없다는 명시적 빈 목록 안내가 표시됐다.

시안의 작업별 glyph·입력 보조 설명·typed 상태,상단 실행/필터 배치,큰 목록 바닥면은 여전히 차이다. 긴 문구·980/1180·hover/pressed·실제 작업 실행/취소는 이 버전에서 미검증이다. 전체 JOB 시각 완성으로 표시하지 않는다.

캡션 닫기 후 새 sky 목록의 r157 경로 필터 결과는 빈 배열이었다. 창 소멸 확인이며 별도 PID 소멸 증명은 아니다.
