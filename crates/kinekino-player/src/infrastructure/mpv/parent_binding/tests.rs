use super::*;
use crate::infrastructure::{
    mpv::{ParentCleanupObservation, ParentCleanupTask},
    thread_completion::{DispatcherHost, DispatcherShutdown},
};
use crate::{
    application::{
        catalog::RawArtifact,
        playback_control::{ControlCompletion, ControlReceipt},
        playback_session::{
            CloseEpisodePlayback, ControlEpisodePlayback, EpisodePlaybackObservation,
            EpisodePlaybackSource, NativeEngineDiagnostics, OpenedEpisodePlayback,
        },
    },
    domain::{
        api_values::DecimalU64, playback_control::PlaybackControl, request_referer::RequestReferer,
    },
    infrastructure::mpv::DeferredEpisodeEngine,
};
use std::sync::{atomic::AtomicUsize, mpsc};
use std::{num::NonZeroUsize, time::Duration};

fn finish_dispatcher(host: &mut DispatcherHost) {
    host.request_shutdown();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        match host.poll_shutdown() {
            DispatcherShutdown::Joined {
                worker_panicked: false,
                faults: 0,
            } => break,
            DispatcherShutdown::Pending => {
                assert!(std::time::Instant::now() < deadline);
                std::thread::yield_now();
            }
            other => panic!("Unexpected dispatcher state: {other:?}"),
        }
    }
}

fn result_fixture() -> (
    crate::infrastructure::mpv::guarded_command::Registry,
    crate::infrastructure::mpv::guarded_command::Ticket,
    crate::application::playback_session::GuardedSeekSubmission,
) {
    use crate::infrastructure::mpv::guarded_command::{Identity, Registry};
    let registry = Registry::default();
    let row = registry
        .reserve(Identity {
            engine: 7,
            resolution: [3; 32],
            connection: 1,
            lifecycle: 1,
            entry: Some(9),
        })
        .unwrap();
    let submission = row.submission(ControlReceipt::enqueued(7)).unwrap();
    (registry, row, submission)
}

#[test]
fn decorated_result_preserves_original_admission_and_terminal_single_consumer_after_revoke() {
    use crate::application::playback_session::{GuardedSeekResultQuery, GuardedSeekState};
    let binding = ParentBinding::new(epoch());
    let engine: Arc<dyn ParentOwnedEngine> = Arc::new(FakeEngine::new());
    binding.bind(epoch(), engine).unwrap();
    let owner = binding.admit(epoch()).unwrap();
    let (registry, row, submission) = result_fixture();
    let original = submission.ticket.clone();
    let decorated = owner.bind_result::<()>(submission, None).unwrap();
    let cloned = decorated.ticket.clone();
    assert!(original.same_request(&cloned));
    assert_eq!(
        owner
            .bind_result::<()>(
                crate::application::playback_session::GuardedSeekSubmission {
                    ticket: original.clone()
                },
                None
            )
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        original.read_original(false).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    drop(owner);
    binding.revoke_admission();
    let proxy = DeferredEpisodeEngine::new(binding.clone(), epoch());
    assert_eq!(
        proxy
            .guarded_seek_result(GuardedSeekResultQuery {
                ticket: cloned.clone(),
                consume_terminal: true
            })
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    registry
        .transition(&row, GuardedSeekState::RejectedBeforeWrite, None)
        .unwrap();
    assert_eq!(
        proxy
            .guarded_seek_result(GuardedSeekResultQuery {
                ticket: original.clone(),
                consume_terminal: true
            })
            .unwrap()
            .state(),
        GuardedSeekState::RejectedBeforeWrite
    );
    assert_eq!(
        proxy
            .guarded_seek_result(GuardedSeekResultQuery {
                ticket: cloned,
                consume_terminal: false
            })
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
}

#[test]
fn different_fixed_binding_rejects_original_read_without_consuming_it() {
    use crate::application::playback_session::GuardedSeekResultQuery;
    let original = ParentBinding::new(epoch());
    let foreign = ParentBinding::new(epoch());
    let engine: Arc<dyn ParentOwnedEngine> = Arc::new(FakeEngine::new());
    original.bind(epoch(), engine).unwrap();
    let (registry, _row, submission) = result_fixture();
    let owner = original.admit(epoch()).unwrap();
    let submission = owner.bind_result::<()>(submission, None).unwrap();
    drop(owner);
    let foreign = DeferredEpisodeEngine::new(foreign, epoch());
    assert_eq!(
        foreign
            .guarded_seek_result(GuardedSeekResultQuery {
                ticket: submission.ticket.clone(),
                consume_terminal: true
            })
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let original = DeferredEpisodeEngine::new(original, epoch());
    assert_eq!(
        original
            .guarded_seek_result(GuardedSeekResultQuery {
                ticket: submission.ticket,
                consume_terminal: false
            })
            .unwrap()
            .state(),
        crate::application::playback_session::GuardedSeekState::Queued
    );
    drop(registry);
}

#[test]
fn read_decoration_does_not_reserve_or_evict_any_of_the_64_rows() {
    use crate::infrastructure::mpv::guarded_command::{Identity, Registry};
    let binding = ParentBinding::new(epoch());
    let engine: Arc<dyn ParentOwnedEngine> = Arc::new(FakeEngine::new());
    binding.bind(epoch(), engine).unwrap();
    let owner = binding.admit(epoch()).unwrap();
    let registry = Registry::default();
    let identity = Identity {
        engine: 7,
        resolution: [3; 32],
        connection: 1,
        lifecycle: 1,
        entry: Some(9),
    };
    let rows: Vec<_> = (0..64)
        .map(|_| registry.reserve(identity.clone()).unwrap())
        .collect();
    assert_eq!(
        registry.reserve(identity.clone()).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    let submission = rows[0].submission(ControlReceipt::enqueued(7)).unwrap();
    let original = submission.ticket.clone();
    let decorated = owner.bind_result::<()>(submission, None).unwrap();
    assert!(original.same_request(&decorated.ticket));
    assert_eq!(
        registry.reserve(identity).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    for row in rows {
        assert_eq!(
            registry.read(&row, false).unwrap().state,
            crate::application::playback_session::GuardedSeekState::Queued
        );
    }
}

#[test]
fn scoped_revoke_closes_original_word_without_taking_held_slot_lock() {
    use crate::infrastructure::mpv::guarded_command::HostScope;
    let scope = HostScope::testing();
    let binding = ParentBinding::new_scoped(epoch(), Arc::downgrade(&scope)).unwrap();
    let slot = binding.0.slot.lock().unwrap();
    binding.revoke_admission();
    assert!(scope.word.is_closed());
    assert!(binding.0.revoked.load(Ordering::Acquire));
    drop(slot);
}

#[test]
fn scoped_revoke_closes_word_while_original_registry_row_lock_is_held() {
    use crate::infrastructure::mpv::guarded_command::HostScope;
    let scope = HostScope::testing();
    let binding = ParentBinding::new_scoped(epoch(), Arc::downgrade(&scope)).unwrap();
    let strong = Arc::strong_count(&scope);
    scope.registry.with_row_lock_for_test(|| {
        binding.revoke_admission();
        assert!(scope.word.is_closed());
        assert_eq!(Arc::strong_count(&scope), strong);
    });
}

#[test]
fn expired_original_scope_revoke_never_closes_a_new_scope_word() {
    use crate::infrastructure::mpv::guarded_command::HostScope;
    let original = HostScope::testing();
    let binding = ParentBinding::new_scoped(epoch(), Arc::downgrade(&original)).unwrap();
    let word = original.word.clone();
    drop(original);
    assert!(word.is_closed());
    let current = HostScope::testing();
    binding.revoke_admission();
    assert!(!current.word.is_closed());
}

#[test]
fn wrong_opaque_read_context_and_missing_context_cannot_authorize_binding_read() {
    use crate::application::playback_session::GuardedSeekResultQuery;
    let binding = ParentBinding::new(epoch());
    let (registry, row, submission) = result_fixture();
    let query = || GuardedSeekResultQuery {
        ticket: submission.ticket.clone(),
        consume_terminal: true,
    };
    assert_eq!(
        binding
            .read_bound_result(epoch(), query())
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    submission.ticket.bind_read_context(Arc::new(7u64)).unwrap();
    assert_eq!(
        binding
            .read_bound_result(epoch(), query())
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        registry.read(&row, false).unwrap().state,
        crate::application::playback_session::GuardedSeekState::Queued
    );
}

#[test]
fn original_result_source_expiry_is_preserved_by_decorated_read() {
    use crate::application::playback_session::GuardedSeekResultQuery;
    let binding = ParentBinding::new(epoch());
    let engine: Arc<dyn ParentOwnedEngine> = Arc::new(FakeEngine::new());
    binding.bind(epoch(), engine).unwrap();
    let (registry, _row, submission) = result_fixture();
    let owner = binding.admit(epoch()).unwrap();
    let submission = owner.bind_result::<()>(submission, None).unwrap();
    drop(owner);
    drop(registry);
    assert_eq!(
        binding
            .read_bound_result(
                epoch(),
                GuardedSeekResultQuery {
                    ticket: submission.ticket,
                    consume_terminal: false
                }
            )
            .unwrap_err()
            .code,
        ErrorCode::SourceExpired
    );
}

type OpenGate = (mpsc::Sender<()>, mpsc::Receiver<()>);
struct FakeEngine {
    active: AtomicUsize,
    opens: AtomicUsize,
    shutdown_calls: AtomicUsize,
    shutdown_step: AtomicUsize,
    open_gate: Mutex<Option<OpenGate>>,
}
impl FakeEngine {
    fn new() -> Self {
        Self {
            active: AtomicUsize::new(0),
            opens: AtomicUsize::new(0),
            shutdown_calls: AtomicUsize::new(0),
            shutdown_step: AtomicUsize::new(2),
            open_gate: Mutex::new(None),
        }
    }
}
fn epoch() -> NonZeroU64 {
    NonZeroU64::new(7).unwrap()
}
fn source() -> EpisodePlaybackSource {
    EpisodePlaybackSource::synthetic_replacement_source(RequestReferer::Absent)
}
fn closed(instance: u64, exited: bool) -> ClosedEpisodePlayback {
    ClosedEpisodePlayback {
        engine_instance_id: DecimalU64(instance),
        engine_exited: exited,
        ipc_worker_exited: exited,
        stderr_worker_exited: exited,
        stdout_worker_exited: exited,
        diagnostics: NativeEngineDiagnostics {
            artifact: RawArtifact {
                artifact_id: "a".repeat(64),
                body_sha256: "b".repeat(64),
                body_bytes: DecimalU64(1),
                body_path: "synthetic-body".into(),
                metadata_path: "synthetic-metadata".into(),
            },
            stderr_sha256: "c".repeat(64),
            stderr_bytes: DecimalU64(0),
            stdout_sha256: "d".repeat(64),
            stdout_bytes: DecimalU64(0),
            complete: true,
            truncated: false,
            read_failed: false,
        },
    }
}
impl EpisodePlaybackEngine for FakeEngine {
    fn options_available(&self) -> bool {
        true
    }
    fn open(
        &self,
        source: EpisodePlaybackSource,
        _: bool,
        _: f64,
    ) -> Result<OpenedEpisodePlayback, ApiError> {
        let gate = self.open_gate.lock().unwrap().take();
        if let Some((started, resume)) = gate {
            started.send(()).unwrap();
            resume.recv().unwrap();
        }
        let id = self.opens.fetch_add(1, Ordering::SeqCst) + 41;
        self.active
            .compare_exchange(0, id, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| ApiError::new(ErrorCode::RevisionConflict, "Already active"))?;
        Ok(OpenedEpisodePlayback {
            engine_instance_id: DecimalU64(id as u64),
            resolution_artifact_id: source.artifact_id().into(),
            completion: ControlCompletion::Unobserved,
        })
    }
    fn close(&self, input: CloseEpisodePlayback) -> Result<ClosedEpisodePlayback, ApiError> {
        self.active
            .compare_exchange(
                input.engine_instance_id.0 as usize,
                0,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .map_err(|_| ApiError::new(ErrorCode::RevisionConflict, "Wrong instance"))?;
        Ok(closed(input.engine_instance_id.0, true))
    }
    fn observe(&self) -> Result<EpisodePlaybackObservation, ApiError> {
        Err(ApiError::new(
            ErrorCode::NeedsInput,
            "No synthetic observation",
        ))
    }
    fn control(&self, input: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
        if self.active.load(Ordering::SeqCst) as u64 != input.engine_instance_id.0 {
            return Err(ApiError::new(ErrorCode::RevisionConflict, "Wrong instance"));
        }
        Ok(ControlReceipt::enqueued(input.engine_instance_id.0))
    }
}
impl ParentOwnedEngine for FakeEngine {
    fn shutdown_for_parent(&self) -> Result<Option<ClosedEpisodePlayback>, ApiError> {
        self.shutdown_calls.fetch_add(1, Ordering::SeqCst);
        match self.shutdown_step.fetch_add(1, Ordering::SeqCst) {
            0 => Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Injected cleanup failure",
            )),
            1 => Ok(Some(closed(
                self.active.load(Ordering::SeqCst) as u64,
                false,
            ))),
            _ => {
                let id = self.active.swap(0, Ordering::SeqCst);
                Ok((id != 0).then(|| closed(id as u64, true)))
            }
        }
    }
}

#[test]
fn unready_stale_and_duplicate_binding_never_select_another_owner() {
    let binding = ParentBinding::new(epoch());
    let proxy = DeferredEpisodeEngine::new(binding.clone(), epoch());
    assert_eq!(
        binding.cleanup(epoch()).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    assert_eq!(
        proxy.open(source(), false, 50.).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    let stale = DeferredEpisodeEngine::new(binding.clone(), NonZeroU64::new(8).unwrap());
    assert_eq!(
        stale.open(source(), false, 50.).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let engine = Arc::new(FakeEngine::new());
    binding.bind(epoch(), engine.clone()).unwrap();
    let other: Arc<dyn ParentOwnedEngine> = Arc::new(FakeEngine::new());
    let refused = binding.bind(epoch(), other.clone()).unwrap_err();
    assert!(Arc::ptr_eq(&refused.engine, &other));
    assert_eq!(refused.error.code, ErrorCode::RevisionConflict);
    let first = proxy.open(source(), false, 50.).unwrap();
    proxy
        .close(CloseEpisodePlayback {
            engine_instance_id: first.engine_instance_id,
        })
        .unwrap();
    let second = proxy.clone().open(source(), true, 25.).unwrap();
    assert_ne!(first.engine_instance_id, second.engine_instance_id);
    assert_eq!(
        proxy
            .control(ControlEpisodePlayback {
                engine_instance_id: first.engine_instance_id,
                control: PlaybackControl::Pause,
            })
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        proxy
            .control(ControlEpisodePlayback {
                engine_instance_id: second.engine_instance_id,
                control: PlaybackControl::Pause,
            })
            .unwrap()
            .engine_instance_id,
        second.engine_instance_id
    );
    assert_eq!(engine.opens.load(Ordering::SeqCst), 2);
    assert!(proxy.options_available());
}

#[test]
fn revoke_before_ready_is_permanent_and_returns_a_late_bind_owner() {
    let binding = ParentBinding::new(epoch());
    binding.revoke_admission();
    let engine: Arc<dyn ParentOwnedEngine> = Arc::new(FakeEngine::new());
    let refused = binding.bind(epoch(), engine.clone()).unwrap_err();
    assert!(Arc::ptr_eq(&engine, &refused.engine));
    assert_eq!(refused.error.code, ErrorCode::Cancelled);
    assert!(matches!(
        binding.cleanup(epoch()).unwrap(),
        ParentCleanupStatus::Complete(None)
    ));
    assert_eq!(
        binding.admit(epoch()).err().unwrap().code,
        ErrorCode::Cancelled
    );
}

#[test]
fn revoke_is_nonblocking_while_slot_is_owned_and_late_bind_is_refused() {
    let binding = ParentBinding::new(epoch());
    let guard = binding.0.slot.lock().unwrap();
    let engine: Arc<dyn ParentOwnedEngine> = Arc::new(FakeEngine::new());
    let worker = {
        let binding = binding.clone();
        let engine = engine.clone();
        std::thread::spawn(move || binding.bind(epoch(), engine))
    };
    binding.revoke_admission();
    assert!(binding.0.revoked.load(Ordering::Acquire));
    drop(guard);
    let refused = worker.join().unwrap().unwrap_err();
    assert_eq!(refused.error.code, ErrorCode::Cancelled);
    assert!(Arc::ptr_eq(&refused.engine, &engine));
    assert_eq!(
        binding.admit(epoch()).err().unwrap().code,
        ErrorCode::Cancelled
    );
}

#[test]
fn in_flight_open_retains_same_owner_until_failed_and_incomplete_cleanup_can_retry() {
    let binding = ParentBinding::new(epoch());
    let proxy = DeferredEpisodeEngine::new(binding.clone(), epoch());
    let engine = Arc::new(FakeEngine::new());
    engine.shutdown_step.store(0, Ordering::SeqCst);
    let weak = Arc::downgrade(&engine);
    let (started_tx, started_rx) = mpsc::channel();
    let (resume_tx, resume_rx) = mpsc::channel();
    *engine.open_gate.lock().unwrap() = Some((started_tx, resume_rx));
    binding.bind(epoch(), engine.clone()).unwrap();
    let worker = {
        let proxy = proxy.clone();
        std::thread::spawn(move || proxy.open(source(), false, 50.))
    };
    started_rx
        .recv_timeout(std::time::Duration::from_secs(2))
        .unwrap();
    binding.revoke_admission();
    assert!(matches!(
        binding.cleanup(epoch()).unwrap(),
        ParentCleanupStatus::Pending
    ));
    assert_eq!(engine.shutdown_calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        proxy
            .clone()
            .control(ControlEpisodePlayback {
                engine_instance_id: DecimalU64(41),
                control: PlaybackControl::Pause,
            })
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    resume_tx.send(()).unwrap();
    let opened = worker.join().unwrap().unwrap();
    assert_eq!(opened.engine_instance_id, DecimalU64(41));
    assert_eq!(
        binding.cleanup(epoch()).unwrap_err().code,
        ErrorCode::EngineFailed
    );
    assert!(
        matches!(binding.cleanup(epoch()).unwrap(), ParentCleanupStatus::Incomplete(ref receipt) if receipt.engine_instance_id == opened.engine_instance_id)
    );
    let receipt = binding.cleanup(epoch()).unwrap();
    assert!(
        matches!(receipt, ParentCleanupStatus::Complete(Some(ref receipt)) if receipt.engine_instance_id == opened.engine_instance_id)
    );
    assert!(matches!(
        binding.cleanup(epoch()).unwrap(),
        ParentCleanupStatus::Complete(Some(_))
    ));
    assert_eq!(engine.shutdown_calls.load(Ordering::SeqCst), 3);
    assert_eq!(
        proxy.open(source(), false, 50.).unwrap_err().code,
        ErrorCode::Cancelled
    );
    drop(engine);
    assert!(weak.upgrade().is_some());
}

#[test]
fn async_cleanup_has_one_joined_wake_after_inflight_drains_without_lost_notification() {
    for release_before_submit in [true, false] {
        let binding = ParentBinding::new(epoch());
        let proxy = DeferredEpisodeEngine::new(binding.clone(), epoch());
        let engine = Arc::new(FakeEngine::new());
        let (started_tx, started_rx) = mpsc::channel();
        let (resume_tx, resume_rx) = mpsc::channel();
        *engine.open_gate.lock().unwrap() = Some((started_tx, resume_rx));
        binding.bind(epoch(), engine.clone()).unwrap();
        let worker = std::thread::spawn(move || proxy.open(source(), false, 50.));
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        binding.revoke_admission();
        let mut worker = Some(worker);
        if release_before_submit {
            resume_tx.send(()).unwrap();
            worker.take().unwrap().join().unwrap().unwrap();
        }
        let mut host = DispatcherHost::start(NonZeroUsize::new(1).unwrap()).unwrap();
        let (wake_tx, wake_rx) = mpsc::channel();
        let mut task = ParentCleanupTask::new(
            binding,
            epoch(),
            host.handle(),
            Arc::new(move || wake_tx.send(()).map_err(|error| error.to_string())),
        );
        task.start_attempt().unwrap();
        if !release_before_submit {
            assert!(matches!(task.observe(), ParentCleanupObservation::Pending));
            assert!(wake_rx.try_recv().is_err());
            assert_eq!(engine.shutdown_calls.load(Ordering::Acquire), 0);
            resume_tx.send(()).unwrap();
            worker.take().unwrap().join().unwrap().unwrap();
        }
        wake_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(
            matches!(task.observe(), ParentCleanupObservation::Finished(Ok(ParentCleanupStatus::Complete(Some(receipt)))) if receipt.engine_instance_id == DecimalU64(41))
        );
        assert_eq!(engine.shutdown_calls.load(Ordering::Acquire), 1);
        finish_dispatcher(&mut host);
        assert!(wake_rx.try_recv().is_err());
    }
}

#[test]
fn refused_dispatch_keeps_unstarted_cleanup_and_original_binding_owned() {
    let binding = ParentBinding::new(epoch());
    let engine = Arc::new(FakeEngine::new());
    let weak = Arc::downgrade(&engine);
    binding.bind(epoch(), engine.clone()).unwrap();
    binding.revoke_admission();
    let mut host = DispatcherHost::start(NonZeroUsize::new(1).unwrap()).unwrap();
    host.request_shutdown();
    let mut task = ParentCleanupTask::new(binding, epoch(), host.handle(), Arc::new(|| Ok(())));
    assert_eq!(task.start_attempt().unwrap_err().code, ErrorCode::Cancelled);
    assert!(matches!(
        task.observe(),
        ParentCleanupObservation::NotStarted
    ));
    assert_eq!(engine.shutdown_calls.load(Ordering::Acquire), 0);
    assert_eq!(task.start_attempt().unwrap_err().code, ErrorCode::Cancelled);
    drop(engine);
    assert!(weak.upgrade().is_some());
    finish_dispatcher(&mut host);
}
