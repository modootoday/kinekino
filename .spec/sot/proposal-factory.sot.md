---
id: sot:kinekino/proposal-factory
title: 제품 Proposal 문서 factory 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 제품 Proposal 문서 factory 계약

후속 미등록 factory·부분 생성 실패 보존 구현 및 얇은 소스 시험의 정확 범위는 [새 factory gate 준비](../../../../plans/player/presentation/product-document-factory-gate.md)에 기록했다. 소스 준비·개별 fmt와 실행 인수는 구별한다.

상태: 제품 root 등록을 위한 입력·소유권 설계. 아직 공용 factory 구현이나 root 실행 인수가 아니다. 기존 [document session](proposal-document-session.md), [제품 조립 인계](../../../../plans/player/presentation/proposal-product-registration.md), [R3 실제 실행 범위](../../../verification/presentation/document-repair-sdk-2026-10-19.md)를 따른다.

## 입력과 선택

제품 Card/Highlight projection·bindings의 미등록 소스와 명시 선택 계약은 [공용 패널 준비](proposal-panel-preparation.md)에 분리했다. 소스 작성은 factory/root 실행의 완료가 아니다.

SDK 조립 지점이 이미 만든 ApplicationClientHandle과 검증된 RequestContext, 명시 InspectProposal, 화면 entry·declared module inventory·원본 bytes/hash, source generation·문서 instance namespace, CSS/viewport/검증 자산, callback intent pins·명시 자원 예산을 받는다. 원문과 mutable registry를 브라우저 JSON에서 받지 않는다. package/item/proposal/source ID·저장 경로의 fixture 기본값은 없다.

proposal은 기존 inspect_proposal로 읽고 canonical StoredProposal을 ReviewSession::new에 전달한다. source ID/revision/SHA는 snapshot에서 얻는다. 외부 선택이 특정 source를 가리키면 snapshot과 대조하고 불일치를 숨기거나 다른 proposal로 대체하지 않는다. 새 생성 결과의 create/source_import·package/item/candidate/companion 선택은 기존 SDK 사용 흐름이며 문서 factory가 임의 후보를 고르거나 raw를 복구하지 않는다.

화면 선택은 공통 Review·DocumentRepair·CardProposalPanel·HighlightProposalPanel의 명시 entry와 inventory를 사용한다. 원본 경로는 제품 등록 인계의 표를 따른다. Card/Highlight 항목 선택은 현재 snapshot/draft generation에 속한 명시 index로 다루고 고정 0 또는 가상의 stable card ID로 대체하지 않는다. generation이 바뀌면 index와 TextInput binding을 다시 얻는다. 전체 source_refs를 공개 typed array 값으로 전달하며 시험 refs cue-a/cue-b를 제품 기본값으로 사용하지 않는다.

DocumentRepair는 draft=None에서 사용자가 원문 항목을 명시 선택한 뒤 별도 수동 시작을 한다. canonical 문서가 허용하는 작업/상태 검증은 SDK에 유지한다. 이미 draft가 있으면 기존 typed payload와 항목 선택을 사용한다. 화면별 props와 bindings 생성기는 UI가 소유하며 fixture의 fixed text/quality label·항목0 선택 코드를 제품 factory로 복사하지 않는다.

## 조립과 반환 소유권

ModuleProgram을 실제 원본 graph/초기 controlled props로 준비하고 InteractiveDocument::from_module_program_with_callbacks에 전달한다. 실제 scope factory 안에서 다음 순서를 따른다.

1. 받은 PluginScope를 기준으로 registry 하나와 RequestContext slot 하나를 만든다. 이미 부모가 registry를 제공하는 호스트라면 그 registry의 실제 scope를 검증하고 같은 것을 사용한다. 두 번째 registry를 만들지 않는다.
2. 공용 제품 manifest·assembly proposal adapter에 기존 client와 같은 context slot을 읽는 ticket factory를 주입하고 SDK registration을 보존한다. grants는 실제 capability descriptor를 따른다.
3. RegistryProposalHost(registry.clone(), sdk_registration.clone(), scope, context_slot, request_namespace)와 ReviewSession(scope, inspected snapshot)을 구성한다. ReviewController와 명시 선택으로 생성한 ReviewBindings를 ProposalDocumentSession::new에 전달한다.
4. export_callbacks를 Native constructor에 반환한다. constructor 실패 때도 factory 안에서 만든 owner/UI registration·SDK registration을 parent가 정리한다. 성공 전 반쯤 구성된 host를 event loop에 게시하지 않는다.

반환할 제품 host는 실제 InteractiveDocument, ProposalDocumentSession, 같은 Rc<RefCell<PluginRegistry>>, SDK RegistrationHandle, context slot, 원본 inventory/source generation과 명시 선택 상태를 함께 소유한다. 이 host의 Rust 타입/모듈은 제품 assembly 등록 시 구현할 대상이다. 현 ProposalDocumentSession은 UI callback owner·현/retired UI registration을 소유하며 SDK registration을 unregister하지 않는다. parent가 SDK registration을 보존해야 한다.

## event loop 호출 순서

Native 입력·키보드·접근성 guard가 끝난 뒤 drain_callbacks의 PendingCallback.handle/typed arguments를 dispatch_handle_after_renderer_guard로 전달한다. dispatch가 끝나고 registry borrow가 해제된 뒤 drain_after_dispatch를 호출한다. 이 단계 전에는 SDK mutation을 실행하지 않는다. UI queue의 처리 결과·canonical ApiError·확정 mutation과 inspect 실패를 구별해 표시한다.

입력/SDK 결과 뒤 최신 review snapshot·draft generation·명시 항목 선택으로 controlled props와 새 ReviewBindings를 만든다. reproject_with의 apply closure에서 update_module_props_with_callbacks(props, export)를 호출한다. 실패하면 기존 문서/lease가 유지되고 새 stage만 정리된다. 성공 후 cleanup_pending이면 성공한 교체를 취소하지 않고 dispatch 밖에서 retry_cleanup한다. 다음 교체 전에 retired binding 정리를 완료한다.

source 이동·unmount·사용자 창 닫기는 document.close_callbacks→session.close→parent SDK unregister 순서다. borrowed/cleanup 실패는 보존하고 dispatch 뒤 재시도한다. 문서와 registry를 먼저 drop해서 정리 오류를 숨기지 않는다. 루트 client/LocalHost/store 수명은 기존 SDK 조립 지점에 남긴다.

## 공개 window hook과 인수 경계

현재 Native window 공개 API 관측과 after-dispatch·redraw·close·전환의 구체 요청은 [window host port 계약](proposal-window-host-port.md)에 분리했다. 이 문서의 제안 동작을 아직 존재하는 공개 함수로 가정하지 않는다.

위 생성·drain·교체·닫기는 현재 문서 API 계약이다. 실제 Native window/event loop가 제품 host에 after-dispatch와 close 지점을 제공하는 공개 hook은 Meari 담당과 확정해야 한다. 단순 mount 성공이나 본체 details/summary 검증은 제품 창의 SDK 연결 성공이 아니다. 기존 eframe Glow와 Native window/GL 기능의 연결은 SDK·Meari 조립 책임이며 UI가 임의 OS loop나 두 번째 player 프로세스를 만들지 않는다.

다음 제품 인수는 고정 fixture 없는 명시 proposal/항목/source 선택, 원본 graph와 모든 refs, actual SDK/store/client 공유, stale/unmount/오류·종료 정리, fresh CLI 재열기를 확인한다. 원 candidate 품질 승인과 수정 output 승인은 분리하며 전체13화면 시각 검증은 계속 별도다.
