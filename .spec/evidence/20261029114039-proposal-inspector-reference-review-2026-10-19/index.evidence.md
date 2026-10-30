---
id: evidence:kinekino/20261029114039
title: 제안 검사기 이관의 참조 확대 — r170 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제안 검사기 이관의 참조 확대 — r170 준비

새 소스의 실행 검토와 구별하는 참조 관측이다. short-recommend-refined.png 전체 및 우측(1130,110,450,810)2배 파생물 `runtime/review-runs/clip-inspector-reference-r170-2x.png`를 직접 확인했다. 기준 이미지를 수정하지 않았다.

참조는 본문 상단에 영상/transport, 그 아래 후보 thumbnail·근거·행동을 유지한다. 우측은 입력 그룹·구분선·모델·실행 사유·하단2개동작을 분리한다. 낮은 대비 비활성 생성 버튼과 amber 주동작, 작은 control radius·얇은 field border·충분한 그룹 여백을 구별한다. 후보3개·시간·모델값·큐상태는 예시이므로 실제 fixture와 혼합하지 않는다.

공통 제안 검토 모드는 우측의 본문/동작 분리 책임을 재사용하되 별도 application인 문서수정/명시채택을 큐추가·미디어편집으로 표현하지 않는다. 작업설정모드의 기존 세부 설정은 유지한다. footer의 원문/저장/채택 상태가 긴 편집 본문에 밀리지 않는지 실제300폭검사기·760drawer에서 확인해야 한다. 자동renderer 회귀는 이 실제검토를 대신하지 않는다.

현재 새 source는 SDK소유 proposal-inspector-ui-r1 묶음에서 검사 중이다. 고정r169 기존GUI와 결과를 섞거나 본문밀림이 실제해결됐다고 미리판정하지 않는다. UI/UX만 GUI에서 확인하고 기능정확성은 SDK/CLI정본에 연결한다.
