---
id: sot:kinekino/sdk-s1-owned-execution
title: SDK S1 owned execution 공유 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# SDK S1 owned execution 공유 계약

화자 편집 후속: 원 PlayerSdk의 subtitles.speakers/edit_speakers에도 같은 control을 연결했다. [화자 계약](../analysis/manual-speaker-annotations.md)의 pre-publication 취소와 post-publication 실제 성공 보존을 적용한다. CLI 원격 ACK·다른 wrapper의 전달·GUI 연결을 완료로 계상하지 않는다.

2026-10-27 후속: [models.verify의 cooperative 실행 취소](../analysis/model-verification-execution-control.md)를 연결했다. worker는 원 operation의 control을 ApplicationClient.execute_controlled로 전달하며 기본 client는 기존 execute를 유지한다. 참여 leaf가 반환한 실제 terminal만 사용하고 cancel Requested 자체를 완료로 계상하지 않는다. 다른 명령·원격 취소 ACK·hard preemption으로 확대하지 않는다.

후속 typed close의 실제 재현·검증 입력과 한계는 [typed close 실행 정본](../../../verification/application/sdk-close-progress-2026-10-20.md)에 분리한다.

2026-10-20 S1 SDK/UI 공유 계약이다. [HP01~HP10](../../../../plans/player/historical-import-pending-integration.md)을 따른다. 구현 소스와 개별 시험 근거는 구별하며 최신 변경 전체의 통과를 이 문서만으로 주장하지 않는다. SDK application은 Meari와 UI 타입을 import하지 않고 기존 ApplicationClientHandle 하나를 사용한다. 새 SDK/store·dependency·persistent job/attempt를 생성하지 않는다.

## SDK Rust port

```rust
// src/application/sdk_execution
pub struct ExecutionBudget {
    pub max_operations: usize,
    pub max_request_bytes: usize,
    pub max_total_request_bytes: usize,
    pub max_terminal_bytes: usize,
    pub max_total_terminal_bytes: usize,
}
// private owner identity + checked sequence; Clone + Eq + Debug, no Deserialize.
pub struct OperationId { /* private */ }
pub enum CancelDisposition {
    QueuedCancelled,
    Requested,
    AlreadyAcknowledged,
}
pub enum ExecutionSnapshot {
    Queued,
    Running { cancellation_requested: bool },
    Terminal(std::sync::Arc<ExecutionTerminal>),
    Acknowledged,
}
pub struct ExecutionTerminal {
    pub request: std::sync::Arc<crate::application::api::Request>,
    pub response: std::sync::Arc<crate::application::api::Response>,
    pub cancellation_requested: bool,
    pub queued_cancelled: bool,
    pub measurement: EncodingMeasurement,
}
pub enum EncodingMeasurement { Exact(usize), LowerBound(usize), SerializationFailed }
pub struct AckReceipt { pub already_acknowledged: bool }
pub struct ExecutionCloseReceipt {
    pub worker_stopped: bool,
    pub unacknowledged: usize,
}
pub trait CompletionNotifier: Send + Sync {
    // Wake is an observation aid, not ACK. Failure never consumes terminal.
    fn notify(&self) -> Result<(), crate::application::api::ApiError>;
}
pub struct ExecutionOwner { /* owned worker + shared state */ }
pub struct ExecutionHandle { /* Clone, same owner identity */ }
// ExecutionOwner/ExecutionHandle public signatures:
// ExecutionOwner::start(client, budget, notifier) -> Result<Self, ApiError>
// ExecutionOwner::handle(&self) -> ExecutionHandle
// ExecutionHandle::submit(&self, Request) -> Result<OperationId, ApiError>
// ExecutionHandle::peek(&self, &OperationId) -> Result<ExecutionSnapshot, ApiError>
// ExecutionHandle::acknowledge(&self, &OperationId) -> Result<AckReceipt, ApiError>
// ExecutionHandle::cancel(&self, &OperationId) -> Result<CancelDisposition, ApiError>
// ExecutionHandle::close(&self) -> Result<ExecutionCloseReceipt, ApiError>
```

submit은 원 Request.validate와 byte/count admission을 SDK 실행 전에 수행한다. 모든 terminal은 ACK까지 operation slot과 예산을 점유한다. queued 취소는 claim과 같은 gate에서 제거한 뒤 canonical Cancelled response를 보존한다. 실행 중/terminal-ready 취소는 Requested이며 실제 SDK Response를 바꾸지 않는다. queued 취소도 UI registry terminal 처리 전에는 raw provider가 Requested를 반환하는 보수적 경로를 사용할 수 있다. AlreadyAcknowledged는 UI terminal 수용/SDK ACK 이후의 제한된 기억에만 사용한다.

peek는 비소비이며 Arc로 동일 terminal을 읽는다. acknowledge는 UI/registry typed terminal 수용 후 호출하며 실패하면 accepted-but-unacknowledged를 보존하고 registry에 재전달하지 않는다. 기억 소거/foreign owner는 Stale에 해당하는 오류이고 새 실행 신호가 아니다.

close는 Closing을 표시하고 신규 admission을 거부한다. 미실행 queue는 취소 결과로 보존하고 실행 중 작업은 실제 결과를 기다린다. terminal ACK·worker/owned resource 종료 전에는 Busy/NeedsInput 오류와 owner를 남긴다. worker 종료를 관측한 경우에만 join하고 close 성공을 반환한다. Drop은 명시 close 영수증이 아니며 caller의 retained handle로 미완료 상태를 관측/정리할 수 있어야 한다.

후속 typed close 입력은 ExecutionOwner/ExecutionHandle::poll_close(&self) -> Result<ExecutionCloseProgress, ApiError>다. 호출은 closing을 개시하고 기존 close와 같은 전이·queue 취소·is_finished 확인 후 join을 사용한다. `WorkerCloseState::{ExitPending { loop_finished }, Joined}`와 `ExecutionCloseProgress::{TerminalAckPending { unacknowledged, worker }, WorkerExitPending { loop_finished }, Faulted { kind, error: Arc<ApiError>, unacknowledged, worker }, Closed(ExecutionCloseReceipt)}`로 구분한다. Faulted.kind는 `ExecutionCloseFaultKind::{Terminal, WorkerJoin}`이며 authority/상태 mutex 오류는 Result::Err다. join panic도 typed Faulted로 보존한다. 최초 제안의 join panic Result::Err 표현은 채택하지 않았으며 legacy close만 같은 canonical Err를 반환한다.

loop_finished는 worker 루프의 논리 종료 표시다. 마지막 notifier가 아직 실행 중일 수 있어 실제 thread 반환이나 join ACK가 아니다. Joined/Closed만 실제 성공 join을 뜻한다. notifier를 barrier로 잡은 현재 동작 재현과 typed 후속 시험의 근거는 별도 실행 정본에 기록한다. 실제 host는 WorkerExitPending에 한해 보존된 owner의 미래 bounded 재시도를 배선해야 하며, SDK가 추가 observer thread·즉시 Busy wake 반복·UI blocking join을 만들지 않는다. TerminalAckPending/Faulted를 자동 timer로 무한 재시도하거나 ACK 없이 폐기하지 않는다. 이 host scheduler 배선은 별도 후속이며 typed API 추가만으로 OS close 자동 진행을 완료로 표시하지 않는다.

Faulted의 같은 diagnostic Arc는 반복 관측되며 원 terminal은 retained_operations/peek로 비소비 조회한다. join 실패는 handle을 take한 뒤에도 별도로 보존해 Closed로 바뀌지 않는다. terminal fault는 명시 ACK와 성공 join이 끝나면 Closed가 될 수 있으나, join fault는 성공 join으로 위조하지 않는다. 기존 close는 정상 receipt, 대기 NeedsInput와 worker_stopped/worker_returned/unacknowledged/closing details, join 실패의 canonical 오류를 유지한다. 이 Rust 전용 관측 타입에는 serde wire를 추가하지 않는다.

ExecutionHandle::attach는 미완료 operation이 없는 상태에서 하나의 ExecutionAttachment만 발급한다. attachment의 submit/peek/ACK/cancel은 동일 상태 mutex 안에서 현재 authority와 일치하는지 검사한다. release는 quiescent 상태에서 연결 권한만 해제하며 worker를 종료하지 않는다. 해제된 attachment는 연결이 없는 동안에도 다시 제출할 수 없고, 새 attachment의 결과를 읽거나 취소·ACK할 수 없다. 이전 release/Drop의 반복도 후속 연결을 해제하지 않는다. 일반 handle의 peek/retained_operations는 신뢰된 owner 진단이며 renderer 권한이 아니다. 일반 submit/ACK/cancel/close는 attachment가 활성인 동안 거부한다.

문서 unbind는 registry pending 수용·ACK와 등록 해제 및 attachment release를 마친다. mount 실패 후 재시도는 같은 ExecutionOwner/SDK/worker에 새 문서 attachment를 연결할 수 있다. 최종 parent 종료는 unbind 뒤 별도로 worker close/join 영수증을 받아야 한다. unbind 성공을 worker 종료로 세지 않는다.

InitialProposalInspection의 초기 inspect는 실제 scope 생성 전 같은 worker에서 실행한다. startup owner가 검증된 immutable typed data와 원 terminal을 보존한 뒤 ACK하고, 그 다음 actual scope를 만들고 같은 ExecutionOwner를 document parent에 넘긴다. 이 ACK는 초기 데이터 수용이며 mount/render ACK가 아니다. typed 검증 또는 ACK가 실패하면 mount하거나 SDK 요청을 다시 제출하지 않는다.

request/terminal count와 encoded bytes 예산은 constructor에서 검증하고 admission 시 terminal 용량을 예약한다. 원 Request 보존과 worker 호출 복사를 위해 request bytes는2배로 예약한다. terminal은 Exact(bytes) 또는 초과를 발견한 LowerBound(limit+1), 크기 초과와 구별되는 SerializationFailed로 측정한다. lower bound를 실제 전체 byte 수로 기록하지 않는다. SDK가 예약 크기보다 큰 Response를 돌려주거나 인코딩이 실패하면 정상 terminal으로 조용히 자르거나 원 결과를 버리지 않는다. fault 원 terminal을 보존하고 신규 admission/추가 SDK claim을 멈춘다. 이는 예산 내 정상 처리 성공이 아니다. 기존 동기 SDK 내부 할당까지 hard memory limit을 보장한다고 주장하지 않는다. overflow 소유/정리와 typed codec 거부는 별도 fault 시험이다.

## UI generic codec와의 경계

UI 패키지의 공유 port는 root Request/Response/OperationId를 직접 import하지 않는다.

```rust
pub trait AsyncSdkPort<I, O> {
    fn start(&mut self, ticket: &PluginTicket, input: I) -> Result<(), PluginError>;
    fn peek_terminal(&mut self, ticket: &PluginTicket)
        -> Result<Option<std::sync::Arc<Result<O, ApiError>>>, PluginError>;
    fn acknowledge(&mut self, ticket: &PluginTicket) -> Result<(), PluginError>;
    fn cancel(&mut self, ticket: &PluginTicket) -> Result<CancelReceipt, PluginError>;
    fn shutdown(&mut self) -> Result<(), PluginError>;
}
```

교차 제안을 조율한 최종 port는 위 actual-ticket AsyncSdkPort<I,O>다. generic typed binding/terminal pump는 UI 소유이며 actual ticket/accepted-but-unacked 상태를 추적한다. SDK assembly의 port 구현체가 actual ticket→OperationId와 canonical DTO/Response 변환을 소유한다. 동일 결과 변환은 캐시해 peek마다 업무를 재호출하지 않는다. Arc<Result>는 non-Clone ApiError/O를 소비하지 않고 읽는다. SDK는 package의 root 역의존을 만들지 않는다. 중간 K/AsyncTerminal 제안은 채택하지 않았다.

QueuedCancelled 이후 SDK peek는 queued_cancelled=true인 canonical Cancelled response를 보존한다. assembly는 원 ApiError(code=Cancelled)를 Err로 유지하고 generic codec은 typed Failed details로 Core에 전달한다. 이것은 실제 queue 제거 확인으로 생긴 취소이며 실행 중 요청을 Cancelled로 위조하는 것이 아니다. provider cancel은 queued 취소도 Requested를 반환하고 실제 terminal accept_event 성공 뒤 ACK한다. terminal-ready는 ACK 전 AlreadyTerminal이 아니다.

invalid typed reply/schema는 S1에서 자동 오류 대체하지 않는 fail-closed 복구 대기로 정한다. Core pending과 canonical 원 Response/실제 effect receipt를 ACK 전 무기한 보존하고 새 SDK 실행·Cancelled/무변경 전이를 하지 않는다. shutdown은 Busy이며 명시 복구가 필요함을 보고한다. 향후 canonical Internal failure로 정리하려면 host의 명시 복구 결정과 원 terminal/effect receipt 보존 위치·아카이브 성공을 함께 계약해야 한다. 이 복구/아카이브 구현 없이 정상 close 완료를 주장하지 않는다.

SDK canonical terminal bytes와 UI typed-view/PluginPayload codec 복사는 별도 예산에 포함한다. codec은 bounded encode/크기 측정 후 admission하고 초과/변환 실패 시 원 terminal을 ACK하지 않는다. S1 constructor의 caller 예산과 Core payload 한도를 명시하며, codec 복사나 숨은 무제한 terminal queue로 예산을 우회하지 않는다.

대상은 create/inspect/edit/accept/import 다섯 가지다. 기존 Immediate Native4 경로는 명시 호환으로 유지한다. 강한 in-flight 취소/게시 reservation(S2), project UI 소비와 actual OS wake(S3)는 미완료 범위다. notifier와 headless pump를 실제 GUI/wake 성공으로 세지 않는다.
