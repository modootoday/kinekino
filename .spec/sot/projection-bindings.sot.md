---
id: sot:kinekino/projection-bindings
title: 제품 공용 패널 projection·bindings 준비
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 제품 공용 패널 projection·bindings 준비

상태: 소스 작성·개별 rustfmt 확인 단계. 미컴파일/미실행이며 마지막 실제 Native SDK27 gate 이후 변경이다. panel_model.rs는 제품 adapter mod/root에 등록하지 않았다. 독립 테스트 panel_preparation.rs만 해당 소스를 path로 소비하도록 준비했다. SDK heavy 중 추가 cargo/GUI는0이다.

`PanelSelection::at(review, scope, index)`는 명시 index만 받고 항목0 기본값은 없다. 선택은 해당 session/draft generation의 opaque TextInput anchor를 보존한다. 준비 시 최신 review_items와 대조하며 범위 초과·다른 scope/세션·편집 뒤 오래된 선택을 거부한다. 배열 index를 stable domain Card ID라고 부르지 않는다. 같은 항목을 계속 표시할 때 factory가 유지한 명시 index로 최신 세대의 선택을 다시 발급해야 한다.

`prepare_panel`은 선택한 Card front/back 또는 Highlight title과 전체 source_refs를 공개 typed array로 투영한다. 참조 원문의 표시 순서도 그 refs를 따른다. 생성 raw·SDK 오류·dirty/refresh/pending 상태를 기존 ReviewSession에서 읽는다. 번역할 revision/source/선택·품질 미연결·결과·시간 표시는 presentation이 공급한다. 품질 플래그나 canonical 시간을 임의로 생성하지 않는다. 원본 TSX·SDK/store/registry/window 생성·새 저장 정책은 포함하지 않는다.

각 준비는 명시 메모리 예산으로 새 bounded ReviewIntents/NodeLease를 만들고 기존 QueueBinding Text 및 Save/Accept/Refresh를 구성해 props와 ReviewBindings를 함께 반환한다. TextInput/callback lease·SDK mutation은 기존 제품 session/controller 경로를 유지한다. 아직 확정되지 않은 NativeDocumentHandle/AfterDispatchHost 타입을 호출하거나 등록하지 않는다.

독립 검증2개는 복수 Card 중 두 번째 명시 선택의 앞/뒤·refs 순서·원문·callback 구성을 대조하고, 범위 초과·foreign session·편집 뒤 stale 및 fresh 선택·첫 항목/raw 보존을 검사하도록 작성했다. 이 테스트 소스는 실제 성공의 근거가 아니며 다음 배정 gate에서 compile/test/strict/fmt로 확인한다. 최신 제품 factory는 이 공용 생성기를 연결한 뒤 실제 원본 Native·SDK 경로를 다시 인수해야 한다.

Close 통합은 Meari의 HostCleanupPending→Complete 후 view 제거, 실패 시 CleanupFailed의 stamp/opaque retry token/단계 완료 기록 보존을 따른다. 현재 session.close 뒤 새 일반 mutation 권한을 활성화하지 않는다. 이미 완료한 callback/session/parent registration 단계는 재시도 때 반복하지 않도록 제품 host가 receipt를 보존해야 한다. forced destroy/Drop은 best-effort/incomplete이며 정상 CloseReceipt로 표시하지 않는다. 해당 새 public 서명과 제품 receipt 구현은 아직 미완료다.
