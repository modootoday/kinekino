---
id: plan:kinekino/20261029113657
title: Native playback child 조립의 최소 후속 범위
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native playback child 조립의 최소 후속 범위

다음 source 준비의 책임별 정본은 [Native 영상 slot 소비 계약](native-video-surface-contract.md)이다. 고정 Meari49443cf의 실제 parent/geometry/LayoutBox 포트와 slot 공백, SDK assembly/mpv 소유 후보, 실제 CLI 기능 검증 대상을 구별했다. whole-window rect 대체·legacy Drop Destroy proof·raw handle 입력 우회를 금지하며 이 문서는 새 제품 구현/OS 지원 인수가 아니다.

SDK5 소비 gate 마감: [shared NativeChild 검증 정본](../../sots/verification/application/shared-native-playback-child-2026-10-20.md). 고정 producer49443cf+SDK5 union58에서 child7/router12/binding6/dispatcher4/launcheractive2/product4 총35pass(기존ignored1 제외), 최종 root default/native·bridge strict 및 양 fmt0. default libtest bridge3 dead_code 원101은 보존했고 native-feature cfg3줄만 추가한 최종r2 pinFC7BF70F94EF7B0A8705A3B1DD936B5E3CBB806AEEBC152D7774782445C13946으로 좁혔다. source5/producer394/보호527 delta0·receipt12 terminal·PID24 부재, heavy/snapshot PM 반환. root 통합/gitlink/커밋은 아직 미실행이며 자막7/API6/PNG 후보를 이 snapshot에 섞지 않았다. 다음 source 업무는 NativeSurfaceSlot 계약과 실제 CLI 검증 대상의 준비다.

후속 가속 배정으로 producer 인수 전 대기는 통합/실행 gate에만 적용하도록 변경됐다. SDK는 독립 checkout `runtime/development/sdk-shared-native-child-r1`에서 bridge2+child3 exact5와 회귀7을 작성했다. frozen `runtime/coordination/sdk-shared-native-child5-r1/pins.json` SHA256 6E5F8449439A1A9792767EC143867DA94E7A3FF9943062504EF95D2763435864, producer 의존 49443cf7e55585937d5a0314826f5642565cbed4/고정 overlay53. PM의 producer 실제 gate 인수를 전달받았지만 SDK5 자체 컴파일/실행은0이다. 직접 rustfmt/diff만 확인했다. 정확한 입력·기능·미검증·후속 gate는 동결 디렉터리 handoff.md 참조. root live/index/gitlink/공유 snapshot은 변경하지 않았다.

동결 SDK5 독립 소스 검토는 확정 수정필요 결함 없이 마감됐으나 실행 인수는 별도다. alias Drop은 coordinator가 machine을 계속 보유하는 범위이며 NativePlaybackChild 최종 Drop/lease/OS surface 증거가 아니다. 직접 child 거부 시험은 Closed만 있고 Full/Faulted는 없다. unknown late signal은 같은 attempt만 검사하며 old-attempt/A-B-C는 기존 router 검증을 참조한다. CleanupMachine 단일 attempt driver 전제, 실제 NativeParentLease/mpv/HWND/OS loop 미검증을 유지한다. 소비 입력은 gate-input/union58.json(고정 producer53+SDK5)과 보호527이며 최신 C21/b0c/검증 launcher2를 되돌리지 않는다.

## Shared owner 관측 불능의 공통 포트 요구 — 읽기 검토, 구현 미배정

router contextual commit은 `a3e9fad7ffbe305240c4fc6e74714a075e2db7b5`(exact4/1460 insertions)이다. index를 PM에 반환했다. 아래는 후속 설계 제안이며 제품 편집/Cargo 실행이 아니다. 읽은 Meari는 현재 제품 submodule 소스이며 고정 gate snapshot의 새 기능 인수로 간주하지 않는다. producer의 `runtime/development/meari-web-icons-clean/.spec/plans/native-host-port.md`와 대조했다.

### 현재 소스가 보장하는 것

- `application/window_lifecycle/ports.rs`: NativeChild의 Err는 no-inflight인 terminal retryable 실패다. 관측 mutex 오류/Internal만으로 이 조건을 증명할 수 없다.
- `coordinator.rs::invoke`: Complete에서만 entry를 제거/drop한다. Err는 Failed로 전이하고 deadline 제거·signal 비활성화, adapter는 entry에 남긴다. 따라서 unknown을 Err로 보내면 retry_failed가 새 attempt를 허용하는 잘못된 근거가 된다.
- `coordinator.rs::poll_inner/next_deadline`: Waiting/TimedOut의 deadline 도달은 TimedOut 표시에 그친다. 종료·취소·join을 증명하지 않는다. 재관측은 signal.take 또는 이미 ForcedIncomplete일 때만 수행한다. next_deadline은 Waiting만 선택하므로 만료 뒤 무한 즉시 timer를 예약하지 않는다.
- `force_loss`: window phase/epoch/surface를 폐기하고 원 adapter.parent_lost를 호출한다. Waiting/TimedOut 원 attempt는 재시작하지 않으며 Active/NeedsCleanup/Failed만 새 cleanup을 준비한다. report는 remaining0이어도 ForcedIncomplete다. 실제 parent 손실을 관측하지 않은 Internal을 OS loss로 위장할 이유는 없다.
- `state.rs`: report의 remaining은 실제 entries 수다. deadline/force_loss 자체가 원 child를 제거하지 않는다. `window/parent.rs::NativeParentLease`는 Rc 권한 객체를 가지지만 ParentBinding/coordinator 연결은 Weak다. lease 보유는 실제 HWND 생존이나 coordinator의 strong owner 보유를 대신하지 않는다.
- `window/driver.rs::Drop`: abandon 통지 후 windows/adapters 등을 정리한다. `window/lifecycle.rs::abandon/removed(true)`는 권한 폐기·ForcedIncomplete이며 정상 cleanup 완료가 아니다. 잔여 child를 계속 소유하려면 trusted assembly가 원 Rc<WindowLifecycle<C>>·router/retirement owner·dispatcher host를 명시 보유해야 한다. driver Drop 뒤 lease만 남기면 충분하다고 설계하지 않는다.
- `native_host/children.rs`: 현재 RetryableFailure만 child retry 경로로 보내고 나머지 비Complete report는 일반 Pending으로 합친다. TimedOut도 이 경로다. 따라서 deadline만 기다리는 설계는 관측 불능을 즉시 드러내지 못한다.

### 권장 최소 의미 확장 — Meari producer와 확정 필요

기존 Err 계약은 유지한다. `CleanupProgress::ObservationUnavailable` 및 `CleanupStatus::ObservationUnavailable`이라는 별도 상태를 제안한다. 이름은 producer 확정 전 제안명이다. 이 상태는 원 work/attempt/owner가 남지만 completion을 관측할 수 없다는 의미이며 Pending·terminal 실패·OS loss·Complete 중 어느 것도 아니다.

NativeChild에 기본값 None인 `observation_error(&self) -> Option<&Self::Error>` 읽기 메서드를 추가하는 안을 제안한다. SDK child는 원 Arc<ApiError> 진단을 내부에 보존하고 새 progress와 함께 노출한다. coordinator는 child를 빼내거나 error를 복제하지 않고 `with_error`의 차용 범위 안에서 기존 entry.error 또는 adapter.observation_error를 읽는다. 이 진단은 권한·joined proof가 아니다.

coordinator는 ObservationUnavailable에서 entry/adapter/lease/원 signal·stamp·deadline을 보존하고 Failed로 만들지 않는다. entry에 별도 observation-unavailable 플래그를 두어 Waiting/TimedOut과 구분한다. report는 이 플래그를 즉시 노출하고 timeout 도달 횟수는 기존 timed_out 필드로 함께 보고한다. retry_failed는 이 entry를 제외한다. 같은 원 signal의 진짜 상태 변화는 같은 attempt를 재관측하며, 성공 관측 회복 때만 unavailable 플래그를 해제한다. deadline 도달은 여전히 성공·terminal 증거가 아니고, 만료 뒤 반복 timer를 예약하지 않는다.

신호가 더 오지 않을 수 있으므로 `WindowLifecycle::reobserve_unavailable`에 해당하는 명시 재관측 유스케이스도 필요하다. 대상/권한 검증 후 기존 request로 invoke(false)만 수행하고 revocation/attempt 증가·begin_cleanup·task 생성·SDK shutdown 재호출은 금지한다. 영구 poison이면 typed 진단과 owner를 계속 보존하며 정상 Pending처럼 숨기지 않는다. 임의 mutex poison 복구로 joined 권한을 만들지 않는다.

ChildWindowHost는 unavailable report를 일반 Pending에 합치지 않고 typed ChildHostError::Children(report)와 원 checkpoint를 노출한다. Meari producer 추가 소스 검토로 기존 CloseDecision::RetryableFailure 재사용안은 제외했다. native_host/runtime.rs가 이 상태에서 HostLifecycle::CleanupFailed로 바꾸고 close_retry를 설정하지 않으며 window.rs::observe_close가 HostSchedule::fail을 호출한다. 이후 lifecycle_turn은 Closing만 관측하고 failed schedule은 ready_at(false)를 거부하므로 late 원 signal이 host close로 이어지지 않는다.

producer 제안의 별도 `CloseDecision::ObservationUnavailable { checkpoint, error }`와 `HostCloseStatus::ObservationUnavailable`을 소비한다. 원 checkpoint/typed error와 HostLifecycle::Closing을 유지하고 HostRetry::OnWake, window schedule의 defer(OnWake)를 연결해야 한다. 새 task·child begin·attempt 증가는 없다. 기존 RetryableFailure는 terminal no-inflight child 실패의 명시 retry용으로 남긴다. 다음 명시 관측은 위 reobserve 포트를 호출하며 child 새 attempt를 시작하지 않는다. 자동 tight retry/자동 force_loss/실제 window 제거는 허용하지 않는다. 이 producer 포트는 source-only 배정 상태이며 인수된 public API가 아니다.

실제 parent loss가 발생하면 기존 forced 경로로 권한을 폐기하고 ForcedIncomplete로 보고한다. 원 child/retirement/dispatcher 소유는 남으며 실제 joined receipt 이후에만 해당 자원을 해제한다. SDK engine/IPC/worker 종료와 OS child surface Destroy receipt는 별도로 대조한다. Legacy Drop의 DestroyWindow 호출은 후자 증거가 아니다.

### 정확한 경로와 호환 경계

Meari 루트 `packages/meari/packages/meari-native/src/` 기준 최소 생산 경로 후보:

1. `domain/window_lifecycle.rs`: progress/status variant 추가. 기존 public enum exhaustive match 소비자는 source 호환 수정이 필요하다. 암묵적 호환 확장이라고 발표하지 않는다.
2. `application/window_lifecycle/ports.rs`: 기본 진단 차용 메서드/새 상태 계약. 기존 NativeChild 구현의 필수 메서드 서명은 유지한다.
3. `application/window_lifecycle/state.rs`: retained-unavailable 표시와 report 우선순위/remaining 보존.
4. `application/window_lifecycle/coordinator.rs`: invoke/poll/next_deadline/with_error/명시 같은-attempt 재관측, 기존 Failed retry 제외. 이 파일의 회귀에도 추가한다.
5. `infrastructure/native_host/children.rs`: unavailable typed 보고·원 checkpoint 보존·명시 재관측 연결 및 이 파일 회귀.
6. `infrastructure/window/lifecycle.rs`: 신규 report/진단을 실제 driver가 노출하는 경계와 deadline 전달을 검토하고 필요할 때만 수정한다. 일반 host 통지로 SDK 오류 문자열을 유실시키지 않는다.

전체 exhaustive consumer는 producer가 rg로 확정해야 하며 6개로 완료된다고 확약하지 않는다. `infrastructure/window/scheduling/` 및 domain native_host/runtime의 close-state 의미를 먼저 대조한다. 변경이 필요한 경우 exact 경로를 producer가 다시 제출한다. 제품 sharedNativeChild 합집합5와 이 공통 Meari 변경은 별도 배정/핀/gate이며 현재 확정5에 몰래 넣지 않는다.

후속 PM 배정: Meari는 검증 A5c459 기초에서 producer9+시험2를 별도 source-only로 준비한다. domain native_host close decision/status·native_host runtime·window close/scheduling 소비 경로까지 producer가 exact 경로를 고정한다. SDK는 producer 소스/핀/회귀·strict 인수 전 sharedNativeChild 합집합5의 production 편집을 시작하지 않는다. 실제 소비 때 full CleanupStamp+reason+signal 대조→원 RetirementHandle 관측→Unavailable 진단 차용→원 signal/같은 attempt의 late 완료→host Closing/OnWake 유지의 전체 흐름을 확인한다. fault 이후 기존 Engine/Incomplete 진단과 관측 오류 진단은 서로 덮어써 유실시키지 않는다. recovery 후 원 joined 결과에서만 Complete 또는 진짜 no-inflight Err를 반환한다.

producer 최신절의 추가 소비 조건: CleanupReport의 unavailable count와 기존 remaining/failed/timed_out를 함께 읽는다. 단일 status만 보고 혼합 Failed를 유실시키거나 unknown에 retry_failed를 적용하지 않는다. generic ChildHostError::Children 로그는 원 ApiError 상세를 보존하는 통로가 아니므로 original ChildLease로 children().with_error를 차용한다.

일반 ChildWindowHost.close/poll은 원 signal 경로만 관측한다. 호출마다 unknown을 무조건 재관측하지 않는다. 신호 없는 명시 재관측은 trusted UI owner가 reobserve_unavailable(expectedBarrierStamp)로 owner/barrier를 검증한 뒤 invoke(false)만 수행한다. 이후 host 관측을 잇는 wake는 아래 고정 r2 대조대로 관측 전용 `wake()`를 쓰며 `request_close`로 대체하지 않는다. 순수 redraw/deadline/일반 completion retry를 이 명시 권한으로 간주하지 않는다. producer가 별도 driver 포트가 필요하다고 판단하면 해당 경로·호환 범위를 먼저 확정한다.

### 고정 producer P2 r2 소비 대조 — 컴파일 전

읽기 입력은 `runtime/development/meari-cleanup-observation-r2`, Git HEAD `e06e433c0009e38d91f476a99486cb877b7f7a17` exact12다. SHA는 디렉터리 하위 경로가 아니라 소스 revision이다. producer compile/gate 미실행 상태이며 public 지원 인수로 표시하지 않는다.

추가 미해결 P2: private `native_host/window_port.rs`에서 runtime.try_borrow_mut가 Busy이면 explicit 의도가 runtime에 전달되기 전에 유실될 수 있다는 독립 검토가 있다. **Busy는 실행 전 미수락**이며 요청 접수/명시 retry 실행으로 표시하지 않는다. PM이 producer r3 최소 수리를 배정했으므로 r2는 인수 전, SDK 제품 후속5는 보류다. r3에서는 원 host/scope에 의도가 보존되고 해당 원 의도만 한 번 소비되는지 대조한다. 이 경계가 해결됐다고 가정해 자동 재시도를 명시 retry로 승격하지 않는다. 별도 새 포트 설계 확장은 하지 않는다.

후속 소스 상태: PM이 `runtime/development/meari-cleanup-observation-r3` revision `49443cf`의 독립 재검토 마감을 인계했다. 원 port host/scope/definition 의도를 Busy 뒤 보존하고 다음 false 관측에서 원 의도만 정확히 한 번 소비하도록 소스 수리됐으며, 추가 확정 결함은 보고되지 않았다. 위 r2 항목은 이 r3 수리로 연결하되 **컴파일/실행0·실제 gate 미인수**로 유지한다. Busy 자체를 수락으로 표시하지 않는다. SDK 제품 후속5는 producer 실제 gate 이후 별도 배정이며, 현재 UI C 통합 gate가 우선이다.

`ChildWindowHost::close_inner`는 ObservationUnavailable의 checkpoint.retry_children를 `report.failed != 0`으로 설정한다. 이전의 무조건 false 제안은 혼합 unknown/실제Failed 형제 상태에는 적용하지 않는다. 대신 **explicit && retry_children**일 때만 retry_failed를 실행한다. unknown은 Waiting/TimedOut+별도 flag여서 retry_failed 대상이 아니며 원 attempt/signal은 보존된다. 자동 observe_close(false)는 flag가 true여도 실제Failed 형제를 재시작하지 않는다. 감싼 business host에도 close/observe_close 선택을 그대로 전달한다.

명시 `request_close`는 private window_port까지 explicit=true를 유지해 실제Failed 형제 retry 권한을 전달한다. 이것이 unknown의 same-attempt reobserve를 자동 실행한다는 뜻은 아니다. unknown의 신호 없는 재관측은 여전히 별도 coordinator.reobserve_unavailable(expectedBarrierStamp)다. 그 뒤 host에 **관측만** 요청하려면 `NativeWindowWake::wake()`를 쓴다. r2 window.rs에서 Closing+COMPLETE bit는 observe_close(false)로 전달된다. `request_close()`를 사용하면 CLOSE bit가 우선되어 Failed 형제의 새 attempt를 허용하므로 관측 전용 UI 동작의 후속 wake로 쓰지 않는다. retry_failed_completion(RETRY-only)은 closing 관측 포트의 대체가 아니다.

자동 late 원 CompletionSignal은 LIFECYCLE→lifecycle_turn→Closing host의 observe_close(false)로 이어진다. underlying dispatcher join의 원 wake는 원 signal을 유지하며 새 signal/task로 교체하지 않는다. deadline/일반 redraw/OnWake 자동 관측도 explicit=false여야 한다. 실제Failed 형제의 명시 retry가 global barrier를 바꿀 수 있으므로 후속 명시 reobserve는 현재 coordinator barrier를 다시 읽어 검증하고, unknown child의 원 CleanupStamp/attempt/signal은 그대로 유지한다. 옛 barrier로 명시 재관측이 거부되는 것은 새 작업 재시작 권한이 아니다.

소스 회귀 `mixed_unavailable_and_failed_children_retry_only_on_explicit_host_request`는 자동 관측3회에서 starts(1,1), 명시 request_close 뒤(1,2), unknown 원 stamp 동일·형제 옛 signal 거부·Closing/OnWake 유지·두 원 완료 관측 뒤 business close1을 단언한다. 현재는 **작성된 단언의 읽기 확인**이며 실행 통과 주장은 하지 않는다. trusted assembly의 strong coordinator/router/dispatcher owner와 typed 진단 차용, 이후 실제 surface Destroy receipt 분리 조건은 유지한다.

producer9의 추가 exact 후보는 `infrastructure/native_host/{types,runtime}.rs`와 `infrastructure/window.rs`이며 `window/scheduling/state.rs`는 기존 defer(OnWake)/failedfalse를 재사용해 편집하지 않는 안이다. 시험2는 `tests/window_lifecycle/completion.rs`와 `infrastructure/native_host/scheduling/tests.rs`다. 이 경로 목록은 producer 제안/배정 인계이며 SDK의 Meari 편집 권한이 아니다.

필수 producer 회귀: unavailable가 terminal Err/정상 Pending으로 합쳐지지 않음; deadline 만료 후 remaining/원 adapter/lease/attempt 보존; retry_failed는 새 작업0; 명시 재관측도 shutdown/task0; late 원 signal로 같은 attempt 완료; 영구 unavailable에서 진단 노출·타이머 spin0; 실제 force_loss 뒤 권한폐기/원 owner 보존; driver Drop을 Complete로 세지 않음. SDK는 후속5에서 원 RetirementHandle/전체 CleanupStamp/원 진단 유지와 actual joined 권한만 사용함을 검증한다. headless 회귀가 OS HWND Destroy/clip/영상합성을 입증하지 않는다.

2026-10-20 router gate 마감: [검증 정본](../../sots/verification/application/native-parent-router-2026-10-20.md). r2 실제 router12, r3 binding6/dispatcher4, r4 default/native all-target strict와 fmt가 통과했다. 최종 source4 핀 FE219612964C28256AFDA5237CFF32FC39538D214928108F4B2A293C3279EFD7, 보호551 delta0/receipt8 terminal/PID16 부재. 원0test·r3 strict101을 보존하고 성공으로 합산하지 않는다. router source 검증 완료와 NativeChild/main/OS surface 실제 조립 완료는 구별한다. heavy/snapshot은 PM에 명시 반환하며 root index는 PM 소유다.

2026-10-20 SDK 조립 계획. [P3 계획](native-resolution-handoff.md)과 [검증된 dispatcher](../../sots/verification/application/native-join-dispatcher-2026-10-20.md)를 잇는다. 첫 cleanup adapter exact4는 r2의 strict 크기 경고 수리 후 r3로 focused5·feature-on all-target strict·default lib/bins strict·fmt를 통과했다. [실행 정본](../../sots/verification/application/native-playback-child-2026-10-20.md)에 원본/실패/수리/한계를 기록했다. 커밋은 `b124226c16697c61fb3ad1e1380f2e4a120bfcb4`이며 실제 OS 연결 완료는 아니다. 검증된8파일은 변경하지 않는다.

## 첫 단계: 실제 cleanup 포트 조립

제안 exact 범위는 신규 `src/assembly/meari/playback_child/{mod,cleanup,tests}.rs`와 기존 `assembly/meari/mod.rs`의 최소 기능 등록이다. main·SDK wire·application DTO·CLI·Cargo·Meari는 바꾸지 않는다.

외부 NativePlaybackChild는 UI thread의 실제 NativeParentLease와 private cleanup machine을 소유한다. 생성 시 lease.window/child를 내부 기대 identity로 저장하고 별도 broker epoch를 전달한다. 두 identity namespace의 연결은 trusted assembly caller 계약이며 생성자가 OS parent와 broker의 연관을 인증하지 않는다. 내부 cleanup machine은 기존 ParentBinding/ParentCleanupTask/DispatcherHandle을 소비하며 정책·엔진 구현을 복제하지 않는다. 테스트는 같은 내부 machine을 공개 WindowLifecycle/CompletionSignal에 붙여 실제 request stamp와 waker를 검사한다. NativeParentLease/OS window 생성 자체는 이 headless 회귀로 입증하지 않는다.

- parent_lost는 revoke_admission만 호출한다. UI에서 shutdown/join/SDK 재호출을 하지 않는다.
- begin_cleanup은 request와 signal의 전체 CleanupStamp(window/child/revocation/attempt), child identity를 대조한다. 같은 진행 attempt에서는 이미 소유한 task만 관측하고 재실행하지 않는다. 새 attempt는 이전 worker의 종료와 retryable 결과가 확인된 뒤에만 시작한다.
- CompletionWake에는 signal clone만 넣는다. Dispatcher가 child join→결과 게시 후 signal.notify를 호출한다. notify의 false는 이미 dirty이거나 비활성일 수 있으므로 전달 실패/cleanup 완료로 단정하지 않는다.
- observe_cleanup은 같은 stamp의 joined 결과를 읽는다. Complete만 Native CleanupProgress::Complete로 대응한다. worker가 종료된 오류·불완전 receipt는 원 task/owner와 증거를 보존하는 retryable failure로 다루고, 미래 wake가 없는 Pending 상태로 숨기지 않는다.
- Host dispatcher의 outstanding delivery와 shutdown/join은 childComplete와 별개다. 조립 owner는 child 정리 뒤 host admission close/drain/join을 별도 마감한다.

필수 회귀는 worker 종료가 늦어도 GUI timer 없이 scoped wake로 재관측, 초기 in-flight open→revoke→동일 attempt 완료, stale/foreign stamp·옛 signal 거부, 중복 begin의 새 task0, 실패 후 명시 retry와 원 owner, host 종료/queue 거부 중 child 결과 보존이다. 실제 OS 없이 기존 공개 lifecycle과 fake engine으로 먼저 검사하고, 일반 callback 반환을 실제 GUI event 소비로 표시하지 않는다.

## 다음 단계: parent-ready와 실제 mpv

NativeChildRegistrar.register는 NativeScopedParent와 !Send lease를 factory에 제공하고 ChildLease를 반환한다. Factory Creating 단계의 parent 접근과 등록 성공 뒤 Active 단계의 readiness는 다르다. engine binding을 factory 중간에 공개해 SDK open이 등록 완료보다 앞서지 않도록, 등록 성공 후 lease.with_parent로 재검증하는 UI-only ready commit을 별도로 설계한다. 실패 시 partial child/engine owner와 cleanup 권한을 보존한다.

Raw handle 처리는 기존 `wry::raw_window_handle` 재노출을 활용할 수 있는 infrastructure adapter에 한정한다. 현재 mpv parent 인수는 u32이므로 checked 변환이 실패하면 명시적으로 거부하고 truncate/None standalone fallback하지 않는다. 기존 VideoSurface.id의 캐스트를 새 경로의 검증으로 쓰지 않는다.

기존 NativePlaybackHost의 Drop은 동기 shutdown이므로 UI child가 그 guard를 무심코 Drop하는 조립은 금지한다. GUI는 기존 NativeEpisodeEngine을 한 번 만들고 broker에 결합한 뒤 검증된 dispatcher 경로로 명시 정리하도록 소유를 배치한다. 새 SDK instance를 만들지 않고 같은 configured SDK에 주입된 deferred engine을 유지한다. CLI의 None standalone은 바꾸지 않는다.

TSX video slot의 geometry/clip/focus와 실제 child surface 소유·Destroy 확인은 아직 별도 계약이 필요하다. whole host HWND에 바로 mpv를 붙이거나 검은 placeholder를 재생 성공으로 표시하지 않는다. 이 조건을 채운 후에만 실제 media decode/시간 진행·제어·parent loss/종료를 CLI 우선으로 배정한다. DPI·VM 보류 범위와 전체 시각 인수 규칙은 유지한다.

## 현재 공개 포트와 공백 대조

| 항목 | 현재 관측한 포트 | 실제 video slot에 남은 계약 |
|---|---|---|
| borrowed parent 권한 | NativeScopedParent의 접근마다 validate, lease.with_parent/with_cleanup_parent, window/child stamp | OS parent 접근 권한이다. 특정 TSX node의 video 영역이나 렌더 준비를 뜻하지 않는다. |
| rect·scale | Registrar.geometry는 revision+HostEnvironment(layout/visual viewport, device_scale, layout_zoom, safe_area)를 반환 | document/source generation·slot node identity·layout revision에 묶인 client-relative slot rect와 좌표 변환 정의가 필요하다. host 전체 viewport를 slot으로 쓰지 않는다. |
| clip | Legacy VideoSurface.layout은 bounds*scale의 SetWindowPos와 visible ShowWindow를 수행 | ancestor/scroll/overlay에 따른 effective clip·가시성·occlusion의 공개 slot 통지가 확인되지 않았다. 임의 parent 크기로 채우지 않는다. |
| z-order | 현재 surface는 SWP_NOZORDER를 사용 | popup·caption·Inspector와의 겹침 순서/합성 방식에 대한 공통 포트가 없다. 실제 overlap 인수 없이 overlay가 된다고 주장하지 않는다. |
| focus·pointer | Legacy adapter는 foreground parent 조건에서 SetFocus(parent)로 돌린다 | Meari 논리 focus owner/keyboard/pointer와 child 표면의 입력 정책이 필요하다. 물리 focus 이동을 typed node focus 인수로 대신하지 않는다. |
| surface 종료 | Legacy Drop은 DestroyWindow 결과를 무시한다 | typed close/실제 종료 확인·실패 시 원 surface owner 보존이 필요하다. 이 Drop을 NativeChild Complete 근거로 쓰지 않는다. |

Meari에 필요한 공통 포트는 제품 정책이 아닌 host-local native surface slot의 안정 식별·rect/clip/visibility/z-order/focus 관측과 generation별 무효화다. UI는 실제 PlayerFrame의 video slot과 overlay 요구를 제공하고 SDK는 해당 권한 안의 mpv child/engine 수명을 조립한다. 현재 public slot port의 존재나 지원을 발명하지 않으며, 먼저 cleanup adapter 단계를 독립 인수한 뒤 공통 slot 요구를 UI·Meari 담당과 조율한다.

## r2 동결 입력과 focused gate 계획 원문

아래는 실행 전 계획이다. 실제 r2 실패와 r3 통과·자원 반환은 위 실행 정본을 따른다. 현재 다음 실행을 예약하거나 Cargo 권한을 부여하는 절이 아니다.

소스 정본은 `runtime/coordination/sdk-native-child4-r2/source/`, exact4 목록은 동폴더 `paths.txt`, SHA/bytes는 `pins.json`이다. pin JSON SHA256은 `36B318EE936ED8734BF488438C926A090C0E8B7D61D3CF6D0AA038030AFD83A6`이다. r1 사본은 별도 폴더로 보존한다. Rust 소스 추가·수정과 Cargo 실행은 다음 배정까지 하지 않는다.

UI Inspector의 explicit 반환 뒤 종료된 snapshot의 revision을 이어받는다. 예상 base는 Meari0589 + UI9 + 기존 SDK3a8/UI11이며, UI9는 검증 통과 입력이 아니다. Inspector runtime4는 Meari0589 공통 mixed-slot preflight 공백으로 실패했고 producer 수리를 기다리는 상태라는 PM 인계를 받았다. SDK gate는 renderer를 거치지 않는 공개 lifecycle/coordinator/dispatcher5 범위이며 이 알려진 UI 실패4를 성공으로 재분류하지 않는다. 실제 인수 시 UI 최종 입력핀·터미널 receipt·PID 종료와 보호 데이터 경로를 확인한다. 실행 중 snapshot은 변경하지 않는다. 새 gate 폴더에 base 식별과 exact4 기존 존재 여부·원 bytes/hash를 보존한다. 신규3파일의 이전 부재도 기록한다. 동결 source4만 overlay하고 final SHA/bytes를 대조하며 그 외 base 보호파일 delta0을 확인한다. SDK/Meari/UI source를 추가 혼합하지 않는다.

단일 heavy owner/기존 Rust resource runner, offline locked -j1, RUST_TEST_THREADS=1 조건으로 아래 명령을 순차 실행할 계획이다. 각 stdout/stderr/receipt·실제 cwd·입력핀을 보존하며 관측 timeout 때문에 재실행하지 않는다.

1. `cargo test -p kinekino-player --offline --locked -j 1 --features native-proposal-window --lib assembly::meari::playback_child::tests -- --test-threads=1`: 정확5시험 실행을 확인한다. 0시험 통과는 인정하지 않는다.
2. `cargo clippy -p kinekino-player --offline --locked -j 1 --features native-proposal-window --all-targets -- -D warnings`: 새 테스트 코드까지 lint한다. unrelated 기존 실패도 분리 기록하고 임의 source 수리는 하지 않는다.
3. `cargo fmt --check`: snapshot 전체 형식 검사를 기록한다.
4. 기본 feature 격리 제안: `cargo clippy -p kinekino-player --offline --locked -j 1 --lib --bins -- -D warnings`. 기본 feature에서는 `cfg(all(windows, feature = "native-proposal-window"))`가 새 모듈과 optional meari-native 참조를 제거하는지 확인한다. 기능 회귀5는 이 문맥에서 중복 실행하지 않는다.

정적 확인: root feature는 optional meari-native를 활성화하고 Windows target dependency는 `window` feature를 켠다. 그 공개 경로의 NativeParentLease.window/child, NativeChild의 Error 제약, WindowLifecycle의 생성·register·close·retry, CompletionSignal.stamp/notify, DispatcherHost.start/handle와 ParentCleanupTask 생성·관측 signature는 r2 사용 형태와 대응한다. 실제 NativeParentLease의 Rc 소유로 outer owner는 !Send이고 worker closure에는 signal과 broker만 이동한다. 이 확인은 Rust typecheck 통과를 대체하지 않는다. 추론·borrow·trait·lint 위험과 타이밍 회귀는 첫 gate에서 확정한다.

r2 회귀5의 범위는 joined 실패 후 explicit retry, broker epoch mismatch, gated worker의 최종 실제 channel wake 후 단일 coordinator 관측과 중복/foreign/stale 입력, closed dispatcher 거부의 owner 보존, incomplete typed receipt의 retry 후 보존이다. 초기 in-flight open→revoke 경계는 기존 broker 검증과 별개로 조립 단계에서는 아직 미검증이다. 반환한 오류 clone은 원 receipt/worker flags/진단을 유지하지만 explicit retry는 adapter 내부 이전 task/진단을 교체한다. 이 adapter는 장기 감사 이력 저장소가 아니다.

실제 OS parent 생성·private lease 생성·NativeSurfaceSlot·main·mpv 실행·GUI event 소비는 이 gate의 인수 범위 밖이며 계속 미검증이다. DPI·VM 보류는 유지한다.

## SDK/main 잔여 계약 재대조 — wire18 커밋 뒤 읽기 제안

2026-10-20. 채택 출력 판정 연결은 별도 커밋 `9c9846c054714df991b3784cf0d2b5a6e01d0b0e`로 끝냈다. 이 절은 native playback 후속 읽기 검토이며 제품 소스·Cargo·미디어/OS 실행은0이다. 이전 standalone 실제 CLI 결과와 NativeChild cleanup5를 실제 Meari video slot 연결 성공으로 합치지 않는다.

| 실제 소스 관측 | 남은 계약/구현 |
|---|---|
| main은 status channel/legacy control port·subtitle/store를 한 SDK에 넣고 current-user LocalHost를 시작한다. NativeProposal은 review용 product native launch로 분기한다. | 플레이어용 CatalogArchive/EpisodePlaybackSessions/EpisodeTransitions와 부모 라우팅을 같은 configured SDK에 주입하는 조립이 없다. review 전용 창을 full player로 부르지 않는다. |
| CLI native_host는 pinned executable+CatalogFiles+NativePlaybackHost(parent=None)로 transport를 시작하고 stdin EOF에 명시 shutdown한다. | 이 standalone 관측은 GUI TSX child 연결이 아니다. CLI에서 GUI를 조작하지 않고 향후 GUI configured LocalHost endpoint에 기존 playback API를 호출해야 한다. |
| ParentBinding은 한 NonZero epoch/엔진을 한 번 bind하고 revoke는 영구적이다. DeferredEpisodeEngine은 그 binding/epoch를 불변 보유한다. | parent 파괴/새 window generation 뒤 같은 SDK를 유지하려면 명시 generation route 교체가 필요하다. 기존 binding을 reset/재사용하거나 새 SDK로 우회하지 않는다. 일반 media close/reopen은 기존 엔진 수명과 구분한다. |
| NativePlaybackChild는 실제 !Send lease를 보존하고 cleanup task를 dispatcher로 보낸다. main/Registrar factory의 실제 연결은 없다. | registration 성공 뒤 Active lease를 재검증한 ready commit, engine/child partial failure 소유, dispatcher host의 별도 shutdown/drain/join 조립이 남는다. |
| NativeEpisodeEngine은 parent Option<u32>를 생성 때 고정한다. NativePlaybackHost Drop은 동기 shutdown이다. | 새 GUI slot generation은 새 engine owner를 필요로 한다. parent handle은 checked 변환으로 전달하고 whole-host HWND·None fallback·UI blocking Drop은 허용하지 않는다. |
| 현재 scoped source 검색에서 NativeSurfaceSlot/SurfaceSlot/surface_slot 공개 native port가 확인되지 않았다. registrar geometry는 host viewport/environment다. | 공통 slot identity/rect·scale/effective clip·z-order/focus/visibility/파괴확인 포트가 선행한다. 기존 VideoSurface의 SetWindowPos/ShowWindow/Drop만으로 이 계약을 충족했다고 하지 않는다. |

### 첫 최소 source 후보: 세대 라우팅 exact4

Meari slot 계약과 별개로 먼저 검증할 수 있는 SDK 측 후보는 신규 `src/infrastructure/mpv/parent_router/{mod.rs,route.rs,tests.rs}`와 기존 `src/infrastructure/mpv/mod.rs` 최소 등록 exact4다. 아직 배정·구현하지 않았다. 기존 ParentBinding/DeferredEpisodeEngine/ParentCleanupTask의 검증된 정책을 복제하거나 수정하지 않는다.

router는 한 configured EpisodePlaybackSessions에 주입할 EpisodePlaybackEngine 포트로서 trusted assembly의 현재 route를 선택하고 기존 deferred engine에 동일9 메서드를 위임한다. OS/window handle이나 !Send lease를 보유하지 않는다. route는 broker epoch/binding/engine owner를 보존하며 준비 전·영구 revoke·retiring 상태를 분리한다. activation/retirement는 SDK 공개 Request나 웹 메시지로 받지 않고 assembly 전용 계약으로 한정한다. route 선택은 OS parent 유효성 증명이 아니며 실제 ready commit은 아래 별도 단계다.

교체는 이전 route를 revoke하고 그 owner/cleanup 권한을 계속 보유한 상태에서만 진행한다. 새 route가 준비되더라도 이전 join/cleanup 결과가 불완전하면 자동 활성화하지 않는 첫 정책을 제안한다. 거절/오류는 제안된 새 owner와 이전 owner를 보존해야 한다. 이미 admit된 작업은 기존 binding의 in-flight 계약을 따라 끝나며 late control/close/observe가 새 instance에 적용되지 않아야 한다. 공개 metadata나 caller의 Complete 주장만으로 cleanup 증명 token을 발명하지 않는다. 구체 release/ack 타입과 반환 소유는 source 배정 전에 PM 검토로 좁힌다.

첫 fake 회귀 후보는 한 SDK 핸들의 prepare→ready→retire→new generation, 잘못된 epoch/중복 activation의 owner 보존, 이미 admit된 open 완료와 추가 admission 차단, 불완전 cleanup 중 새 route 거부, 완료 뒤 새 instance 선택·옛 instance 명령 거부, 실패 후 같은 owner 재시도다. 이 exact4가 통과해도 parent-ready/slot/main/실제 mpv 성공으로 표시하지 않는다. 새 CLI/wire 명령은 첫 단계에 필요하지 않으며 기존 playback.engine의 NeedsInput/Cancelled와 instance ID 오류 계약을 보존한다.

### 실제 조립의 후속 의존 순서

1. Meari 공통 native slot producer와 UI PlayerFrame의 slot/overlay 요구를 책임 담당이 먼저 확정한다. source node/document generation과 surface 종료 확인을 확보하기 전 root HWND로 대체하지 않는다.
2. UI-only parent-ready 조립은 Registrar factory에서 lease/partial child 소유만 구성하고 registration 성공 후 lease.with_parent와 slot generation을 재검증한다. 현재 lease는 child 내부에 move되므로 clone 가능한 raw 권한을 꺼내지 않고 UI 내부 단일 lease owner와 제한된 ready controller 수명을 설계해야 한다. ready 전 SDK open은 NeedsInput이며 constructor 성공/transport ready를 media ready로 표시하지 않는다.
3. root assembly의 별도 player host module에서 CatalogFiles/지정 mpv/dispatcher/라우팅을 준비하고 main은 조립 호출만 한다. 검증된 engine을 같은 configured SDK의 sessions/transitions에 주입하고 LocalHost client와 UI client를 공유한다. 새 SDK 생성·UI 전용 정책·CLI GUI 자동화는 없다. 원 review launch와 standalone CLI는 별도 scope를 유지한다.
4. CLI→동일 GUI host endpoint의 기존 playback.open/engine/control/options/transitions/close를 실제 미디어로 검증한다. transport/process/media loaded·decode/시간진행·명령접수/적용·child cleanup/dispatcher 종료를 각각 기록한다. parent loss/suspend/resume 새 generation·stale ID·EOF·partial startup 실패 소유·자원 종료를 포함한다. 실제 renderer/OS/시각 검증은 별도 배정하며 DPI·VM 보류는 건드리지 않는다.

full main 단계의 exact 경로는 slot/ready producer 계약 인수 후 다시 제시한다. 현재 존재하지 않는 공통 slot 타입의 signature나 조립 성공을 만들어내지 않는다. 첫 exact4 제안도 구현 권한이나 실행 슬롯 배정이 아니다.

## Router exact4의 서명·소유 전이 구체안

PM 후속 읽기 배정에 따른 source 대조다. 아직 새 Rust 코드/Cargo는0이다. 첫 scope는 기존 제안4경로이며 아래 서명은 trusted assembly에 대한 `pub(crate)` lifecycle API 후보다. JSON·CLI·web message로 token/cleanup proof를 받지 않는다. `Arc<ParentEngineRouter>` 한 개를 기존 EpisodePlaybackSessions에 넣은 뒤 generation마다 SDK를 다시 만들지 않는다.

```rust
impl ParentEngineRouter {
    fn new(dispatcher: DispatcherHandle) -> Self;
    fn prepare(&self, engine: Arc<NativeEpisodeEngine>)
        -> Result<ParentRouteHandle, Box<RoutePreparationFailure>>;
    fn activate(&self, prepared: &ParentRouteHandle) -> Result<(), ApiError>;
    fn retire(&self, route: &ParentRouteHandle, wake: CompletionWake)
        -> Result<RetirementStart, ApiError>;
    fn observe_retirement(&self, handle: &RetirementHandle)
        -> Result<RetirementObservation, ApiError>;
    fn retry_retirement(&self, handle: &RetirementHandle) -> Result<(), ApiError>;
}
// All fields are private Rust ownership, not serializable authority.
// RoutePreparationFailure returns the proposed engine with its original error.
// RetirementStart retains the original handle even when dispatch_error is Some.
impl EpisodePlaybackEngine for ParentEngineRouter { /* existing nine methods */ }
```

위 코드 주석의 fields는 모두 private Rust 소유를 뜻한다. 공개 `ParentCleanupStatus::Complete`, 호출자 bool·epoch 숫자·Finished DTO를 인수해 다음 parent를 허용하는 메서드는 만들지 않는다. diagnostic epoch getter가 생겨도 권한이 아니다.

### Token과 유일한 완료 권한

- `ParentRouteHandle`은 private Weak router identity와 private route sequence/binding clone을 결합한다. 외부 struct literal·serde 생성·숫자 재구성은 불가하다. router가 engine owner를 내부에 보존하므로 handle Drop만으로 엔진이 유실되지 않는다. `prepare` 거부는 proposed Arc<NativeEpisodeEngine>를 boxed failure에 반환한다.
- engine을 넘기는 공개 lifecycle 경계는 NativeEpisodeEngine으로 한정하는 안이다. 임의 backend의 반복 instance ID를 새 wire 세대로 remap하지 않는다. 순수 fake는 `tests.rs`가 private cfg(test) 준비 경계에 주입하고 native의 process-wide unique ID 계약을 모사한다.
- `RetirementHandle`은 router와 caller가 함께 보유하는 opaque Arc<RetirementOwner>다. owner 안에 **그 route의 원 ParentBinding/epoch와 실제 ParentCleanupTask 하나**를 넣는다. submit 거부·panic·partial receipt·observer handle Drop에도 router가 원 owner를 유지한다. 오류/Incomplete receipt는 typed 원정보를 보존한다.
- task.observe가 `Finished(Ok(ParentCleanupStatus::Complete(_)))`를 직접 관측한 경우에만 private joined_complete 상태를 release-publish한다. 이 값은 실제 dispatcher의 worker join 뒤 result cell에서 나온다. activate/새 prepare는 자기 router에 등록된 같은 RetirementOwner identity와 acquire-load된 상태만 사용한다. 상태값이나 완료 receipt를 caller가 넣는 통로는 없다. NotStarted/Pending/Err/Incomplete는 모두 activate 불가다.
- 첫 정책은 한 active 또는 prepared route만 허용하고 이전 retiring route의 joined Complete 관측 전 새 prepare를 거부해 proposed engine을 반환한다. Drop/revoke/queue 접수/engine 종료 추정은 prepare 허용 증거가 아니다. old complete 뒤에도 새 prepare/activate는 명시 호출이며 자동 재시작하지 않는다.
- retry는 원 task의 실제 terminal/dispatch-refused 상태에서만 start_attempt를 다시 호출한다. Pending 중 재시작0·Complete 후 재시작0이다. 같은 dispatcher가 Full이면 후속 여유 뒤 명시 retry가 가능하며 Closed/Faulted이면 오류와 owner를 계속 보존한다. 첫 단계는 dispatcher를 몰래 교체하거나 원 요청을 새 engine으로 옮기지 않는다. 종료된 dispatcher의 복구/강제 종료 정책은 host 별도 계약이다.

### 상태와 오류

| 내부 상태 | engine port의 새 호출 | lifecycle 허용/소유 |
|---|---|---|
| EmptyNeverReady | NeedsInput, options/capture availability=false | prepare 허용. engine owner 없음. |
| PreparedNotActive | NeedsInput, availability=false | route/engine은 router가 소유. activate 또는 같은 route retire로 명시 cleanup. 아직 OS ready를 증명하지 않음. |
| Active | 선택한 route의 실제 binding.admit 뒤 원 engine으로 전달 | 같은 route retire만 허용. 다른 prepare/activate는 conflict와 proposed owner 반환. |
| Retiring/DispatchRefused/JoinedFailure/Incomplete | Cancelled, availability=false | 원 binding/task/receipt 보존. observe와 명시 retry. 새 prepare/activate 거부. |
| JoinedCompleteNoRoute | Cancelled, availability=false | private 실제 완료 관측 뒤 새 prepare 허용. 원 receipt는 다음 명시 전이/host 마감까지 보존. |
| 새 Prepared/Active generation | 위 준비/활성 규칙 | epoch는 checked 증가하며 old handle은 새 route에 사용 불가. counter 소진은 새 owner 반환 오류. |

Wrong/foreign token은 RevisionConflict다. caller가 이미 보유한 잘못된 token의 binding을 먼저 revoke해 다른 router를 망가뜨리지 않도록 immutable owner identity를 먼저 검사한다. 같은 router의 과거 handle은 과거 binding만 가리키므로 새 current binding을 revoke하지 않는다. private state/정확 route도 확인해 stale lifecycle 요청을 거부한다. EngineAdmission 이후 backend가 내는 NotFound/RevisionConflict/EngineFailed 등 원 오류는 다른 generation 재시도로 바꾸지 않는다.

### retire 즉시 차단과 lock 순서

선택과 admission의 선형화는 짧은 router state mutex(R) 아래 `ParentBinding::admit(epoch)`를 얻는 지점이다. route만 clone한 뒤 나중에 아무 current를 다시 선택하지 않는다. 원 EngineAdmission을 취득한 뒤 R을 해제하고 **그 admission.engine**으로 호출하며 함수 종료까지 admission을 보유한다. engine open/observe/close/options·shutdown은 R 안에서 실행하지 않는다.

retire는 opaque route의 owner identity를 먼저 검사한 뒤 handle에 고정된 원 binding의 `revoke_admission()`을 호출한다. 이는 atomic이며 R/slot/engine 잠금을 기다리지 않는다. 그 다음 R에서 해당 route를 retiring owner로 옮긴다. 따라서 R을 기다리는 retire라도 signal 이후 새 binding.admit는 Cancelled이고, signal 전에 성공한 admission만 기존 owner로 마칠 수 있다. duplicate retire는 같은 handle/원 시도를 반환하고 cleanup을 중복 제출하지 않는다. foreign token은 revoke 전 거부한다. stale old handle의 revoke가 있다 해도 그 원 binding뿐이며 새 binding을 선택해 revoke하지 않는다.

R 안에서는 route/ownership metadata만 변경한다. ParentCleanupTask::start_attempt는 state 전이 뒤 별도 task mutex(T) 아래에서 실행하고 dispatcher submit도 R 밖이다. observe는 T 아래 result cell을 읽어 private complete 상태를 게시하고 R을 취득하지 않는다. activate는 R 아래 그 owner identity와 complete atomic만 읽으며 T를 취득하지 않는다. wake callback은 raw parent/engine owner를 운반하지 않는 scoped notification이며 R/T 안에서 UI callback을 직접 실행하지 않는다.

기존 binding은 admit 동안 slot mutex(B)를 짧게 보유하고 곧 해제한다. cleanup은 cleanup mutex(C)→B, in-flight wait에서 Condvar가 B를 해제하며 engine.shutdown 전에 B를 놓는다. EngineAdmission Drop은 B만 취득하고 R/T를 취득하지 않는다. 따라서 허용 순서는 R→B, T→dispatcher/result cell, C→B 후 C→engine이며 engine I/O→R, T→R, R→T/engine/worker join 경로는 없다. poisoning/submit 오류는 fail-closed와 원 owner 보존으로 처리한다. mutex 보유를 OS/네트워크 작업으로 연장하지 않는다.

### stale instance/session의 재해석 금지

`control.rs::NEXT_INSTANCE`는 process-wide AtomicU64의 checked 증가이며 소진 시 새 instance 생성을 실패시킨다. `NativeEpisodeEngine::Active::require_id`와 close/control/options/capture의 기존 instance 검사를 유지한다. 이 근거 때문에 공개 prepare를 NativeEpisodeEngine으로 제한하며 원 engine ID를 generation 번호로 덮거나 재매핑하지 않는다. 새 parent에는 새 engine owner를 구성하되 old raw engine을 wire에서 찾거나 `None` standalone으로 우회하지 않는다.

old admitted open/observe가 retire와 겹쳐 늦게 반환해도 원 engine ID/receipt를 그대로 반환한다. 이를 새 generation의 응답으로 다시 포장하지 않는다. 이후 caller가 그 old ID로 새 Active route에 close/control/rate/track/capture를 보내면 새 native engine의 require_id가 RevisionConflict를 반환하고 실제 새 instance에 적용0이다. observe 자체는 request에 instance ID가 없는 기존 계약이므로 호출 때 선택한 generation의 결과이며 영구 최신성은 약속하지 않는다. 새 current를 다시 관측해야 하고 UI는 원 engine ID 비교 없이 이전 관측을 새 session의 성공으로 사용하면 안 된다. 같은 SDK 포트와 LocalHost를 유지하며 wire의 임의 parent/epoch 인수는 추가하지 않는다.

### 기존 binding 변경0 가능 여부와 통합 한계

source 근거상 **독립 router4 단계는 기존 binding 변경0으로 가능**하다. 같은 `mpv` 하위 모듈은 `pub(super) ParentBinding::admit`와 `EngineAdmission.engine`에 접근할 수 있다. public new/bind/revoke와 ParentCleanupTask new/start_attempt/observe/notification으로 원 소유와 joined 완료를 조합할 수 있다. 기존 cleanup은 실제 receipt의 engine/IPC/stdout/stderr 종료 flags를 모두 확인한다. 결과는 borrowed observation이므로 router가 직접 typed 오류/receipt를 보존하고 opaque 완료 상태를 만든다.

단, 현재 NativePlaybackChild cleanup machine도 자체 ParentCleanupTask를 만든다. 실제 GUI 통합 때 router task와 child task를 병렬로 만드는 방안은 채택하지 않는다. 같은 RetirementOwner를 child cleanup에 명시 주입해 begin/observe/signal을 위임하는 후속 source 변경이 필요하다. 후보 추가 경로는 `assembly/meari/playback_child/{mod.rs,cleanup.rs,tests.rs}` exact3이며 router4와 지금 섞지 않는다. 그 단계에서 Meari CleanupStamp/CompletionSignal과 router retirement handle을 전체 identity로 연결한다. ParentBinding/cleanup_task의 기존검증8 수정은 필요 없고, 만약 기존 외부 task를 그대로 사용해야 한다면 sealed task-owned proof accessor 추가가 별도 최소 변경 대안이다. 공개 Complete DTO를 받을 수 있게 넓히지는 않는다.

### 정확 첫 회귀 후보12 — 아직 작성/실행0

1. 같은 SDK engine Arc에 generation A prepare/activate→retire→실제 joined Complete→B prepare/activate. SDK/LocalHost 재생성0.
2. Empty/Prepared NeedsInput과 Retiring/Failed/JoinedNoRoute Cancelled 구분·availability=false.
3. R을 테스트 gate로 보유한 동안 retire가 원 binding atomic revoke를 먼저 수행해 새 admit를 즉시 거부. UI engine/shutdown 호출0.
4. 이미 admitted gated open은 원 owner로 끝나고 cleanup이 drain을 기다림. 새 준비 owner는 거부 시 그대로 반환.
5. retire와 동시에 open/observe/close가 겹쳐도 선택한 EngineAdmission만 사용하고 engine I/O 중 R을 보유하지 않음. 새 current 재선택0.
6. wrong/stale/foreign handle이 다른 router/current generation을 revoke/activate하지 않음. proposed owner/원 task 유지.
7. dispatch Full의 request 보존·명시 retry, Closed/Faulted의 owner 보존·자동교체/활성화0.
8. shutdown Err→Incomplete typed flags/진단→Complete의 원 task/engine 재시도. Pending 중 begin 중복0.
9. worker가 결과를 준비해도 실제 join 전 완료권한 없음; 최종 scoped wake 뒤 observe에서만 새 준비 허용.
10. 공개 Complete metadata를 lifecycle API에 넣는 경로가 없고 drop/revoke/단순 bool/epoch로 권한 생성0. opaque owner identity를 구조와 시험으로 확인.
11. B 활성 뒤 A의 close/control/rate/track/capture ID를 거부하고 새 instance 적용0. 늦은 A observe/open 응답은 원 ID 유지.
12. sequence 소진·중복 activate·prepared handle Drop·retirement handle Drop 뒤 router의 원 owner 보존과 명시 host cleanup. dispatcher host 종료는 child 완료와 별도.

가짜 backend 주입은 test-private 경계이며 native global ID 성질을 모사한다. actual OS parent/미디어/slot geometry/GUI race를 이12 합성 회귀로 통과했다고 하지 않는다. signature와 상태표의 PM 인수 뒤 exact4 source를 배정하며 현재 Cargo/제품 편집은 계속0이다.

### Router4 source 준비와 한정 lint 유예

PM 별도 배정으로 기존 `mpv/mod.rs`와 신규 `parent_router/{mod.rs,route.rs,tests.rs}` 정확4를 작성했다. 직접 rustfmt --check와 기존 등록 diff --check를 통과했으나 Cargo/typecheck/회귀 실행은 아직0이다. 기존 ParentBinding/cleanup_task/NativeChild/source9c9846c를 수정하지 않았으며 실제 main/parent-ready/slot 연결은 없다.

lifecycle new/prepare/activate/retire/observe/retry/status/current_route는 `pub(crate)`이고 root에는 engine port ParentEngineRouter만 재export한다. 아직 assembly consumer가 없어서 발생할 dead_code에 대해 PM이 **신규 parent_router 모듈 한 곳의 cfg_attr(not(test), expect(dead_code, reason=...))만** 임시 허용했다. crate 전체 allow·다른 lint 유예·test 문맥 유예는 없다. 후속 NativeChild retirement owner 주입/실제 assembly 연결 시 이 expect를 제거하고 사용되지 않는 API를 재검토한다. test-only cfg로 제품 구현 전체를 숨기지 않는다.

루틴 signature 보강은 `retry_retirement(handle, fresh_wake)`와 `current_route()`다. 전자는 새 attempt의 scoped notification을 명시하고 후자는 handle Drop 뒤에도 router가 보유한 같은 opaque route를 찾아 명시 retirement할 수 있게 한다. metadata epoch나 Complete를 authority로 받지는 않는다. production prepare는 Arc<NativeEpisodeEngine>으로 고정하고 fake engine 준비는 private helper에만 있다.

원 ParentCleanupTask 하나를 유지하면서 attempt별 FIFO wake relay를 사용한다. 이전 callback이 늦게 반환하더라도 다음 callback을 새 signal로 바꾸어 호출하지 않는다. 거부된 submit은 해당 attempt wake만 제거하고 원 작업/request owner를 보존한다. callback은 relay mutex에서 원 closure를 꺼낸 뒤 lock을 해제하고 호출하며 R/T 아래에서 호출하지 않는다. 현재 dispatch_error와 이전 typed engine/Incomplete 진단을 별도 보존해 retry 거부가 원 shutdown 오류를 덮거나 반대로 새 dispatch 오류를 숨기지 않는다. task Pending 중 재시작은 거부하며 전체 dispatcher 종료는 별도 host 책임이다.

새 회귀는 정확12다. 같은 SDK engine port 두 generation, NeedsInput/Cancelled·prepared retirement와 Drop recovery, R 경합 전 atomic revoke, admitted open drain, observe/close 동시 호출의 원 admission, foreign/stale token, queue Full/Closed 보존·retry, 원 task Err→Incomplete→Complete와 flags, 실제 join 전 metadata Complete 무권한·최종 wake, late callback과 capacity1거부/capacity2수락의 signal 분리, stale instance 옵션/제어/close/capture 거부, sequence 소진/중복 activate/handle Drop owner 보존을 작성했다. 마지막 시험은 production prepare failure가 정확 NativeEpisodeEngine Arc를 반환하는 타입/소유를 확인하기 위해 pinned inert 파일로 native owner를 구성하지만 그 파일을 실행하거나 OS parent를 접근하지 않는다. actual mpv/미디어 재생 성공이 아니다.

다음 최소 gate 후보는 router12 + 기존 parent_binding6 + thread_completion4와 root default/native all-target strict/fmt다. 기존 NativeChild5의 실제 shared-owner 연결 변경은 후속 exact3 단계에서 검증한다. 현재 UI가 heavy/snapshot을 소유하므로 source4만 동결하고 source 복사/실행/index는 수행하지 않는다.

## NativeChild shared owner exact3 책임 설계 — 추가 제품 편집0

PM 배정 읽기 검토다. router4 r1/source/pin은 동결 유지한다. 목표는 `assembly/meari/playback_child/{mod.rs,cleanup.rs,tests.rs}` exact3에서 기존 직접 ParentCleanupTask 생성을 제거하고 router의 **같은 RetirementOwner**를 사용하도록 바꾸는 것이다. 실제 창/slot/main 연결은 이 단계에도 포함하지 않는다.

### 정확3만으로 아직 부족한 내부 접근 경계

현재 root mpv는 ParentEngineRouter만 reexport하고 opaque handle/observation 타입은 private parent_router 모듈 아래에 있다. 또한 router.retire는 atomic revoke 뒤 R/T/dispatcher 단계까지 진행하므로 `NativeChild::parent_lost`의 atomic-only signal 대용으로 그대로 호출할 수 없다. 이 둘을 숨긴 Any/클로저 타입 우회나 infrastructure→assembly 역의존으로 해결하지 않는다.

따라서 실제 구현 전 별도 **최소 bridge exact2**를 먼저 배정해야 한다. 기존 `src/infrastructure/mpv/mod.rs`에는 필요한 handle/observation 타입의 `pub(crate) use`만 추가하고, 기존 `parent_router/mod.rs`에는 `pub(crate) validate_route_identity(&handle)` 및 `pub(crate) revoke_route_admission(&handle)`를 추가하는 안이다. 전자는 immutable Weak owner identity 확인만 하고, 후자는 그 확인 뒤 handle에 고정된 원 binding의 atomic revoke만 수행한다. R/T 취득·cleanup 시작·engine 호출0이며 foreign router를 먼저 revoke하지 않는다. 원 binding/cleanup_task/route.rs 수정은 필요 없다. 이2와 child3의 합집합은5이고 **현재 source4에 조용히 섞지 않는다**. router gate 뒤 PM이 별도 범위를 확정한다. 임의 public lifecycle/serialization 확장은 없다.

### 세 책임과 형식

| exact3 경로 | 변경 책임 |
|---|---|
| `playback_child/mod.rs` | 실제 !Send NativeParentLease outer owner 유지. 생성 인수의 별도 ParentBinding/epoch/dispatcher 대신 Arc<ParentEngineRouter>와 opaque ParentRouteHandle을 받음. 원 lease.window/child와 route owner identity를 고정. 원 engine/task는 clone해 새로 만들지 않음. construction failure는 lease/router/route owner를 보존하는 typed failure/partial child 경계로 반환하고 Drop으로 cleanupComplete를 만들지 않음. |
| `playback_child/cleanup.rs` | 기대 WindowStamp/child와 고정 router/route, private Meari attempt 기록만 소유. 첫 유효 begin에서 router.retire를 호출해 받은 원 RetirementHandle을 보존. 이후 observe는 같은 handle만 사용하고 새 유효 retry는 router.retry_retirement(handle,fresh scoped wake)에 위임. ParentCleanupTask::new/start 직접 호출 제거. |
| `playback_child/tests.rs` | 기존 공개 WindowLifecycle/coordinator/waker 목적을 유지해 shared owner 회귀로 전환. 실제 NativeParentLease를 위조하지 않고 metadata-only private machine+fake engine 경계로 검사. 기존 cleanup5의 목적/부정 사례를 제거하거나 open 성공만으로 약화하지 않음. |

내부 결합은 `(expected WindowStamp 전체, child ID, opaque route owner identity/generation)`이다. Meari window epoch와 broker generation을 숫자가 같다고 간주하지 않는다. 기대 window/child는 실제 lease에서 가져오고 route는 해당 trusted assembly 등록의 opaque 인수로 결합한다. 이 생성 관계도 실제 OS parent와 engine의 연관 인증은 아니다. 그 OS/slot 연관은 post-registration ready commit 단계의 책임이다.

`Attempt`는 전체 CleanupRequest(stamp+reason), 해당 CompletionSignal의 실제 stamp, 원 RetirementHandle, 관측한 terminal/error/retained diagnostics를 보존한다. signal에는 Meari window owner/epoch/surface/child/revocation/attempt 전체가 들어 있다. route handle은 바꾸지 않으며 전체 child 수명에 고정한다. caller가 CleanupStamp 숫자·router generation·공개 Complete receipt로 다른 owner를 지정할 수 없다.

### begin/observe/parent_lost 순서

1. parent_lost는 bridge의 fixed-route atomic revoke만 호출한다. 원 lease/router/route/retirement handle은 유지한다. SDK call·R/T wait·dispatcher submit·OS parent 접근0이다. signal 오류는 private diagnostics로 보존하고 정상 완료로 변환하지 않는다.
2. begin은 먼저 lease에서 고정한 전체 window/child와 request.stamp, signal.stamp의 완전 일치 및 reason을 검사한다. 실패 입력은 원 attempt/signal/route를 교체하지 않는다. 원 cleanup이 Pending이면 Pending을 유지해 "Err=no work in flight"를 거짓으로 만들지 않는다. terminal인 경우에만 identity 오류를 반환한다.
3. 첫 유효 begin은 그 signal.clone만 캡처한 CompletionWake로 router.retire를 한 번 호출한다. RetirementStart의 handle은 dispatch_error가 있어도 먼저 보존한다. queue 거부는 실제 worker가 시작하지 않은 retryable terminal failure이며 원 owner/request는 router가 유지한다.
4. 같은 전체 stamp+reason의 중복 begin/observe는 원 handle을 읽기만 한다. retire/retry 호출·signal 교체·새 task 생성0이다. 외부 host가 별도로 retry를 운전하지 않도록 이 machine을 **해당 child의 유일한 Meari attempt driver**로 둔다. host shutdown은 coordinator cleanup을 통해 들어오고 child가 없는 factory 실패의 prepared route만 별도 router owner가 정리한다.
5. 더 큰 Meari revocation/attempt는 같은 expected window/child/route에 대해 이전 시도가 실제 terminal retryable failure일 때만 새 retry로 연결한다. stale/same/foreign/in-flight/이미 Complete 상태에 새 cleanup을 시작하지 않는다. 새 signal은 새 attempt에만 귀속된다. FIFO relay가 거부된 새 wake만 제거하고 이미 실행 중인 옛 callback은 원 signal을 유지한다.
6. observe는 same request/reason과 고정 route의 원 RetirementHandle만 사용한다. request가 바뀌었거나 handle을 외부 metadata로 교환하려는 호출은 정상 성공으로 처리하지 않는다. UI에서 worker join/wait·engine shutdown·새 SDK 구성은 없다.

### 관측 대응과 실패 보존

| router 원 관측 | Meari 대응 |
|---|---|
| Pending | Ok(Pending). 이전 typed 실패/receipt는 진단으로 보존하지만 현재 시도를 terminal로 오표시하지 않음. |
| RetryableFailure + 현재 dispatch_error | 원 Arc<ApiError>의 code/message/details/remediation/request ID를 보존하는 Err. 원 handle/route/lease 유지. |
| RetryableFailure + Engine 실패 | 원 typed API error를 보존하는 Err. 자동 retry0. |
| RetryableFailure + Incomplete receipt | 원 ClosedEpisodePlayback/worker flags/진단을 보존하는 Err. boolean false를 없애거나 미래 wake 없는 정상 Pending으로 숨기지 않음. |
| Complete | 원 task actual join 관측만 Ok(Complete). router bool이나 caller receipt로 분기하지 않음. 이것도 실제 surface Destroy/dispatcher host Joined의 증거가 아님. |
| NotStarted/소유권 관측 Internal fault | 원 owner를 보존하고 정상 Complete로 변환0. 관측 불능 상태에서 work in flight가 없다고 거짓 Err를 반환하지 않음. host의 timeout/ForcedIncomplete와 retained diagnostics로 귀결되는 별도 비정상 경계를 source 검토 때 확정해야 함. |

마지막 내부 잠금 fault는 정상 joined EngineFailure/Incomplete와 구분한다. 이를 무기한 정상 Pending이나 성공으로 숨기지 않고 host의 명시 비정상 마감이 필요하다. 현재 Meari NativeChild 포트는 Err를 "cleanup work 없음"으로 정의하므로 확인 불능을 terminal Err로 바꾸는 구현을 승인한 것이 아니다. 실제 exact3 구현 배정 전에 timeout/ForcedIncomplete 인계의 구체 symbol/기존 host report 범위를 대조한다.

### 기존 목적을 보존할 회귀 후보

- 공개 coordinator의 close→actual channel wake→한 번 observe로 shared task 완료. worker join 전에는 Complete/다음 route 준비 불가.
- 같은 attempt 중복·foreign/stale request/signal은 task/attempt/원 callback 교체0. 기존 owner가 Pending이면 Err로 종료했다고 표시0.
- broker/router owner 교환·다른 child/window/surface stamp를 거부하고 원 !Send outer owner/engine owner를 유지.
- dispatch Full/Closed, Engine Err, Incomplete를 각각 원 typed 정보로 유지하고 Meari explicit retry만 같은 RetirementOwner를 재사용. task 생성 수1/engine shutdown 횟수/relay callback 수를 독립 확인.
- attemptA callback 지연 중 attemptB signal의 조기/교차 소비0; B 제출 거부 후 C retry도 A/B/C 신호와 diagnostics가 섞이지 않음.
- prepared 상태 registration 실패/partial child 정리, parent_lost atomic-only, child handle Drop 뒤 router owner 유지, coordinator가 원 lease를 놓기 전에 joined cleanup barrier 확인.
- child Complete와 dispatcher callback/runtime Joined를 별도 인수. OS surface 종료·실제 media/main 성공 주장은0.

기존5의 정확 시험 수를 유지할지 추가 회귀로 나눌지는 source 배정 시 인계한다. 현재는 설계만 기록했고 router4 source/pin·기존 child3는 변경하지 않았다.
