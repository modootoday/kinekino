---
id: plan:kinekino/20261029113804
title: "과거 기록: 제품 document factory 소스 준비와 gate"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 과거 기록: 제품 document factory 소스 준비와 gate

이 파일은 과거 준비 상태를 보존한 이력이다. 현재 상태는 [완료 gate](../product-document-factory-gate.md)와 [별도 bridge 준비](../native-window-bridge-gate.md)를 따른다.

현재 [새 factory 실제 gate](../../../../sots/verification/presentation/product-document-factory-2026-10-19.md)에서 첫선택9/전체36pass·0fail·0ignored를 확인했다. strict 최초2지적의 의미 동일 최소수리 뒤 선택9/strict/fmt0·외부shared/manifest차이0·소유생존[]까지 확인했다. 아래 준비/미실행 기록은 이전 단계다. 최종source에서 전체36을 다시 실행한 것은 아니며 실제 NativeHost/window·root 등록·freshCLI/project/provenance·전체 시각 인수는 여전히 후속이다.

상태: 미등록 소스 준비·개별 rustfmt 통과, 컴파일/실행0. SDK heavy 중 추가 cargo/GUI·root/SDK/manifest/lock 변경0이다. 이전 [Native SDK27](../../../../sots/verification/presentation/original-review-sdk-2026-10-19.md) 결과를 이 새 소스에 상속하지 않는다.

## 작성한 책임

`web_ui_adapter/proposals/document_factory.rs`는 기존 SDK inspect·명시 proposal/source identity·명시 항목 index·원본 inventory/ModuleProgram·정확 초기 props 대조와 실제 scope 기반 단일 registry/context/session 조립을 담당한다. 패널 생성기는 panel_model.rs를 재사용하며 source/state/SDK CAS 정책을 새로 구현하지 않는다. initial props는 caller가 준비하되 실제 mounted root identity의 session projection과 완전히 일치해야 한다. 제품이 테스트의 probe scope·fixture ID·항목0을 기본값으로 복사하지 않는다.

외부 register_parent closure와 ParentProposalLease port는 SDK의 `runtime/coordination/product-host/proposal_registration.pending.rs` register/handle/close 후보에 대응한다. factory는 새 SDK/store를 만들지 않고 주입된 client를 inspect와 parent 등록에만 사용한다. SDK concrete adapter impl과 root 등록은 SDK 담당 소유다. 후보 parent lease를 현재 컴파일된 제품 API라고 부르지 않는다.

실패 객체는 typed 원인·cleanup 오류·부분 session/parent/registry와 완료 단계를 보존한다. 명시 cleanup이 실패하면 같은 owner를 보유해 재시도한다. callbacks/session/parent 완료 단계를 반복하지 않으며 이것을 Native driver의 정상 CloseReceipt로 표시하지 않는다. driver/view/OS 종료와 future host turn은 별도 미확정 port다.

기존 ProposalDocumentSession::new는 호환 유지하며 새 new_retaining이 원 PluginError와 부분 owner를 반환한다. factory는 이 생성자를 소비한다. 중간 stage 등록/바인딩 실패 시 retired registration을 포함한 session을 호출자가 보유하고 정리를 재시도할 수 있다. mounted/retired 각각 callback revoke 완료를 기록해 cleanup Busy 재시도 때 callback revision invalidate를 반복하지 않는다. Drop은 best effort이며 성공 receipt가 아니다.

호환 new는 failure에서 error만 꺼내므로 partial owner가 Drop되어 cleanup 실패를 caller가 재시도할 수 없는 한계가 남는다. 제품 factory는 new_retaining 사용을 필수로 한다. Native 콜백 정리 증거는 bool 대신 NotCreated/Unconfirmed/Active/NativeCloseCalled/SessionCloseConfirmed/NativeAndSessionClosed로 구별한다. 실패 cleanup에서 session.close 성공이 확인되면 SessionCloseConfirmed로 기록하며 관측하지 않은 Native close 호출을 임의 완료로 바꾸지 않는다. 부분 세션이 없으면 NotCreated를 유지한다.

unregister의 pending-ticket Busy와 RefCell borrow 충돌은 다르다. parent가 소유한 실제 tickets를 cancel하고 registry의 실제 terminal event/drain을 처리한 뒤에만 unregister를 다시 시도한다. CancelReceipt::Requested는 terminal 완료가 아니며 단순 borrow 해제·반복 close로 해결됐다고 표시하지 않는다. factory는 ticket을 만들어 취소하거나 terminal 결과를 위조하지 않는다. 얇은 기술 adapter 시험은 Pending→unregister Busy→cancel Requested→여전히 Busy→실제 fixture terminal Cancelled→retry 성공과 중복 종료 방지를 확인하도록 추가했다.

## 준비된 얇은 소스 시험

panel_preparation2는 두 번째 명시 항목·다중 refs 순서·독립 필드/원문/raw·foreign/stale/range/fresh 선택을 검사하도록 작성했다. document_factory5는 실제 원본 module·SDK/store/parent registration 주입·실제 scope와 close parent1회, source/item/초기 props 불일치의 등록 전 거부, 기술 parent handle 실패+cleanup Busy→재시도, renderer preflight 실패+registry borrow로 retired cleanup Busy→borrow 해제 후 retry·rejected handle stale, 중복 UI epoch stage 등록 실패의 원인/부분 owner 보존·기존 export 유지를 검사하도록 작성했다. 기술 오류 주입과 실제 SDK 정책 오류는 구별한다.

이 소스 시험7개는 아직 컴파일·실행하지 않았다. Meari 공개 host port 이름이나 미등록 타입을 import하지 않는다. 새 factory/panel은 제품 부모 mod/root에 등록하지 않았고 독립 테스트가 path로 소비하도록 준비했다. 기존 document_session의 recoverable 생성자/정리 보완과 export는 UI 소유 범위의 변경이다.

이후 감사 보완으로 콜백 정리 증거/parent 실패 뒤 retry·pending ticket cancellation/terminal acknowledgement 소스 시험도 추가했다. 기존7은 당시 준비 수이며 최종 수는 실제 다음 실행에서 확정한다. 현재까지 추가 실행은0이다.

## 다음 실행

### 고정된 소스 준비 묶음

`runtime/coordination/ui-product-factory-prepared-r1`에 owned6파일 원본 복사본과 owned-pins.json을 보존했다. inventory SHA256은 `25e9f654f13185e1ea738fb06a7d08f64bf0d3bd1d274c6cc6e86ee2cae325bc`다. 파일은 proposals/{panel_model,document_factory,document_session,mod}.rs 및 독립 tests/{panel_preparation,document_factory}.rs다. SDK parent lease/manifest/context 초안3개와 독립 Cargo/lock은 별도 reference pins로 구별했다. 이는 소스 준비 snapshot이며 Meari 최종 green 뒤 실제 gate 입력은 다시 대조한다.

declared-unexecuted-tests.txt에는 source의 test 함수9개를 나열했다. panel2는 두 번째 명시 선택/foreign·stale·fresh, factory7은 실제scope/close1회·source/item/초기props거부·parent실패보존·cleanup evidence/retry·pending ticket cancel/terminal·stagecleanup Busy·recoverable constructor다. 9개는 선언 목록이며 실행 통과 수가 아니다. 기존 source가 추가로 소비하는 helper·assembly·원본TSX·SDK/Meari 등록 source와 문서 pins도 실제 gate에서 함께 고정한다.

실제 parent 연결은 SDK 초안 ProposalSdkRegistration::register(client,registry,&scope,context)→handle/close를 기존 factory ParentProposalLease::registration/close에 위임하는 SDK 소유 wrapper/impl이다. tests의 Lease는 실패 주입/수명 확인용이며 제품에 등록하지 않는다. adapter/manifest/context를 UI에서 복제하거나 별도 SDK/store를 만들지 않는다. source와 item은 factory가 명시 identity/index로 대조하고 초기props는 actual session projection과 일치해야 한다. 제품 caller의 초기props 준비가 시험용 StateProgram/ProbeScope의 권한을 가져오지 않도록 순수 DTO view 값만 전달한다.

SDK actual 반환→Meari host port gate/정확 공개 서명 인계 뒤 중앙 슬롯을 받는다. 신규 panel/factory 선택→기존 R3/CardHL 포함 전체 --no-fail-fast→strict all targets→fmt를 새 snapshot/pins로 실행한다. source 최초 실패/최소 수리/receipt·typed cleanup 결과를 보존하고 단일 runner·2CPU/BelowNormal/offline locked-j1/기존 cache를 유지한다. 미확정 host-turn 연결은 검증된 API 인계 후 별도 실제 창 수직에서 실행한다.

root 제품 등록·fresh CLI 재열기·project binding·provenance 대상 결합/원 candidate와 수정 output decision 구별·전체13화면 시각 비교·가상 모니터 실행은 후속 미완료다.
