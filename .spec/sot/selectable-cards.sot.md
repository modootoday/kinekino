---
id: sot:kinekino/selectable-cards
title: 선택 카드 표시 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 선택 카드 표시 계약

2026-10-18. MODEL 후보 목록에 먼저 적용한다. 기준 models-refined.png 후보 행과 공통 선택 규칙을 조합한다. 기존 r67에서는 이름 영역만 선택/포커스를 표시하고 검사기는 첫 행 또는 선택 행을 보여줬다.

공용 card::selectable은 stable ID·접근성 이름·선택값과 본문 slot을 받는다. 전체 행이 클릭 영역이며 선택은 SELECTED 면과 amber 좌측2선, focus는 내부 FOCUS2 외곽선으로 분리한다. 면·padding12·radius5·border1은 기존 카드와 공유한다. 본문 안에 별도 선택 버튼을 중첩하지 않는다. MODEL 제목과 기능/정밀도/설치 예시는 화면 조합 책임이며 다운로드 정책을 컴포넌트에 넣지 않는다.

모델 목록은 검사기와 같은 실제 선택 행 또는 첫 행 fallback을 시각적으로 표시한다. 선택은 stable row_id, Up/Down/Home/End는 기존 공용 helper를 사용한다. 샘플의 설치 상태·정밀도는 디자인 예시이며 실제 설치/지원으로 승격하지 않는다.1440/760 클릭·방향키·설정/Escape와 Empty를 실제 확인하기 전 완료로 표시하지 않는다.
