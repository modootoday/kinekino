---
id: sot:kinekino/proposal-document
title: Proposal document 수명 조립
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Proposal document 수명 조립

최신 [Native Correction 실제 gate](../../../verification/presentation/native-review-sdk-2026-10-19.md)는 원본 Review·실제 module scope·제품 staged 수명·단일 registry·실제 SDK/store 경로의 Native1/전체23/strict/fmt를 통과했다. 아래 source 준비/미컴파일 문구는 당시 이력이다. Cards/Highlights·모든 Drop 실패 조합·GUI/root등록/전체시각 완료를 뜻하지 않는다.

원본 Review→실제 Native document→단일 registry/SDK/store 수직 source는 [후속 gate 계획](../../../../plans/player/presentation/native-review-sdk-gate.md)에 준비했다. 현재 manifest 상속 오류의 소유 수리 전이며 direct rustfmt만 확인했다. 예상23개 실행과 실제 관측은 아직 없다.

2026-10-19 ProposalDocumentSession source를 작성하고 fmt/check 종료0을 확인했다. 단일 registry22 gate 이후의 미컴파일 변경이다. 새 registry·SDK 정책·합성 검사 수를 추가하지 않았다. 실제 listener/TSX/GUI는 미연결이며 다음 중앙 슬롯에서 기존 실제 회귀를 이 소유 경계로 소비해야 한다.

## 소유 책임

ProposalDocumentSession은 기존 ReviewController<RegistryProposalHost>, ReviewIntents, UI intent registration, HostCallbacks 및 callback signature inventory를 소유한다. parent 실제 document가 주입한 registry와 SDK registration은 생성·교체하지 않는다. controller host와 document session의 registry Rc가 동일한지도 확인한다. queue는 동일 ReviewSession owner/scope의 열린 큐여야 한다.

ReviewBindings는 actual renderer 노드에 대응시킬 host NodeLease/현재 TextInput·큐·manifest pins·targets를 주입한다. export_callbacks는 문서가 보관할 CallbackBindings를 반환하고 references는 Component props용 serializable CallbackReference만 반환한다. 참조 문자열은 실행 권한이 아니다. 공개 Meari의 현재 schema6/함수 props/module forwarding/export lease를 소비하며 과거 parser unsupported를 현재 차단으로 취급하지 않는다.

## dispatch와 재투영

actual node/event/disabled guard가 끝난 후 dispatch_after_renderer_guard(export, reference, arguments) 또는 opaque handle port를 호출한다. 둘 다 기존 HostCallbacks와 같은 registry로 enqueue만 수행한다. 실제 document가 export를 소유하고 resolve한 경우 handle port를 사용할 수 있다. port 자체는 node guard가 아니며 browser/source 문자열만으로 호출 권한을 생성하지 않는다.

dispatch가 반환된 뒤 drain_after_dispatch가 기존 controller를 실행한다. 새로운 field generation의 재투영은 reproject에 새 열린 queue/NodeLease/field binding을 전달한다. prepare 단계에서 scope·owner·새 큐 identity·Core callback reference/signature·adapter manifest를 먼저 검증하며 이 실패는 기존 binding을 무효화하지 않는다. 같은 큐를 재사용하면 닫힌 큐를 새 callback에 연결하게 되므로 명시 거부한다.

prepare 성공 후 reproject_with가 별도의 임시 UI registration과 HostCallbacks/export를 stage한다. UI registration의 논리 base ID는 공통 상수이며 문서 내부 epoch를 붙여 기존/임시 binding이 같은 registry에 공존하게 한다. 새 registry나 SDK registration을 추가하지 않는다. host의 apply(export) closure가 renderer props/export 교체를 수행하고, 성공한 뒤에만 새 binding을 현재 소유로 승격하고 이전 owner/queue를 폐기한다.

apply 실패는 ReprojectionError::Renderer로 반환하며 새 stage만 revoke/정리한다. 기존 callback owner·export·SDK registration과 수정본은 보존한다. 세션의 이미 적용된 입력 generation을 되돌리지는 않으므로 새 props 게시 실패는 명시 오류/재시도 필요 상태다. renderer 성공 후 이전 registration cleanup만 실패하면 성공 receipt의 cleanup_pending=true로 구분하고 retry_cleanup으로 처리한다. retire된 묶음은 하나만 유지하며 cleanup이 남아 있으면 새 prepare를 거부해 누적하지 않는다. 실제 Native prepare/commit/last-good 인수는 아직 하지 않았다.

callback 조립 중복은 bind_review_callbacks로 공통화했다. 제품 document 소유 객체와 기존 실제SDK fixture가 같은 signature→HostCallbacks binding 함수를 소비한다. fixture도 공개 export_bindings/resolve/invalidate 경로를 사용하도록 source를 정리했으며 raw host handle 조립을 별도로 복제하지 않는다. 추가 synthetic 시험은 없고 기존22개 수를 유지한다. 이 정리와 document 수명 source는 아직 fmt만 확인했으며 이전22통과를 이번 변경의 실행 근거로 확대하지 않는다.

## close·Drop

close는 queue의 공유 atomic close signal을 먼저 닫고 callback owner를 invalidate한 뒤 UI registration을 unregister한다. queue를 외부에서 borrow 중이어도 Drop은 close signal과 opaque export 권한을 폐기할 수 있다. 큐는 시작 및 각 항목에서 close 상태를 재검사해 뒤따르는 저장/채택을 실행하지 않는다. 이미 완료한 SDK mutation을 되돌리거나 재전송하지 않는다.

registry/adapter가 borrow 중이라 unregister하지 못하면 explicit close는 오류를 반환하고 registration을 유지해 이후 cleanup을 재시도할 수 있게 한다. Drop은 권한을 먼저 폐기하고 cleanup을 시도하지만 오류를 반환할 수 없다. parent document는 dispatch 종료 뒤 explicit close를 완료하고 최종 registry teardown으로 잔여 adapter를 회수해야 한다. SDK registration/cancel·기존 ticket drain은 parent/Meari registry 책임이다.

## PM 경유 Meari 인수 요청

### Native 공개 소스 서명에 대한 제품 적응점

PM이 중계한 아래 서명은 아직 Meari의 실제 gate/서명 안정화 전 인계다. UI가 본체 API를 추정해 추가 구현하지 않고 대응 지점만 기록한다.

| Native 인계 | 제품 연결 지점·남은 조건 |
|---|---|
| from_module_program_with_callbacks(program, css, viewport, assets, source_generation, FnOnce(PluginScope)→Result<CallbackBindings,String>) | opaque CallbackReference props로 원본 ModuleProgram을 준비하고, factory가 받은 실제 root scope에서 parent가 registry/SDK registration을 한 번 조립한다. 같은 scope의 ReviewSession/제품 document session을 보존하고 export_callbacks를 반환한다. ProbeScope를 복사하지 않는다. constructor 실패 시 factory에서 만든 제품 session/registrations도 parent가 정리해야 한다. |
| callback_scope() | 주입된 registry/ReviewSession/실제 module root의 scope를 대조한다. 임의 root instance를 먼저 만들어 맞추지 않는다. |
| drain_callbacks()→PendingCallback{handle,invocation} | native node/event/disabled guard가 끝난 요청의 handle과 invocation.arguments를 dispatch_handle_after_renderer_guard에 전달한다. registry borrow를 해제한 뒤 drain_after_dispatch를 실행하며 원본 prop/span은 진단에 보존한다. 이 대응은 실제 listener 인수 후 사용한다. |
| update_module_props_with_callbacks(props,export) | 최신 field/controlled props와 새 export를 reproject_with의 apply closure에서 전달한다. 제품 staged 소스는 renderer 성공 뒤에만 old binding을 닫으며 실패 시 새 stage만 폐기한다. 성공 후 old export/이미 drain된 old handle 무효 및 실패 시 old lease 생존은 본체 실제 gate와 제품 수직에서 각각 인수해야 한다. |
| close_callbacks()/Drop | native queue와 export lease를 먼저 폐기하고 제품 close로 로컬 큐/owner를 닫는다. SDK registration/ticket cancel·drain은 parent 책임이다. 이미 drain된 opaque handle도 Core dispatch에서 stale이어야 한다. |

현재 제품은 새 stage 준비→renderer apply→성공 후 old 폐기의 소스를 구현하고 fmt만 확인했다. 본체 Convert case의 실패/new export Drop·old lease 생존·성공 old handle stale 근거는 PM 중계이며, Review는 원본 dl 미지원으로 본체 수리 중이다. 원본을 바꾸지 않고 전체 gate 인계 뒤 같은 제품 source로 검증한다. 제품 source 구현이나 일부 Convert 성공을 실제 Native 원본 전체 인수로 확대하지 않는다.

Native/a11y 동일 guard queue는 본체 연결 중이고 Web listener는 미지원으로 인계됐다. schema6/함수 props/forwarding/export API 지원은 유지하며 실제 listener와 제품 조립을 그 성공으로 대신하지 않는다.

### 실제 gate에서 인수할 항목

1. Native 실제 module document에 CallbackBindings를 이전하는 공개 constructor/setter의 정확 서명과 소유·교체·Drop 시점. 같은 원본 proposal-review-intent.tsx와 실제 instance/source generation을 유지해야 한다.
2. node/event/disabled/source generation 검사를 수행한 뒤 bounded callback 요청을 꺼내는 API와 결과 타입. opaque CallbackHandle+typed CallbackArguments 또는 reference+export resolve 경로 중 어느 것을 인계하는지 명시한다. UI는 guard 결과를 위조하지 않는다.
3. callback export 교체와 update_module_props 재투영의 prepare/commit·실패 시 last-good/오류 처리, 이전 node/props의 대기 요청 폐기 보장. UI는 최신 field binding으로 교체하고 SDK registration은 유지한다.
4. unmount/source 교체 때 native queue 폐기·export lease revoke 순서와 기존 registry ticket drain의 책임. 문서 lease가 닫힌 요청이 전달돼도 Core dispatch가 거부해야 한다.
5. Native와 Web listener의 각각 실제 지원/검증 범위 및 원문 source/span을 보존하는 missing-dispatcher 진단. Native 성공을 Web 성공으로 집계하지 않는다.

필요 API 인계 뒤 같은 최소 TSX의 실제 typed 입력→enqueue→dispatch 반환→drain→SDK save/inspect/accept/refresh·reopen을 확인한다. root Cargo/GUI 등록은 SDK 소유를 유지하며 전체13화면·시각 인수는 별도 남는다.

### 현재 인수 상태와 문서 수동 시작 확장

후속 [실제 R3 Document Native gate](../../../verification/presentation/document-repair-sdk-2026-10-19.md)에서 아래 문서 선택/수동 시작 경로와 부정 입력을 실행했다. 합산24pass/원본 CardHL map intake1fail/0ignored·strict/fmt0이며 아래 실행 전 문장은 소스 준비 당시 기록이다. fresh CLI/사람 품질/전체 GUI 인수로 확대하지 않는다.

위 API 준비 기록 이후 원본 Review의 Native→SDK/store Correction23 gate는 [실행 근거](../../../verification/presentation/native-review-sdk-2026-10-19.md)로 인수됐다. 기술 Cue fixture의 성공이며 실제 R3 Document/NeedsRepair, Web, root 등록 및 전체 시각 일치의 성공으로 확대하지 않는다.

문서 수동 시작의 소스 계약은 draft=None에서 원문 항목 번호를 명시 입력하고 immutable DocumentSourceSnapshot의 text_reference로 선택을 해석하는 것이다. 잘못된 번호는 선택을 지우고 InvalidSelection을 반환한다. 별도 BeginDocumentRepair는 선택한 전체 DocumentTextRef와 빈 입력으로 working DocumentCorrection만 만든다. 원문 선택·시작·입력에는 SDK mutation이 없고 기존 draft가 있으면 시작/재선택을 거부한다. Save/Accept/Refresh는 기존 controller·canonical SDK 계약을 그대로 사용한다. 원문/generated raw/historical import_error/저장 draft와 working draft는 구별한다.

제품 TSX와 실제 R3 reader를 사용하는 독립 Native 테스트의 소스가 준비됐으며 아직 컴파일·실행 전이다. 안정된 Meari checkpoint 뒤 소비한다. [유형별 후속 계획](../../../../plans/player/presentation/review-type-native-follow-up.md)에 실행 범위와 미검증 항목을 기록한다.

작업 유형·저장 상태의 허용 여부는 ReviewSession::new가 호출하는 canonical StoredProposal::validate에 위임한다. SourceSnapshot::supports_task는 Document에 Correction만 허용하며, 저장 상태 검증은 Editable과 draft.is_some의 동치를 요구한다. 현재 DraftState가 Editable/NeedsRepair 두 값이므로 유효한 Document+draft=None 세션은 Correction/NeedsRepair로 제한된다. 세션의 저장 snapshot은 외부 가변 참조로 노출하지 않는다. 이 불변식을 UI에 복제하지 않고, R3 테스트에 Cards/Highlights task 및 Editable+draft=None의 세션 생성 거부와 working draft 시작 뒤 재선택/재시작 거부를 추가했다. 추가 검증도 실행 전이다.
