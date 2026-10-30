---
id: plan:kinekino/20261029113816
title: 공개 P1b Inspector의 제품 소비 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공개 P1b Inspector의 제품 소비 제안

2026-10-20 공개 인계 문서 `runtime/development/meari-p1b/.spec/evidence/handoffs/native-controlled-public-2026-10-20.md`와 docs/native-controlled-widgets.md를 읽었다. 고정 인계 commit은0589b58a9e8b40be1ff10b8ad27b3dfd27f68d61, 구현d60c68db632cef6b3f856a12c41b895d61677f0f, 최종362 inventory D8E991…이다. 현재는 제안이며 제품 소스/manifest/라이브러리 경로를 변경하지 않는다.

## 인수된 표면과 유지 경계

Boolean signature/arguments/source가 추가됐고 명시 controlled Dialog/AlertDialog.open 및 Switch/Checkbox.checked만 Native 공개 허용된다. scoped 등록/export와 typed Boolean을 사용한다. 원 InspectorDock의 asChild Button/forceMount/nonmodal/dirty/focus·accept/reject/rollback은 라이브러리 CPU DOM 근거이며 제품 SDK 인수는 별도다. source-only/default-only/child 추가handler 합성은 fail-closed다. Web/WASM/GL/전체HMR/제품 OS 인수로 확대하지 않는다.

Core·Native·paint는 같은0589 checkout으로 조립해야 한다. live2c Core와 새 Native를 혼합하지 않는다. PM이 별도 승인한 fast-forward/별도 snapshot 전환·gitlink/Cargo owner 작업 이후 제품 검증을 시작한다. 기존 UI11 snapshot은old2c의 완료 입력으로 보존하고 거기에 새 Meari를 무기록 덮어쓰지 않는다. P2 branch와도 섞지 않는다.

## 제안 exact writer 경계

| 경로 | 최소 변경 책임 |
|---|---|
| proposals/frame_composition.rs | 원frame+원inspector-dock graph를 실제 editor/provider와 결합. inspector/children 안정key 유지, nav/queue는 명시Empty인 P1b 조합을 별도 표현 |
| studio/inspector_state.rs | 현재 view-state 초안의 accept/rollback 경계 확정. editor/SDK draft·dirty·selection을 소유하지 않음 |
| 신규 studio/inspector_intents.rs | bool intent의 bounded mailbox·source scope/node lifetime·accepted drain을 presentation 책임으로 분리. 새 SDK/registry 아님 |
| proposals/callback_adapter.rs | 같은 실제 registry 등록/export 안에 명시Boolean binding을 추가해 새 mailbox에 enqueue. prop 문자열 우회·Text/Action 변환0 |
| proposals/document_factory.rs | 같은 root scope/registry/context/ACK/worker 경로에서 추가 binding/mailbox 수명을 조립. 기존 실패owner/recovery 보존 |
| proposals/window_bridge.rs | 기존 admit→after_dispatch에서 guarded dispatch/drain과 staged Inspector projection을 같은 document atomic commit에 결합 |
| proposals/initial_panel.rs·product_window/mod.rs | 기존 선택/준비 흐름을 재사용하는 opt-in Inspector composition 인자 전달만. 기본 launcher 전환은 별도 |
| proposals/mod.rs·studio/mod.rs | 필요한 단일 module/export 등록 |
| 신규 consumer focused test | 동일 실제 SDK/editor의 열기/닫기·dirty/useId/NodeId·reject/rollback·stale export·ACK/typed join 검증 |

원 inspector-dock.tsx/frame.tsx/공통style는 유지한다. production application7의 navigation scalar capture는 P2 미인수이므로 그 원 source를 제거하거나 callback 단순화로 진단을 숨기지 않는다. product graph 전체 성공과 P1b-only graph 성공을 구분한다. 공통 component를 임의 복사한 전용 Dock을 만들지 않는다.

## 상태와 원자성

입력 admission은 HostStamp/실제 registry scope/CallbackHandle/Boolean payload와 queue capacity를 검증하고 enqueue만 한다. SDK mutation과 document/view-state 변경은 renderer guard 뒤다. 기존 ReviewIntents의 SDK TextInput/Save/Accept로 Boolean을 전달하지 않는다. 별도 mailbox를 쓰더라도 기존 suffix/용량/순서 계약과 같은 host turn에서 통합하며 pending SDK 작업을 재전송하지 않는다.

after_dispatch에서는 현재 InspectorViewState의 사본에 intent를 적용해 next props를 준비한다. 원본 view-state를 먼저 바꾸고 실패 후 DOM만 유지하면 state/DOM이 어긋나므로, same document try_reproject와 callback export 교체가 성공한 뒤에만 committed state로 승격한다. reject/pending/실패는 현재 open/modal/hidden·dirty editor·focus·기존 export를 유지한다. next export의 실패 cleanup과 옛 lease revoke는 기존 session 경로를 재사용한다.

라이브러리는 승인된 props 반영 후 focus를 이동하므로 제품의 InspectorFocusRequest를 별도 OS 명령으로 중복 실행하지 않는다. trigger 복귀·nonmodal·숨긴 forceMount의 hit/a11y/focus 차단을 실제 같은 document에서 확인한다. wide dismiss를 무시하는 기존 초안 규칙은 제품 임의정책으로 굳히지 않고 원 Dock effectiveOpen과 공개 controlled 동작에 맞춰 인수한다.

실제 SDK/client/registry/host/worker는 하나다. source/저장 historical 표시·unknown quality 경계는 현재 인수된 UI11 결과를 유지하되 새 source 입력으로 다시 구분한다. 공개 Boolean enum이 늘어난 exhaustiveness 영향은 제품·consumer의 모든 기존 match에서 정확히 확인하고 wildcard로 삼키지 않는다.

## 후속 배정 요청 범위

P3 후속의 영상 결합은 [Native playback child 조립 계획](../native-playback-child-assembly.md)의 geometry 권한 대조를 따른다. window Registrar.geometry와 borrowed NativeParentLease는 특정 TSX 영상 slot의 rect/clip/z-order/focus 권한을 뜻하지 않는다. 공통 NativeSurfaceSlot은 document/source generation·slot identity/layout revision·client rect/ancestor clip·visibility/z-order/focus·무효화/Destroy 관측을 별도 제공해야 한다. UI는 실제 영상 영역과 overlay 요구를, Meari는 일반 slot 계약을, SDK는 OS surface/mpv 어댑터를 소유한다. Inspector/P2 gate 이후의 범위이며 whole parent HWND나 별도 mpv 창으로 최종 결합을 대체하지 않는다. 이번 source9에는 영상 포트를 추가하지 않았다.

구현 후속: PM이 source writer와 live Meari fast-forward를 배정해0589 전환 및 제품 source9 작성을 완료했다. 기존 initial_panel/product_window 위임은 그대로 재사용했으므로 해당2파일 변경0이다. 소스 경계·bounded queue/상태 사본·원자적 commit 및 새4개 실제 SDK consumer 시험은 [구현 기록](../../../sots/verification/presentation/native-inspector-consumer-2026-10-20.md)에 있다. 아직 compile/test/strict/GUI 인수 전이며 아래 미래형 제안은 그 배정과 소스 기록으로 구체화됐다.

우선 source 조립 기준과 exact writer를 PM이 확정한다. 다음 consumer gate는 원 Dock+actual SDK editor의 fixed P1b composition, same scope registry/Boolean typed dispatch, accept/reject/failed atomic export, stable editor dirty/focus/useId, viewport wide/drawer, source-generation rejection, typed cleanup/join이다. SDK edit/save/accept는 기존 유스케이스이고 Dock 열기/닫기로 mutation0을 확인한다. GUI/OS는 필요한 실제 입력 공백에 한해 별도 배정한다. 새 Cargo·GUI·fast-forward·gitlink·index 변경은 이 준비 단계에 포함하지 않는다.
