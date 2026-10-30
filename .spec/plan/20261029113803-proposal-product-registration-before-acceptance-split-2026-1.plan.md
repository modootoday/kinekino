---
id: plan:kinekino/20261029113803
title: Proposal UI 제품 조립 인계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Proposal UI 제품 조립 인계

## 다음 제품 등록 checkpoint

후속 [NativeWindowHost bridge r2](../../../../sots/verification/presentation/native-window-bridge-2026-10-19.md)는 단일 factory budget 수리 후 전체46pass·strict/fmt0으로 인수했다. 이 결과는 headless runtime/명시 Completion이며 실제 driver 창·backend wake·SDK concrete parent/root 등록은 다음 통합이다.

새 [명시선택/factory/partial cleanup gate](../../../../sots/verification/presentation/product-document-factory-2026-10-19.md)도 첫선택9/전체36pass 및 최소 lint정리 후 영향9·strict/fmt0으로 인수했다. 현재 제품 root 미등록 source이며 NativeWindowDriver와 factory를 실제 창에 연결하는 다음 수직과 구별한다.

추가 [원본 Card/Highlight Native SDK gate](../../../../sots/verification/presentation/original-review-sdk-2026-10-19.md)도 전체27pass/0fail/0ignored·strict/fmt0으로 인수했다. 기술 typed fixture이며 product root/window 등록은 그대로 미완료다. 아래 R3 시점24pass/map1fail은 이전 checkpoint이고 현재 원본 map intake와 두 유형 실제 SDK 경로는 통과했다.

현재 실제 제품 인수 범위는 [Review Native23](../../../../sots/verification/presentation/native-review-sdk-2026-10-19.md)과 [R3 Document Native 경로](../../../../sots/verification/presentation/document-repair-sdk-2026-10-19.md)다. R3 후속은24pass/원본 CardHL intake1fail/0ignored·strict/fmt0이며 root 등록은 여전히 없다. 이후 Meari Repeat의 원본 parser/초기 Native mount·refs·Actions·disclosure 통과 인계는 본체 범위다. 제품 원본 CardHL의 SDK 저장·채택과 동적 Native/Web 최종 인수를 대신하지 않는다. 아래 과거 단계별 기록과 이 현재 범위를 구별한다.

### 실제 런처와 등록 파일

| 관측한 지점 | 다음 연결과 소유 |
|---|---|
| `src/main.rs`의 기존 PlayerSdk 조립 | SubtitleFiles/frame capture/playback status·control와 선택한 ProposalFiles/ProjectFiles/ReviewFiles로 이미 만든 ApplicationClientHandle을 그대로 전달한다. 두 번째 PlayerSdk/store/default client를 UI에서 만들지 않는다. 루트 변경은 SDK 소유다. |
| 같은 main의 LocalHost::start 및 run_with_native_playback | 기존 client clone을 로컬 CLI host와 GUI가 공유한다. UI 등록 때문에 named-pipe CLI를 GUI 자동화로 바꾸지 않는다. 현재 DesignPreview 분기는 SDK 구성 전에 반환하며 저장소 옵션과 함께 사용할 수 없으므로 R3 편집을 이 경로에 몰래 연결하지 않는다. 등록할 실행 모드·명시 저장소 동작은 application::launch 담당 SDK가 정한다. |
| `src/lib.rs`, `src/presentation/mod.rs` | lib에 assembly 미등록, presentation에 web_ui_adapter 미등록이다. SDK가 assembly/meari/proposals와 presentation/web_ui_adapter/proposals의 부모 모듈을 등록하고 실제 document host를 조립한다. 현재 adapter 부모 mod.rs도 없으므로 중간 모듈 선언이 필요하다. |
| `src/assembly/meari/proposals.rs::adapter` | 기존 ApplicationClientHandle·manifest·ticket context factory를 주입한다. `kinekino_player::` self import는 SDK가 crate 경로 또는 self alias로 한 번 정리한다. 같은 소스를 소비하며 별도 제품 복사본은 만들지 않는다. |
| `tools/player-ui-sdk-bridge/src/proposal_bridge.rs::proposal_manifest`, `src/lib.rs::scoped_context` | 현재 검증 도구에만 있는 공통 manifest·ticket correlation을 제품 assembly의 단일 책임 모듈로 이동하고 도구가 재사용하도록 바꿔야 한다. manifest의 include_bytes 경로·provider/contract/schema pin을 이동 후 다시 고정한다. root가 tools에 의존하는 순환이나 정책/descriptor 복제는 금지한다. 이동 구현은 SDK/UI 소유 범위 협의 후 진행한다. |
| root `Cargo.toml`/lock | Windows GUI용 meari·meari-native·player-ui-plugins path 의존과 실제 필요한 renderer feature를 SDK가 등록한다. 독립 bridge는 Native default-features=false/local paint patch로 문서 실행만 검증했다. 제품 window/GL feature와 root의 local blitz-paint patch 필요 여부는 실제 dependency resolution으로 별도 확인한다. 독립 lock을 root lock으로 복사하지 않는다. |

현재 제품 런처는 이미 eframe::Renderer::Glow를 사용한다. Meari 문서 검증의 CPU 렌더러와 제품의 기존 GL 창을 혼동하지 않는다. 실제 Meari Native window/event-loop에 document·SDK callback drain을 주입하는 호스트와 기존 playback/WebView 수명 연결은 아직 제품에 없다. 기존 eframe 창에 문서 객체만 넣으면 동작한다고 가정하거나 Meari의 CLI run 함수를 제품 조립 API로 대체하지 않는다. Meari 담당이 공개 호스트/renderer 경계를 제공하고 SDK가 런처를 연결하며, UI는 문서와 presentation owner를 제공한다.

### 단일 문서 조립 순서

fixture 상수 없는 실제 factory의 입력·명시 항목 선택·반환 소유권·after-dispatch/교체/닫기 순서는 [제품 문서 factory 계약](../../../../sots/player/presentation/shared/proposal-document-factory.md)에 정본화했다. 현 테스트 helper를 그대로 제품 factory로 등록하지 않는다.

1. 기존 SDK reader로 명시 선택한 proposal을 inspect하고 저장 snapshot을 가져온다. 화면 entry와 module inventory를 선택해 원본 ModuleProgram을 준비한다. 다른 source로 이동할 때 이전 문서를 닫고 source generation을 갱신한다.
2. InteractiveDocument의 실제 scope factory 안에서 Rc<RefCell<PluginRegistry>> 하나와 Rc<RefCell<RequestContext>> 하나를 만들고 기존 assembly adapter를 SDK registration으로 등록한다. context factory는 같은 slot을 읽으며 opaque ticket correlation helper를 사용한다.
3. 같은 registry/registration/scope/context slot/문서별 request namespace로 RegistryProposalHost를 만들고 ReviewSession→ReviewController→ProposalDocumentSession을 조립한다. 실제 field/callback bindings를 export해 constructor에 반환한다. SDK registration은 parent host가 소유하고 UI intent registration은 document session이 소유한다.
4. Native guard 뒤 drain_callbacks의 typed 요청을 session에 전달한다. callback dispatch와 registry borrow가 반환된 다음 drain_after_dispatch로 기존 SDK 유스케이스를 실행한다. 변경 뒤 최신 props/field bindings를 reproject_with→update_module_props_with_callbacks로 교체한다.
5. source 전환/창 종료 시 Native callbacks를 닫고 UI session close·queue/export 폐기 뒤 parent가 SDK registration을 unregister한다. 정리 오류는 보존한다. 사용자가 데모 player를 종료한다는 기존 운영 규칙을 유지한다.

### 유지할 원본 식별과 남은 검증

| 화면 | 원본 entry·inventory | 제품 연결 상태 |
|---|---|---|
| 공통 Review | `verification/proposal-review-intent.tsx` | Native→SDK 기술 cue 경로 검증. 제품 root 미등록 |
| R3 문서 수동 수정 | `studio/subtitles/document-repair.tsx` + 위 Review, `verification/document-repair.source-graph.json` | 실제 companion/raw/NeedsRepair→명시 선택·수동 시작→SDK 경로 검증. fixture package/item과 실제 사용자의 선택을 구별 |
| Cards | `studio/cards/proposal-panel.tsx` + 공용 ProposalSource/ProposalActions, `verification/card-proposal.source-graph.json` | 원본 유지. 앞/뒤 독립 입력과 typed refs2·SDK 실제 인수는 후속 |
| Highlights | `studio/clips/highlight-proposal-panel.tsx` + 같은 공용 두 모듈, `verification/highlight-proposal.source-graph.json` | 원본 유지. 제목·시간 미상·typed refs2·SDK 실제 인수는 후속 |

기술 시험의 module mount 이름은 화면 source inventory를 대체하는 저장 식별자가 아니다. proposal/source ID·revision/SHA는 canonical snapshot을 사용하고 표시 제품명과 분리한다. root 등록 인수는 기존 SDK/CLI 회귀, 실제 창·원본 문서 callback/오류/종료, fresh CLI 재열기·project 연결을 확인한다. 후보 품질 승인·전체13화면 및 시각 비교는 별도 미완료다. 현재 Meari 변경 중 새 consumer cargo/GUI 실행은0으로 유지한다.

독립 consumer의 [원본 Native Correction23 gate](../../../../sots/verification/presentation/native-review-sdk-2026-10-19.md)는 통과했다. root Cargo/GUI 제품 등록은 변경하지 않았고 SDK 소유의 다음 작업이다. 아래 source 준비·headless·미컴파일 문구는 단계별 이력이며 현재 범위는 최신 실제 근거를 따른다.

후속 [ProposalDocumentSession](../../../../sots/player/presentation/shared/proposal-document-session.md)은 소유/Drop·재투영·export source를 작성하고 fmt만 확인했다. 기존 단일 registry22 gate 이후의 미컴파일 변경이며 실제 listener 연결에 필요한 공개 API 요청을 정리했다.

최신 [단일 registry22 gate](../../../../sots/verification/presentation/proposal-single-registry-2026-10-19.md)는 RegistryProposalHost와 기존 actualSDK 회귀의 한 registry 구성을 test22/strictClippy/fmt로 통과했다. 아래 source 준비/미컴파일 문구는 당시 이력이며 실제 InteractiveDocument/listener/TSX·root 제품 등록은 아직 미완료다.

2026-10-19 SDK/PM 소유 root 등록 제안이다. UI가 root Cargo/lib/main/presentation 등록을 직접 수정하지 않는다. [controller16 gate](../../../../sots/verification/presentation/proposal-controller-2026-10-19.md)는 headless consumer의 실제 SDK 경로를 입증했으며 제품 GUI 등록을 입증하지 않는다.

## 현재 관측

root Cargo에는 Meari/player-ui-plugins 의존성이 없고 lib.rs는 application/domain/infrastructure 및 Windows presentation만 등록한다. presentation/mod.rs는 기존 eframe player 경로다. src/assembly/meari/{proposals,subtitle_conversion}.rs와 web_ui_adapter/proposals는 독립 bridge가 path-mod로 소비했으며 root 모듈에는 등록되지 않았다.

tools의 ProposalBridge는 검사용 StateProgram/PluginScope를 만든다. 이를 제품 화면의 registry/scope로 복사해서 등록하면 안 된다. 제품은 실제 Meari document/instance/source generation의 registry와 CallbackHandle 수명을 사용한다. 검사용 ProposalReviewHost를 player GUI의 최종 host로 표시하지 않는다.

## SDK/PM이 검토할 구체 등록

| 책임/파일 | 필요한 연결 | 인수 |
|---|---|---|
| root Cargo.toml/lock | 검증된 Meari·player-ui-plugins 경로 의존성, GUI/runtime 범위 명시 | root SDK/CLI 소비자 회귀와 실제 GUI target compile. 독립 bridge에 root가 의존하는 순환을 만들지 않음 |
| root lib/assembly module | 기존 assembly/meari host와 UI adapter 모듈 등록 | 현재 path-mod의 kinekino_player 절대 import를 root 내부에서도 해석할 방식 검토. SDK 소유자가 crate 경로 조정 또는 self alias를 선택하며 소스 복제 금지 |
| GUI 조립 지점 | 실제 ApplicationClientHandle·RequestContext factory·document registry/registration/capability descriptor 주입 | probe scope를 대체한 실제 instance·foreign/stale/unmount 경계 |
| presentation 런타임 | 같은 ReviewSession/fields/ReviewController 및 원본 TSX props/callback 연결 | input→save→fresh inspect→accept→refresh·오류 보존. 기존 상태와 실제 module graph identity 유지 |
| CLI | 같은 application 유스케이스 유지 | GUI 클릭 자동화 의존이나 GUI 전용 기능 정책 추가 없음 |

실제 SDK store·파일 권한·ApplicationClient 수명은 현재 조립 책임을 따른다. 제품 표시 이름·프로토콜 식별·CLI 경로를 UI에서 별도 상수로 만들지 않는다. root 전체 기본 실행을 미검증 renderer로 즉시 전환하는 변경은 이 제안에 포함하지 않는다.

## 단일 document registry의 조립 경로

이 절이 제품 등록 요청의 정본이며 callback 동작 계약은 shared 문서에 링크한다. 현재 headless 두 registry를 제품에 그대로 등록하지 않는다.

| 현재 경로 | 제품의 대체 책임 | 유지/제거 경계 |
|---|---|---|
| tests/support/callback_fixture.rs가 별도 PluginRegistry 생성 | 실제 Meari document 조립 지점이 하나의 registry를 소유 | 시험 helper는 회귀용 유지. 제품 조립은 이 생성기를 호출하지 않음 |
| tools/ProposalBridge가 검사용 StateProgram과 SDK registry 생성 | 같은 실제 document registry에 기존 product_proposals::adapter 등록 | Probe StateProgram/Bridge를 제품 host로 가져오지 않음. 검증 도구는 삭제하지 않음 |
| ProposalReviewHost가 ProbeBridge로 typed request 전달 | 제품 document host port가 실제 registry·SDK registration·정본 descriptor/context를 주입받음 | 기존 typed request/ApiError 경계를 재사용/공통화하고 SDK 우회 호출·두 번째 registry 생성 없음 |
| QueueCallbackAdapter | 같은 registry의 별도 UI intent registration | enqueue만 수행. SDK registration과 plugin ID/capability inventory를 구분 |
| ReviewController 및 ReviewIntents | 실제 ReviewSession과 event loop가 소유 | controller/SDK 정책 복제 없이 callback 반환 뒤만 drain |

제품 소유 구조는 실제 document → registry 하나 → SDK proposal registration + UI intent registration이다. ReviewController의 host port는 registry에 대한 공유 Rust 소유권과 SDK registration을 보유하며 registry를 생성하지 않는다. callback dispatch가 registry의 mutable borrow를 반환한 다음 queue drain이 해당 port를 호출한다. UI adapter에 실제 ApplicationClient를 직접 넣어 callback 안에서 실행하지 않는다.

제품 측 독립 source `src/presentation/web_ui_adapter/proposals/registry_host.rs`의 RegistryProposalHost를 추가했다. 실제 registry/registration/scope/context slot/request namespace를 주입받아 ReviewHost를 구현하며 새 registry·ProbeScope·SDK store를 만들지 않는다. descriptor는 주입된 실제 registration의 manifest에서 조회하고, 매 호출 scope를 재검사한다. canonical Request.validate 후 context slot borrow를 해제하고 registry.dispatch하며 canonical ApiError를 그대로 복원하고 나머지 plugin 오류는 구조화된 details로 보존한다. registry/SDK registration의 종료는 document 조립 책임에 남긴다.

이 relay는 후속 단일 registry gate에서 compile/기존22회귀/strictClippy/fmt를 통과했다. 신규 synthetic 검사 수를 늘리지 않았다. 실제 InteractiveDocument/listener 실행은 별도이며 root 제품 등록에서 이 주입 port를 실제 document 소유 registry에 연결해야 한다.

완료한 compile checkpoint는 같은 tools/player-ui-sdk-bridge Cargo manifest/공유 native target을 소비했다. 기존 tests/callback_sdk.rs의 실제SDK fixture를 단일 registry 구성으로 변경했다. fixture가 StateProgram scope에서 registry를 한 번 생성하고 기존 product_proposals::adapter와 QueueCallbackAdapter를 함께 등록한다. callback helper는 registry를 생성하지 않고 주입받는다. RegistryProposalHost는 같은 registry/SDK registration/context slot을 소비한다. 이전 두 registry/ProposalBridge 경로를 이 회귀에서 제거했으며 별도 Probe 회귀는 보존한다. 시험 수는22개 그대로 모두 통과했다.

소비 변경 파일은 product adapter의 registry_host.rs/mod.rs, tool src/lib.rs/proposal_bridge.rs, tests/callback_sdk.rs/support/callback_fixture.rs다. tool의 기존 proposal manifest와 ticket correlation helper를 재사용하도록 공개했으며 metadata/정책은 복제하지 않았다. root Cargo/API/기존 assembly source는 변경하지 않았다. 검사 중 고정 범위는 root SDK/API 및 현재 Meari core/host_callbacks/registry·media contracts·player-ui-plugins의 소비 source와 독립 tool source다. Meari strict 반환 뒤 중앙 슬롯에서 offline lock 동기화·fmt·기존22회귀·strict Clippy를 수행하고 즉시 동결을 반환한다. 실제 InteractiveDocument/listener/TSX gate는 그 뒤 별도다.

SDK registration은 기존 assembly/meari/proposals.rs의 ApplicationClient/context factory binding을 사용한다. 각 typed 호출은 canonical Request.validate 후 요청 context slot을 갱신하고 slot borrow를 먼저 해제한 뒤 registry.dispatch한다. SDK adapter의 per-ticket context factory가 그 값을 읽는다. mutation expected_revision을 inspect에 복사하지 않으며 request correlation은 기존 host ticket 방식을 공통화한다. 이 역할은 제품 host relay이며 도메인 CAS/timeout/idempotency 정책을 새로 만들지 않는다.

Meari 담당이 실제 document registry 소유 API와 CallbackReference/Handle 주입·node/event guard를 인계한 뒤 UI는 제품 host port source를 해당 공개 서명에 연결한다. SDK는 root Windows GUI 의존성/assembly·presentation 등록과 기존 ApplicationClient를 연결한다. root 등록 전 소비자 path-mod의 crate import 경계는 SDK가 결정하고 승인된 원본을 중복 복사하지 않는다.

### 모델 재투영과 수명

입력 적용 후 TextInput의 draft generation이 바뀌므로 최초 field handle을 영구 callback에 붙이지 않는다. drain 완료 후 최신 field handle과 controlled props를 준비하고 이전 UI callback binding/reference·NodeLease를 무효화한다. 실제 Meari가 제공하는 reference/handle 교체 API를 사용하며 같은 참조 문자열을 다른 필드 권한으로 몰래 재사용하지 않는다.

현재 QueueCallbackAdapter는 host-owned 고정 QueueBinding을 받는다. 제품의 첫 연결에서는 UI intent registration만 교체하고 새 큐/lease/HostCallbacks에 최신 field를 bind하는 경계를 검토한다. 기존 queue는 drain 후 닫으며 SDK proposal registration/ReviewSession·수정본은 유지한다. document unmount/source 교체에서는 두 registration의 수명을 모두 정리한다. 실제 renderer node guard가 낡은 노드/props의 이벤트를 거부하는지 별도 인수한다. 이 수명 경계는 source 준비를 위한 설계이며 실제 제품 구현/실행 근거는 아직 없다.

공개 source에서 확인한 export API는 HostCallbacks::export_bindings→CallbackBindings, CallbackBindings::scope/resolve/invalidate다. export를 drop하면 해당 lease가 무효화되고 HostCallbacks owner invalidate/drop도 기존 handle을 무효화한다. 이 API는 reference를 opaque handle로 resolve하는 경계이며 renderer/node listener 연결을 입증하지 않는다. source 교체/재투영에서 이전 export를 폐기하고 최신 export를 실제 document가 소유하는 순서를 Meari 실행 인계 후 적용한다. Core 정적 검사 중에는 해당 source를 변경하지 않는다.

### 담당별 인계 요청

- Meari: 실제 document instance/generation과 registry 연결 지점, 함수 props의 원본 span/전달, Native/Web node listener·disabled·stale 검사, callback reference 교체/무효화 API 및 실행 근거.
- UI: 기존22 source를 유지한 제품 registry 주입 host relay, enqueue→dispatch 반환→drain 조립, 최신 field/controlled props 재투영과 correction/Card/Highlight 원본 소비.
- SDK: root Windows GUI Meari/plugins 의존·모듈 등록, self import 경계, 실제 ApplicationClient/store/context 주입. root protocol foundation과 UI 등록 gate 분리.

다음 수직 gate는 원본 proposal-review-intent.tsx에서 실제 document 단일 registry·입력·명시 save/inspect/accept/refresh·stale/unmount를 확인한다. 기존 headless22 통과를 이 gate의 완료로 대신하지 않는다. Meari Core5 실행 중에는 core/API를 변경하거나 별도 무거운 검사를 실행하지 않는다.

## callback dispatch와 재진입 경계 상세

Meari source 인계의 HostCallbacks::bind는 typed PayloadProducer를 등록하고 dispatch는 기존 PluginRegistry의 capability를 호출한다. onDraftChange는 working draft 입력이며 매 keystroke를 SDK proposals.edit로 저장하는 동작으로 바꾸지 않는다. onSave/onAccept는 명시 mutation, onRefresh는 inspect만 수행한다.

현재 controller는 동기 ReviewHost를 소비한다. 같은 registry의 callback adapter 실행 중 controller가 그 registry로 재진입하면 mutable borrow/dispatch 수명 문제가 생긴다. 제품 연결은 callback에서 scoped presentation intent를 수신하고 현재 dispatch가 반환된 뒤 event loop가 controller/SDK를 실행하는 경계를 갖춰야 한다. presentation 입력 큐는 SDK job queue나 새 도메인 정책이 아니다. 실제 source callback/renderer 인계 시 이 경계를 Meari 담당과 검증한다.

해당 [bounded presentation intent](../../../../sots/player/presentation/shared/proposal-intent-dispatch.md)의 queue/drain/공개 HostCallbacks→PluginAdapter enqueue·실제SDK callback/store/reopen을 포함한22검사/fmt/strictClippy가 통과했다. 실제 renderer callback 연결은 미검증이다. 실제SDK headless도 callback registry와 검증 probe의 SDK registry가 분리되어 있으므로 제품 실제 document의 단일 registry 등록을 대체하지 않는다.

저장/채택 성공 receipt를 보존한 뒤 inspect 실패를 별도 표시한다. callback 문자열에서 프로그램·URL·SDK 명령을 임의 실행하지 않으며 schema/grant/opaque handle·node/event guard를 우회하지 않는다. 원문·quality unknown·source_refs·None PTS는 현재 canonical 계약을 유지한다.

## 다음 수직 인수

### 원문 비교·사람 수정·명시 채택의 SDK 요청 경계

현재 ApplicationClient와 source_import source를 읽기 대조했다. 다음 표의 기존 계약을 먼저 재사용하며 SDK 속도/tracks·runtime 등록 파일은 UI가 수정하지 않는다. SDK native-r3 실제 회차 CLI frame/4controls/close/reopen/staleID/EOF 성공은 playback checkpoint이며 proposal Native23이나 모델 품질/전체 UI 인수와 합치지 않는다.

| 목적 | 이미 있는 계약 | 인계할 남은 요청/소유 |
|---|---|---|
| 실제 원문·후보 선택 | inspect_review(InspectReviewPackage), read_review_text_asset(ReadReviewTextAsset) | package/item/source asset/candidate asset를 명시 선택하는 제품 UI 연결. 파일을 UI가 직접 신뢰/파싱해 SDK 검증을 우회하지 않음 |
| snapshot 구성 | application::proposals::source_import::from_request_asset | canonical SHA/bytes 검증·DocumentTextRef 추출·Cards/Highlights None PTS를 그대로 재사용. 새 UI parser 요청 없음 |
| 생성 raw/repair | create_proposal/inspect_proposal 및 generated.raw_text/import_error/draft/state | NeedsRepair의 draft=None에서 명시 수동 typed draft 시작은 UI working-state port 책임. 첫 cue/참조를 몰래 채우거나 raw를 자동 복구하는 SDK 정책을 요청하지 않음 |
| 사람 수정/저장/채택 | edit_proposal/accept_proposal, canonical expected_revision/target_source_revision/receipt | 동일 SDK client/store/context를 실제 제품 document에 주입하는 root 조립은 SDK 소유. 매 입력 저장·자동 재채택·옵션 삭제 금지 |
| 품질 검토 | record_review_decision/review_decisions(ListReviewDecisions) | 기존 review 결정 API를 소비하며 proposal accept와 별도로 표시. 재사용 가능한 query를 새로 복제하지 않음 |
| proposal provenance | 현재 GeneratedResult는 task/raw_text만 보유 | 필요 SDK 요청은 package SHA/item ID/candidate asset SHA/원본 companion 및 HF revision·license·precision/decision을 proposal과 명시적으로 연결·재검증하는 정본 계약. 단순 UI문자열합치기나 외부 verdict 추정으로 bound/quality를 true로 바꾸지 않음 |
| project/출력 연결 | create_project/inspect_project/save_project_revision | proposal receipt와 실제 미디어/CardDocument/ClipDocument의 연결·저장·내보내기를 별도 application 계약으로 인계. 현재 player/프로젝트에 암묵 적용하지 않음 |

provenance 연결 계약의 추가가 다른 원문 비교/수정/채택 전체를 멈추는 선행조건은 아니다. 현재 source SHA/revision/raw/unknown 표시와 기존 API로 진행하고, 실제 NotFound/corrupt/reference mismatch의 원 오류·remediation 표시를 유형별 후속 gate에서 검증한다. SDK API 요청은 이 계약 차이에 한정하며 root source 변경은 합의된 SDK checkpoint 뒤 담당자가 수행한다.

### 검토 대상과 provenance 의미의 후속 인수

AI 읽기 대조 인계에서 GeneratedResult/StoredProposal에 package/item/candidate/companion/decision의 정본 결합이 없고 ai_provenance source는 root 미등록·reader/atomic persist 미구현임을 확인했다. 기존 review/source_import/proposal/project API를 재사용하고 새 decision API를 복제하지 않는다. 이 차이는 후속 계약이며 현재 빌드의 추가 선행조건으로 만들지 않는다.

원 candidate/item에 결합된 verdict/decision SHA를 사람이 edit/accept한 다른 output에 자동 전이하지 않는다. UI는 원 candidate의 review와 현재 proposal draft/acceptance output의 검토 대상을 별도로 명시해야 한다. 문구가 비슷하거나 source가 같다는 이유로 품질 승인을 상속하지 않는다. 출력에 대한 새 검토 연결은 정본 대상 식별과 별도 명시 decision이 있을 때만 적용한다.

evidence_verified는 증거 저장/무결성 검증, review_ready는 baseline 존재 의미다. baseline이 ASR 원입력일 수 있으므로 human gold/정답으로 표시하지 않는다. ModelCondition 메타데이터는 license 승인이나 실제 runtime 정밀도 측정 결과를 입증하지 않는다. generation_provenance_bound/quality_decision_known 미연결 상태를 이 플래그로 true로 바꾸지 않는다.

후속 인수 순서는 명시 package/item/candidate/companion 선택→검증 API로 원 raw 수입/NeedsRepair 보존→사람의 명시 edit/accept→fresh CLI process 재열기와 project binding 확인→동일 원 candidate/item의 decision 별도 재조회다. 현재 Native23은 same-process disk reopen이며 이 fresh CLI/project/원candidate 결정 재조회 전체를 입증하지 않는다. 별도 decision 표시와 수정된 output의 상태를 함께 보여주되 서로 합치지 않는다.

원본 verification/proposal-review-intent.tsx의 Text onDraftChange와 Action onSave/onAccept/onRefresh를 실제 module document에 bind한다. stale field handle·stale document/node·signature mismatch·unmount 뒤 입력을 거부하고, explicit mutation/refresh의 호출 횟수를 확인한다. 이후 correction/Card/Highlight 원본 graph와 동일 controller를 소비한다. 각 패널을 전체13화면/모든 하위탭의 완료로 집계하지 않는다.
