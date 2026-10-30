---
id: plan:kinekino/20261029113806
title: UI 세션 목표 본문 갱신 제안·도구 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# UI 세션 목표 본문 갱신 제안·도구 관측

2026-10-21 사용자 직접 active 전환 후 get_goal을 한 번 재조회해 status=active, 기존 전체13화면/Meari consumer 본문이 유지됨을 확인했다. 과거 blocked 응답을 현재 상태로 승계하지 않는다. 이 문서는 **목표 본문 편집 성공 기록이 아니라 요청된 갱신안**이다.

노출된 goal 도구는 create_goal(objective/optional budget), get_goal, update_goal(status)뿐이다. 기존 objective를 편집하는 인자가 없다. 미완료 기존 목표에 create 반복·가짜 complete·DB 변경을 하지 않았으며 현재 active 목표 아래 명시 배정된 작업을 계속한다. [OpenAI 공식 Goals 문서](https://developers.openai.com/cookbook/examples/codex/using_goals_in_codex)를 검색하고 실제 페이지를 열어 사용자/시스템의 lifecycle 제어 범위를 대조했다. 문서가 이 세션에 없는 objective 편집 도구를 제공하는 것은 아니다.

## 제안 본문

승인된 PotPlayer 스타일 native-first 제품 UI를 React/TSX 기반 Meari의 실제 parser/generator/builder/runtime consumer로 구현하고, 공통 Rust SDK 계약에 연결된 실제 사용 가능한 presentation을 완성·검증·.spec 정본화한다. 시청·제작 전체13화면군/모든 하위탭과 공통 프레임·색상/여백/border/radius 등 Tailwind 범위 토큰·SVG·Pretendard·다국어·세부 상태·입력/키보드/패널/접근성 및 기준 시안의 확대 1:1 시각 일관성 범위를 보존한다. 비활성 목업·임의 flatten·다른 JS 정책 런타임·SDK 정책 복제로 완료 범위를 축소하지 않는다.

인수된 V1 및 frame/V2a/V2b CPU 계약을 토대로 실제 Viewer NoMedia public prepare/run의 원 startup/registrar bind·현재 Frame N proof 소비·owned Completion 반영·child-first close/remaining0와 startup 전 원 문서 변경 bind 거절을 별도 근거로 검증한다. 이어 실제 media·native controls/cursor/audio/episodes/전체화면 등 전체 미완료 기능과 13화면 시각 검증을 담당 세션/PM과 조율하며 완성한다. CPU/포트 시험·최종 DTO·컴파일 성공을 OS/시각/영상 완료로 승격하지 않는다.

Rust/SRP/모듈화/계층 규칙·기존 SDK/LocalHost/ExecutionOwner/loop와 원 callback/ACK/cleanup 정책·출처와 허가·고정 pin/원 실패/before/binary/receipt·PID 신원 증거를 유지한다. Meari 기능 부족은 실제 재현·진단·기대 계약을 담당과 조율하고 같은 consumer로 재검증한다. 관련 modoo.today skill 본문과 실제 적용 근거도 기존 요구대로 추적한다. root/index/commit은 중앙 PM 책임이며 단일 heavy/OS/media/AI 실행은 개별 배정에 따른다. 수동 GUI/시각·Pages/VM/DPI·새 영상/장시간 작업·사용자 앱 보존의 현재 제한을 유지하고 최신 사용자 해제/승인만 반영한다. 실제 막힌 외부 경계와 다음 안전한 구현/검증준비 업무를 구분해 계속 진행한다.
