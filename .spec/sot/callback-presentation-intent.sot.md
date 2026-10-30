---
id: sot:kinekino/callback-presentation-intent
title: 제안 검토 callback의 presentation intent 경계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 제안 검토 callback의 presentation intent 경계

최신 [단일 registry22 검증](../../../verification/presentation/proposal-single-registry-2026-10-19.md)에서 RegistryProposalHost·주입된 한 registry와 실제SDK 회귀가 통과했다. 이전 분리 registry/준비 단계는 이력으로 보존한다. 실제 document/listener/TSX 검증은 포함하지 않는다.

준비된22 source snapshot/pins와 실제 Component·renderer 인수 순서는 [동일 원본 인계](proposal-renderer-handoff.md)를 따른다. 신규 synthetic 수량을 늘리지 않고 본체 인계 뒤 같은 TSX 경로에 집중한다.

2026-10-19 [headless22 gate](../../../verification/presentation/proposal-callback-sdk-2026-10-19.md)에서 intent queue/adapter·실제SDK callback을 포함한 fmt/test22/strictClippy가 종료0이다. 이전 controller16 checkpoint는 보존한다. 실제 TSX callback 연결·node/event guard·GUI는 미검증이다.

## 수명과 실행 순서

ReviewIntents는 실제 ReviewSession의 private owner 및 ReviewScope에 묶인다. source generation/제안/원문 scope가 다르거나 동일 PluginScope로 세션을 재생성해도 이전 큐를 drain하지 않는다. 새 SDK job queue·저장소·application 정책이 아닌 메모리의 presentation mailbox다.

callback은 enqueue_text 또는 enqueue_action(Save/Accept/Refresh)만 호출한다. 이때 SDK를 호출하거나 working draft를 수정하지 않는다. 현재 Meari PluginRegistry dispatch가 반환된 후 event loop가 drain(controller)을 실행한다. Text는 session.change_text, 명시 Action만 기존 controller.save/accept/refresh로 전달한다. 동일 registry를 callback adapter 안에서 재진입하지 않는다.

QueueCallbackAdapter는 공개 PluginAdapter를 구현한다. host가 주입한 scope·queue·노드/필드별 QueueBinding과 원본 byte hash 메타데이터로 manifest를 만들며 새 registry를 구현하지 않는다. capability별 CallbackArguments의 signature/schema를 검사하고 해당 typed intent만 enqueue한다. QueuedAck는 큐 접수 확인이며 SDK 저장 완료가 아니다. queue의 document scope와 adapter scope가 다르면 생성 시 거부한다. plugin unregister/shutdown은 큐를 닫고 남은 입력을 폐기한다. 외부 IO/프로세스·SDK 명령·controller 호출은 adapter에 없다.

NodeLease는 host가 실제 renderer 노드에 연결해 보관하는 opaque 수명 핸들이다. 노드 교체/제거 시 retire_node, document unmount/source 교체 시 close를 호출한다. 큐는 foreign/retired lease를 거부하며 drain에서도 재검사한다. NodeLease가 Meari의 실제 node ID·event·disabled·signature 검증을 대신하지 않는다. 해당 guard를 통과한 host callback만 enqueue할 수 있어야 한다.

## bounded 입력과 오류

큐 항목/동시 node lease는 host가 지정한 capacity(1..64), 대기 텍스트 합계는 max_text_bytes(1..1048576)로 제한한다. 이는 presentation transport 메모리 예산이며 제품 문구의 길이 정책이 아니다. UTF-8 bytes로 계산하고 잘라내지 않는다. 초과·foreign·closed·duplicate action은 명시 IntentError를 반환하며 renderer가 오류 표시와 입력 보존을 처리해야 한다.

같은 노드/동일 TextInput의 연속 대기 입력만 마지막 값으로 병합한다. Action 경계를 건너 병합하거나 다른 필드의 입력을 합치지 않는다. 같은 Action의 대기 중 중복 enqueue는 거부한다. TextInput의 session/generation/kind/index 비교는 opaque 값 내부에서만 수행한다. 최신 draft generation 검증은 drain 시 기존 session.change_text에 맡긴다.

stale text/node나 view 오류가 있으면 해당 항목과 뒤의 대기 Action을 취소하고 DrainReport.discarded/view_error로 보고한다. 실패한 입력 뒤 예전 초안을 저장하지 않는다. canonical SDK 오류는 session에 보존하고 나머지 큐를 취소하며 sdk_error를 표시한다. processed는 intent 처리 횟수이며 SDK 성공/품질 승인 수가 아니다. 재시도·자동 저장·자동 채택하지 않는다.

## 준비된 회귀와 남은 인수

신규4개는 연속 typing 병합/명시 drain 전 SDK호출0/명시save1회, retired node 뒤 save취소, byte/capacity/foreign/unmount 오류, stale field·재생성 session 뒤 save0을 검사하도록 작성했다. synthetic QueueHost이며 실제 TSX 이벤트/SDK 네트워크 인수로 확대하지 않는다. 기존 실제 SDK controller와 raw9 회귀는 유지한다.

추가 headless1개는 실제 Meari HostCallbacks.bind/dispatch→기존 PluginRegistry→QueueCallbackAdapter→bounded queue→dispatch 반환 후 ReviewController drain을 통과했다. Text/Action signature mismatch·foreign CallbackHandle·중복 Action·invalidate/unregister 뒤 입력 거부와 dispatch 중 저장0/drain 후 저장1을 확인했다. 이 fixture의 controller host는 synthetic이며 실제 제품 GUI 인수로 주장하지 않는다. 실제 renderer node guard는 호출하지 않았다.

실제SDK 후속1개는 tests/callback_sdk.rs와 support/callback_fixture.rs다. 실제 ApplicationClient/ProposalFiles 및 기존 ProposalReviewHost를 같은 callback enqueue→drain 흐름에 연결했다. source.json의 실제 bytes SHA를 SourceSnapshot에 사용하고 generated.txt를 따로 보존한다. callback Text/Save/Accept/Refresh dispatch 동안 소유 store의 파일별 해시와 canonical inspect가 불변인지 비교하고, drain 후 저장1/채택1·SDK inspect2를 확인하도록 작성했다. 별도 post-accept inspect Timeout은 host wrapper의 명시 기술 주입이며 실제 SDK가 Timeout을 일으켰다는 증거가 아니다. inspect attempts3 중 주입1을 구별한다. Refresh는 저장·채택을 재전송하지 않는다.

이 후속은 host close/drop 후 새 SDK/ProposalFiles로 재열어 canonical document/receipt 전체, 원문 SHA·generated raw·3/2/3 revisions·automaticfalse·품질/provenance unknown을 비교해 통과했다. runtime/quality-review/player-ui-callback-sdk-r1/93920-1792393854813529100의 calls.json/restored.json/dispatch-store-pins.json을 직접 대조했다. edits1/accepts1/sdk_inspects2/주입실패1 및 mutation재전송0이다. 소유 technical fixture이며 실제 raw9의 품질 검토나 GUI 입력을 대체하지 않는다.

통과한22 checkpoint는 callback용 PluginRegistry와 기존 ProposalBridge의 SDK용 registry를 각각 소비했다. 이후 같은 실제SDK 회귀 source를 한 registry에 두 adapter를 등록하고 RegistryProposalHost를 주입하는 구성으로 변경했다. test 수는22개 유지했고 fmt만 확인한 다음 compile checkpoint다. 실제 제품 document·node guard·CallbackHandle 조립은 입증하지 않으며 SDK 인계 순서에서 실제 Meari document/generation으로 별도 연결한다.

다음은 scoped Meari callback adapter가 실제 registry에서 typed intent를 enqueue하고 반환한 뒤 같은 controller로 drain하는 최소 수직 인수다. 문서/node generation·unmount·capacity·입력보존·중복저장0·explicit refresh를 원본 proposal-review-intent.tsx로 확인한다. root 등록 책임은 [SDK 인계안](../../../../plans/player/presentation/proposal-product-registration.md)을 따르며 UI가 root Cargo를 수정하지 않는다.
