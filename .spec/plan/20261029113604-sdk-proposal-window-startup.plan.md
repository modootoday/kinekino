---
id: plan:kinekino/20261029113604
title: Proposal Native startup 조립 후보
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Proposal Native startup 조립 후보

최신 상태(2026-10-20): 등록1~4단계를 인수하고 5단계 main Native 분기·owned cleanup·고정 CSS/font/icon 연결을 구현·기술 검증했다. 최종 source13 4F4AE9…B0F0E/새 binary6610D2…FDC6F의 실제6·오류5와 필요한 trusted 실패/종료 게이트를 통과하고 단일 heavy를 반환했다. 정확 결과와 한계는 [제품 실행 계약](../../sots/player/application/native-proposal-execution.md)과 runtime/quality-review/product-native-main-r1/handoff.md를 따른다. 과거 후보/읽기 전용 단계와 후속 실제 실행을 구별한다. 전체 제품·UI P1 공통 프레임·시각13·새 exe 편집/수락 상호작용·AI 품질 완료가 아니다.

아래 초기 후보 기록은 당시 PM 배정의 SDK 소유 소스 인계다. 당시 **미등록·미컴파일 후보**였으며 production 런처/OS 연결 성공이 아니었다. 기존 [typed close SDK](../../sots/verification/application/sdk-close-progress-2026-10-20.md)의 E1959DC…8BF5 입력과 UI16·Native 최종2827a5…3132a를 소비했다. 이후 승인된 단계별 변경은 본문 후속 상태와 각 정본/핀으로 구분하며, 당시 동결 범위를 현재 전체 source 불변 주장으로 사용하지 않는다.

## 정확 파일과 API

- `src/infrastructure/meari/mod.rs`와 `completion_notifier.rs`: NativeStartupCompletionNotifier::new(NativeStartupWake), 공통 CompletionNotifier 구현. bool true/false 모두 성공이며 Native 오류를 명시적인 canonical ApiError로 반환한다. SDK Shared::notify의 notification_error가 마지막 오류 Arc를 보존한다. adapter에 새 mailbox·setter·등록·ACK·thread는 없다.
- `src/assembly/meari/proposal_window_startup.rs`: prepare_proposal_execution(client, execution_budget, schedule_limits) → Result<PreparedProposalExecution, Box<ProposalExecutionFailure>>. 성공 bundle.into_parts()는 같은 NativeLoopOwner/NativeStartup/ExecutionOwner를 한 번 이동한다.
- ProposalExecutionFailure는 Native/SDK typed cause와 이미 생성한 loop/startup 및 같은 client를 보존한다. retry_execution(budget)은 SDK 생성 실패에만 기존 loop/startup을 재사용한다. 성공한 owner를 만들고 다시 만드는 재시도 API가 아니다. 초기 inspect나 UI 조립 실패의 기존 owner는 UI InitialStartFailure/Handoff/MountFailure가 계속 보존한다. Drop은 정상 cleanup receipt가 아니다.

순서는 NativeLoopOwner::new → startup → wake_handle → 얇은 notifier → 기존 client로 ExecutionOwner::start 한 번 → UI InitialPanelStartup::start(같은 owner·신뢰한 context·명시 선택) → controller handoff/run_startup이다. assembly는 context/selection 기본값이나 probe scope·SDK 정책을 만들지 않는다. 실제 scope·registry·Rc context는 UI/parent에만 남기고 worker에는 기존 canonical snapshot을 전달한다. Native 자체의 startup dirty/attach forwarding을 재구현하지 않는다.

## 소비와 Cargo 경계

root infrastructure/mod.rs 및 assembly/meari/mod.rs에는 새 export를 추가하지 않았다. 두 소스는 미래 root 등록에 맞는 crate::application/crate::infrastructure::meari 경로를 쓴다. 현재 consumer integration crate는 kinekino_player::application을 crate root에 re-export하고, 새 infrastructure/meari 부모를 path module로 읽은 뒤 crate::infrastructure::meari alias를 제공해 같은 소스를 소비한다. Native 의존 소스를 consumer lib로 옮기지 않는다. 실제 production optional Native dependency/export/launch 주입은 다음 필수 단계다.

consumer Cargo.toml의 기존 Native dev-dependency에 features=["window"]만 추가하고 default-features=false를 유지했다. PM이 별도로 허용한 offline dependency resolution 한 번은 consumer cwd의 cargo update --offline --workspace로 수행했다. resource-runner2CPU/BelowNormal,3246ms/exit0이었다. 컴파일·fetch·다운로드는 없었다.

Cargo.lock에는 window graph39개 package가 추가됐다. blitz-shell0.3.0-beta.4, accesskit_xplat0.2.0, winit0.31.0-beta.3 및 softbuffer renderer와 해당 target 의존성이다. 기존 package의 version/source/checksum 제거·이동은 없고, 기존 dependency 문자열9곳은 복수 버전 명시로 바뀌었다. root/Meari lock이나 Native feature 정의는 바꾸지 않았다. consumer 새 manifest SHA256은 EF84652B28CD789F865C66127002515AE397DDFF35F50C838E5FD6CEC79C71BC, lock은 E5C8BB86E0F1D192F390AD8BCD2B094D4100B68BCB6FD3ED11A3AAB92F9504D0다.

## 인계 근거와 다음 시험 요청

### trusted notifier wrapper 후속 후보

UI가 원 compile-first session81631의 exit0을 확인하고 SDK 파일 수정 시점을 명시 인계한 뒤 `proposal_window_startup.rs`만 확장했다. 원9CA03C…6579C 후보 사본은 그대로 보존한다. 기존 후보의 compile 성공을 아래 새 wrapper 입력의 통과로 승격하지 않는다.

`prepare_proposal_execution_with_notifier_wrapper(client, execution_budget, schedule_limits, wrap: impl FnOnce(Arc<dyn CompletionNotifier>) -> Arc<dyn CompletionNotifier>)`를 추가했다. 기존 prepare_proposal_execution은 identity wrapper를 전달해 같은 생성 경로를 사용한다. Native loop/startup 생성 후 실제 Native notifier를 만든 다음 wrap을 한 번만 호출한다. SDK 시작 실패에는 private notifier Arc도 보존하며 retry_execution은 같은 Arc를 재사용한다. Native 생성 실패는 wrap을 호출하지 않는다. 새 직렬화 입력·late setter·worker·mailbox나 SDK policy는 없다.

wrapper는 신뢰한 Rust 조립/시험 전용 주입 경계다. 실제 Native notifier를 먼저 위임한 뒤 통제된 마지막 worker-exit 호출만 barrier로 보류해야 wake-before-return 경쟁을 검증할 수 있다. notify에는 종료 인자가 없으므로 고정 fixture의 알림 순번/빈 queue/완료된 terminal ACK를 대조하며 arbitrary sleep이나 '다음 알림' 추정으로 대체하지 않는다. 실패→retry 후 wrapper 생성1회와 같은 instance 사용을 시험하고, 기존 identity 경로의 초기 준비/ACK/close 회귀도 유지한다.

wrapper 변경의 before 사본/핀과 diff·새5파일 inventory는 `runtime/quality-review/proposal-window-startup-wrapper-r1`에 별도 보존한다. 개별 rustfmt/--check 외 새 compile/test/GUI는 SDK가 실행하지 않았으며 UI가 다음 검증 슬롯에서 소비한다.

`runtime/quality-review/proposal-window-startup-source-r1`에 consumer 원본 사본/핀, manifest·lock diff와 resolver receipt/stdout/stderr 및 신규 소스 사본·최종 pin을 보존한다. 개별 rustfmt/--check는 파싱을 통과했지만 typecheck·test·clippy나 실제 SDK/OS 실행의 증거가 아니다. UI가 단일 heavy 슬롯에서 같은 소스를 검증한다.

필수 검증 요청은 다음과 같다.

1. window feature graph의 locked consumer compile과 기존83 호환/strict/fmt. alias 때문에 SDK 정책·worker를 복제하지 않는다.
2. invalid ExecutionBudget로 SDK 시작 실패 시 같은 loop/startup stamp/client 보존, valid budget의 명시 retry_execution 후 성공 owner1회. 새 loop fallback이나 inspect 재실행을 하지 않는다.
3. 초기 loop 실행 전/Preparing 중/attach 직후의 terminal 보존과 coalesced Ok(false) 성공, 같은 owner의 초기 inspect→명시 typed ACK→actual scope/mount handoff.
4. startup가 닫힌 뒤 notifier 실패가 canonical code/kind로 SDK notification_error에 보존되고 원 terminal은 그대로 남는지 확인. notifier 오류를 terminal 실패나 취소 ACK로 대체하지 않는다.
5. NativeRunOutcome 및 UI 초기/부분 조립 실패에서 반환 owner를 보존하고, worker exit와 실제 Native 종료는 각각 receipt로 검사한다. 이 소스 인계만으로 NW01~08·시각·전체 제품 완료를 표시하지 않는다.

## 다음 제품 등록 배정안 — 읽기 전용 결정

2026-10-20 [등록 경계 리뷰](../../evidence/reviews/product-native-registration-boundary-2026-10-20.md)를 현재 root Cargo/lib/main/launch와 대조한 제안이다. 아래 feature·옵션·DTO는 아직 구현/지원 API가 아니다. 이번 배정에서는 이 문서만 갱신하며 SDK/Cargo/lock·빌드·GUI 입력은 동결한다.

### feature·의존성과 import 결정 제안

- feature 이름은 범위를 드러내는 `native-proposal-window`로 한정한다. 기존 기본 기능은 유지하고 Windows optional meari-native에 default-features=false/features=["window"]를 지정한다. feature는 dep:meari-native만 활성화하며 gl/all-features나 bridge 의존성은 추가하지 않는다. 관련 Rust export는 cfg(all(windows, feature="native-proposal-window")) 아래 둔다.
- root workspace의 `[patch.crates-io] blitz-paint = { path="packages/meari/packages/meari-paint" }`를 기존 consumer와 같은 승인 source로 고정하는 변경이 필요하다. 이 patch 자체는 Cargo feature별로 조건화할 수 없다. optional feature off에서도 lock 해석에 영향을 줄 수 있으므로 'off graph는 byte-identical'이라고 가정하지 않고 resolver diff와 실제 활성 graph를 따로 검증한다. root lock만 이 배정에서 갱신하고 Native lock을 복사·수정하지 않는다.
- 최소 import 방안은 root lib의 feature-gated `extern crate self as kinekino_player;`다. 기존 UI의 kinekino_player::application/domain imports와 SDK assembly의 crate::application/crate::infrastructure를 그대로 해석하며 source-wide 치환을 피한다. self alias는 같은 crate의 이름일 뿐 두 번째 SDK 인스턴스가 아니다. root→player-ui-sdk-bridge 역의존은 금지한다.
- UI가 root façade를 등록한 뒤 consumer feature-on 경로는 root의 presentation::web_ui_adapter::proposals를 re-export해 소비하는 방향으로 인수한다. 등록된 문서 타입과 같은 source를 별도 path module로 다시 컴파일한 타입을 섞지 않는다. 기존 independent83/path 경로를 feature-off 호환 시험으로 남기려면 consumer의 같은 이름 feature와 kinekino-player/native-proposal-window 전달을 명시하고, 어떤 시험이 canonical root façade를 쓰는지 구분해야 한다. self alias 하나만으로 이 중복 타입 문제가 해결됐다고 주장하지 않는다.

### 명시 시작 입력과 기존 client

제안하는 시작 선택은 `player.exe --native-proposal-request <absolute-json-file> --proposal-root <absolute-directory>`다. 일반 실행은 계속 Player/eframe이고 DesignPreview는 기존 규칙을 유지한다. Native 선택과 --ui-demo/--smoke-test/--start-url의 혼합 및 중복 옵션은 application launch에서 거부한다. feature가 없는 빌드에서 이 옵션을 조용히 무시해 eframe으로 fallback하지 않고 명시 Unsupported로 실패시킨다. CLI의 기존 RPC/명령 wire는 바꾸지 않는다.

새 요청 DTO 후보는 application/launch/native_proposal.rs의 NativeProposalLaunchRequest { schema_version, context: RequestContext, selection }이다. selection은 proposal ID/document revision/source ID·revision·SHA와 명시 target(Card/Highlight index, CueCorrection index+cue ID, DocumentCorrection index+full DocumentTextRef, 미선택 DocumentRepair)를 domain 값으로 담는다. application이 presentation::BootstrapSelection 타입을 역참조하지 않고 UI가 한 번 변환한다. index0 fallback/최신 revision 자동 선택은 하지 않는다. 실제 일치 판단은 기존 SDK inspect와 UI project_initial의 공통 검증에 남긴다. 최초 inspect가 거부하는 expected_revision/idempotency_key/timeout을 assembly가 지워서 통과시키지 않는다.

파일 읽기·크기 한도·기존 filesystem guard는 infrastructure가, DTO/옵션 중복·상호 배제는 application launch가 맡는다. 요청에서 TSX 임의 경로·실행 파일·verifier pin·쉘을 받지 않는다. 모듈/asset inventory·entry·labels·budget은 제품의 고정 source 조립에서 주입한다. 특히 현재 BootstrapPanel::Correction.entry()의 verification 경로를 production 기본값으로 승격하지 않고 실제 product correction-panel entry와 UI 소유 매핑을 확정해야 한다.

src/main.rs의 현재 PlayerSdk::new→frame/status/control→configured proposal/project/review store→ApplicationClientHandle 생성은 한 번만 사용한다. Native 분기를 위해 PlayerSdk::default나 새 store를 만들지 않는다. 명시 모드/요청 오류는 가급적 client/host 생성 전 반환하고, 유효한 Native 선택은 그 같은 client를 prepare_proposal_execution에 넘긴다. 현재 LocalHost를 유지한다면 같은 client.clone() 하나를 공유하고 backend 하나만 실행한다. Native 모드에서 기존 eframe/WebView event loop를 함께 시작하지 않는다.

### 다음 launch 단계의 현재 API 대조 인계 (2026-10-20, 읽기 전용)

후속 상태: 3단계에서 plain DTO/schema "1"/64 KiB/u32 checked index와 bounded adapter를 구현·검증했다. 정확한 현재 입력/선택 계약 및 원 결함·검증은 [launch 정본](../../sots/player/application/native-proposal-launch.md)을 따른다. 아래는 당시 읽기 전용 인계 근거이며 후속 UI helper/owned cleanup/main 연결은 여전히 미구현이다. 현재 desktop_mode는 off/on 모두 Native 실행을 명시 거부하는 staged guard로 eframe fallthrough를 막는다. guard 해제는 main Native 실행 연결과 동시에 수행한다. DocumentRepair Selected는 DTO에 포함하며 UI InitialPanelStartup::prepare의 Some 거부는 후속 UI 소유로 인계한다.

이 절은 UI canonical 등록 중 현재 source를 읽어 구체화한 구현 후보다. source/Cargo/lock/실행 파일은 변경하지 않았고, 아래 신규 DTO·adapter·제품 outcome은 아직 존재하지 않는다. UI 등록 최종 핀 이후 별도 배정으로 구현한다.

#### 최소 입력과 실제 재사용 타입

`src/application/launch.rs::{DesktopMode,desktop_mode,storage_root}`는 현재 Player/DesignPreview만 판단한다. `desktop_mode`는 --ui-demo가 없으면 즉시 Player를 반환하므로 Native 옵션을 추가할 때 이 조기 반환 앞에서 단일 선택을 파싱해야 한다. `storage_root`는 절대 경로/중복/preview 혼합만 확인하며 파일 존재·reparse·바이트 한도를 검사하지 않는다. 기존 `proposal_root`를 다시 임의 기본 경로로 바꾸지 않고 Native 선택에는 명시 값을 요구한다. request 파일 옵션은 공백형과 =형을 동일하게 처리하고 중복·누락·상충 옵션을 먼저 거부한다. feature-off도 옵션을 인식한 뒤 Unsupported로 반환해야 하므로 DTO/옵션 파서는 Native 의존성과 함께 숨기지 않는다.

신규 `src/application/launch/native_proposal.rs`의 최소 후보는 `NativeProposalLaunchRequest { schema_version: String, context: RequestContext, selection: NativeProposalSelection }`다. schema_version은 launch 입력 자체 버전 "1"로 고정하는 후보이며 context.api_version과 별개로 검사한다. 모든 DTO는 unknown field를 거부한다. context는 `src/application/api/commands.rs::RequestContext`를 그대로 사용하고 `validate()`를 호출한다. `expected_revision/idempotency_key/timeout_ms`를 삭제하거나 default context로 덮지 않는다. validate 통과와 SDK inspect에서 해당 context가 지원되는지는 별도 판단이다.

`NativeProposalSelection`의 값은 proposal_id/source_id/source_sha256은 String, document_revision/source_revision은 기존 `domain::api_values::DecimalU64`, target은 신규 application enum이다. 현재 domain에는 BootstrapTarget과 같은 선택 enum이 없으므로 기존 domain 타입이라고 표기하지 않는다. target은 Card/Highlight의 명시 index, CueCorrection의 index+cue_id, DocumentCorrection의 index+`domain::proposals::DocumentTextRef`, DocumentRepair의 명시 미선택 또는 full ref를 담는다. JSON index는 u32 후보로 받고 UI 변환 때 usize로 checked conversion한다. DocumentRepair는 omitted와 unselected를 혼동하지 않도록 명시 tagged Unselected/Selected{source_ref} 형태를 후보로 한다. `DocumentTextRef`는 source_asset_sha256/outer_pointer/extraction(`SourceExtraction`)/inner_pointer 전체를 재사용한다. ID/SHA 정합성과 실제 index/ref 존재는 `bootstrap.rs::project_initial`과 SDK inspect의 기존 canonical 검증에서 확정한다. 입력 단계에서 같은 정책을 새로 복제하지 않는다.

application enum을 UI `BootstrapTarget`으로 한 번 변환하는 helper는 presentation 소유다. `BootstrapSelection`에는 문서 revision도 있지만 factory `ExplicitPanelSelection`에는 문서 revision이 없으므로 factory 선택만으로 초기 revision 검증을 대신할 수 없다. `InitialPanelStartup::start(owner, context, selection, max_typed_wire_bytes)` → 기존 preparation/ACK → handoff 경로를 유지한다.

#### bounded 파일 adapter와 기존 guard

신규 Windows `src/infrastructure/native_proposal_request.rs` 후보는 선택된 절대 request 파일만 읽는다. `immutable_storage_paths.rs::root(parent)`가 반환하는 canonical parent와 모든 ancestor File guard를 읽기 종료까지 보존하고, 그 parent와 원래 단일 file_name을 조합해 `immutable_storage_paths::file`로 연다. root는 Disk prefix/ParentDir/ancestor reparse를 검사하고 file은 ordinary file/reparse 및 공유 모드를 검사한다. request 파일에 root()를 직접 호출하면 디렉터리 검사에서 실패하므로 parent에만 적용한다. UNC 등 이 guard가 지원하지 않는 경로를 별도 우회로 허용하지 않는다.

한도는 작은 launch DTO에 맞춘 64 KiB 후보로 application 계약 상수에 두고 adapter는 `Read::take(limit+1)` 후 실제 바이트 수로 판정한다. metadata 길이만 신뢰하지 않는다. 기존 `catalog/results.rs::CatalogFiles::read_bounded`는 private이며 상대 경로와 catalog root에 결합되어 있어 직접 재사용할 public 범용 reader가 아니다. 그 bounded 읽기 패턴과 기존 immutable guard를 재사용하되 catalog 의존성을 추가하지 않는다. adapter는 바이트/IO 오류까지만 담당하고 JSON decode/버전/context 검사는 application이 맡는다. 새 writer lock, 임의 디렉터리 생성, TSX/실행 파일 선택은 필요 없다.

#### main의 한 client와 실제 owned 반환

`src/main.rs::main`의 args/roots/historical/desktop_mode 처리 구간에서 Native 옵션 및 request decode 오류를 먼저 확정한다. 이후 기존 `PlayerSdk::new` → frame/status/control → configured proposal/project/review store → `ApplicationClientHandle` 생성 구간을 그대로 한 번 사용한다. 현재 `LocalHost::start(client.clone(), ...)` 뒤 `presentation::run_with_native_playback(...)` 앞이 단일 backend 분기 위치다. Native도 localhost를 유지한다면 이 같은 client를 공유한다. Native 분기는 기존 eframe 실행 함수를 함께 호출하지 않는다. paths/status/control의 실제 사용 여부는 backend별 조립 책임으로 정리하되 새 SDK나 store를 만들지 않는다.

현재 `assembly/meari/proposal_window_startup.rs::prepare_proposal_execution`은 `(NativeLoopOwner, NativeStartup, ExecutionOwner)`를 `PreparedProposalExecution::into_parts`로 반환한다. 실패 `Box<ProposalExecutionFailure>`는 cause/client/가능한 loop_owner/startup을 보존하고 `retry_execution`은 보존한 notifier와 같은 Native owner로 SDK 시작만 재시도한다. 이 함수는 launch 전체 실행/정리 outcome이 아니다. 다음 `InitialPanelStartup::start` 실패의 `InitialStartFailure { error, owner }`, `ProposalWindowStartup::new` 실패의 `Box<WindowStartupInput>`도 각각 기존 owner를 반환한다. 조립 실패를 anyhow 오류 하나로 즉시 변환해 이 객체들을 버리지 않는다.

실제 실행은 같은 `NativeLoopOwner::run_startup(startup, controller)`이며 반환 `NativeRunOutcome<ProposalWindowStartup>`의 controller/startup/runtime/recovery/loop_owner/result 전체를 신규 제품 assembly outcome 후보가 인수한다. `packages/meari/.../startup/runner.rs::run`은 Stale 사전 실패에는 loop_owner Some을 반환하지만 실제 run_app 이후에는 None을 반환한다. 따라서 모든 오류를 같은 loop로 다시 run할 수 있다고 설계하지 않는다. main은 이 outcome의 원 오류와 미정리 소유 상태를 검사·기록하는 제품 조립 함수를 호출하고, cleanup 완료가 확정된 경우에만 최종 종료 결과로 변환해야 한다. main에 callback/ACK 정책을 구현하지 않는다.

기존 정리 포트는 `InitialPanelStartup::{cancel,retained_terminal,poll_close}`(자동 ACK 없음), `ProposalWindowStartup::{request_initial_close,initial_mut,recover_mount_for_retry}`와 public failure/runtime 보존 필드, `InitialPanelMountFailure::recover_for_retry`, `BridgeAssemblyFailure::retry_cleanup`, factory의 retry_cleanup/retry_unbind/take_unbound_parent다. `NativeHostRuntime::request_close`도 있으나 `finish_headless_close`를 OS Destroyed 증거로 대체하지 않는다. controller의 request_initial_close는 initial 준비 단계용이고 attached host 전체 정리 함수가 아니다. 현재 startup controller에는 반환된 initial/handoff/mount_failure/bridge_failure/attach_failure/runtime 모든 상태를 하나로 인수해 bounded cleanup하고 terminal ACK의 명시 수락 및 실제 SDK join/OS teardown 결과를 돌려주는 public 통합 포트가 없다. 반환 객체 보존은 이미 가능하지만 이 통합 recovery/cleanup 조립은 미구현이다. loop 종료 뒤 OnWake만 기다리는 것으로 join이 끝난다고 가정하거나 새 loop/worker를 생성하지 않는다. 이 경계의 최소 additive port 필요 여부는 다음 구현 배정에서 UI/Native와 확정하며, 이번 문서만으로 존재한다고 간주하지 않는다.

#### UI 소유 요구사항 — 단일 후속 인계

UI에는 다음 묶음 하나만 전달한다: (1) application DTO selection → BootstrapSelection checked 변환, (2) 제품 고정 module/assets inventory·labels·budget·IntentPins·WindowOptions/observer·HostScheduleLimits·worker_exit_delay를 조립하는 helper, (3) 같은 InitialPanelHandoff를 `mount`하고 assembly의 기존 async 등록을 actual registry/scope/context로 연결하는 PanelMount closure, (4) production Correction entry와 위 failure 상태를 보존하는 반환 계약. `WindowStartupInput`의 initial_labels/correction_labels/live_correction_labels/labels/close_policy는 기존 public 필드대로 채우고 시험 seam인 commit은 제품 기본 None 후보로 둔다. `BootstrapPanel::Correction.entry()`는 현재 `verification/proposal-review-intent.tsx`이므로 그대로 production 기본값으로 쓰지 않는다. 새 helper가 어떤 고정 production entry를 쓰며 초기 projection과 live reprojection에 같은 entry/계약을 적용하는지 UI가 소유·확정한다. 이 묶음은 UI 현재 canonical 등록 단계의 변경 요구가 아니라 그 최종 핀 뒤 launch 조립 단계의 요구사항이다.

### 파일 소유와 구현 순서

| 순서·담당 | 정확 변경 후보 | 종료 조건 |
| --- | --- | --- |
| 1 SDK | root Cargo.toml/Cargo.lock, lib.rs, infrastructure/mod.rs, assembly/meari/mod.rs | optional Windows window graph·paint patch·self alias 및 notifier/startup export를 함께 고정. Core/Native source와 root 기본 backend 불변 |
| 2 UI | presentation/mod.rs, 신규 web_ui_adapter/mod.rs, proposals/mod.rs 및 consumer façade | original sibling 모듈의 canonical root 등록. Native 의존 factory/initial/bridge/startup을 feature 경계 안에 등록하고 path 타입 중복 금지 |
| 3 SDK/application·infrastructure | application/launch.rs와 신규 launch/native_proposal.rs, launch 요청 파일 adapter 후보 | bounded local 요청→plain DTO, 중복/충돌/미지원 mode 오류. presentation 타입/플랫폼 호출을 application에 추가하지 않음 |
| 4 SDK assembly·UI | proposal_window_startup.rs의 launch 조립 확장과 UI WindowStartupInput 생성 helper | 같은 client/context/명시 selection→기존 owner1회→InitialPanelStartup→factory/bridge→동일 Native loop. trusted wrapper는 시험 seam이며 직렬화 입력에 노출하지 않음 |
| 5 SDK | main.rs의 단일 모드 분기와 owned launch outcome 처리 | main은 조립/실행만. 실제 player.exe 분기의 초기·부분 실패 owner와 정상 SDK/Native 종료 receipt 인수 |

현재 성공 bundle.into_parts 이후 UI InitialStartFailure는 기존 ExecutionOwner를, WindowStartupInput 생성 실패는 boxed input을 반환한다. 실행 후에는 NativeRunOutcome의 controller/startup/runtime/recovery/loop_owner를 그대로 인수한다. main에서 result만 anyhow로 변환해 `?`로 빠져나가 owner를 버리지 않는다. root 조립의 owned outcome/명시 cleanup 경로가 원 오류와 남은 owner를 보존하고, incomplete/forced 상태를 정상 close로 출력하지 않아야 한다. 정상 모드 복귀를 이유로 새 loop·worker·inspect를 생성하지 않는다. 부분 cleanup의 실제 재개 가능 범위는 Native 최종 반환 API와 구현 배정 때 대조한다.

### 검증 순서

제품 등록 1단계의 구현·검증 증거는 `runtime/quality-review/product-native-registration-step1-r1/`에 보존한다. root의 `native-proposal-window`는 Windows optional `meari-native`의 `window` 기능을 선택하며, 기본 기능은 끈다. 공통 paint patch와 조건부 self alias 및 infrastructure/assembly export까지 등록했다. UI façade와 launch/main 연결은 후속 단계이며, 이 등록만으로 실제 제품 실행이나 시각 검토를 완료했다고 판단하지 않는다.

최초 fmt 실패는 두 부모 모듈 선언 정렬이며, 해당 부모 파일만 정렬한 fmt-r2는 exit 0이다. lock 감사 도구는 package name/version/source/checksum identity 집합의 기존 항목 보존을 검사한다. dependency 문자열의 의미와 실제 활성 feature graph는 별도 Cargo 관측으로 구분한다. 전체 게이트 결과와 최종 핀은 같은 증거 디렉터리의 최종 인계 문서에 기록한다.

1. 소비 gate의 최종 pin을 먼저 인수하고 feature off의 root fmt/test/strict 및 기존 GUI·CLI 계약을 검증한다. resolver diff에서 의도한 Native graph/paint source와 예상 밖 version/source 이동을 구별한다.
2. feature on root와 canonical consumer façade의 compile/기존 호환·typed ownership·초기 오류/원 owner 회수 시험을 실행한다. consumer-only path 성공을 root 등록 성공으로 상속하지 않는다.
3. 실제 `player.exe --native-proposal-request ...`를 현재 빌드로 실행해 준비→명시 ACK→actual scope→실제 wake/commit→worker join 및 실제 Native 종료를 확인한다. feature-off 명시 미지원·입력 오류·SDK start/partial mount 실패도 같은 입력/원로그로 보존한다.
4. root registration 이후의 새 binary/source pin·소유[]를 반환한다. 기존73 CLI는 영향에 따라 재실행 여부를 결정하며 과거 binary 실행을 새 입력 근거로 바꾸지 않는다. 물리 입력·시각13화면·AI 품질·S2 취소는 별도 미완료로 유지한다.

### 다음 main 연결 checkpoint의 검증표 (4단계 UI 인계 전 준비)

현재 관측 기준은 main의 단일 PlayerSdk/store 조립과 LocalHost::start(client.clone()) 및 launch.rs의 off/on staged guard다. 아래 항목은 아직 실행하지 않은 승인 후 검증 계획이다. UI 4단계의 최종 source/API/핀을 인수한 뒤 정확 포트 이름과 반환 타입을 고정한다. main/Cargo/SDK8을 이 계획 준비로 수정하거나 heavy/GUI를 실행하지 않는다. staged guard 해제와 실제 Native backend 분기는 한 checkpoint로 검토한다.

#### 조립과 증거의 구분

main은 옵션/파일 adapter/application decode → 기존 configured SDK/client 조립 → 단일 backend 선택 → 제품 assembly의 owned 실행/정리 결과 인수만 담당한다. 입력 오류는 ExecutionOwner/NativeLoopOwner 생성 전 반환한다. 기존 store를 두 번 open하거나 Native 전용 PlayerSdk::default를 만드는 경로는 허용하지 않는다. LocalHost가 유지되면 같은 client clone을 사용하며 반환된 host도 최종 cleanup 판단까지 보존한다. SDK 생성 자체와 SDK execution worker 시작을 구별하고, 오류 검사 순서의 기준은 후자의 시작 전이다.

제품 assembly 반환 후보는 단계별 실패 owner 또는 NativeRunOutcome와 정리 결과를 함께 소유하는 enum/struct다. UI의 실제 cleanup 포트가 아직 확정되지 않았으므로 특정 `cleanup()/finish()` 함수나 `Complete/Incomplete` 타입이 이미 존재한다고 가정하지 않는다. 필요한 공통 정보는 원 cause, 남은 권한/owner 객체, 마지막 ExecutionCloseProgress, Native 종료 관측, 명시 재개 가능 여부다. 원 ApiError/Native 오류를 단순 문자열로 덮지 않는다. 이러한 owner가 생긴 뒤 `outcome.result?`, `map_err(anyhow)?` 또는 UI helper 실패를 anyhow 하나로 바꾸어 미정리 소유 객체가 drop되게 하지 않는다. owner 생성 전 순수 입력/IO 실패의 일반 오류 전파와 구별한다.

실제 exe 검증은 새 binary hash·명령 argv·요청 원 바이트/SHA·store의 원 revision/source SHA·고정 UI module/assets pin·원 stdout/stderr·실행/종료 receipt를 묶는다. Rust 조립 시험은 같은 production 조립 경로에 trusted seam을 연결해 강제 실패와 권한 보존을 검사하는 별도 증거다. DTO/CLI에 failure injection/임의 TSX·exe 필드를 추가하지 않는다. Rust seam 성공을 실제 exe의 동일 강제 실패 재현으로 표기하지 않는다. thread 미생성은 child PID 없음만으로 증명할 수 없으므로 조립 순서 및 worker-start 경계의 관측/시험을 함께 사용한다. 그 관측 seam이 필요하면 후속 배정에서 최소 범위를 확정한다.

| 검증 항목 | 입력/유발 조건 | 확인할 결과와 증거 | 실행 종류/완료 제한 |
| --- | --- | --- | --- |
| client/store 1개 | proposal/project/review 설정을 사용하는 Native 선택, 기존 Player 선택 | 조립의 단일 생성 경로, LocalHost/ExecutionOwner에 전달한 같은 Arc의 identity, 동일 저장 revision 관측. 기존 store 설정과 context가 Native 분기에서 유실되지 않음 | trusted 조립 시험+새 exe 실제 store 관측. 단순 같은 값만으로 같은 client라고 단정하지 않음 |
| backend/event loop 1개 | feature-on Native 선택 및 기존 Player/DesignPreview 선택 | Native는 같은 NativeLoopOwner 한 번만 run; eframe/WebView 시작 함수 미호출. 기존 모드는 해당 backend만 실행. 종료/실패 후 새 loop·worker를 자동 생성하지 않음 | 조립 분기 시험+새 exe Native 실행. run_app 이후 loop_owner None이면 재실행 가능으로 표시하지 않음 |
| worker 전 입력 실패 | 누락/중복/상충 옵션, 상대 경로, reparse/초과 파일, malformed/중복/unknown JSON, 잘못된 schema/context | 기존 typed 오류/원 진단, 비정상 종료, worker/loop 시작 경계 미도달. 파일이 잘못됐는데 UI가 열린 뒤 실패하는 동작 없음 | 영향 Rust 시험+새 exe 대표 malformed/64KiB 초과 실행. 원 요청 바이트 보존 |
| feature-off 거부 | feature-off 새 exe와 유효 Native 옵션 | 빌드 미지원 Unsupported, request를 정상 Player로 처리하지 않음, worker/loop 미시작 | 별도 off binary pin과 실제 exit/로그. on 빌드 실행 미연결 진단과 혼동하지 않음 |
| Card 명시 선택 | 저장된 cards proposal의 index 1 이상, 정확 document/source revision·SHA | 준비→명시 ACK→actual scope→해당 card 초기/갱신 props와 원 선택 일치, index0 fallback 없음 | 새 player.exe+실제 Native 창/문서 관측. index1을 가진 실제 저장 fixture를 기존 Rust SDK/CLI로 준비 |
| Highlight 명시 선택 | 저장된 highlights proposal의 index 1 이상과 정확 pins | 같은 유스케이스/정책으로 해당 highlight 준비·ACK·commit, 미선택 항목과 canonical source/generated 보존 | 새 player.exe. 기존 CLI73 결과를 새 launcher 성공으로 대체하지 않음 |
| Correction 명시 선택 | cue index+cue_id 또는 document correction index+full DocumentTextRef | production Correction entry 사용, 초기·live projection 계약 일치, 명시 대상만 수정/수락, 공통 revision 정책 | 새 player.exe에서 실제 저장된 correction fixture. cue/document 경로는 입력 종류를 구분해 기록 |
| DocumentRepair Unselected | draft 없는 document repair와 {kind:unselected} | 초기 미선택 유지, 임의 첫 source/ref 없음, 이후 사용자 선택 전 대상 문구를 제안 결과로 승격하지 않음 | 새 player.exe. 빈 화면 성공만이 아니라 미선택 props/state 관측 필요 |
| DocumentRepair Selected | 같은 종류 source의 첫 항목이 아닌 full ref를 명시 | DTO→BootstrapSelection→InitialPanelStartup::prepare→초기/live correction projection에 ref가 유지. Some 거부/None 전환/첫 항목 대체 없음 | UI 4단계 Selected 보완 핀을 전제로 새 player.exe에서 검증. 아직 지원 완료로 간주하지 않음 |
| SDK start 실패 | trusted SDK 시작 seam에서 원 시작 오류, loop/startup은 이미 확보 | ProposalExecutionFailure의 원 cause/client/가능한 loop_owner/startup 보존. 같은 notifier/owner로 허용된 SDK 재시도만 가능, main이 임의 owner 재생성/성공 변환하지 않음 | production 조립의 Rust seam 시험. 일반 CLI 입력으로 SDK 실패 주입 기능을 만들지 않음 |
| mount 실패 | 같은 InitialPanelHandoff의 partial actual scope factory 실패 | 원 prepared/raw/worker/partial scope 보존, 기존 unbind/cleanup 진행 상태와 원 오류 동시 반환. 재시도 가능 여부는 실제 quiescent 결과에 따름 | trusted Rust mount seam. actual exe 부분 실패를 실행하면 별도 receipt로 추가하며 미실행 상태를 분리 |
| attach 실패 | Native attachment 준비/실제 attach 경계의 원 오류 | controller의 attach_failure와 NativeRunOutcome의 runtime/recovery/startup 등 남은 권한을 원 상태대로 인수. startup 이전/이후 실패를 혼합하지 않음 | 확정 Native/UI public 포트의 trusted 시험. 실제 Native 창 사용 여부를 별도 표기 |
| cleanup incomplete/fault | terminal ACK 미완료, worker exit pending, 원 join fault 또는 forced Native 종료 | 원 실패와 cleanup 미완료를 함께 반환, 마지막 close progress 및 남은 소유 객체 보존. 정상 종료/완료로 출력하거나 재시작으로 숨기지 않음 | UI 확정 cleanup 포트로 Rust 시험, 가능한 실제 exe 오류 경로는 별도 실행. process 종료 후 객체가 살아 있다고 주장하지 않음 |
| 정상 종료 | 실제 창을 정상 close하고 진행 중 작업을 계약대로 정리 | ExecutionCloseProgress::Closed의 실제 join 완료, terminal ACK 잔여 없음, actual Native owned windows의 Destroyed 관측 및 정상 NativeRunOutcome. 두 결과 모두 확보 후 최종 성공 | 새 player.exe 원 observer/종료 receipt. logical Closed/handle Drop/finish_headless_close/is_finished만으로 완료 판정하지 않음 |

정상 종료의 SDK/Native 증거를 각각 기록하고 둘 중 하나가 미완료면 전체를 정상 완료로 합치지 않는다. 반환 객체가 메모리에 소유되는 것과 파일로 남긴 진단/receipt는 별개다. process 종료 후 Rust owner 자체를 직렬화해 복원할 수 있다고 주장하지 않으며, 강제 종료/미정리 상태의 최종 exit와 보존 진단을 구분한다. bounded cleanup의 시간/호출 예산과 미완료 상태 반환 포맷은 UI 실제 포트를 인수한 뒤 확정한다. main에서 새 timer/event loop/ACK 정책을 작성하지 않는다.

actual 3task와 repair fixture는 기존 canonical source/generated/draft/revision을 가진 저장 데이터를 사용한다. 모델 추론/다운로드를 재개하거나 새 synthetic 성공을 실제 추론 품질 근거로 바꾸지 않는다. 모든 actual launcher 검증에는 그 실행에 사용한 새 player.exe pin을 붙이며 과거 CLI73·Native OS9·headless/consumer 성공을 상속하지 않는다. 기능·소유/종료 검증과 물리 입력/시각 1:1 검토/AI 품질은 계속 구분한다. 최종 source·binary·입력 핀, 원 실패 및 성공 로그, 정확 담당 프로세스 owned[]와 heavy 슬롯 반환까지 묶어 checkpoint를 닫는다. 이 checkpoint가 전체 제품 목표 완료를 의미하지 않는다.
