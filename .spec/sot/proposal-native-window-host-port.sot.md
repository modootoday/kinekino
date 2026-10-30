---
id: sot:kinekino/proposal-native-window-host-port
title: Proposal Native window host port 요청
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Proposal Native window host port 요청

## Meari 제안 대조와 남은 서명

### typed 후보의 제품 적용과 미확정점

추가된 HostStamp/HostTurn·HRTB try_with/try_with_mut·HostEffects/HostUpdate·CloseRequest/CloseDecision 초안을 직접 읽었다. 기존 PluginScope/CallbackBindings/PendingCallback을 재사용하고 Error/CloseCheckpoint에 Send/Sync/Clone/serde를 강제하지 않는 계약은 현재 제품 session/Rc registry와 맞는다. 아직 제안 타입이므로 UI source에서 import하지 않는다.

| 현재 제품 준비 | typed 후보로 가능한 연결 | 구현 전 꼭 확정할 gap |
|---|---|---|
| PreparedPanel {props, bindings} | explicit selection을 최신 세대로 재발급하고 turn당 새 props/bindings를 한 번 준비 | 제품 PanelSelection의 TextInput generation은 HostStamp가 아니며 source/definition/projection/turn 검사를 대체하지 않음 |
| ProposalDocumentSession::reproject_with | 새 callback export를 준비하고 renderer 성공 후에만 owner binding을 commit | HostEffects::Props를 반환해 driver가 나중에 commit하면 동기 apply closure의 성공 시점을 알 수 없음. driver commit/abort 결과를 owner stage에 돌려주는 typed ACK 또는 handle 안에서 기존 reproject를 즉시 한 번 실행하는 명시 profile 중 하나를 정해야 함. 두 번 update하지 않음 |
| 성공 재투영 뒤 stamp | 같은 doc 모델은 한 번만 갱신하고 redraw만 요청 | host가 짧은 handle borrow 안에서 직접 update하는 profile이면 projection_revision 증가와 Effects.expected/AlreadyApplied 처리의 기준 stamp를 확정해야 함. 기존 stamp를 재사용해 두 번째 적용으로 만들지 않음 |
| callback export 필수 | 현재 ProposalDocumentSession은 항상 콜백 문서용 export를 생성 | Props.export=None을 현재 callback 문서가 받으면 명시 거부하는 정확 오류와 기존 lease 보존 확인. 비콜백 raw markup과 구별 |
| explicit close | !Send consumer CloseCheckpoint가 callback revoke/session.close/parent unregister의 완료 단계를 소유 | handle이 Closing 상태가 된 뒤 일반 try_with_mut가 거부될 때 document.close_callbacks를 누가 수행하는지 확정. driver 완료 receipt를 host가 받거나 close 전용 권한을 제공해야 함. 정상 mutation 권한은 복구하지 않음 |
| close retry | session 완료 뒤 parent unregister만 재시도하고 completed 단계를 반복하지 않음 | 현재 session.close는 상위 typed checkpoint를 제공하지 않음. 제품 checkpoint 구현·초기 실패/재시도/forced Drop 인수가 별도 필요하며 기존 Drop 성공을 정상 CloseReceipt로 표시하지 않음 |
| batch/queue 예산 | 기존 document/plugin/registry 한도와 UI의 더 작은 bounded queue를 모두 유지 | HostTurn을 take한 뒤 UI queue가 Busy/Capacity/TextBudget을 반환할 때 남은 owned batch의 보존/재개/오류 소유권 확정. silent drop·SDK mutation 재실행·새 무제한 queue로 우회하지 않음 |

현재 제품 패널 예산은 caller가 명시하고 ReviewIntents가 entries1..64/text bytes1..1MiB로 검증한다. 본체 pending1024/건1MiB/총4MiB 및 PluginPayload depth64/nodes65536/wire1MiB, registry max_pending1..4096과 별도 경계다. 이 한도들을 합산해 더 큰 허용량으로 해석하지 않는다. JSON payload의 wire 검증도 기존 plugin layer에 유지한다. 한 normalized event에서 받은 batch를 한 군데서 처리한다.

즉시 적용 가능한 것은 fixture 없는 명시 선택·기존 SDK/controller·Props/ReviewBindings 준비·opaque close checkpoint의 책임 설계다. 남은 public 단계는 stage commit ACK/profile·close 특권/receipt·batch backpressure이며 owner와 확정 후 소스 등록/컴파일한다. 동시 SDK heavy 동안 새빌드·공유 등록 source 변경은0이다.

UI가 Meari의 `.spec/sots/integration/native-host-port.md`, `.spec/adrs/0015-ui-thread-document-host-port.md`, `.spec/plans/native-host-port.md`, `.spec/evidence/reviews/native-host-port-source-audit-2026-10-19.md`를 직접 읽었다. 상태는 모두 제안·구현/컴파일/OS 검증0이다. 아래는 읽기 대조이며 기능 인수가 아니다.

| UI 요청 | Meari 후보와 일치/수정 | 실제 공개 인계 때 필요한 확정 |
|---|---|---|
| 같은 스레드의 한 document/제품 owner | !Send/!Sync NativeDocumentHandle·단일 내부 accessor와 일치. OS/file/checked render/a11y의 기존 downcast를 모두 동일 accessor로 연결해야 함 | handle 생성·제한 borrow·closed/epoch 오류의 정확 Result 서명. Rc wrapper 추가만으로 완료하지 않음 |
| guard 밖 after-dispatch | owned batch+turn stamp 후보가 더 엄격함. 기존 UI 표의 mutable document+host 접근은 동시 borrow를 뜻하지 않도록 수정 | batch의 owned CallbackHandle/CallbackInvocation 접근, 정규화 이벤트당 turn 호출, registry borrow 반환 뒤 새 문서 borrow의 update 적용 서명 |
| 입력→저장 순서 | 단일 이벤트마다 draft/reproject를 끝낸 뒤 다음 이벤트 처리라는 후보를 수용 | driver가 다음 입력을 소비하는 시점과 pending/Busy·worker completion·wake 순서. 오래된 props로 text와 Save를 한 batch에 합치지 않음 |
| 재투영·오류 | props/export preflight/commit·last-good·poisoned 구분과 일치 | CallbackBindings 이전·실패 Drop·성공 lease 교체 결과, update effect/redraw/AccessKit 갱신 신호, typed 오류의 소유권 |
| close·source 전환 | close request host turn→허용 commit과 forced destroy/suspend 구분을 수용 | close 승인/보류/정리 실패의 typed 결과·정확 한 번 호출·강제 종료 통지, source replacement와 host session/registry 교체 순서 |
| 현재 player 창/GL | winit driver와 standalone convenience의 분리는 일치하나 eframe 연결은 여전히 공백 | 제품의 기존 loop에 실제 위임 가능한 API·공유 GL context/FBO/state/소멸 순서의 별도 조사. 문서만으로 eframe 지원을 주장하지 않음 |

최소 port의 타입 역할은 (1) document handle의 제한된 borrow, (2) owner/session/source/definition/revision을 담은 opaque turn stamp와 owned pending batch, (3) 소비자 host의 guard 없는 batch 처리, (4) UI thread의 typed update/effects 적용, (5) close request/commit·wake queue다. 이 역할에 대한 구체 함수명·struct 필드·generic lifetime은 Meari가 얇은 소비자 타입검사로 확정한 후 UI가 연결한다. 여기서 임의 공개 Rust 시그니처를 발명하지 않는다.

UI adapter는 현재 Native 문서에서 batch를 꺼낸 뒤 dispatch→drain을 실행하므로 owned batch를 받는 새 제품 pump로 해당 부분만 분리할 수 있다. 그 동안 document/renderer guard를 보유하지 않는다. 최신 props와 CallbackBindings 준비는 제품 session의 UI thread에서 수행하고 적용 때만 짧게 document를 빌린다. CallbackBindings/registry Rc·문서/GL guard를 worker로 보내지 않는다. async SDK 작업이 추가되면 기존 typed deferred 계약을 사용하며 돌아온 결과의 Meari stamp와 canonical SDK expected_revision 검증을 서로 대체하지 않는다.

SDK ApiError는 기존 ReviewSession 화면 상태로 남기고 보통 host infrastructure 실패로 승격하지 않는다. Meari의 turn/lease/DOM/scene 오류는 별도 유지한다. 재시도 요청이 없는데 SDK 오류 때문에 매 redraw마다 mutation/inspect를 재호출하지 않는다. source/close 후 worker 결과를 차단하는 opaque stamp 구현은 Meari 소유이며 UI에서 해시 문자열로 권한을 재구성하지 않는다.

현재 gap은 정확 public 서명과 전체 accessor/driver 구현·소비자 compile·OS 실행이며, 신규 SDK domain mutation이 아니다. SDK 회차전환 heavy 중에는 문서만 정본화하며 src/manifest/lock 변경·빌드0을 유지한다.

상태: UI 제품 등록을 위한 Meari 소유 공개 API 요청. 구현·실행 완료가 아니다. [factory 계약](proposal-document-factory.md)과 [제품 등록 계획](../../../../plans/player/presentation/proposal-product-registration.md)을 따른다. SDK 변경 중 UI 추가 cargo/GUI·공유 등록 source 변경은 하지 않는다.

## 현재 공개 경계

읽은 Native infrastructure/window.rs의 공개 show(document, seconds)와 show_with_options(document, seconds, WindowOptions)는 document를 window loop에 넘긴다. 현 인자는 제품 ProposalDocumentSession/parent SDK registration 또는 after-dispatch·close handler를 받지 않는다. 내부 application은 접근성 act와 일반 이벤트, redraw와 파일 요청을 처리한다. UI가 private Application을 복제하거나 OS 이벤트 루프를 presentation 안에 새로 만들지 않는다.

문서 자체의 공개 constructor scope factory·drain_callbacks·update_module_props_with_callbacks·close_callbacks는 실제 SDK consumer27에서 사용했다. 이 문서 API 성공과 실제 창 loop의 SDK hook 부재는 별개다. 제품의 기존 eframe Glow와 이 VelloCpuWindowRenderer 경로도 구분한다. GL/window host의 선택·표면/핸들 소유는 SDK/Meari 조립에서 확정한다.

## 요청할 최소 공개 port

아래는 동작 계약이며 trait/함수 이름과 정확 Rust 서명은 Meari 담당이 소유한다. 제품 DTO나 ApplicationClient를 Meari 패키지에 의존시키지 않는다. UI가 제안한 이름을 이미 공개된 API로 사용하지 않는다.

| port | 필요한 계약 |
|---|---|
| window host 주입 | 실제 InteractiveDocument와 같은 스레드에서 살아있는 제품 host state를 loop가 소유하거나 수명 보장된 참조로 받는다. UI의 Rc/RefCell registry를 Send/Sync로 강제하지 않는다. 기존 show API의 비제품 사용은 유지한다. |
| after-dispatch | 일반 pointer/text/keyboard 및 접근성 입력 모두 Native guard·dispatch가 끝난 뒤 owned batch를 추출해 document borrow를 반환하고 제품 host를 호출한다. Core plugin registry borrow도 남아 있지 않아야 한다. update 때만 짧은 새 문서 borrow를 사용한다. |
| redraw 결정 | 제품 host는 큐를 꺼내 dispatch 반환 후 SDK drain·새 props/bindings 재투영을 수행하고 실제 변경 시 redraw를 요청할 수 있어야 한다. 단순 repaint가 반복 SDK 호출이나 자동 mutation/inspect 재시도를 만들지 않는다. |
| error 보존 | canonical SDK 오류는 화면 상태로 표시하고 문서를 유지한다. port/renderer 인프라 오류와 구별하며 루프 오류 보고에서 원인·last-good를 잃지 않는다. error 발생만으로 세션/창을 닫지 않는다. |
| 사용자 close | document를 제거하기 전에 host가 close_callbacks→session.close→parent SDK unregister를 수행할 기회를 준다. callback/registry borrow가 반환된 상태에서 호출하며 cleanup 실패면 오류와 재시도 경계를 보존한다. teardown을 이중 호출하지 않는다. |
| source/window 전환 | 기존 문서와 owner의 cleanup·새 document/owner 준비를 함께 다룬다. source generation/실제 instance가 바뀐 이전 handle은 사용할 수 없어야 한다. window만 보존하는 전환과 새 window 생성을 구분한다. |

데모 확인 창은 사용자가 종료한다. 제품 연결에서는 seconds=None에 해당하는 무기한 사용자 종료 경로를 사용하며 임의 timeout·프로세스 kill을 추가하지 않는다. 가상 모니터 검증 배정은 중앙 스케줄을 따른다.

## 제품 host가 수행할 일

부모 host는 실제 SDK client·context slot·registry·SDK registration과 명시 선택을 보유한다. after-dispatch에서 document.drain_callbacks→session.dispatch_handle_after_renderer_guard를 마치고 borrow를 해제한 뒤 session.drain_after_dispatch를 실행한다. 최신 view/selected index·generation에서 새 bindings/controlled props를 만들어 reproject_with와 Native update를 수행하고 cleanup_pending을 처리한다. scope·generation·순서 검증은 기존 모듈을 사용한다.

UI 공통 projection/bindings 생성기는 제품 코드로 제공할 다음 구현 범위다. tests의 cue-a/cue-b·고정 항목0·기술 ID·seed를 제품 기본값으로 이관하지 않는다. 제품 host는 SDK error를 새 문자열 정책으로 해석하거나 승인/저장을 자동 실행하지 않는다.

## SDK API 재사용과 후속

| 필요 기능 | 기존 계약·미완료 경계 |
|---|---|
| 저장 proposal 선택·읽기·수정·채택 | InspectProposal/CreateProposal/EditProposal/AcceptProposal과 기존 ApplicationClient. GUI 전용 API를 만들지 않는다. |
| 원 candidate/companion 선택 | InspectReviewPackage/ReadReviewTextAsset와 source_import. 명시 package/item/asset 선택을 유지하며 원 candidate와 수정 output 품질은 분리한다. |
| 프로젝트 연결 | CreateProject/SaveProjectRevision/InspectProject와 ProposalRevisionRef의 Live/Saved·working_draft 계약을 재사용한다. UI factory가 별도 project storage/capture 정책을 만들지 않는다. |
| decision 표시 | 기존 review decisions/record 계약을 재사용한다. 현재 StoredProposal에 package/item/candidate/decision의 정본 결합이 없다는 provenance gap을 해결한 것처럼 표시하지 않는다. |
| CLI 동등성 | 같은 SDK/store를 사용하는 fresh CLI process 재열기·project binding은 후속 실제 인수다. Native27의 same-process reopen을 이 성공으로 집계하지 않는다. |

이 요청 자체에 필요한 신규 domain/SDK mutation API는 없다. 실제 선택 목록·호스트 수명·기존 storage 이용 가능성 또는 provenance 식별의 추가 요구가 확인되면 정확한 누락 계약을 SDK 담당과 협의한다. 제품 등록·창 입력/정리·전체13화면의 시각 비교는 아직 완료가 아니다.
