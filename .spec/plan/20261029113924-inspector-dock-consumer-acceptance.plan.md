---
id: plan:kinekino/20261029113924
title: P1b 원 InspectorDock 소비 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# P1b 원 InspectorDock 소비 인수

2026-10-20 소스 작성/검토 준비. Meari 공개 Boolean/Dialog 계약과 해당 consumer 배정 전 새 Cargo/GUI 실행은 하지 않는다. `studio/inspector-dock.tsx`와 `studio/application.tsx`는 P1a 동결 원본을 유지한다. 별도 JS modal, 임의 flatten, 두 개의 inspector 복제, source Boolean 삭제로 우회하지 않는다.

원 props는 narrow/open/title/children/onOpenChange(Boolean)다. 부모의 observed viewport가 narrow를 제공하고 view state가 open을 소유한다. wide의 effectiveOpen=`!narrow || open`, modal=narrow, content hidden=`narrow && !open`이다. narrow 전환·drawer 닫힘에도 같은 children slot key/reference를 유지한다. Dialog의 변경 payload는 실제 Boolean으로 부모에 전달하며 문자열/텍스트 callback으로 변환하지 않는다. 제품 정책/SDK 값을 이 상태 모듈에 복제하지 않는다.

| 입력/전환 | 필요한 실제 인수 |
|---|---|
| wide1280 이상, open=false | content 표시·nonmodal, trigger/header 감춤. editor identity/입력/스크롤 유지 |
| narrow1279 이하, open=false | content 숨김·focus 접근 차단, 설정 trigger 표시. forceMount는 focus 접근 허용이 아님 |
| trigger Enter/Space 또는 pointer | controlled true payload1회, drawer 표시·초기 focus·modal trap, 같은 editor 유지 |
| Escape/닫기/허용된 바깥 dismiss | false payload1회·trigger focus복귀, SDK 편집 내용 보존; dirty 정책은 명시 SDK/host 계약에 따름 |
| 열린 상태 narrow→wide→narrow | 같은 slot/DOM/useId와 pending/dirty/selection 유지. 크기 전환 자체가 SDK mutation을 호출하지 않음 |
| forceMount와 hidden 동시 | 숨긴 content의 paint/hit/accessibility/focus 모두 차단, show 후 기존 identity 유지 |
| invalid/stale Boolean callback | 타입·scope·generation 경계에서 거부, 이전 유효 callback export/화면 보존 |
| 부분 mount/projection 실패 | P1a의 prepared ACK·registry/context·동일 SDK worker·typed cleanup/recovery 경로 유지 |

구현 책임은 공용 presentation 상태/viewport projection, 원 InspectorDock 선언, 기존 SDK frame bridge의 slot composition으로 분리한다. Native Dialog 내부 focus/overlay/typed payload 권위는 Meari 소유다. 부모/자식 두 경로의 controlled state authority가 충돌하면 최소 원source·정확 callback·관측 state로 생산자에 반례를 넘긴다.

작성 초안은 `src/presentation/web_ui_adapter/studio/inspector_state.rs`다. layout/drawer_open만 보유하고 원 Boolean 표현식의 표시 props를 투영한다. Drawer true→false 때 SettingsTrigger 요청을 반환하며 실제 focus 실행을 주장하지 않는다. viewport 전환은 저장된 drawer 선호를 유지한다. wide dismiss를 무시하는 동작은 nonmodal host 소비 시 검증할 provisional 정책이다. 원 TSX/SDK bridge에는 아직 연결하지 않았다. SDK draft/dirty/selection/pending/document를 보유하지 않으므로 상태 복제는 없지만 실제 editor identity/dirty 보존은 향후 동일 slot integration 시험이 필요하다. 집중 상태 시험2개를 작성했으며 중앙 heavy 미배정으로 fmt/test/clippy 모두 미실행이다.

검증은 실제 원 application7 graph의 진단→공개 인계 API에 의한 같은 document projection→집중 Rust 회귀→필수 fmt/test/strict 순서다. 실제 pointer/키보드·760×480/1180/1440·확대 시안 검토는 별도 중앙 GUI 슬롯 이후다. headless 입력을 물리 입력으로, source 선언을 동작으로, CPU raster를 compositor로 승격하지 않는다. SVG/font/assets는 기존 product resolver를 재사용한다.
