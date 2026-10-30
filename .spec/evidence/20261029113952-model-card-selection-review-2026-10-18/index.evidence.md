---
id: evidence:kinekino/20261029113952
title: MODEL 후보 카드 선택 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# MODEL 후보 카드 선택 검토

2026-10-18. 기준 models-refined.png 후보 영역(288,241,784,147)의2배 확대 `runtime/design-inspection/model-selection-r68-2x.png`와 [r67 실제 관측](system-regression-review-2026-10-18.md)을 대조했다. 시안은 후보를 가로 행으로 묶으며 현재 이름만 선택되는 차이를 공통 선택 카드로 수정한다. 기능별 그룹·상태 badge·검색/filter는 이번 수정 범위가 아니다.

## 구현

공용 card::selectable에 stable ID·접근성 이름·선택·본문 slot을 전달한다. 카드 전체 click target, SELECTED 면/amber 좌측2선, FOCUS 내부2선으로 구분한다. Frame 기본 외형은 기존 card::show와 공유한다. MODEL은 별도 selectable 이름 위젯을 제거하고 모델명/기능·정밀도·설치 예시를 같은 카드에 조합한다. 검사기와 동일한 선택 또는 첫 행 fallback을 강조하고 기존 키보드 helper로 선택 ID를 변경한다.

## 실제 실행

r68(PID46488, `runtime/review-runs/ui-model-selection-20261018-r68/player.exe`)을 기존 가상 모니터에서 실행했다.

- 1440×900: 초기 첫 후보 전체 면과 좌측선이 검사기의 첫 후보와 일치했다. 두 번째 카드의 이름 밖 본문 영역을 클릭한 뒤 Down으로 세 번째 전체 카드에 focus/선택이 이동했고 검사기도 세 번째를 표시했다.
- 760×480: 첫 카드 본문 클릭→End가 마지막 후보를 선택하고 해당 카드가 보이도록 스크롤했다. 전체 카드 외곽 focus와 좌측 선택선이 표시됐다.
- 설정을 열면 세 번째 후보가 표시되고 다운로드는 비활성이다. Escape 후 후보 전체 선택면/좌측선과 scroll 위치는 유지되고 focus는 설정 버튼으로 이동한 상태와 구분됐다.
- 760 Empty는 후보0개와 기능별 접기 구조를 유지하며 이전 카드·선택 표시를 제거했다. Empty 검사기 자체는 이번 표본에서 다시 열지 않았다.
- 최종 fmt 검사·전체 cargo test·Clippy all-targets warnings deny·player 빌드 통과. 전체 fixture의 headless 통과는 수동 전 화면 시각 인수와 구분한다.

소유 r68 창 닫기로 종료했고 player.exe/mpv.exe 조회 결과는 비어 있었다. VM·DPI·모니터 구성은 변경하지 않았다.

## 미검증과 남은 범위

hover/pressed, LongLabels/NeedsInput·980/1180·세부기능 그룹/태그·설치 badge·검색/filter·모델 실제 설치/정밀도 지원·revision/license 검증은 별도다. 선택 외형 변경을 전체 MODEL 일치로 표시하지 않는다. 현재 후보들을 하나의 그룹에 둔 구조와 시안의 기능별 후보 그룹 차이도 남아 있다.
