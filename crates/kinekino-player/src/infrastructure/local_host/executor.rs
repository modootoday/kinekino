use super::{TransportDiagnostics, diagnostics::emit};
use crate::application::{
    api::{ApiError, ApplicationClientHandle, ErrorCode, Request, Response},
    local_host::transport_error,
};
use std::sync::{Arc, Mutex, mpsc};
use tokio::sync::oneshot;
#[cfg(test)]
#[path = "executor_tests.rs"]
mod tests;

type Job = (Request, oneshot::Sender<Response>, &'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DispatchState {
    Idle,
    Accepted,
    Running,
    /// SDK invocation returned or its panic was caught; this is not operation success.
    Completed,
}
struct Admission {
    jobs: Option<mpsc::SyncSender<Job>>,
    phase: DispatchState,
}
/// One admitted synchronous SDK call, including work accepted before close.
#[derive(Clone)]
pub(super) struct Executor {
    admission: Arc<Mutex<Admission>>,
    trace: Option<TransportDiagnostics>,
}
impl Executor {
    #[cfg(test)]
    pub(super) fn start(
        client: ApplicationClientHandle,
    ) -> Result<(Self, std::thread::JoinHandle<()>), ApiError> {
        Self::start_observed(client, None)
    }
    pub(super) fn start_observed(
        client: ApplicationClientHandle,
        trace: Option<TransportDiagnostics>,
    ) -> Result<(Self, std::thread::JoinHandle<()>), ApiError> {
        let (jobs, receiver) = mpsc::sync_channel::<Job>(1);
        let admission = Arc::new(Mutex::new(Admission {
            jobs: Some(jobs),
            phase: DispatchState::Idle,
        }));
        let worker_admission = admission.clone();
        let worker_trace = trace.clone();
        let thread = std::thread::Builder::new()
            .name("player-local-sdk".into())
            .spawn(move || {
                while let Ok((request, reply, class)) = receiver.recv() {
                    // Close stops new admission, not an already accepted SDK call.
                    match worker_admission.lock() {
                        Ok(mut state) => state.phase = DispatchState::Running,
                        Err(_) => {
                            let _ = reply.send(Response::failure(
                                request.context.request_id,
                                transport_error(ErrorCode::Internal, true),
                            ));
                            break;
                        }
                    }
                    emit(&worker_trace, class, "sdk_running", None);
                    emit(&worker_trace, class, "sdk_execute_begin", None);
                    let response = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        client.execute(request.clone())
                    }))
                    .unwrap_or_else(|_| {
                        Response::failure(
                            request.context.request_id,
                            transport_error(ErrorCode::Internal, true),
                        )
                    });
                    emit(&worker_trace, class, "sdk_execute_returned", None);
                    match worker_admission.lock() {
                        Ok(mut state) => state.phase = DispatchState::Completed,
                        Err(_) => break,
                    }
                    let sent = reply.send(response).is_ok();
                    emit(
                        &worker_trace,
                        class,
                        if sent {
                            "sdk_reply_sent"
                        } else {
                            "sdk_reply_receiver_gone"
                        },
                        None,
                    );
                }
            })
            .map_err(|_| transport_error(ErrorCode::Internal, false))?;
        Ok((Self { admission, trace }, thread))
    }
    pub(super) fn close(&self) {
        // Recover only to revoke admission; poison never reopens execution.
        let mut state = self
            .admission
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        state.jobs.take();
    }
    pub(super) fn state(&self) -> Option<DispatchState> {
        self.admission.lock().ok().map(|state| state.phase)
    }
    fn admit(&self, request: Request) -> Result<oneshot::Receiver<Response>, ApiError> {
        let mut state = self
            .admission
            .lock()
            .map_err(|_| rejected(ErrorCode::Internal))?;
        let Some(jobs) = state.jobs.as_ref() else {
            return Err(rejected(ErrorCode::Cancelled));
        };
        if matches!(
            state.phase,
            DispatchState::Accepted | DispatchState::Running
        ) {
            return Err(rejected(ErrorCode::ResourceExhausted));
        }
        let (tx, rx) = oneshot::channel();
        let class = match request.command.as_str() {
            "jobs.get" => "jobs.get",
            "jobs.submit" => "jobs.submit",
            "jobs.cancel" => "jobs.cancel",
            _ => "other",
        };
        jobs.try_send((request, tx, class))
            .map_err(|_| rejected(ErrorCode::EngineFailed))?;
        state.phase = DispatchState::Accepted;
        emit(&self.trace, class, "sdk_admitted", None);
        Ok(rx)
    }
    pub(super) async fn execute(&self, request: Request) -> Response {
        let id = request.context.request_id.clone();
        match self.admit(request) {
            Ok(rx) => rx.await.unwrap_or_else(|_| {
                Response::failure(id, transport_error(ErrorCode::Cancelled, true))
            }),
            Err(error) => {
                emit(&self.trace, "sdk", "sdk_admission_refused", None);
                Response::failure(id, error)
            }
        }
    }
}
fn rejected(code: ErrorCode) -> ApiError {
    let mut error = ApiError::new(code, "Local host request was not dispatched");
    error.details = serde_json::json!({"delivery":"not_dispatched","automatic_retry":false});
    error
}
