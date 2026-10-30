//! Source-authored owner lifecycle regressions; no live media or GUI claims.
use super::*;
use crate::{
    application::playback_session::{GuardedSeek, GuardedSeekResultQuery},
    domain::request_referer::RequestReferer,
    infrastructure::{
        catalog::CatalogFiles,
        mpv::{
            control::MpvControlHandle,
            guarded_command::{HostScope, Identity, RequestIds, Worker},
            owned_startup,
        },
    },
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::AtomicUsize,
    time::{Duration, Instant},
};

const PROBE: &str = "PLAYER_GUARDED_LIFECYCLE_CHILD";
#[test]
fn bounded_lifecycle_child_probe() {
    if std::env::var(PROBE).ok().as_deref() == Some("child") {
        std::thread::sleep(Duration::from_secs(3));
    }
}
fn child_command() -> Command {
    let full = format!("{}::bounded_lifecycle_child_probe", module_path!());
    let (_, name) = full.split_once("::").unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args(["--exact", name]).env(PROBE, "child");
    command
}
struct Fixture {
    owner: Option<NativeEpisodeEngine>,
    root: PathBuf,
}
impl Fixture {
    fn new(enabled: bool) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "player-guarded-lifecycle-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir(&root).unwrap();
        let executable = root.join("mpv.exe");
        let bytes = b"Synthetic trusted identity; only the Rust child probe is executable";
        std::fs::write(&executable, bytes).unwrap();
        let executable =
            TrustedMpvExecutable::open(&executable, &format!("{:x}", Sha256::digest(bytes)))
                .unwrap();
        let catalog = root.join("catalog");
        std::fs::create_dir(&catalog).unwrap();
        let store = Arc::new(CatalogFiles::open(&catalog).unwrap());
        let owner = if enabled {
            NativeEpisodeEngine::new_with_scope(executable, None, store, HostScope::testing())
        } else {
            NativeEpisodeEngine::new(executable, None, store)
        }
        .unwrap();
        Self {
            owner: Some(owner),
            root,
        }
    }
    fn owner(&self) -> &NativeEpisodeEngine {
        self.owner.as_ref().unwrap()
    }
    fn attach(&self) -> MpvControlHandle {
        let owner = self.owner();
        let scope = owner.scope.clone();
        let mpv = owned_startup::spawn_owned_scoped(
            child_command(),
            format!(r"\\.\pipe\never-guarded-lifecycle-{}", std::process::id()),
            scope,
        )
        .unwrap_or_else(|_| panic!("Rust lifecycle probe startup"));
        let control = mpv.control_handle();
        control
            .state
            .native_observation
            .lock()
            .unwrap()
            .connect()
            .unwrap();
        control.state.connected.store(true, Ordering::Release);
        let source = EpisodePlaybackSource::synthetic_replacement_source(RequestReferer::Absent);
        *owner.active.lock().unwrap() = Some(Active::from_mpv(
            mpv,
            &source,
            owner.diagnostics_store.clone(),
        ));
        control
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(owner) = self.owner.take() {
            owner
                .shutdown()
                .expect("Retain and join every owned lifecycle probe");
            drop(owner);
        }
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}
fn next<T>(receiver: &mut tokio::sync::mpsc::Receiver<T>) -> T {
    let until = Instant::now() + Duration::from_secs(1);
    loop {
        if let Ok(value) = receiver.try_recv() {
            return value;
        }
        assert!(Instant::now() < until, "Original Observe was not enqueued");
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn reply(wire: &Value, data: Value) -> Value {
    json!({"request_id":wire["request_id"],"error":"success","data":data})
}
fn complete_observe(
    control: &MpvControlHandle,
    worker: &mut Worker,
    ids: &mut RequestIds,
    start: impl FnOnce(
        &mut Worker,
        &mut RequestIds,
    ) -> Vec<crate::infrastructure::mpv::guarded_command::Packet>,
) {
    let host = &control.state.guarded;
    let before = start(worker, ids).remove(0);
    let playlist = json!([{"id":9,"playing":true}]);
    let properties = worker
        .receive(
            &reply(&before.wire, playlist.clone()),
            Some((1, 1)),
            ids,
            host,
            Instant::now(),
        )
        .unwrap();
    assert_eq!(properties.len(), 6);
    let mut after = Vec::new();
    for (query, value) in properties.into_iter().zip([
        json!(3.),
        json!(60.),
        json!(false),
        json!(false),
        json!(true),
        json!(false),
    ]) {
        after = worker
            .receive(
                &reply(&query.wire, value),
                Some((1, 1)),
                ids,
                host,
                Instant::now(),
            )
            .unwrap();
    }
    assert_eq!(after.len(), 1);
    assert!(
        worker
            .receive(
                &reply(&after[0].wire, playlist),
                Some((1, 1)),
                ids,
                host,
                Instant::now()
            )
            .unwrap()
            .is_empty()
    );
}

#[test]
fn owner_releases_active_before_observe_wait_and_uses_original_ticket_after_close() {
    let fixture = Fixture::new(true);
    let owner = fixture.owner();
    let control = fixture.attach();
    let mut receiver = control.state.guarded.take_receiver().unwrap();
    let consumer = owner.clone();
    let waiting = std::thread::spawn(move || consumer.observe_native_guard());
    let input = next(&mut receiver);
    assert!(
        owner.active.try_lock().is_ok(),
        "Observe must not hold Active during barrier wait"
    );
    let mut worker = Worker::new();
    let mut ids = RequestIds::default();
    complete_observe(&control, &mut worker, &mut ids, |worker, ids| {
        worker
            .begin(input, ids, &control.state.guarded, Instant::now())
            .unwrap()
    });
    let observation = waiting.join().unwrap().unwrap();
    let (view, guard) = observation.into_guarded_parts().unwrap();
    assert_eq!(
        view.identity().engine_instance_id().0,
        control.instance_id()
    );
    assert_eq!(view.playback().position_us, Some(DecimalU64(3_000_000)));
    let submitted = owner
        .submit_guarded_seek(GuardedSeek {
            guard,
            position_us: DecimalU64(4_000_000),
        })
        .unwrap();
    let (_, ticket) = submitted.into_parts();
    let input = next(&mut receiver);
    let packets = worker
        .begin(input, &mut ids, &control.state.guarded, Instant::now())
        .unwrap();
    assert_eq!(packets.len(), 1);
    drop(packets);
    let old_id = DecimalU64(control.instance_id());
    owner
        .close(CloseEpisodePlayback {
            engine_instance_id: old_id,
        })
        .unwrap();
    assert!(
        !owner.scope.word.is_closed(),
        "Episode close must preserve logical owner scope"
    );
    let next_control = fixture.attach();
    assert_ne!(next_control.instance_id(), control.instance_id());
    let next_generation = next_control.state.guarded.fence.generation();
    control.state.close();
    assert_eq!(owner.scope.word.generation(), next_generation);
    assert!(!next_control.state.guarded.fence.is_closed());
    let result = owner
        .guarded_seek_result(GuardedSeekResultQuery {
            ticket,
            consume_terminal: false,
        })
        .unwrap();
    assert_eq!(result.original_identity().engine_instance_id(), old_id);
    assert!(result.write_commit());
}

#[test]
fn shutdown_closes_original_word_before_waiting_for_active_lock() {
    let fixture = Fixture::new(true);
    let owner = fixture.owner();
    let active = owner.active.lock().unwrap();
    let scope = owner.scope.clone();
    let consumer = owner.clone();
    let shutdown = std::thread::spawn(move || consumer.shutdown());
    let until = Instant::now() + Duration::from_secs(1);
    while !scope.word.is_closed() {
        assert!(Instant::now() < until);
        std::thread::yield_now();
    }
    assert!(owner.closed.load(Ordering::Acquire));
    drop(active);
    assert!(shutdown.join().unwrap().unwrap().is_none());
}

#[test]
fn shutdown_drains_original_permit_without_holding_active_lock() {
    let fixture = Fixture::new(true);
    let owner = fixture.owner();
    let permit = owner
        .scope
        .word
        .commit(owner.scope.word.generation())
        .unwrap();
    let consumer = owner.clone();
    let scope = owner.scope.clone();
    let (done, finished) = std::sync::mpsc::channel();
    let shutdown = std::thread::spawn(move || {
        let result = consumer.shutdown();
        done.send(()).unwrap();
        result
    });
    let until = Instant::now() + Duration::from_secs(1);
    while !scope.word.is_closed() {
        assert!(Instant::now() < until);
        std::thread::yield_now();
    }
    loop {
        if owner.active.try_lock().is_ok() {
            break;
        }
        assert!(Instant::now() < until);
        std::thread::yield_now();
    }
    assert!(finished.try_recv().is_err());
    assert_eq!(scope.word.in_flight(), 1);
    drop(permit);
    assert!(shutdown.join().unwrap().unwrap().is_none());
    finished.recv_timeout(Duration::from_secs(1)).unwrap();
    assert_eq!(scope.word.in_flight(), 0);
}

#[test]
fn pre_child_spawn_failure_retires_connection_without_ending_logical_scope() {
    let fixture = Fixture::new(true);
    let owner = fixture.owner();
    let generation = owner.scope.word.generation();
    let command = Command::new(fixture.root.join("absent-native-child.exe"));
    let failure = owned_startup::spawn_owned_scoped(
        command,
        "unused-pre-child-failure".into(),
        owner.scope.clone(),
    )
    .err()
    .expect("Missing executable must fail before child creation");
    let (error, recovery) = (*failure).into_parts();
    assert_eq!(error.details["process_created"], false);
    assert!(recovery.is_none());
    assert!(!owner.scope.word.is_closed());
    assert_ne!(owner.scope.word.generation(), generation);
    let (sender, _receiver) = tokio::sync::mpsc::channel(64);
    let replacement = MpvControlHandle::new_scoped(sender, owner.scope.clone()).unwrap();
    assert!(!replacement.state.guarded.fence.is_closed());
}

#[test]
fn production_constructor_remains_diagnostic_only_with_same_legacy_controls() {
    let fixture = Fixture::new(false);
    let control = fixture.attach();
    let owner = fixture.owner();
    let observation = owner.observe_native_guard().unwrap();
    assert!(!observation.observation_authorized);
    assert!(observation.guard.is_none());
    let legacy = control
        .submit(crate::domain::playback_control::PlaybackControl::Pause)
        .unwrap();
    assert_eq!(legacy.engine_instance_id.0, control.instance_id());
    assert_eq!(legacy.completion, ControlCompletion::Unobserved);
    let identity = Identity {
        engine: control.instance_id(),
        resolution: [0xbb; 32],
        connection: 1,
        lifecycle: 1,
        entry: None,
    };
    assert!(
        matches!(control.state.guarded.observe(identity,false,Instant::now()+Duration::from_secs(1)),Err(error) if error.code==ErrorCode::Unsupported)
    );
}

#[test]
fn original_resolution_requires_full_hex_and_does_not_rehash_diagnostics() {
    assert_eq!(resolution_identity(&"b".repeat(64)).unwrap(), [0xbb; 32]);
    for invalid in ["b".repeat(63), "g".repeat(64), "가".repeat(22)] {
        assert!(resolution_identity(&invalid).is_err());
    }
}

#[test]
fn clones_share_original_scope_and_weak_consumers_do_not_keep_owner_alive() {
    let mut fixture = Fixture::new(true);
    let first = fixture.owner().guard_scope();
    let consumer = fixture.owner().clone();
    assert!(std::sync::Weak::ptr_eq(&first, &consumer.guard_scope()));
    let word = consumer.scope.word.clone();
    drop(fixture.owner.take());
    assert!(first.upgrade().is_some());
    drop(consumer);
    assert!(first.upgrade().is_none());
    assert!(word.is_closed());
}

fn observed_guard(
    fixture: &Fixture,
) -> (
    MpvControlHandle,
    crate::application::playback_session::NativeObservationGuard,
    impl FnMut() -> Vec<Value>,
) {
    let control = fixture.attach();
    let mut receiver = control.state.guarded.take_receiver().unwrap();
    let consumer = fixture.owner().clone();
    let waiting = std::thread::spawn(move || consumer.observe_native_guard());
    let input = next(&mut receiver);
    let mut worker = Worker::new();
    let mut ids = RequestIds::default();
    complete_observe(&control, &mut worker, &mut ids, |worker, ids| {
        worker
            .begin(input, ids, &control.state.guarded, Instant::now())
            .unwrap()
    });
    let (_, guard) = waiting
        .join()
        .unwrap()
        .unwrap()
        .into_guarded_parts()
        .unwrap();
    let writer = control.clone();
    (control, guard, move || {
        let mut wires = Vec::new();
        while let Ok(input) = receiver.try_recv() {
            let packets = worker
                .begin(input, &mut ids, &writer.state.guarded, Instant::now())
                .unwrap();
            wires.extend(packets.iter().map(|packet| packet.wire.clone()));
        }
        wires
    })
}
fn foreign_submit(closed: bool) {
    let original = Fixture::new(true);
    let foreign = Fixture::new(true);
    let (_, guard, mut writer) = observed_guard(&original);
    if closed {
        foreign.owner().shutdown().unwrap();
    }
    assert_eq!(
        foreign
            .owner()
            .submit_guarded_seek(GuardedSeek {
                guard,
                position_us: DecimalU64(4_000_000)
            })
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert!(writer().is_empty());
}
#[test]
fn foreign_open_host_cannot_submit_original_guard() {
    foreign_submit(false);
}
#[test]
fn foreign_closed_host_is_rejected_as_foreign_before_closed_admission() {
    foreign_submit(true);
}

#[test]
fn same_clone_submits_and_original_closed_host_reads_while_foreign_reads_are_rejected() {
    let original = Fixture::new(true);
    let foreign = Fixture::new(true);
    let (control, guard, mut writer) = observed_guard(&original);
    let consumer = original.owner().clone();
    let submission = consumer
        .submit_guarded_seek(GuardedSeek {
            guard,
            position_us: DecimalU64(4_000_000),
        })
        .unwrap();
    let (_, ticket) = submission.into_parts();
    assert_eq!(writer().len(), 1);
    for closed in [false, true] {
        if closed {
            foreign.owner().shutdown().unwrap();
        }
        assert_eq!(
            foreign
                .owner()
                .guarded_seek_result(GuardedSeekResultQuery {
                    ticket: ticket.clone(),
                    consume_terminal: false
                })
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
    }
    original.owner().shutdown().unwrap();
    let result = consumer
        .guarded_seek_result(GuardedSeekResultQuery {
            ticket,
            consume_terminal: false,
        })
        .unwrap();
    assert_eq!(
        result.original_identity().engine_instance_id().0,
        control.instance_id()
    );
    assert!(result.write_commit());
}

#[test]
fn own_closed_host_cannot_submit_its_original_guard() {
    let fixture = Fixture::new(true);
    let (_, guard, mut writer) = observed_guard(&fixture);
    fixture.owner().shutdown().unwrap();
    assert_eq!(
        fixture
            .owner()
            .submit_guarded_seek(GuardedSeek {
                guard,
                position_us: DecimalU64(4_000_000)
            })
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    assert!(writer().is_empty());
}

#[test]
fn wrong_opaque_issuer_type_is_rejected_without_upgrading_owner_or_consuming_capability() {
    let fixture = Fixture::new(true);
    let (_, guard, mut writer) = observed_guard(&fixture);
    let owner = fixture.owner();
    let count = Arc::strong_count(&owner.scope);
    assert_eq!(
        guard.require_issuer(&7u64).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    guard.require_issuer(&owner.guard_scope()).unwrap();
    assert_eq!(Arc::strong_count(&owner.scope), count);
    let submission = owner
        .submit_guarded_seek(GuardedSeek {
            guard,
            position_us: DecimalU64(4_000_000),
        })
        .unwrap();
    let (_, ticket) = submission.into_parts();
    assert_eq!(
        ticket.require_issuer(&"foreign-type").unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    ticket.require_issuer(&owner.guard_scope()).unwrap();
    assert_eq!(Arc::strong_count(&owner.scope), count);
    assert_eq!(writer().len(), 1);
}

fn trusted_standalone(fixture: &Fixture) -> crate::infrastructure::mpv::NativePlaybackHost {
    crate::infrastructure::mpv::NativePlaybackHost::start_trusted_standalone(
        &fixture.root.join("mpv.exe"),
        fixture.owner().executable.sha256(),
        fixture.owner().diagnostics_store.clone(),
    )
    .unwrap()
}

#[test]
fn verified_standalone_factory_enables_original_scope_without_starting_media() {
    let fixture = Fixture::new(false);
    let host = trusted_standalone(&fixture);
    let engine = host.engine();
    engine.scope.require_enabled().unwrap();
    assert_eq!(engine.parent, None);
    assert!(engine.active.lock().unwrap().is_none());
    assert_eq!(
        engine.executable.sha256(),
        fixture.owner().executable.sha256()
    );
    assert_eq!(engine.executable.path(), fixture.owner().executable.path());
    let clone = host.engine();
    assert!(std::sync::Weak::ptr_eq(
        &engine.guard_scope(),
        &clone.guard_scope()
    ));
    assert_eq!(
        engine.observe_native_guard().unwrap_err().code,
        ErrorCode::NotFound
    );
}

#[test]
fn standalone_factory_rejects_relative_name_malformed_and_mismatched_sha() {
    use crate::infrastructure::mpv::NativePlaybackHost;
    let fixture = Fixture::new(false);
    let executable = fixture.root.join("mpv.exe");
    let hash = fixture.owner().executable.sha256();
    let store = fixture.owner().diagnostics_store.clone();
    let error = NativePlaybackHost::start_trusted_standalone(
        std::path::Path::new("mpv.exe"),
        hash,
        store.clone(),
    )
    .err()
    .expect("Relative executable must fail");
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    let error = NativePlaybackHost::start_trusted_standalone(
        &fixture.root.join("other.exe"),
        hash,
        store.clone(),
    )
    .err()
    .expect("Non-mpv filename must fail before file access");
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    let error = NativePlaybackHost::start_trusted_standalone(&executable, "bad", store.clone())
        .err()
        .expect("Malformed digest must fail");
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    let error = NativePlaybackHost::start_trusted_standalone(&executable, &"0".repeat(64), store)
        .err()
        .expect("Wrong digest must fail before scope creation");
    assert_eq!(error.code, ErrorCode::SourceExpired);
    assert!(fixture.owner().active.lock().unwrap().is_none());
    assert_eq!(
        fixture.owner().scope.require_enabled().unwrap_err().code,
        ErrorCode::Unsupported
    );
}

#[test]
fn legacy_engine_and_parented_or_unparented_host_constructors_remain_disabled() {
    use crate::infrastructure::mpv::NativePlaybackHost;
    let fixture = Fixture::new(false);
    assert_eq!(
        fixture.owner().scope.require_enabled().unwrap_err().code,
        ErrorCode::Unsupported
    );
    for parent in [None, Some(1)] {
        let host = NativePlaybackHost::start(
            &fixture.root.join("mpv.exe"),
            fixture.owner().executable.sha256(),
            parent,
            fixture.owner().diagnostics_store.clone(),
        )
        .unwrap();
        let engine = host.engine();
        assert_eq!(engine.parent, parent);
        assert_eq!(
            engine.scope.require_enabled().unwrap_err().code,
            ErrorCode::Unsupported
        );
        assert!(engine.active.lock().unwrap().is_none());
    }
}

#[test]
fn standalone_host_drop_closes_clone_permission_and_retains_verified_file_until_last_engine() {
    let mut fixture = Fixture::new(false);
    let host = trusted_standalone(&fixture);
    let engine = host.engine();
    let original = engine.guard_scope();
    let word = engine.scope.word.clone();
    let executable = fixture.root.join("mpv.exe");
    drop(fixture.owner.take());
    assert_eq!(
        std::fs::OpenOptions::new()
            .write(true)
            .open(&executable)
            .is_err(),
        crate::infrastructure::os_policy::has_mandatory_share_modes()
    );
    drop(host);
    assert!(word.is_closed());
    assert!(original.upgrade().is_some());
    assert_eq!(
        engine
            .open(
                EpisodePlaybackSource::synthetic_replacement_source(RequestReferer::Absent),
                false,
                50.
            )
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    assert!(engine.active.lock().unwrap().is_none());
    assert_eq!(
        std::fs::OpenOptions::new()
            .write(true)
            .open(&executable)
            .is_err(),
        crate::infrastructure::os_policy::has_mandatory_share_modes()
    );
    drop(engine);
    assert!(original.upgrade().is_none());
    assert!(
        std::fs::OpenOptions::new()
            .write(true)
            .open(&executable)
            .is_ok()
    );
}
