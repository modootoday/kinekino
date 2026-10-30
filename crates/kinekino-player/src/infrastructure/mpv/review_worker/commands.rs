//! Source/instance-bound worker mailbox. Queue acceptance alone is not IPC publication.
use super::*;
use crate::{
    application::{
        playback_control::{ControlReceipt, PlaybackControlPort},
        review_playback::ReviewInputIdentity,
    },
    domain::{
        playback_control::PlaybackControl,
        review_source::{MediaReference, ReviewCase},
    },
};
use std::sync::atomic::{AtomicU8, Ordering};
const QUEUED: u8 = 0;
const SENDING: u8 = 1;
const FINISHED: u8 = 2;
const NOT_SENT: u8 = 3;
pub(super) type Gate = Arc<Mutex<bool>>;
#[derive(Clone)]
struct Binding {
    case: ReviewCase,
    media: MediaReference,
    instance: u64,
}
pub(super) struct CommandRequest {
    binding: Binding,
    action: PlaybackControl,
    expires: Instant,
    phase: Arc<AtomicU8>,
    reply: mpsc::SyncSender<Result<ControlReceipt, ApiError>>,
}
#[derive(Clone)]
pub struct ReviewWorkerControl {
    sender: mpsc::SyncSender<CommandRequest>,
    gate: Gate,
    binding: Binding,
}
impl ReviewWorkerControl {
    pub(super) fn new(
        sender: mpsc::SyncSender<CommandRequest>,
        gate: Gate,
        case: ReviewCase,
        media: MediaReference,
        instance: u64,
    ) -> Result<Self, ApiError> {
        if case.validate().is_err() || media.validate().is_err() || instance == 0 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Exact review source/instance required",
            ));
        }
        Ok(Self {
            sender,
            gate,
            binding: Binding {
                case,
                media,
                instance,
            },
        })
    }
    fn submit_until(
        &self,
        action: PlaybackControl,
        expires: Instant,
    ) -> Result<ControlReceipt, ApiError> {
        action
            .validate()
            .map_err(|message| error(ErrorCode::InvalidArgument, message, false))?;
        if *self
            .gate
            .lock()
            .map_err(|_| error(ErrorCode::Internal, "Review control gate poisoned", false))?
        {
            return Err(error(ErrorCode::NotFound, "Review owner is closing", false));
        }
        let phase = Arc::new(AtomicU8::new(QUEUED));
        let (reply, result) = mpsc::sync_channel(1);
        self.sender
            .try_send(CommandRequest {
                binding: self.binding.clone(),
                action,
                expires,
                phase: phase.clone(),
                reply,
            })
            .map_err(|failure| match failure {
                mpsc::TrySendError::Full(_) => error(
                    ErrorCode::ResourceExhausted,
                    "Review command queue full",
                    false,
                ),
                mpsc::TrySendError::Disconnected(_) => {
                    error(ErrorCode::NotFound, "Review worker ended", false)
                }
            })?;
        match result.recv_timeout(expires.saturating_duration_since(Instant::now())) {
            Ok(result) => result,
            Err(failure) => {
                let _ =
                    phase.compare_exchange(QUEUED, NOT_SENT, Ordering::AcqRel, Ordering::Acquire);
                let uncertain = matches!(phase.load(Ordering::Acquire), SENDING | FINISHED);
                let code = if uncertain {
                    ErrorCode::EngineFailed
                } else if failure == mpsc::RecvTimeoutError::Timeout {
                    ErrorCode::Timeout
                } else {
                    ErrorCode::NotFound
                };
                Err(error(
                    code,
                    "Review command response unavailable; do not automatically retry",
                    uncertain,
                ))
            }
        }
    }
}
impl PlaybackControlPort for ReviewWorkerControl {
    fn submit(&self, action: PlaybackControl) -> Result<ControlReceipt, ApiError> {
        let expires = Instant::now()
            .checked_add(Duration::from_secs(2))
            .ok_or_else(|| {
                error(
                    ErrorCode::ResourceExhausted,
                    "Review command clock exhausted",
                    false,
                )
            })?;
        self.submit_until(action, expires)
    }
}
fn error(code: ErrorCode, message: &str, uncertain: bool) -> ApiError {
    let mut error = ApiError::new(code, message);
    error.details = serde_json::json!({"delivery":if uncertain {"execution_unknown"} else {"not_sent"},"automatic_retry":false});
    error
}
fn not_sent(mut failure: ApiError) -> ApiError {
    failure.details =
        serde_json::json!({"delivery":"not_sent","automatic_retry":false,"cause":failure.details});
    failure
}
fn reject(request: CommandRequest, failure: ApiError) {
    if request
        .phase
        .compare_exchange(QUEUED, NOT_SENT, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        let _ = request.reply.try_send(Err(not_sent(failure)));
    }
}
pub(super) fn process_pending(
    requests: &mpsc::Receiver<CommandRequest>,
    source: &ReviewInputIdentity,
    control: &MpvControlHandle,
    runtime: Deadline,
    gate: &Gate,
    check: &dyn Fn(&PlaybackControl) -> Result<(), ApiError>,
) {
    for request in requests.try_iter().take(control::COMMAND_CAPACITY) {
        if request.phase.load(Ordering::Acquire) != QUEUED {
            continue;
        }
        if request.expires <= Instant::now()
            || runtime.operation_remaining(Instant::now()).is_zero()
        {
            reject(
                request,
                error(
                    ErrorCode::Timeout,
                    "Original review command/runtime deadline expired",
                    false,
                ),
            );
            continue;
        }
        if request.binding.case != source.case
            || request.binding.media != source.media
            || request.binding.instance != control.instance_id()
        {
            reject(
                request,
                error(
                    ErrorCode::RevisionConflict,
                    "Review control source/instance differs",
                    false,
                ),
            );
            continue;
        }
        if gate.lock().map(|closing| *closing).unwrap_or(true) {
            reject(
                request,
                error(ErrorCode::NotFound, "Review owner is closing", false),
            );
            continue;
        }
        if let Err(failure) = check(&request.action) {
            reject(request, failure);
            continue;
        }
        // Serialize final command publication against close. Proof checks happen
        // before this small critical section; the IPC enqueue is nonblocking.
        let closing = match gate.lock() {
            Ok(closing) => closing,
            Err(_) => {
                reject(
                    request,
                    error(ErrorCode::Internal, "Review control gate poisoned", false),
                );
                continue;
            }
        };
        if *closing
            || request.expires <= Instant::now()
            || runtime.operation_remaining(Instant::now()).is_zero()
        {
            reject(
                request,
                error(
                    ErrorCode::NotFound,
                    "Review control closed or expired before publication",
                    false,
                ),
            );
            continue;
        }
        if request
            .phase
            .compare_exchange(QUEUED, SENDING, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            continue;
        }
        let result = control.submit(request.action);
        request.phase.store(
            if result.is_ok() { FINISHED } else { NOT_SENT },
            Ordering::Release,
        );
        let _ = request.reply.try_send(result.map_err(not_sent));
    }
}
#[cfg(test)]
mod tests;
