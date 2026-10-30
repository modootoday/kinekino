//! Local review engine worker. Its original non-Send source session never leaves this thread.
use super::{MpvControlHandle, control, guarded_command::HostScope, ipc};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        native_playback::NativePlayback,
    },
    infrastructure::{
        immutable_storage_paths, model_review::GuardedAdmittedReviewMedia,
        review_runtime::ReviewJobReservation,
    },
};
use kinekino_process::{
    application::supervisor::{ProcessFamily, Supervisor, SystemClock},
    domain::{deadline::Deadline, observation::FamilyObservation},
    infrastructure::{
        evidence_files::PreparedEvidence,
        family::{SpawnSpec, spawn_prepared},
    },
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};

pub struct ReviewWorkerLimits {
    pub total: Duration,
    pub cleanup: Duration,
    pub raw_output_bytes: u64,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct ReviewWorkerSnapshot {
    pub source: Option<crate::application::review_playback::ReviewInputIdentity>,
    pub process_created: bool,
    pub pid: Option<u32>,
    pub creation_filetime: Option<u64>,
    pub engine_instance_id: Option<u64>,
    pub playback: NativePlayback,
    pub closing: bool,
    pub physical_closed: bool,
    pub child_exit: Option<u32>,
    pub family_empty: bool,
    pub forced: bool,
    pub ipc_exited: bool,
    pub first_error: Option<String>,
    pub first_error_code: Option<ErrorCode>,
    owner_thread_ended: bool,
    #[serde(skip)]
    published_at: Instant,
}
type State = Arc<Mutex<ReviewWorkerSnapshot>>;
/// Dropping the handle requests close; the original worker retains custody until physical exit.
pub struct ReviewPlaybackWorker {
    close: mpsc::SyncSender<()>,
    state: State,
    control: Arc<Mutex<Option<MpvControlHandle>>>,
    worker: thread::JoinHandle<()>,
    commands: mpsc::SyncSender<commands::CommandRequest>,
    gate: commands::Gate,
}
/// Consumed on the original owner thread. The caller can retain non-Send
/// evidence leases in its stack for the entire synchronous run, including cleanup.
pub struct ReviewWorkerLaunch {
    root: PathBuf,
    deadline: Deadline,
    limits: ReviewWorkerLimits,
    close: mpsc::Receiver<()>,
    state: State,
    controls: Arc<Mutex<Option<MpvControlHandle>>>,
    commands: mpsc::Receiver<commands::CommandRequest>,
    gate: commands::Gate,
}
impl ReviewWorkerLaunch {
    pub fn run(
        self,
        input: GuardedAdmittedReviewMedia<ReviewJobReservation>,
        check_custody: &dyn Fn() -> Result<(), ApiError>,
    ) -> Result<(), ApiError> {
        self.run_with_control_guard(input, check_custody, &|_| Ok(()))
    }
    pub fn run_with_control_guard(
        self,
        input: GuardedAdmittedReviewMedia<ReviewJobReservation>,
        check_custody: &dyn Fn() -> Result<(), ApiError>,
        check_control: &dyn Fn(
            &crate::domain::playback_control::PlaybackControl,
        ) -> Result<(), ApiError>,
    ) -> Result<(), ApiError> {
        self.run_with_policy(
            input,
            check_custody,
            &|intent, _| check_control(intent),
            &|_| Ok(false),
        )
    }
    pub fn run_with_policy(
        self,
        input: GuardedAdmittedReviewMedia<ReviewJobReservation>,
        check_custody: &dyn Fn() -> Result<(), ApiError>,
        check_control: &dyn Fn(
            &crate::domain::playback_control::PlaybackControl,
            &NativePlayback,
        ) -> Result<(), ApiError>,
        should_pause: &dyn Fn(&NativePlayback) -> Result<bool, ApiError>,
    ) -> Result<(), ApiError> {
        run(
            || Ok(input),
            self,
            check_custody,
            check_control,
            should_pause,
        )
    }
}
impl ReviewPlaybackWorker {
    pub fn spawn(
        prepare: impl FnOnce() -> Result<GuardedAdmittedReviewMedia<ReviewJobReservation>, ApiError>
        + Send
        + 'static,
        evidence_root: PathBuf,
        limits: ReviewWorkerLimits,
    ) -> Result<Self, ApiError> {
        Self::spawn_scoped(
            move |launch| launch.run(prepare()?, &|| Ok(())),
            evidence_root,
            limits,
        )
    }
    /// Trusted assembly creates and keeps its original proof on this thread;
    /// no unsafe Send or deserialized capability crosses the boundary.
    pub fn spawn_scoped(
        prepare: impl FnOnce(ReviewWorkerLaunch) -> Result<(), ApiError> + Send + 'static,
        evidence_root: PathBuf,
        limits: ReviewWorkerLimits,
    ) -> Result<Self, ApiError> {
        let deadline =
            Deadline::new(Instant::now(), limits.total, limits.cleanup).map_err(io_error)?;
        if limits.raw_output_bytes == 0 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Positive review raw output bound required",
            ));
        }
        let (close, receiver) = mpsc::sync_channel(1);
        let (commands, command_requests) = mpsc::sync_channel(control::COMMAND_CAPACITY);
        let gate = Arc::new(Mutex::new(false));
        let worker_gate = gate.clone();
        let state = Arc::new(Mutex::new(ReviewWorkerSnapshot {
            source: None,
            process_created: false,
            pid: None,
            creation_filetime: None,
            engine_instance_id: None,
            playback: NativePlayback::default(),
            closing: false,
            physical_closed: false,
            child_exit: None,
            family_empty: false,
            forced: false,
            ipc_exited: false,
            first_error: None,
            first_error_code: None,
            owner_thread_ended: false,
            published_at: Instant::now(),
        }));
        let controls = Arc::new(Mutex::new(None));
        let (worker_state, worker_control) = (state.clone(), controls.clone());
        let worker = thread::Builder::new()
            .name("player-review-owner".into())
            .spawn(move || {
                let _publication = PublicationLifetime(worker_state.clone());
                if let Err(error) = prepare(ReviewWorkerLaunch {
                    root: evidence_root,
                    deadline,
                    limits,
                    close: receiver,
                    state: worker_state.clone(),
                    controls: worker_control,
                    commands: command_requests,
                    gate: worker_gate,
                }) {
                    update(&worker_state, |s| {
                        let first = s.first_error.is_none();
                        first_error(s, format!("{error:?}"));
                        if first {
                            s.first_error_code = Some(error.code);
                        }
                        if !s.process_created {
                            s.physical_closed = true;
                            s.ipc_exited = true;
                        }
                    });
                }
            })
            .map_err(io_error)?;
        Ok(Self {
            close,
            state,
            control: controls,
            worker,
            commands,
            gate,
        })
    }
    pub fn snapshot(&self) -> Result<ReviewWorkerSnapshot, ApiError> {
        self.state
            .lock()
            .map(|s| s.clone())
            .map_err(|_| ApiError::new(ErrorCode::Internal, "Review worker state poisoned"))
    }
    pub fn status_reader(
        &self,
        case: crate::domain::review_source::ReviewCase,
        media: crate::domain::review_source::MediaReference,
        engine_instance_id: u64,
    ) -> Result<ReviewWorkerStatusReader, ApiError> {
        ReviewWorkerStatusReader::new(&self.state, case, media, engine_instance_id)
    }
    pub fn control(&self) -> Result<MpvControlHandle, ApiError> {
        self.control
            .lock()
            .map_err(|_| ApiError::new(ErrorCode::Internal, "Review control state poisoned"))?
            .clone()
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::NeedsInput,
                    "Review engine control is not connected",
                )
            })
    }
    pub fn close(&self) {
        if let Ok(mut closing) = self.gate.lock() {
            *closing = true;
        }
        let _ = self.close.try_send(());
    }
    pub fn scoped_control(
        &self,
        case: crate::domain::review_source::ReviewCase,
        media: crate::domain::review_source::MediaReference,
        engine_instance_id: u64,
    ) -> Result<ReviewWorkerControl, ApiError> {
        ReviewWorkerControl::new(
            self.commands.clone(),
            self.gate.clone(),
            case,
            media,
            engine_instance_id,
        )
    }
    pub fn is_finished(&self) -> bool {
        self.worker.is_finished()
    }
}
impl Drop for ReviewPlaybackWorker {
    fn drop(&mut self) {
        self.close();
    }
}
fn update(state: &State, f: impl FnOnce(&mut ReviewWorkerSnapshot)) {
    if let Ok(mut state) = state.lock() {
        f(&mut state);
        state.published_at = Instant::now();
    }
}
struct PublicationLifetime(State);
impl Drop for PublicationLifetime {
    fn drop(&mut self) {
        update(&self.0, |s| {
            s.owner_thread_ended = true;
            if thread::panicking() {
                first_error(
                    s,
                    "Review owner thread panicked; physical exit is not inferred".into(),
                );
            }
        });
    }
}
fn first_error(state: &mut ReviewWorkerSnapshot, error: String) {
    if state.first_error.is_none() {
        state.first_error = Some(error);
        state.first_error_code = Some(ErrorCode::EngineFailed);
    }
}
fn io_error(error: std::io::Error) -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        format!("Review engine operation failed: {error}"),
    )
}

mod arguments;
mod boundary_pause;
mod commands;
mod launch_record;
mod run;
mod status;
pub use commands::ReviewWorkerControl;
use run::run;
pub use status::ReviewWorkerStatusReader;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prepare_failure_preserves_first_error_and_never_creates_child() {
        let worker = ReviewPlaybackWorker::spawn(
            || {
                Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "original preimage changed",
                ))
            },
            PathBuf::from("unused-before-preparation"),
            ReviewWorkerLimits {
                total: Duration::from_secs(2),
                cleanup: Duration::from_millis(100),
                raw_output_bytes: 1024,
            },
        )
        .unwrap();
        let cutoff = Instant::now() + Duration::from_secs(2);
        while !worker.is_finished() && Instant::now() < cutoff {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(worker.is_finished());
        let state = worker.snapshot().unwrap();
        assert!(!state.process_created && state.physical_closed && state.ipc_exited);
        assert_eq!(state.first_error_code, Some(ErrorCode::RevisionConflict));
        assert!(
            state
                .first_error
                .unwrap()
                .contains("original preimage changed")
        );
        assert!(matches!(worker.control(), Err(e) if e.code == ErrorCode::NeedsInput));
    }
}
