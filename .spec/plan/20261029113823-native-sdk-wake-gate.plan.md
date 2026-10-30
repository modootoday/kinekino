---
id: plan:kinekino/20261029113823
title: 실제 SDK notifier와 Native wake 소비 gate
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 SDK notifier와 Native wake 소비 gate

2026-10-20 읽기 검토·후속 인수 준비. 제품 OS 조립 구현/실행 성공이 아니다. UI source16=5b6515510b154805e5d8d199fdf778f3d439486a0fd6f7a15113def151d17a2f 및 Native=8cf2ab2f6eb431a6138288f6475be73c7506fecb096cc3dde481d8df4fed4739를 기준으로 한다. SDK typed close 최종 입력 E1959DC775A0EBA92619F83AB26EB48672367298CD5441D80D66B265DA298BF5를 담당 인계·실제 source 재hash와 [SDK 검증 정본](../../../sots/verification/application/sdk-close-progress-2026-10-20.md)으로 확인했다. 이 입력에서 기존 UI83 호환 회귀는409776ms/exit0·83pass/0fail/0ignore다. UI가 새 poll_close를 배선하거나 Native startup/scheduling을 지원한다는 증거는 아니다. [직전 headless83 근거](../../../sots/verification/presentation/pending-document-sdk-2026-10-20.md), [단일 창 전체 범위](native-window-minimal-vertical.md), [중앙 조율](../current-coordination.md).

범용 Native API의 관측·계약 정본은 [Meari 경계 검토](../../../../packages/meari/.spec/evidence/reviews/plugin-pending-sdk-boundary-2026-10-20.md)다. 이 문서는 제품 notifier/startup/parent 조립과 제품 소비 인수기준만 소유한다. 아래 port 목록은 라이브러리 새 API 정본이나 구현 배정이 아니라 담당에게 전달하는 제품 요구다.

Meari의 [ADR0015 scheduling/startup 정확 제안](../../../../packages/meari/.spec/adrs/0015-ui-thread-document-host-port.md)은 PM이 다음 Native 구현 방향으로 인수했다. 최신 엄격 경로는 NativeLoopOwner가 pinned loop를 내부 소유하고 NativeStartupController/Context가 Pending(HostRetry)·opaque NativeAttachment·Stop으로 인계하는 제안이다. 초기 borrowed-loop startup/외부 raw driver 인계안 대신 이 마지막 수정 후보를 참조한다. HostRetry/opt-in schedule limits/default retry hint를 포함한 모든 새 이름은 아직 현재 지원 API가 아니다. SDK 최종 gate→Native 포트 구현/실기 인수→제품 OS 조립 순서를 지킨다.

## 관측 사실

| 소유 경계 | 실제 source에서 확인한 내용 | 아직 증명하지 못한 것 |
|---|---|---|
| 초기 SDK 준비 | initial_panel.rs는 실제 document scope 전에 plain inspect·immutable 준비·ACK를 수행한다. | 실제 NativeWindowWake가 생기기 전 완료 알림을 OS 소비에 연결하는 제품 notifier |
| Native wake | window/wake.rs의 wake()는 COMPLETE bit를 coalesce하고, proxy_wake_up는 take()로 bit를 소비한다. COMPLETE는 Closing/CleanupFailed에서도 close를 재관측하지만 child LIFECYCLE은 Closing에서만 진행한다. UI 객체/registry가 worker로 이동하지 않는다. | host Busy 후 동일 completion의 재관측 보장. child 신호를 SDK 알림의 대체로 사용하지 않음 |
| UI guard/host | native_host/window_port.rs는 runtime borrow 실패를 문자열 오류로 반환한다. NativeHostRuntime.after_dispatch도 guard 진입 실패를 반환할 수 있다. window.rs는 해당 오류를 보고하지만 완료 비트의 재보존 경로는 보이지 않는다. | 문자열 분기 없이 transient Busy와 영구 오류를 구별하는 공개 계약 |
| host close | CloseDecision::Pending은 checkpoint와 Closing을 보존한다. RetryableFailure는 canonical host error와 CleanupFailed를 보존한다. | Pending에 대한 자동 재시도 시각/시한 |
| parent close | InitialExecutionParent는 unbind 뒤 owner.close를 호출하고 canonical ApiError를 유지하지만 실패는 PluginError::Busy로 전달한다. | typed worker join-wait와 실제 cleanup 실패의 UI 구별 |
| deadline | window.rs.about_to_wait의 deadline은 WindowLifecycle child cleanup에서 온다. NativeWindowHost/HostEffects/CloseDecision에는 host join-wait deadline 전달이 없다. | host와 child deadline의 합성·진짜 오류에서 timer 중단 |

이는 소스 검토이며 Busy 유실·OS hang의 실행 재현이 아니다. 기존 headless 시험의 수동 Completion pump/yield close retry를 실제 OS 자동 진행 근거로 바꾸지 않는다. SDK worker_finished는 thread join receipt가 아니다.

## 책임과 필요한 최소 범용 port

SDK는 typed close progress와 실제 join 완료 receipt를 소유한다. 인수한 poll_close의 WorkerExitPending만 future bounded retry 대상으로 소비하고, TerminalAckPending은 실제 operation·명시 수용을 유지한다. Faulted의 Terminal/WorkerJoin 종류·원 diagnostic Arc와 authority/poison outer Err를 실패로 보존한다. loop_finished는 join ACK가 아니며 Closed의 실제 receipt만 최종 완료다. 이 제품 분류는 SDK 담당과 대조했고, 실제 코드는 Native 최종 port/PM 배정 뒤 연결한다.

제품 assembly/infrastructure는 SDK CompletionNotifier와 실제 Native wake handle을 연결한다. startup 전에는 아직 실제 Native document/wake가 없으므로 early completion을 유한 dirty 상태로 보존하고, 신뢰한 같은 owner의 실제 wake가 bind된 뒤 첫 관측을 보장해야 한다. ACK나 SDK 업무를 notifier에서 실행하지 않는다. dirty clear와 bind/notify 경쟁은 lost wake가 없도록 한 책임에서 다룬다. 오래된 window/owner의 wake를 새 문서에 이식하거나 직렬화된 ID로 권한을 복원하지 않는다.

dirty 보존만으로 pre-window startup event loop가 자동 깨어나는 것은 아니다. Meari 담당이 현재 공개 startup port 부재를 확인했고 PM은 범용 pre-document startup wake port 제안·보완을 우선 배정했다. [제품 결정·대안·비용](../../../adrs/player/presentation/native-async-startup-wake.md)을 따른다. 공개 API와 실제 인계/실패 회수는 아직 미구현·미검증이다. initial inspect를 UI thread에서 block하거나 가짜 ProbeScope·새 winit 버전/다른 JS runtime·임시 loading-root로 우회하지 않는다. 최종 backend/런처 교체 결정으로 확대하지 않는다.

UI parent/bridge는 SDK typed progress를 소비해 최종 join-wait만 retained Pending+bounded future retry로 투영한다. 진짜 API/업무 실패를 timer 대상으로 바꾸지 않는다. session/registration/worker close checkpoint와 canonical error를 보존하고 새 작업·inspect·mutation을 재실행하지 않는다. 실제 terminal ACK 대기는 terminal 알림/명시 수용을 사용하며 timer로 자동 수용하지 않는다.

Native 라이브러리는 SDK를 모르는 범용 재관측 port를 소유해야 한다. 현재 공개 API만으로 host retry deadline을 driver에 전달할 수 있다는 근거는 없다. 필요한 기능은 다음과 같으며 타입 이름은 Meari 담당과 확정한다.

- host의 다음 재관측 시각을 typed하게 얻거나 Pending과 함께 전달하고 실제 child deadline·렌더러 event-loop deadline과 최소값으로 합성한다.
- 서로 다른 clock origin의 duration을 그대로 비교하지 않는다. Native monotonic origin을 기준으로 absolute Instant로 정규화하며 외부 ControlFlow::Poll 및 더 이른 WaitUntil을 보존한다.
- due 이전에는 callback/SDK 효과를 재실행하지 않고, due에서 같은 checkpoint를 한 번 관측한다. 계속 wait면 미래 deadline만 재설정하고 Complete/오류/forced/owner 폐기 시 제거한다.
- event/runtime/document의 transient Busy로 관측하지 못한 Completion/close 요청을 유한 dirty 상태로 보존한다. 즉시 wake 연쇄가 아닌 guard 해제 후 kick 또는 bounded retry 계약으로 재개한다.
- Busy와 영구 오류를 구조화하여 구별한다. root SDK 타입·JSON details parser·제품 command/저장 정책을 Native에 넣지 않는다.

HostEffects.redraw/committed는 성공한 after_dispatch의 표현이고 close deadline의 대체가 아니다. CloseDecision::Pending만 추가 사용하는 것으로 실제 자동 wake를 구현했다고 표시하지 않는다. 공용 trait 확장이 필요하면 기존 Immediate/Standalone/Child host에 source 호환 default와 명시 동작을 제공하고 Meari 소유 검증 후 제품 consumer에서 재검증한다.

## 후속 실행 인수 기준

| ID | 정확 자극 | 필요한 근거 |
|---|---|---|
| NW01 | 초기 inspect가 실제 Native wake bind 전에 완료 | same raw/typed result·SDK execute1·ACK 전 mount0·bind 후 첫 관측·숨겨진 첫 항목0 |
| NW02 | bind 직전/직후와 dirty clear 사이 worker notify 경쟁 | deterministic barrier/channel·lost wake0·중복 SDK 실행0·유한 wake coalescing |
| NW03 | Completion 소비 시 runtime/document guard Busy | same owner retained dirty·guard 해제 후 실제 kick·callback/terminal 정확1회·SDK mutation replay0 |
| NW04 | SDK 마지막 notifier를 barrier로 막고 close 관측 | typed join-wait→Pending·UI 비차단·bounded future deadline·마지막 notify를 join ACK로 오인0 |
| NW05 | notifier release 후 추가 외부 입력/SDK 알림 없음 | 실제 OS timer에서 재관측→정확1 join·unbind1·child/host/parent 정상 close 순서·owned process/window[] |
| NW06 | terminal ACK 대기/정상 업무 오류/진짜 cleanup 실패 | canonical 오류 보존·자동 ACK/업무 재실행0·join-only timer와 구별·실패 시 원 checkpoint 유지 |
| NW07 | 독립 deadline fake scheduler 또는 Open의 child release+host deferred 관측, 별도로 정상 child-first close | 독립 상태의 deadline min 합성·외부 Poll/더 이른 WaitUntil 보존. 정상 close는 child barrier 완료→host join deadline 순차 인계. 시험을 위해 SDK join을 child 이전에 강제로 시작하지 않음 |
| NW08 | partial attach/forced destroy/오래된 wake/new owner | 원 owner 회수·foreign/stale 거부·ForcedIncomplete 보존·정상 Complete 위조0 |

최초 단계는 headless fake monotonic clock/실제 SDK helper·barrier로 결정적 경계를 확인한다. 이후 중앙 슬롯에서 실제 OS single-window gate를 따로 배정한다. actual user input/IME/a11y/시각과 SDK notifier·timer 진행을 구별한다. 기존83을 새 SDK/Native pin의 통과로 상속하지 않는다. 기존 전체13화면과 모든 하위탭·기능 범위는 유지한다.

외부 consumer 인수 전에 두 공개 경계를 추가 확인한다. 첫째, 제품 infrastructure가 별도 winit 버전 없이 같은 loop에서 startup→driver를 인계할 수 있어야 한다. 최신 엄격 제안은 OS ApplicationHandler를 제품에 강요하지 않고 owner-runner/controller만 사용하며 legacy embedding에만 loop_api facade를 둔다. 기존 document 이후 show_with_runtime convenience만으로 이 인수를 증명하지 않는다. 둘째, 문서0인 초기 canonical failure/cancel/준비 실패의 worker join-wait도 같은 limits/clock에서 Pending(After)로 관측하고, canonical 오류는 controller가 유지한 채 cleanup 완료 뒤 Stop/원오류를 전달해야 한다. NativeHostRuntime 이후 timer만으로 pre-document 종료를 보장하지 않는다. 두 공백은 최신 수정 후보에 반영됐으나 실제 구현/external compile/단일 OS loop·문서0 barrier 증거는 아직 없다.

## 스킬 적용과 자원 근거

modoo.today의 `modootoday/build-concurrency-guard/concurrency-policy`를 `mcp__modoo_today_skills__skill_call`로2026-10-20 본문 전체 읽었다. 적용: 단일 parent fan-out와 cargo task/thread 한도를 별도로 기록하고, 느린 wall time만으로 CPU/lock/AV 원인을 단정하지 않으며, 호스트 여유와 process 예산을 구별한다. 직전 compile-first의 실제 낮은 CPU·I/O 증가를 원인 미확정으로 기록한 판단과 후속 중앙 단일 heavy 배정에 적용한다. cap 비교 벤치마크는 수행하지 않았으므로 affinity3가 최적이라는 주장은 없다. 실행 규칙은 [기존 자원 정본](../resource-budget.md)을 유지하며 새 프로파일·우선순위·시스템 변경을 도입하지 않는다.

현재 source/manifest/cargo/GUI 변경0. 문서·조율 작업은 SDK의 단일 heavy 구현 슬롯과 병행하며, 실제 UI 수리는 SDK 최종 API/pin과 Meari 범용 port를 인수한 뒤 배정한다.
