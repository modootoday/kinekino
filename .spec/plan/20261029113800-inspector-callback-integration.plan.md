---
id: plan:kinekino/20261029113800
title: Inspector Boolean의 제품 연결 경계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Inspector Boolean의 제품 연결 경계

2026-10-20 현재 원본 코드 직접 대조. Meari 격리 Core의 Boolean enum은 작성됐지만 기존 live 라이브러리에는 아직 인수되지 않았다. Native 생산자 구현/검증 전 제품 adapter를 새 enum으로 컴파일하거나 공개 지원으로 표시하지 않는다. OBJ 추가 source는 동결한다.

## 현재 실제 연결

`proposals/callback_adapter.rs`의 QueueBinding은 DocumentChoice/UnboundText/Text/Action만 있다. bind_review_callbacks는 타입별 HostCallbacks를 같은 등록에 묶고 QueueCallbackAdapter는 ReviewIntents에 enqueue한다. `window_bridge.rs`의 admit_callback은 stamp/registry scope를 확인한 뒤 session.dispatch_handle_after_renderer_guard에 전달한다. after_dispatch는 기존 queue를 drain하고 SDK completion을 pump한 다음, 남은 callback suffix가 없을 때 원 document의 atomic props/export 교체를 수행한다.

Inspector의 Boolean은 Save/Accept/Refresh 같은 ReviewAction이나 text field가 아니다. 기존 ReviewIntents에 임의 Boolean text를 넣거나 callback.invocation.prop 문자열만 보고 권한을 우회해 상태를 바꾸지 않는다. validation/handle dispatch/export revocation은 Meari 공개 포트와 같은 registry 경계를 유지한다. 추가 application SDK/registry/event loop는 만들지 않는다.

## 생산자 인수 후 구현 순서

1. 원 inspector-dock.tsx의 Boolean signature/표현식/forceMount/hidden/controlled open이 실제 Native callback으로 전달되는 최소 graph를 인수한다. production application7은 P2 navigation capture를 포함하므로, P1b만 검증할 때 원 StudioFrame/InspectorDock와 실제 editor graph의 nav/queue Empty 조합을 별도로 명시한다. 원 production graph의 미지원 코드는 삭제하지 않는다.
2. 현재 owner와 registration/export 수명 안에 Inspector의 typed Boolean callback binding을 추가한다. Callback payload 타입을 확인해 bounded presentation intent로 enqueue하고 admission 중 SDK/state/문서를 변경하지 않는다. ReviewAction·TextInput으로 변환하지 않는다. Meari의 Boolean variant 실제 인계 전 정확 Rust enum/함수명을 새로 발명하지 않는다.
3. after_dispatch에서 Inspector intent를 InspectorViewState에 반영하고 needs_projection을 설정한다. 같은 renderer/document guard 밖에서 실행하며 SDK edit queue/completion pump 경로는 보존한다. 복수 queue의 용량·FIFO·rejected/remaining suffix 처리는 기존 HostTurn 권위와 일치시킨다. 단순 latest-value overwrite나 무제한 queue는 쓰지 않는다.
4. frame composition에 원 InspectorDock component reference와 안정된 inspector/children key를 보존한다. 기존 동일 document.node_slot과 atomic props/export 교체에 포함하고 실패 시 기존 draft/dirty/DOM/callback export를 보존한다. pointer/keyboard focus 실행은 실제 Native focus 포트 인수 후 추가하며 InspectorFocusRequest 반환만으로 성공 처리하지 않는다.
5. 초기/등록 후 실패 recovery·동일 prepared ACK·registry/context/client/worker·typed join을 기존 경로에서 확인한다. Inspector 열기/닫기/viewport 전환의 SDK mutation 호출 수는0이어야 한다. 편집/Save 등의 기존 실제 SDK 유스케이스는 그대로 유지한다.

## 현재 초안과 미확정

InspectorViewState는 layout/drawer_open만 소유하고 wide visibility, modal, hidden, trigger/header projection을 계산한다. Drawer close의 focus 요청은 의도만 나타낸다. wide dismiss를 무시해 drawer 선호를 유지하는 동작은 provisional이며 Native/Dialog의 controlled 권위와 함께 인수해야 한다. draft·dirty·pending·selection은 SDK/editor 소유라 이 상태에 복사하지 않는다.

미확정은 Boolean enqueue용 registry callback 표면, renderer widget의 controlled authority, hidden forceMount의 a11y/hit/focus 차단, 호스트 focus 복귀 포트다. Core enum 통과만으로 이 네 경계를 완료하지 않는다. 실제 dirty/focus 보존·실패 rollback·stale export·viewport 변경은 [인수 표](inspector-dock-consumer-acceptance.md)를 사용한다. 현재 source 작성/읽기만 수행했고 새 Cargo/GUI0이다.
