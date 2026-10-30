---
id: sot:kinekino/tsx-proposal-fixture-component-renderer
title: 동일 TSX proposal fixture의 Component·renderer 인수 경계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 동일 TSX proposal fixture의 Component·renderer 인수 경계

원본 Review의 [Native Correction 실제 gate](../../../verification/presentation/native-review-sdk-2026-10-19.md)가 Native1/전체23/strict/fmt를 통과했다. 실제 module document의 입력/disabled/lease·제품 수명·SDK/store를 인수했으며 아래 준비/미연결 기록은 당시 이력이다. Web·Cards/Highlights·제품 GUI 등록·전체시각은 여전히 미완료다.

최신 [단일 registry22 gate](../../../verification/presentation/proposal-single-registry-2026-10-19.md)는 RegistryProposalHost와 기존 actualSDK 회귀를 한 registry로 소비해 test22/strictClippy/fmt를 통과했다. 아래 준비 기록은 당시 이력이다. 실제 InteractiveDocument/listener/TSX/시각 인수는 여전히 남는다.

2026-10-19 인계 정본. 고정22 source는 [headless gate](../../../verification/presentation/proposal-callback-sdk-2026-10-19.md)에서 fmt/test22/strictClippy를 통과했다. 추가 synthetic 검사를 늘리지 않고 실제 같은 TSX Component/renderer 연결에 집중한다. 이전 controller16 통과·현재headless22 통과와 실제renderer 미인수를 구분한다.

소유 source snapshot은 `runtime/quality-review/player-ui-callback-sdk-r1/prepared-22-r1`이다. adapter·consumer src/tests/support·독립 Cargo/lock·최소 TSX/manifest와 source-pins.json을 보존했다. upstream Meari/root SDK source를 동결한 snapshot은 아니며 실행 시 담당자가 인계한 실제 revision/checkpoint를 별도로 고정한다. 이 복사본은 증거 보존용이고 include_bytes/path-mod 상대 경로를 재작성한 독립 빌드 입력은 아니다.

## 입력 원본

entry는 `src/presentation/web_ui/verification/proposal-review-intent.tsx`, inventory는 같은 디렉토리의 proposal-review-intent.source-graph.json이다. 원문을 단순화하거나 직접 만든 CallbackInvocation으로 실제 renderer 이벤트를 대신하지 않는다. 과거 consumer r2 span281..320은 snapshot에 보존된 옛 원문 진단으로 현재 변환 성공/실패를 판정하지 않는다.

| 선언/props | host 소유 연결 | 실제 인수 |
|---|---|---|
| proposalId/sourceSha256/documentRevision | canonical 현재 ReviewSession snapshot | 표시값과 SDK source/revision 일치, 현재 player로 암묵 재바인딩 없음 |
| workingText·onDraftChange(text) | 현재 선택 TextInput·QueueBinding::Text·CallbackSignature::Text | 실제 textarea 입력→currentTarget.value→opaque CallbackHandle→bounded enqueue. dispatch 중 store 불변 |
| onSave() | Action·ReviewAction::Save | callback 반환 뒤 drain→기존 controller→SDK edit→fresh inspect. 저장1회와 결과 표시 |
| onAccept() | Action·ReviewAction::Accept | dirty 초안은 SaveRequired, 명시 실행 후 receipt의3/2/3 구분·automaticfalse |
| onRefresh() | Action·ReviewAction::Refresh | 확인된 mutation 뒤 inspect만 수행, mutation 재전송0 |
| input/save/accept/refresh Disabled | 각각의 현재 입력 가능 상태 | renderer가 disabled 노드 이벤트를 차단. enqueue ack를 저장 성공으로 표시하지 않음 |
| useId·label/status | 실제 component instance의 ID·접근성 트리 | label/input 연결·키보드 접근·결과 알림, 재투영에서 동일 instance 유지 |

## 실제 document와 실행 경계

제품 host는 실제 Component/module document의 instance/source generation에서 PluginRegistry와 HostCallbacks를 생성한다. queue는 같은 ReviewSession owner/scope에 연결하며 실제 renderer 노드를 NodeLease에 대응시킨다. source 교체/unmount는 callbacks.invalidate→node retire/queue close→registry cancellation/unregister의 소유 수명을 정리한다. 기존 ticket drain/cancel은 Meari registry 책임을 따른다.

guard 순서는 실제 node/event/disabled/source generation 검사→scoped CallbackHandle/signature→registry grant/schema→enqueue→dispatch 반환→event loop drain이다. callback adapter에서 동일 registry로 controller를 재진입시키지 않는다. NodeLease는 host 수명 표시일 뿐 실제 Meari node/event guard가 아니다. 낡은 노드나 잘못된 signature를 native document host API에 직접 넣어도 거부되는 근거가 필요하다.

통과한22 checkpoint의 callback registry와 기존 ProposalBridge SDK registry는 분리되어 있었다. 이후 같은 actualSDK 회귀를 하나의 주입 registry와 RegistryProposalHost로 바꾼 source를 준비했으며 아직 컴파일하지 않았다. 이 headless scope도 실제 제품 document 인수가 아니다. root 등록 시 SDK 담당이 실제 registry/registration을 기존 ApplicationClient host에 주입하며 ProbeScope를 복사하지 않는다. [제품 조립 인계](../../../../plans/player/presentation/proposal-product-registration.md)를 따른다.

그 대체 경계의 RegistryProposalHost는 후속 단일 registry22 gate에서 compile/회귀/strictClippy/fmt를 통과했다. 공개 CallbackBindings export/resolve/invalidate·drop lease API는 source에서 확인했으며 실제 document가 export를 소유하고 교체하는 renderer 경로는 본체 인계 뒤 검증한다. headless22회귀나 API source 읽기로 이 renderer 경계를 완료 처리하지 않는다.

## 인계 후 같은 source로 확인할 결과

Meari 담당의 Core 인계는 schema6·Props Callback(Action/Text)·CallbackReference·StateProjection.callbacks·ModuleProjection.callback_bindings 및 CallbackBindings(scope/resolve/invalidate/Drop)까지다. 원본2개 byte 동일/Core96·영향19 통과는 본체 담당 실행 근거이며 UI가 이 원본을 실제 renderer로 실행한 결과가 아니다. Native/Web listener·실제 document 단일 registry·GUI checked-present는 미연결로 인계됐다.

UI의 다음 준비 checkpoint는 RegistryProposalHost를 같은 실제SDK 회귀에 주입한 단일 registry 구성이다. tests/support/callback_fixture는 registry를 생성하지 않고 주입받으며, 같은 fixture의 SDK adapter와 callback adapter가 한 registry를 공유한다. 새 합성 수량 없이22개를 유지했고 fmt만 확인했다. 현재 Meari 정적 검사 종료 뒤 이 Rust 소비 compile을 먼저 인수하고 actual renderer/listener gate는 분리한다.

Meari 담당은 실제 parser/lowering의 callback prop/handler 원본 span과 renderer 연결 API·document/node guard·Native/Web 타깃 범위를 인계한다. UI는 그 API로 위 원본을 변환/마운트하고 실제 입력·store 불변·명시 저장/채택/refresh·재열기를 확인한다. Native 성공을 Web 성공으로, headless callback 성공을 TSX 입력 성공으로 확대하지 않는다.

미지원/실패는 같은 source·code/span·예상 동작·타깃·호출 API와 출력 근거를 기록한다. 같은 소스를 유지한 본체 수리 후 재검증하며 별도 JS 런타임·flatten·비활성 목업으로 우회하지 않는다. 최소 경로가 입증된 뒤 correction/Card/Highlight 원본 graph로 확장한다. 전체13화면군의 시각·키보드·패널 인수는 별도 범위로 유지한다.
