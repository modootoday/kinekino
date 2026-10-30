use super::*;
#[cfg(feature = "native-proposal-window")]
#[path = "tests/bootstrap_late.rs"]
mod bootstrap_late;
#[cfg(feature = "native-proposal-window")]
#[path = "tests/failed_engine_cleanup.rs"]
mod failed_engine_cleanup;
#[cfg(feature = "native-proposal-window")]
#[path = "tests/routed_activation.rs"]
mod routed_activation;
#[cfg(feature = "native-proposal-window")]
#[path = "tests/routed_replacement.rs"]
mod routed_replacement;
use crate::{
    application::{
        api::{PlayerSdk, Request},
        catalog::{CaptureMetadata, RawArtifact},
        episode_resolve::{EpisodeResolutionResults, StoredEpisodeResolution},
        jobs::VerifiedJobOutput,
        playback_control::ControlCompletion,
        playback_status::{PlaybackStatus, PlaybackStatusOrigin},
    },
    domain::{
        api_values::DecimalU64,
        playback_control::PlaybackControl,
        playback_options::{PlaybackRate, TrackKind, TrackSelection},
        request_referer::RequestReferer,
    },
    infrastructure::thread_completion::{DispatcherHost, DispatcherShutdown, JoinRequest},
};
use std::{
    cell::RefCell,
    num::NonZeroUsize,
    sync::{
        Weak,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
const LIMIT: Duration = Duration::from_secs(5);

struct ExportFixture {
    engine: Option<Arc<NativeEpisodeEngine>>,
    directory: std::path::PathBuf,
}
impl ExportFixture {
    fn new() -> Self {
        Self::with_routed_scope(false)
    }
    fn with_routed_scope(routed: bool) -> Self {
        use sha2::{Digest, Sha256};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "player-routed-export-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).unwrap();
        let executable = directory.join("mpv.exe");
        let bytes = b"Synthetic pinned export owner; never executed";
        std::fs::write(&executable, bytes).unwrap();
        let trusted = super::super::TrustedMpvExecutable::open(
            &executable,
            &format!("{:x}", Sha256::digest(bytes)),
        )
        .unwrap();
        let engine = if routed {
            NativeEpisodeEngine::new_with_scope(
                trusted,
                Some(1),
                Arc::new(UnusedArtifacts),
                super::super::guarded_command::HostScope::testing(),
            )
        } else {
            NativeEpisodeEngine::new_trusted_standalone(trusted, Arc::new(UnusedArtifacts))
        }
        .unwrap();
        Self {
            engine: Some(Arc::new(engine)),
            directory,
        }
    }
    fn engine(&self) -> &Arc<NativeEpisodeEngine> {
        self.engine.as_ref().unwrap()
    }
}
impl Drop for ExportFixture {
    fn drop(&mut self) {
        drop(self.engine.take());
        std::fs::remove_dir_all(&self.directory).unwrap();
    }
}
fn export_provenance() -> NativeResultProvenance {
    NativeResultProvenance {
        registry_generation: "skip-generation".into(),
        session: "skip-session".into(),
        pass: 1,
        segment: "segment-1".into(),
        expected_revision: 1,
    }
}
fn export_input(guard: NativeObservationGuard) -> ExportGuardedSeek {
    ExportGuardedSeek {
        seek: GuardedSeek {
            guard,
            position_us: DecimalU64(8_000_000),
        },
        export: PreparedNativeResultExport {
            provenance: export_provenance(),
        },
    }
}

fn wire_input(guard: NativeObservationGuard) -> v1::ExportGuardedSeekV1 {
    let origin = v1::NativeExportOriginV1::try_from(v1::NativeExportOriginInputV1 {
        registry_generation: "g".into(),
        session: "s".into(),
        pass: DecimalU64(1),
        segment: "t".into(),
        request_revision: DecimalU64(1),
        engine_instance_id: DecimalU64(7),
        resolution_artifact_id: "03".repeat(32),
    })
    .unwrap();
    let annotation = v1::NativeExportAnnotationV1::try_from(v1::NativeExportAnnotationInputV1 {
        annotation_revision: DecimalU64(2),
        verified_revision: DecimalU64(2),
        timebase: crate::domain::skip_segments::SegmentTimebase::EnginePlayback,
    })
    .unwrap();
    v1::ExportGuardedSeekV1 {
        seek: GuardedSeek {
            guard,
            position_us: DecimalU64(8_000_000),
        },
        export: v1::PreparedNativeResultExportV1::new(origin, annotation),
    }
}
fn seed_wire_scope(scope: &super::super::guarded_command::HostScope, nonce: u8) {
    scope.set_issuer_for_test(Ok(v1::NativeIssuerScopeV1::from_original_nonce(
        [nonce; 16],
    )
    .unwrap()));
}
#[test]
fn wire_routed_and_deferred_preserve_original_cell_and_reads_after_replacement() {
    let mut dispatcher = host(8);
    let first = ExportFixture::with_routed_scope(true);
    let second = ExportFixture::with_routed_scope(true);
    let router = ParentEngineRouter::new(dispatcher.handle());
    let route = router.prepare(first.engine().clone()).unwrap();
    router.activate(&route).unwrap();
    let scope = first.engine().guard_scope().upgrade().unwrap();
    seed_wire_scope(&scope, 11);
    assert_eq!(
        first.engine().native_export_issuer().unwrap_err().code,
        ErrorCode::Unsupported
    );
    assert_eq!(
        router.native_export_issuer().unwrap_err().code,
        ErrorCode::Unsupported
    );
    let issuer = router.capture_native_export_issuer(&route).unwrap();
    let deferred = super::super::DeferredEpisodeEngine::new(route.binding.clone(), route.epoch);
    let deferred_issuer = deferred.native_export_issuer().unwrap();
    assert_eq!(issuer.scope(), deferred_issuer.scope());
    let (guard, _link, mut receiver) =
        super::super::guarded_command::standalone_export_guard_for_test(&scope);
    let input = wire_input(guard);
    let origin = input.export.origin().clone();
    let output = router.submit_exported_guarded_seek_v1(input).unwrap();
    assert!(receiver.try_recv().is_ok());
    let (submission, reference, metadata) = output.into_parts();
    let original_rows = scope
        .registry
        .exported_references(NativeResultProvenance {
            registry_generation: "g".into(),
            session: "s".into(),
            pass: 1,
            segment: "t".into(),
            expected_revision: 1,
        })
        .unwrap();
    assert!(original_rows[0].ticket.shares_read_cell(&submission.ticket));
    assert_eq!(metadata.actual_target_us(), DecimalU64(8_000_000));
    assert!(submission.ticket.read_context().is_some());
    assert_eq!(
        submission
            .ticket
            .bind_read_context(Arc::new(()))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        first
            .engine()
            .guarded_seek_result(GuardedSeekResultQuery {
                ticket: submission.ticket.clone(),
                consume_terminal: false
            })
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let (guard, _link2, mut receiver2) =
        super::super::guarded_command::standalone_export_guard_for_test(&scope);
    let other = deferred
        .submit_exported_guarded_seek_v1(wire_input(guard))
        .unwrap();
    assert!(receiver2.try_recv().is_ok());
    assert_eq!(issuer.list(&origin).unwrap().items().len(), 2);
    assert_eq!(
        issuer.consume(reference, &origin).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    complete(&router, &route);
    let next = router.prepare(second.engine().clone()).unwrap();
    router.activate(&next).unwrap();
    assert!(issuer.peek(reference, &origin).is_ok());
    assert!(deferred_issuer.peek(other.reference(), &origin).is_ok());
    issuer.consume(reference, &origin).unwrap();
    deferred_issuer.consume(other.reference(), &origin).unwrap();
    assert!(issuer.list(&origin).unwrap().items().is_empty());
    assert_eq!(
        router
            .observe_native_guard_for_export(&issuer)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    complete(&router, &next);
    drop(next);
    drop(deferred);
    drop(route);
    drop(router);
    drop(scope);
    finish(&mut dispatcher);
}
#[test]
fn wire_foreign_binding_epoch_and_router_are_rejected_before_native_observe() {
    let mut dispatcher = host(8);
    let fixture = ExportFixture::with_routed_scope(true);
    let router = ParentEngineRouter::new(dispatcher.handle());
    let route = router.prepare(fixture.engine().clone()).unwrap();
    router.activate(&route).unwrap();
    let scope = fixture.engine().guard_scope().upgrade().unwrap();
    seed_wire_scope(&scope, 12);
    let issuer = router.capture_native_export_issuer(&route).unwrap();
    let other_router = ParentEngineRouter::new(dispatcher.handle());
    assert_eq!(
        other_router
            .capture_native_export_issuer(&route)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let epoch = std::num::NonZeroU64::new(route.epoch.get() + 1).unwrap();
    let binding = ParentBinding::new_scoped(epoch, Arc::downgrade(&scope)).unwrap();
    binding.bind(epoch, fixture.engine().clone()).unwrap();
    let other = super::super::DeferredEpisodeEngine::new(binding.clone(), epoch);
    let other_issuer = other.native_export_issuer().unwrap();
    assert_eq!(other_issuer.scope(), issuer.scope());
    assert_eq!(
        router
            .observe_native_guard_for_export(&other_issuer)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        other
            .observe_native_guard_for_export(&issuer)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let stale = super::super::DeferredEpisodeEngine::new(binding.clone(), route.epoch);
    assert_eq!(
        stale.native_export_issuer().unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    complete(&router, &route);
    drop(other);
    drop(stale);
    drop(binding);
    drop(other_router);
    drop(route);
    drop(router);
    drop(scope);
    finish(&mut dispatcher);
}
#[test]
fn wire_captured_issuer_keeps_only_weak_binding_and_router_owners() {
    let mut dispatcher = host(8);
    let fixture = ExportFixture::with_routed_scope(true);
    let router = ParentEngineRouter::new(dispatcher.handle());
    let route = router.prepare(fixture.engine().clone()).unwrap();
    router.activate(&route).unwrap();
    let scope = fixture.engine().guard_scope().upgrade().unwrap();
    seed_wire_scope(&scope, 13);
    let issuer = router.capture_native_export_issuer(&route).unwrap();
    let (guard, _link, mut receiver) =
        super::super::guarded_command::standalone_export_guard_for_test(&scope);
    let input = wire_input(guard);
    let origin = input.export.origin().clone();
    let output = router.submit_exported_guarded_seek_v1(input).unwrap();
    assert!(receiver.try_recv().is_ok());
    complete(&router, &route);
    issuer.peek(output.reference(), &origin).unwrap();
    let weak = Arc::downgrade(&router.shared);
    drop(route);
    drop(router);
    assert!(weak.upgrade().is_none());
    assert_eq!(
        issuer.peek(output.reference(), &origin).unwrap_err().code,
        ErrorCode::SourceExpired
    );
    assert_eq!(
        scope
            .registry
            .exported_references(NativeResultProvenance {
                registry_generation: "g".into(),
                session: "s".into(),
                pass: 1,
                segment: "t".into(),
                expected_revision: 1
            })
            .unwrap()
            .len(),
        1
    );
    drop(scope);
    finish(&mut dispatcher);
}
#[test]
fn wire_nonce_failure_and_disabled_or_unscoped_bindings_never_fall_back() {
    let mut dispatcher = host(8);
    let fixture = ExportFixture::with_routed_scope(true);
    let router = ParentEngineRouter::new(dispatcher.handle());
    let route = router.prepare(fixture.engine().clone()).unwrap();
    router.activate(&route).unwrap();
    let scope = fixture.engine().guard_scope().upgrade().unwrap();
    scope.set_issuer_for_test(Err(()));
    for _ in 0..2 {
        assert_eq!(
            router
                .capture_native_export_issuer(&route)
                .unwrap_err()
                .code,
            ErrorCode::EngineFailed
        );
    }
    let deferred = super::super::DeferredEpisodeEngine::new(route.binding.clone(), route.epoch);
    assert_eq!(
        deferred.native_export_issuer().unwrap_err().code,
        ErrorCode::EngineFailed
    );
    let fake = FakeEngine::new(42);
    let epoch = std::num::NonZeroU64::new(1).unwrap();
    let disabled = super::super::guarded_command::HostScope::disabled();
    let binding = ParentBinding::new_scoped(epoch, Arc::downgrade(&disabled)).unwrap();
    binding.bind(epoch, fake.clone()).unwrap();
    assert_eq!(
        super::super::DeferredEpisodeEngine::new(binding, epoch)
            .native_export_issuer()
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    let legacy = ParentBinding::new(epoch);
    legacy.bind(epoch, fake.clone()).unwrap();
    assert_eq!(
        super::super::DeferredEpisodeEngine::new(legacy, epoch)
            .native_export_issuer()
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    assert_eq!(fake.applied.load(Ordering::SeqCst), 0);
    complete(&router, &route);
    drop(deferred);
    drop(route);
    drop(router);
    drop(scope);
    finish(&mut dispatcher);
}

#[test]
fn wire_foreign_guard_and_failed_send_create_no_export_row_or_second_binding() {
    let mut dispatcher = host(8);
    let first = ExportFixture::with_routed_scope(true);
    let second = ExportFixture::with_routed_scope(true);
    let router = ParentEngineRouter::new(dispatcher.handle());
    let route = router.prepare(first.engine().clone()).unwrap();
    router.activate(&route).unwrap();
    let scope = first.engine().guard_scope().upgrade().unwrap();
    seed_wire_scope(&scope, 14);
    let issuer = router.capture_native_export_issuer(&route).unwrap();
    let foreign = second.engine().guard_scope().upgrade().unwrap();
    let (guard, _foreign_link, mut foreign_receiver) =
        super::super::guarded_command::standalone_export_guard_for_test(&foreign);
    let input = wire_input(guard);
    let origin = input.export.origin().clone();
    assert_eq!(
        router
            .submit_exported_guarded_seek_v1(input)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert!(foreign_receiver.try_recv().is_err());
    let (guard, _link, receiver) =
        super::super::guarded_command::standalone_export_guard_for_test(&scope);
    drop(receiver);
    assert!(
        router
            .submit_exported_guarded_seek_v1(wire_input(guard))
            .is_err()
    );
    assert!(issuer.list(&origin).unwrap().items().is_empty());
    complete(&router, &route);
    drop(route);
    drop(router);
    drop(scope);
    drop(foreign);
    finish(&mut dispatcher);
}
#[test]
fn routed_export_preserves_original_cell_after_route_replacement_and_rejects_raw_read() {
    let mut dispatcher = host(8);
    let first = ExportFixture::new();
    let second = ExportFixture::new();
    let router = ParentEngineRouter::new(dispatcher.handle());
    let old = router.prepare(first.engine().clone()).unwrap();
    router.activate(&old).unwrap();
    let scope = first.engine().guard_scope().upgrade().unwrap();
    let (guard, _link, mut receiver) =
        super::super::guarded_command::standalone_export_guard_for_test(&scope);
    let output = router
        .submit_exported_guarded_seek(export_input(guard))
        .unwrap();
    assert!(receiver.try_recv().is_ok());
    assert!(
        output
            .reference
            .ticket
            .shares_read_cell(&output.submission.ticket)
    );
    assert!(output.reference.ticket.read_context().is_some());
    assert_eq!(
        output
            .reference
            .ticket
            .bind_read_context(Arc::new(7_u64))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        first
            .engine()
            .guarded_seek_result(GuardedSeekResultQuery {
                ticket: output.submission.ticket.clone(),
                consume_terminal: false
            })
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        router
            .exported_seek_result(NativeExportResultQuery {
                reference: output.reference.clone(),
                consume_terminal: true
            })
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    assert_eq!(
        router
            .exported_seek_references(export_provenance())
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    complete(&router, &old);
    let current = router.prepare(second.engine().clone()).unwrap();
    router.activate(&current).unwrap();
    assert!(
        router
            .exported_seek_result(NativeExportResultQuery {
                reference: output.reference.clone(),
                consume_terminal: false
            })
            .unwrap()
            .state()
            .is_terminal()
    );
    router
        .exported_seek_result(NativeExportResultQuery {
            reference: output.reference.clone(),
            consume_terminal: true,
        })
        .unwrap();
    assert_eq!(
        router
            .exported_seek_result(NativeExportResultQuery {
                reference: output.reference,
                consume_terminal: true
            })
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    complete(&router, &current);
    drop(current);
    drop(old);
    drop(scope);
    drop(router);
    finish(&mut dispatcher);
}

#[test]
fn routed_export_foreign_and_duplicate_preparation_never_enqueue() {
    let mut dispatcher = host(8);
    let first = ExportFixture::new();
    let second = ExportFixture::new();
    let router = ParentEngineRouter::new(dispatcher.handle());
    let route = router.prepare(first.engine().clone()).unwrap();
    router.activate(&route).unwrap();
    let admission = route.binding.admit(route.epoch).unwrap();
    let scope = second.engine().guard_scope().upgrade().unwrap();
    let (guard, _link, mut receiver) =
        super::super::guarded_command::standalone_export_guard_for_test(&scope);
    assert_eq!(
        guard
            .prepare_routed_export(
                admission
                    .prepare_export_read_owner(Some(Arc::downgrade(&router.shared)))
                    .unwrap()
            )
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert!(receiver.try_recv().is_err());
    let original = first.engine().guard_scope().upgrade().unwrap();
    let (guard, _original_link, mut original_receiver) =
        super::super::guarded_command::standalone_export_guard_for_test(&original);
    let guard = guard
        .prepare_routed_export(
            admission
                .prepare_export_read_owner(Some(Arc::downgrade(&router.shared)))
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        guard
            .prepare_routed_export(
                admission
                    .prepare_export_read_owner(Some(Arc::downgrade(&router.shared)))
                    .unwrap()
            )
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert!(original_receiver.try_recv().is_err());
    drop(admission);
    complete(&router, &route);
    drop(route);
    drop(router);
    drop(scope);
    drop(original);
    finish(&mut dispatcher);
}

#[test]
fn routed_export_expired_router_and_revoked_owner_reject_before_enqueue() {
    let mut dispatcher = host(8);
    let fixture = ExportFixture::new();
    let router = ParentEngineRouter::new(dispatcher.handle());
    let route = router.prepare(fixture.engine().clone()).unwrap();
    router.activate(&route).unwrap();
    let scope = fixture.engine().guard_scope().upgrade().unwrap();
    let (guard, _link, mut receiver) =
        super::super::guarded_command::standalone_export_guard_for_test(&scope);
    let admission = route.binding.admit(route.epoch).unwrap();
    let guard = guard
        .prepare_routed_export(
            admission
                .prepare_export_read_owner(Some(Weak::<Shared>::new()))
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        fixture
            .engine()
            .submit_exported_guarded_seek(export_input(guard))
            .unwrap_err()
            .code,
        ErrorCode::SourceExpired
    );
    assert!(receiver.try_recv().is_err());
    let (guard, _link2, mut receiver2) =
        super::super::guarded_command::standalone_export_guard_for_test(&scope);
    let guard = guard
        .prepare_routed_export(
            admission
                .prepare_export_read_owner(Some(Arc::downgrade(&router.shared)))
                .unwrap(),
        )
        .unwrap();
    drop(admission);
    route.binding.revoke_admission();
    assert_eq!(
        fixture
            .engine()
            .submit_exported_guarded_seek(export_input(guard))
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    assert!(receiver2.try_recv().is_err());
    complete(&router, &route);
    drop(route);
    drop(router);
    drop(scope);
    finish(&mut dispatcher);
}

#[test]
fn deferred_export_uses_fixed_binding_and_reads_after_revoke_without_new_admission() {
    use crate::infrastructure::mpv::DeferredEpisodeEngine;
    let fixture = ExportFixture::new();
    let scope = fixture.engine().guard_scope().upgrade().unwrap();
    let epoch = std::num::NonZeroU64::new(1).unwrap();
    let binding = ParentBinding::new_scoped(epoch, Arc::downgrade(&scope)).unwrap();
    binding.bind(epoch, fixture.engine().clone()).unwrap();
    let deferred = DeferredEpisodeEngine::new(binding.clone(), epoch);
    let (guard, _link, mut receiver) =
        super::super::guarded_command::standalone_export_guard_for_test(&scope);
    let output = deferred
        .submit_exported_guarded_seek(export_input(guard))
        .unwrap();
    assert!(receiver.try_recv().is_ok());
    assert!(
        deferred
            .exported_seek_references(export_provenance())
            .unwrap()[0]
            .same_request(&output.reference)
    );
    let foreign = ParentBinding::new_scoped(epoch, Arc::downgrade(&scope)).unwrap();
    let other = DeferredEpisodeEngine::new(foreign, epoch);
    assert_eq!(
        other
            .exported_seek_result(NativeExportResultQuery {
                reference: output.reference.clone(),
                consume_terminal: false
            })
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    binding.revoke_admission();
    fixture.engine().shutdown().unwrap();
    deferred
        .exported_seek_result(NativeExportResultQuery {
            reference: output.reference.clone(),
            consume_terminal: true,
        })
        .unwrap();
    assert_eq!(
        deferred
            .exported_seek_result(NativeExportResultQuery {
                reference: output.reference,
                consume_terminal: true
            })
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    drop(other);
    drop(deferred);
    drop(binding);
    drop(scope);
}

#[test]
fn routed_export_weak_owner_expires_and_foreign_router_cannot_recover_it() {
    let mut dispatcher = host(8);
    let mut fixture = ExportFixture::new();
    let second = ExportFixture::new();
    let router = ParentEngineRouter::new(dispatcher.handle());
    let foreign = ParentEngineRouter::new(dispatcher.handle());
    let route = router.prepare(fixture.engine().clone()).unwrap();
    router.activate(&route).unwrap();
    let scope = fixture.engine().guard_scope().upgrade().unwrap();
    let weak = Arc::downgrade(&scope);
    let strong = Arc::strong_count(fixture.engine());
    let (guard, link, mut receiver) =
        super::super::guarded_command::standalone_export_guard_for_test(&scope);
    let output = router
        .submit_exported_guarded_seek(export_input(guard))
        .unwrap();
    assert!(receiver.try_recv().is_ok());
    assert_eq!(Arc::strong_count(fixture.engine()), strong);
    assert_eq!(
        foreign
            .exported_seek_result(NativeExportResultQuery {
                reference: output.reference.clone(),
                consume_terminal: false
            })
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    complete(&router, &route);
    let current = router.prepare(second.engine().clone()).unwrap();
    router.activate(&current).unwrap();
    drop(route);
    drop(scope);
    drop(link);
    drop(fixture.engine.take());
    let deadline = Instant::now() + LIMIT;
    while router.shared.dispatcher.status().outstanding != 0 {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert!(weak.upgrade().is_none());
    assert_eq!(
        router
            .exported_seek_result(NativeExportResultQuery {
                reference: output.reference,
                consume_terminal: false
            })
            .unwrap_err()
            .code,
        ErrorCode::SourceExpired
    );
    complete(&router, &current);
    drop(current);
    drop(router);
    drop(foreign);
    finish(&mut dispatcher);
}

#[test]
fn routed_export_keeps_original64_and_terminal_consume_has_one_winner() {
    let mut dispatcher = host(8);
    let fixture = ExportFixture::new();
    let router = ParentEngineRouter::new(dispatcher.handle());
    let route = router.prepare(fixture.engine().clone()).unwrap();
    router.activate(&route).unwrap();
    let scope = fixture.engine().guard_scope().upgrade().unwrap();
    let mut outputs = Vec::new();
    for _ in 0..64 {
        let (guard, _link, mut receiver) =
            super::super::guarded_command::standalone_export_guard_for_test(&scope);
        outputs.push(
            router
                .submit_exported_guarded_seek(export_input(guard))
                .unwrap(),
        );
        assert!(receiver.try_recv().is_ok());
    }
    assert_eq!(
        scope
            .registry
            .exported_references(export_provenance())
            .unwrap()
            .len(),
        64
    );
    assert_eq!(
        scope
            .registry
            .reserve(super::super::guarded_command::Identity {
                engine: 7,
                resolution: [3; 32],
                connection: 1,
                lifecycle: 2,
                entry: Some(9)
            })
            .unwrap_err()
            .code,
        ErrorCode::ResourceExhausted
    );
    assert_eq!(
        router
            .exported_seek_result(NativeExportResultQuery {
                reference: outputs[0].reference.clone(),
                consume_terminal: true
            })
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    complete(&router, &route);
    let mut workers = Vec::new();
    for _ in 0..2 {
        let reader = router.clone();
        let reference = outputs[0].reference.clone();
        workers.push(std::thread::spawn(move || {
            reader.exported_seek_result(NativeExportResultQuery {
                reference,
                consume_terminal: true,
            })
        }));
    }
    let outcomes: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter(|result| matches!(result,Err(error) if error.code==ErrorCode::NotFound))
            .count(),
        1
    );
    assert_eq!(
        scope
            .registry
            .exported_references(export_provenance())
            .unwrap()
            .len(),
        63
    );
    drop(route);
    drop(router);
    drop(scope);
    finish(&mut dispatcher);
}

#[test]
fn routed_export_without_original_scope_or_active_route_has_no_native_fallback() {
    let mut dispatcher = host(8);
    let fixture = ExportFixture::new();
    let router = ParentEngineRouter::new(dispatcher.handle());
    let scope = fixture.engine().guard_scope().upgrade().unwrap();
    let (guard, _link, mut receiver) =
        super::super::guarded_command::standalone_export_guard_for_test(&scope);
    assert_eq!(
        router
            .submit_exported_guarded_seek(export_input(guard))
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    assert!(receiver.try_recv().is_err());
    let fake = FakeEngine::new(900);
    let route = ready(&router, &fake);
    let (guard, _link2, mut receiver2) =
        super::super::guarded_command::standalone_export_guard_for_test(&scope);
    assert_eq!(
        router
            .submit_exported_guarded_seek(export_input(guard))
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    assert!(receiver2.try_recv().is_err());
    assert_eq!(fake.applied.load(Ordering::SeqCst), 0);
    complete(&router, &route);
    drop(route);
    drop(router);
    drop(scope);
    finish(&mut dispatcher);
}

#[test]
fn routed_export_send_failure_drops_tentative_original_row_and_read_owner() {
    let mut dispatcher = host(8);
    let fixture = ExportFixture::new();
    let router = ParentEngineRouter::new(dispatcher.handle());
    let route = router.prepare(fixture.engine().clone()).unwrap();
    router.activate(&route).unwrap();
    let scope = fixture.engine().guard_scope().upgrade().unwrap();
    let strong = Arc::strong_count(fixture.engine());
    let (guard, _link, receiver) =
        super::super::guarded_command::standalone_export_guard_for_test(&scope);
    drop(receiver);
    assert!(
        router
            .submit_exported_guarded_seek(export_input(guard))
            .is_err()
    );
    assert!(
        scope
            .registry
            .exported_references(export_provenance())
            .unwrap()
            .is_empty()
    );
    assert_eq!(Arc::strong_count(fixture.engine()), strong);
    complete(&router, &route);
    drop(route);
    drop(router);
    drop(scope);
    finish(&mut dispatcher);
}
struct Gate {
    entered: mpsc::Sender<()>,
    resume: mpsc::Receiver<()>,
}
impl Gate {
    fn wait(self) {
        self.entered.send(()).unwrap();
        self.resume.recv_timeout(LIMIT).unwrap();
    }
}
fn gate() -> (Gate, mpsc::Receiver<()>, mpsc::Sender<()>) {
    let (tx, entered) = mpsc::channel();
    let (resume, rx) = mpsc::channel();
    (
        Gate {
            entered: tx,
            resume: rx,
        },
        entered,
        resume,
    )
}
struct ExitGate(Gate);
impl Drop for ExitGate {
    fn drop(&mut self) {
        self.0.entered.send(()).unwrap();
        self.0.resume.recv_timeout(LIMIT).unwrap();
    }
}
thread_local! {static EXIT_GATE:RefCell<Option<ExitGate>>=const {RefCell::new(None)};}
struct FakeEngine {
    id: u64,
    active: AtomicBool,
    shutdowns: AtomicUsize,
    step: AtomicUsize,
    applied: AtomicUsize,
    gates: [Mutex<Option<Gate>>; 3],
    exit: Mutex<Option<ExitGate>>,
    router: Mutex<Weak<Shared>>,
}
impl FakeEngine {
    fn new(id: u64) -> Arc<Self> {
        Arc::new(Self {
            id,
            active: AtomicBool::new(false),
            shutdowns: AtomicUsize::new(0),
            step: AtomicUsize::new(2),
            applied: AtomicUsize::new(0),
            gates: std::array::from_fn(|_| Mutex::new(None)),
            exit: Mutex::new(None),
            router: Mutex::new(Weak::new()),
        })
    }
    fn call(&self, index: usize) {
        if let Some(router) = self.router.lock().unwrap().upgrade() {
            assert!(
                router.state.try_lock().is_ok(),
                "Engine I/O retained the router lock"
            );
        }
        let gate = self.gates[index].lock().unwrap().take();
        if let Some(gate) = gate {
            gate.wait();
        }
    }
    fn require(&self, id: u64) -> Result<(), ApiError> {
        if id != self.id {
            return Err(conflict("Synthetic instance changed"));
        }
        if !self.active.load(Ordering::SeqCst) {
            return Err(ApiError::new(
                ErrorCode::NotFound,
                "Synthetic engine is not active",
            ));
        }
        Ok(())
    }
}
impl EpisodePlaybackEngine for FakeEngine {
    fn options_available(&self) -> bool {
        true
    }
    fn capture_frame_available(&self) -> bool {
        true
    }
    fn open(
        &self,
        source: EpisodePlaybackSource,
        _: bool,
        _: f64,
    ) -> Result<OpenedEpisodePlayback, ApiError> {
        self.call(0);
        self.active
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| conflict("Synthetic already active"))?;
        Ok(OpenedEpisodePlayback {
            engine_instance_id: DecimalU64(self.id),
            resolution_artifact_id: source.artifact_id().into(),
            completion: ControlCompletion::Unobserved,
        })
    }
    fn close(&self, input: CloseEpisodePlayback) -> Result<ClosedEpisodePlayback, ApiError> {
        self.require(input.engine_instance_id.0)?;
        self.call(2);
        self.active.store(false, Ordering::SeqCst);
        self.applied.fetch_add(1, Ordering::SeqCst);
        Ok(closed(self.id, true))
    }
    fn observe(&self) -> Result<EpisodePlaybackObservation, ApiError> {
        self.require(self.id)?;
        self.call(1);
        Ok(EpisodePlaybackObservation {
            engine_instance_id: DecimalU64(self.id),
            resolution_artifact_id: "a".repeat(64),
            playback: PlaybackStatus {
                origin: PlaybackStatusOrigin::NativeObservation,
                published_age_ms: DecimalU64(0),
                connected: true,
                paused: Some(true),
                idle: Some(false),
                position_us: None,
                duration_us: None,
                volume_percent: None,
                muted: None,
                video_size: None,
                audio_channels: None,
                has_error: false,
            },
            diagnostics: None,
            options: None,
        })
    }
    fn control(&self, input: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
        self.require(input.engine_instance_id.0)?;
        self.applied.fetch_add(1, Ordering::SeqCst);
        Ok(ControlReceipt::enqueued(self.id))
    }
    fn set_rate(&self, input: SetEpisodeRate) -> Result<ControlReceipt, ApiError> {
        self.require(input.engine_instance_id.0)?;
        self.applied.fetch_add(1, Ordering::SeqCst);
        Ok(ControlReceipt::enqueued(self.id))
    }
    fn select_track(&self, input: SelectEpisodeTrack) -> Result<TrackControlReceipt, ApiError> {
        self.require(input.engine_instance_id.0)?;
        self.applied.fetch_add(1, Ordering::SeqCst);
        Ok(TrackControlReceipt::enqueued(
            ControlReceipt::enqueued(self.id),
            input.expected_tracks_revision,
        ))
    }
    fn capture_frame(&self, input: CaptureEpisodeFrame) -> Result<CapturedEpisodeFrame, ApiError> {
        self.require(input.engine_instance_id.0)?;
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Synthetic capture never writes a file",
        ))
    }
}
impl ParentOwnedEngine for FakeEngine {
    fn shutdown_for_parent(&self) -> Result<Option<ClosedEpisodePlayback>, ApiError> {
        self.shutdowns.fetch_add(1, Ordering::SeqCst);
        if let Some(exit) = self.exit.lock().unwrap().take() {
            EXIT_GATE.with(|slot| *slot.borrow_mut() = Some(exit));
        }
        match self.step.fetch_add(1, Ordering::SeqCst) {
            0 => Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Synthetic shutdown failure",
            )),
            1 => Ok(Some(closed(self.id, false))),
            _ => Ok(self
                .active
                .swap(false, Ordering::SeqCst)
                .then(|| closed(self.id, true))),
        }
    }
}
fn closed(id: u64, complete: bool) -> ClosedEpisodePlayback {
    ClosedEpisodePlayback {
        engine_instance_id: DecimalU64(id),
        engine_exited: complete,
        ipc_worker_exited: complete,
        stdout_worker_exited: complete,
        stderr_worker_exited: complete,
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
            complete,
            truncated: false,
            read_failed: false,
        },
    }
}
fn source() -> EpisodePlaybackSource {
    EpisodePlaybackSource::synthetic_replacement_source(RequestReferer::Absent)
}
fn host(capacity: usize) -> DispatcherHost {
    DispatcherHost::start(NonZeroUsize::new(capacity).unwrap()).unwrap()
}
fn finish(host: &mut DispatcherHost) {
    host.request_shutdown();
    let deadline = Instant::now() + LIMIT;
    loop {
        match host.poll_shutdown() {
            DispatcherShutdown::Joined {
                worker_panicked: false,
                faults: 0,
            } => break,
            DispatcherShutdown::Pending => {
                assert!(Instant::now() < deadline);
                std::thread::yield_now();
            }
            other => panic!("Unexpected dispatcher state: {other:?}"),
        }
    }
}
fn quiet() -> CompletionWake {
    Arc::new(|| Ok(()))
}
fn wake() -> (CompletionWake, mpsc::Receiver<()>) {
    let (tx, rx) = mpsc::channel();
    (Arc::new(move || tx.send(()).map_err(|e| e.to_string())), rx)
}
fn prepare(
    router: &ParentEngineRouter,
    engine: &Arc<FakeEngine>,
) -> Result<ParentRouteHandle, ApiError> {
    *engine.router.lock().unwrap() = Arc::downgrade(&router.shared);
    let owner: Arc<dyn ParentOwnedEngine> = engine.clone();
    router.prepare_engine(&owner)
}
fn ready(router: &ParentEngineRouter, engine: &Arc<FakeEngine>) -> ParentRouteHandle {
    let route = prepare(router, engine).unwrap();
    router.activate(&route).unwrap();
    route
}

#[test]
fn original_routed_ticket_reads_after_new_route_and_rejects_foreign_router_or_binding() {
    use crate::infrastructure::mpv::{
        DeferredEpisodeEngine,
        guarded_command::{Identity, Registry},
    };
    let mut host = host(2);
    let router = ParentEngineRouter::new(host.handle());
    let other = ParentEngineRouter::new(host.handle());
    let first = FakeEngine::new(100);
    let old = ready(&router, &first);
    let registry = Registry::default();
    let row = registry
        .reserve(Identity {
            engine: 100,
            resolution: [3; 32],
            connection: 1,
            lifecycle: 1,
            entry: Some(9),
        })
        .unwrap();
    let raw = row
        .submission(crate::application::playback_control::ControlReceipt::enqueued(100))
        .unwrap();
    let original = raw.ticket.clone();
    let admission = router.admitted().unwrap();
    let submission = admission
        .bind_result(raw, Some(Arc::downgrade(&router.shared)))
        .unwrap();
    drop(admission);
    assert!(original.same_request(&submission.ticket));
    let ticket = submission.ticket;
    complete(&router, &old);
    let next = FakeEngine::new(200);
    let current = ready(&router, &next);
    assert_eq!(
        other
            .guarded_seek_result(GuardedSeekResultQuery {
                ticket: ticket.clone(),
                consume_terminal: true
            })
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let foreign_binding = DeferredEpisodeEngine::new(current.binding.clone(), current.epoch);
    assert_eq!(
        foreign_binding
            .guarded_seek_result(GuardedSeekResultQuery {
                ticket: ticket.clone(),
                consume_terminal: true
            })
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let result = router
        .guarded_seek_result(GuardedSeekResultQuery {
            ticket: ticket.clone(),
            consume_terminal: false,
        })
        .unwrap();
    assert_eq!(
        result.original_identity().engine_instance_id(),
        DecimalU64(100)
    );
    assert_eq!(result.state(), GuardedSeekState::Queued);
    assert_eq!(next.applied.load(Ordering::SeqCst), 0);
    drop(old);
    assert_eq!(
        router
            .guarded_seek_result(GuardedSeekResultQuery {
                ticket,
                consume_terminal: false
            })
            .unwrap_err()
            .code,
        ErrorCode::SourceExpired
    );
    complete(&router, &current);
    finish(&mut host);
}

#[test]
fn prepared_scoped_route_cannot_observe_before_activation_and_old_revoke_never_closes_new_word() {
    use crate::infrastructure::mpv::guarded_command::HostScope;
    let mut host = host(2);
    let router = ParentEngineRouter::new(host.handle());
    let old_scope = HostScope::testing();
    let first: Arc<dyn ParentOwnedEngine> = FakeEngine::new(100);
    let old = router
        .prepare_engine_scoped(&first, Some(Arc::downgrade(&old_scope)))
        .unwrap();
    assert_eq!(
        router.observe_native_guard().unwrap_err().code,
        ErrorCode::NeedsInput
    );
    router.activate(&old).unwrap();
    complete(&router, &old);
    assert!(old_scope.word.is_closed());
    let new_scope = HostScope::testing();
    let second: Arc<dyn ParentOwnedEngine> = FakeEngine::new(200);
    let current = router
        .prepare_engine_scoped(&second, Some(Arc::downgrade(&new_scope)))
        .unwrap();
    router.activate(&current).unwrap();
    old.binding.revoke_admission();
    assert!(!new_scope.word.is_closed());
    assert_eq!(
        current
            .binding
            .admit(current.epoch)
            .unwrap()
            .engine
            .observe_native_guard()
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    complete(&router, &current);
    finish(&mut host);
}

#[test]
fn scoped_word_revoke_precedes_held_router_state_lock() {
    use crate::infrastructure::mpv::guarded_command::HostScope;
    let mut host = host(2);
    let router = ParentEngineRouter::new(host.handle());
    let scope = HostScope::testing();
    let engine: Arc<dyn ParentOwnedEngine> = FakeEngine::new(100);
    let route = router
        .prepare_engine_scoped(&engine, Some(Arc::downgrade(&scope)))
        .unwrap();
    router.activate(&route).unwrap();
    let held = router.shared.state.lock().unwrap();
    let retiring = router.clone();
    let original = route.clone();
    let worker = std::thread::spawn(move || retiring.retire(&original, quiet()));
    let until = Instant::now() + LIMIT;
    while !scope.word.is_closed() {
        assert!(Instant::now() < until);
        std::thread::yield_now();
    }
    drop(held);
    let start = worker.join().unwrap().unwrap();
    while !matches!(
        router.observe_retirement(&start.handle).unwrap().progress,
        RetirementProgress::Complete(_)
    ) {
        assert!(Instant::now() < until);
        std::thread::yield_now();
    }
    finish(&mut host);
}
fn complete(router: &ParentEngineRouter, route: &ParentRouteHandle) {
    // Test driver explicitly waits for the preceding notification job to drain.
    let deadline = Instant::now() + LIMIT;
    while router.shared.dispatcher.status().outstanding != 0 {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    let (wake, rx) = wake();
    let start = router.retire(route, wake).unwrap();
    assert!(start.dispatch_error.is_none());
    rx.recv_timeout(LIMIT).unwrap();
    assert!(matches!(
        router.observe_retirement(&start.handle).unwrap().progress,
        RetirementProgress::Complete(_)
    ));
    drop(start.handle);
}
struct UnusedResults;
impl EpisodeResolutionResults for UnusedResults {
    fn publish(
        &self,
        _: &StoredEpisodeResolution,
        _: &CaptureMetadata,
    ) -> Result<VerifiedJobOutput, ApiError> {
        panic!("Router must not publish resolution data")
    }
    fn read(&self, _: &str) -> Result<StoredEpisodeResolution, ApiError> {
        Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Synthetic router test does not resolve media",
        ))
    }
}
struct UnusedArtifacts;
impl crate::application::catalog::CatalogArtifacts for UnusedArtifacts {
    fn save(&self, _: &[u8], _: &CaptureMetadata) -> Result<RawArtifact, ApiError> {
        panic!("Rejected native preparation must not write diagnostics")
    }
}

#[test]
fn same_sdk_engine_port_observes_two_generations_without_rebuilding_client() {
    // Production lifecycle accepts only NativeEpisodeEngine; no public fake constructor.
    type PrepareNative = fn(
        &ParentEngineRouter,
        Arc<NativeEpisodeEngine>,
    ) -> Result<ParentRouteHandle, Box<RoutePreparationFailure>>;
    let _native_signature: PrepareNative = ParentEngineRouter::prepare;
    let mut host = host(2);
    let router = ParentEngineRouter::new(host.handle());
    let port: Arc<dyn EpisodePlaybackEngine> = Arc::new(router.clone());
    let sdk = PlayerSdk::default().with_episode_playback(EpisodePlaybackSessions::new(
        Arc::new(UnusedResults),
        port.clone(),
    ));
    assert_eq!(
        sdk.execute(Request::new("playback.engine"))
            .error
            .unwrap()
            .code,
        ErrorCode::NeedsInput
    );
    let a = FakeEngine::new(100);
    let route = ready(&router, &a);
    port.open(source(), true, 50.).unwrap();
    assert_eq!(
        sdk.execute(Request::new("playback.engine")).data.unwrap()["engine_instance_id"],
        "100"
    );
    complete(&router, &route);
    let b = FakeEngine::new(200);
    let route = ready(&router, &b);
    port.open(source(), true, 50.).unwrap();
    assert_eq!(
        sdk.execute(Request::new("playback.engine")).data.unwrap()["engine_instance_id"],
        "200"
    );
    complete(&router, &route);
    finish(&mut host);
}

#[test]
fn not_ready_and_retired_states_differ_and_abandoned_prepared_route_can_be_cleaned() {
    let mut host = host(1);
    let router = ParentEngineRouter::new(host.handle());
    assert_eq!(router.status().unwrap(), RouterStatus::EmptyNeverReady);
    assert!(router.current_route().unwrap().is_none());
    assert_eq!(router.observe().unwrap_err().code, ErrorCode::NeedsInput);
    assert!(!router.options_available());
    let engine = FakeEngine::new(100);
    let weak = Arc::downgrade(&engine);
    let route = prepare(&router, &engine).unwrap();
    drop(route);
    drop(engine);
    assert!(weak.upgrade().is_some());
    assert!(matches!(
        router.status().unwrap(),
        RouterStatus::Prepared { .. }
    ));
    assert_eq!(
        router.open(source(), false, 50.).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    let route = router.current_route().unwrap().unwrap();
    complete(&router, &route);
    assert_eq!(router.observe().unwrap_err().code, ErrorCode::Cancelled);
    assert!(!router.capture_frame_available());
    finish(&mut host);
}

#[test]
fn retirement_atomic_revoke_precedes_waiting_for_router_state_lock() {
    let mut host = host(1);
    let router = ParentEngineRouter::new(host.handle());
    let engine = FakeEngine::new(100);
    let route = ready(&router, &engine);
    let held = router.shared.state.lock().unwrap();
    let retiring = router.clone();
    let token = route.clone();
    let worker = std::thread::spawn(move || retiring.retire(&token, quiet()));
    let deadline = Instant::now() + LIMIT;
    loop {
        match route.binding.admit(route.epoch) {
            Err(error) if error.code == ErrorCode::Cancelled => break,
            Ok(owner) => drop(owner),
            other => panic!("Unexpected admission: {}", other.is_ok()),
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert_eq!(engine.shutdowns.load(Ordering::SeqCst), 0);
    drop(held);
    let start = worker.join().unwrap().unwrap();
    let deadline = Instant::now() + LIMIT;
    while !matches!(
        router.observe_retirement(&start.handle).unwrap().progress,
        RetirementProgress::Complete(_)
    ) {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    finish(&mut host);
}

#[test]
fn already_admitted_open_retains_original_owner_and_blocks_next_generation() {
    let mut host = host(1);
    let router = ParentEngineRouter::new(host.handle());
    let engine = FakeEngine::new(100);
    let route = ready(&router, &engine);
    let (gate, entered, resume) = gate();
    *engine.gates[0].lock().unwrap() = Some(gate);
    let caller = router.clone();
    let opening = std::thread::spawn(move || caller.open(source(), true, 50.));
    entered.recv_timeout(LIMIT).unwrap();
    let (wake, done) = wake();
    let retired = router.retire(&route, wake).unwrap();
    assert_eq!(
        router.open(source(), false, 50.).unwrap_err().code,
        ErrorCode::Cancelled
    );
    assert_eq!(engine.shutdowns.load(Ordering::SeqCst), 0);
    let next = FakeEngine::new(200);
    let weak = Arc::downgrade(&next);
    assert_eq!(
        prepare(&router, &next).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    assert!(weak.upgrade().is_some());
    resume.send(()).unwrap();
    assert_eq!(opening.join().unwrap().unwrap().engine_instance_id.0, 100);
    done.recv_timeout(LIMIT).unwrap();
    assert!(matches!(
        router.observe_retirement(&retired.handle).unwrap().progress,
        RetirementProgress::Complete(_)
    ));
    let next_route = ready(&router, &next);
    complete(&router, &next_route);
    finish(&mut host);
}

#[test]
fn concurrent_observe_and_close_use_old_admissions_without_holding_router_lock() {
    let mut host = host(1);
    let router = ParentEngineRouter::new(host.handle());
    let engine = FakeEngine::new(100);
    let route = ready(&router, &engine);
    router.open(source(), false, 50.).unwrap();
    let (observe_gate, observe_entered, observe_resume) = gate();
    let (close_gate, close_entered, close_resume) = gate();
    *engine.gates[1].lock().unwrap() = Some(observe_gate);
    *engine.gates[2].lock().unwrap() = Some(close_gate);
    let view = router.clone();
    let observation = std::thread::spawn(move || view.observe());
    observe_entered.recv_timeout(LIMIT).unwrap();
    let closer = router.clone();
    let closing = std::thread::spawn(move || {
        closer.close(CloseEpisodePlayback {
            engine_instance_id: DecimalU64(100),
        })
    });
    close_entered.recv_timeout(LIMIT).unwrap();
    let (wake, done) = wake();
    let retired = router.retire(&route, wake).unwrap();
    assert_eq!(engine.shutdowns.load(Ordering::SeqCst), 0);
    observe_resume.send(()).unwrap();
    close_resume.send(()).unwrap();
    done.recv_timeout(LIMIT).unwrap();
    router.observe_retirement(&retired.handle).unwrap();
    let next = FakeEngine::new(200);
    let token = ready(&router, &next);
    assert_eq!(
        observation.join().unwrap().unwrap().engine_instance_id.0,
        100
    );
    assert_eq!(closing.join().unwrap().unwrap().engine_instance_id.0, 100);
    complete(&router, &token);
    finish(&mut host);
}

#[test]
fn foreign_and_stale_handles_never_revoke_or_activate_current_other_route() {
    let mut host = host(2);
    let a = ParentEngineRouter::new(host.handle());
    let b = ParentEngineRouter::new(host.handle());
    let ea = FakeEngine::new(100);
    let eb = FakeEngine::new(200);
    let old = ready(&a, &ea);
    let foreign = ready(&b, &eb);
    assert_eq!(
        a.retire(&foreign, quiet()).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert!(b.options_available());
    assert_eq!(
        a.activate(&foreign).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    complete(&a, &old);
    let next = FakeEngine::new(300);
    let current = ready(&a, &next);
    assert_eq!(
        a.retire(&old, quiet()).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        a.activate(&old).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert!(a.options_available());
    complete(&a, &current);
    complete(&b, &foreign);
    finish(&mut host);
}

#[test]
fn queue_full_preserves_attempt_for_retry_and_closed_dispatch_retains_owner() {
    let mut host = host(1);
    let (block, entered, resume) = gate();
    let mut blocker = host
        .handle()
        .submit(JoinRequest::new(move || block.wait(), quiet()))
        .unwrap();
    entered.recv_timeout(LIMIT).unwrap();
    let mut queued = host
        .handle()
        .submit(JoinRequest::new(|| (), quiet()))
        .unwrap();
    let router = ParentEngineRouter::new(host.handle());
    let engine = FakeEngine::new(100);
    let route = ready(&router, &engine);
    let start = router.retire(&route, quiet()).unwrap();
    assert_eq!(
        start.dispatch_error.as_ref().unwrap().code,
        ErrorCode::ResourceExhausted
    );
    let Some(RetirementFailure::Dispatch(error)) = router
        .observe_retirement(&start.handle)
        .unwrap()
        .retained_failure
    else {
        panic!("Dispatch error lost")
    };
    assert_eq!(error.code, ErrorCode::ResourceExhausted);
    assert_eq!(engine.shutdowns.load(Ordering::SeqCst), 0);
    resume.send(()).unwrap();
    let deadline = Instant::now() + LIMIT;
    let mut joined = false;
    let mut queued_joined = false;
    loop {
        if !joined && let Some(result) = blocker.try_take() {
            result.unwrap();
            joined = true;
        }
        if !queued_joined && let Some(result) = queued.try_take() {
            result.unwrap();
            queued_joined = true;
        }
        if joined && queued_joined && host.handle().status().outstanding == 0 {
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    let (wake, done) = wake();
    router.retry_retirement(&start.handle, wake).unwrap();
    done.recv_timeout(LIMIT).unwrap();
    assert!(matches!(
        router.observe_retirement(&start.handle).unwrap().progress,
        RetirementProgress::Complete(_)
    ));
    assert_eq!(engine.shutdowns.load(Ordering::SeqCst), 1);
    finish(&mut host);
    let mut closed_host = self::host(1);
    let closed = ParentEngineRouter::new(closed_host.handle());
    finish(&mut closed_host);
    let engine = FakeEngine::new(400);
    let weak = Arc::downgrade(&engine);
    let route = ready(&closed, &engine);
    drop(engine);
    let start = closed.retire(&route, quiet()).unwrap();
    assert_eq!(start.dispatch_error.unwrap().code, ErrorCode::Cancelled);
    drop(start.handle);
    drop(route);
    assert!(weak.upgrade().is_some());
    assert_eq!(weak.upgrade().unwrap().shutdowns.load(Ordering::SeqCst), 0);
}

#[test]
fn original_task_preserves_error_and_incomplete_receipt_across_explicit_retries() {
    let mut host = host(2);
    let router = ParentEngineRouter::new(host.handle());
    let engine = FakeEngine::new(100);
    engine.step.store(0, Ordering::SeqCst);
    let route = ready(&router, &engine);
    router.open(source(), false, 50.).unwrap();
    let (wake, done) = self::wake();
    let start = router.retire(&route, wake).unwrap();
    let identity = Arc::as_ptr(&start.handle.owner);
    done.recv_timeout(LIMIT).unwrap();
    let Some(RetirementFailure::Engine(error)) = router
        .observe_retirement(&start.handle)
        .unwrap()
        .retained_failure
    else {
        panic!("Engine error lost")
    };
    assert_eq!(error.code, ErrorCode::EngineFailed);
    let (wake, done) = self::wake();
    router.retry_retirement(&start.handle, wake).unwrap();
    done.recv_timeout(LIMIT).unwrap();
    let failed = router.observe_retirement(&start.handle).unwrap();
    let Some(RetirementFailure::Incomplete(receipt)) = failed.retained_failure else {
        panic!("typed receipt missing")
    };
    assert!(!receipt.ipc_worker_exited && !receipt.stdout_worker_exited);
    assert_eq!(receipt.engine_instance_id.0, 100);
    let (wake, done) = self::wake();
    router.retry_retirement(&start.handle, wake).unwrap();
    done.recv_timeout(LIMIT).unwrap();
    let complete = router.observe_retirement(&start.handle).unwrap();
    let RetirementProgress::Complete(Some(receipt)) = &complete.progress else {
        panic!("Complete receipt lost")
    };
    assert_eq!(receipt.engine_instance_id.0, 100);
    assert!(
        receipt.engine_exited
            && receipt.ipc_worker_exited
            && receipt.stdout_worker_exited
            && receipt.stderr_worker_exited
    );
    assert!(matches!(
        complete.retained_failure,
        Some(RetirementFailure::Incomplete(_))
    ));
    assert_eq!(Arc::as_ptr(&start.handle.owner), identity);
    assert_eq!(engine.shutdowns.load(Ordering::SeqCst), 3);
    assert_eq!(
        router
            .retry_retirement(&start.handle, quiet())
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    finish(&mut host);
}

#[test]
fn completion_metadata_cannot_enable_prepare_before_actual_worker_join_and_final_wake() {
    let mut host = host(1);
    let router = ParentEngineRouter::new(host.handle());
    let engine = FakeEngine::new(100);
    let route = ready(&router, &engine);
    router.open(source(), true, 50.).unwrap();
    let (exit, entered, resume) = gate();
    *engine.exit.lock().unwrap() = Some(ExitGate(exit));
    let (wake, done) = wake();
    let start = router.retire(&route, wake).unwrap();
    entered.recv_timeout(LIMIT).unwrap();
    let binding = route.binding.clone();
    let epoch = route.epoch;
    let metadata = std::thread::spawn(move || binding.cleanup(epoch))
        .join()
        .unwrap()
        .unwrap();
    assert!(matches!(
        metadata,
        super::super::ParentCleanupStatus::Complete(_)
    ));
    assert!(matches!(
        router.observe_retirement(&start.handle).unwrap().progress,
        RetirementProgress::Pending
    ));
    assert!(done.try_recv().is_err());
    assert_eq!(
        prepare(&router, &FakeEngine::new(200)).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    resume.send(()).unwrap();
    done.recv_timeout(LIMIT).unwrap();
    assert!(matches!(
        router.observe_retirement(&start.handle).unwrap().progress,
        RetirementProgress::Complete(_)
    ));
    finish(&mut host);
}

#[test]
fn late_previous_callback_and_retry_keep_distinct_wakes_outside_task_locks() {
    for capacity in [1, 2] {
        let mut host = host(capacity);
        let router = ParentEngineRouter::new(host.handle());
        let engine = FakeEngine::new(100);
        engine.step.store(0, Ordering::SeqCst);
        let route = ready(&router, &engine);
        let (block, entered, resume) = gate();
        let block = Mutex::new(Some(block));
        let old_wake: CompletionWake = Arc::new(move || {
            block.lock().unwrap().take().unwrap().wait();
            Ok(())
        });
        let start = router.retire(&route, old_wake).unwrap();
        entered.recv_timeout(LIMIT).unwrap();
        assert!(matches!(
            router.observe_retirement(&start.handle).unwrap().progress,
            RetirementProgress::RetryableFailure
        ));
        engine.step.store(2, Ordering::SeqCst);
        // The running callback has already left the queue. Fill its one waiting
        // slot explicitly; capacity two still accepts the retry behind this job.
        let mut queued = host
            .handle()
            .submit(JoinRequest::new(|| (), quiet()))
            .unwrap();
        let (next_wake, done) = wake();
        let attempted = router.retry_retirement(&start.handle, next_wake);
        assert!(done.try_recv().is_err());
        if capacity == 1 {
            assert_eq!(attempted.unwrap_err().code, ErrorCode::ResourceExhausted);
            let refused = router.observe_retirement(&start.handle).unwrap();
            assert_eq!(
                refused.dispatch_error.unwrap().code,
                ErrorCode::ResourceExhausted
            );
            assert!(matches!(
                refused.retained_failure,
                Some(RetirementFailure::Engine(_))
            ));
            resume.send(()).unwrap();
            let deadline = Instant::now() + LIMIT;
            while host.handle().status().outstanding != 0 {
                assert!(Instant::now() < deadline);
                std::thread::yield_now();
            }
            let (retry_wake, retry_done) = wake();
            router.retry_retirement(&start.handle, retry_wake).unwrap();
            retry_done.recv_timeout(LIMIT).unwrap();
            assert!(done.try_recv().is_err());
        } else {
            attempted.unwrap();
            assert!(matches!(
                router.observe_retirement(&start.handle).unwrap().progress,
                RetirementProgress::Pending
            ));
            assert_eq!(
                router
                    .retry_retirement(&start.handle, quiet())
                    .unwrap_err()
                    .code,
                ErrorCode::RevisionConflict
            );
            resume.send(()).unwrap();
            done.recv_timeout(LIMIT).unwrap();
        }
        assert!(matches!(
            router.observe_retirement(&start.handle).unwrap().progress,
            RetirementProgress::Complete(_)
        ));
        let deadline = Instant::now() + LIMIT;
        loop {
            if let Some(result) = queued.try_take() {
                result.unwrap();
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        finish(&mut host);
    }
}

#[test]
fn old_instance_commands_are_not_remapped_to_new_native_generation() {
    let mut host = host(1);
    let router = ParentEngineRouter::new(host.handle());
    let a = FakeEngine::new(100);
    let old = ready(&router, &a);
    router.open(source(), false, 50.).unwrap();
    complete(&router, &old);
    let b = FakeEngine::new(200);
    let current = ready(&router, &b);
    router.open(source(), false, 50.).unwrap();
    assert_eq!(
        router
            .close(CloseEpisodePlayback {
                engine_instance_id: DecimalU64(100)
            })
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        router
            .control(ControlEpisodePlayback {
                engine_instance_id: DecimalU64(100),
                control: PlaybackControl::Pause
            })
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        router
            .set_rate(SetEpisodeRate {
                engine_instance_id: DecimalU64(100),
                rate: PlaybackRate::new(1.).unwrap()
            })
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        router
            .select_track(SelectEpisodeTrack {
                engine_instance_id: DecimalU64(100),
                expected_tracks_revision: DecimalU64(1),
                track_kind: TrackKind::Subtitle,
                selection: TrackSelection::Disabled
            })
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        router
            .capture_frame(CaptureEpisodeFrame {
                engine_instance_id: DecimalU64(100),
                output_path: "C:/synthetic-not-written.png".into()
            })
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(b.applied.load(Ordering::SeqCst), 0);
    assert_eq!(router.observe().unwrap().engine_instance_id.0, 200);
    complete(&router, &current);
    finish(&mut host);
}

#[test]
fn exhausted_sequence_duplicate_activation_and_dropped_handles_preserve_owned_routes() {
    let mut host = host(1);
    let router = ParentEngineRouter::new(host.handle());
    router.shared.state.lock().unwrap().sequence = u64::MAX;
    let candidate = FakeEngine::new(100);
    assert_eq!(
        prepare(&router, &candidate).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    assert_eq!(Arc::strong_count(&candidate), 1);
    // Construct a pinned inert native owner solely to exercise the production failure
    // ownership type. The bytes are never executed and no OS parent is accessed.
    use sha2::{Digest, Sha256};
    static FIXTURE: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "router-native-owner-{}-{}",
        std::process::id(),
        FIXTURE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&directory).unwrap();
    let executable = directory.join("mpv.exe");
    let inert = b"Synthetic pinned owner; never execute";
    std::fs::write(&executable, inert).unwrap();
    let trusted = super::super::TrustedMpvExecutable::open(
        &executable,
        &format!("{:x}", Sha256::digest(inert)),
    )
    .unwrap();
    let native =
        Arc::new(NativeEpisodeEngine::new(trusted, Some(1), Arc::new(UnusedArtifacts)).unwrap());
    let failure = router.prepare(native.clone()).unwrap_err();
    assert_eq!(failure.error.code, ErrorCode::ResourceExhausted);
    assert!(Arc::ptr_eq(&failure.engine, &native));
    drop(failure);
    drop(native);
    std::fs::remove_dir_all(directory).unwrap();
    let router = ParentEngineRouter::new(host.handle());
    let route = ready(&router, &candidate);
    assert_eq!(
        router.activate(&route).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let (exit, entered, resume) = gate();
    *candidate.exit.lock().unwrap() = Some(ExitGate(exit));
    let weak = Arc::downgrade(&candidate);
    drop(candidate);
    let start = router.retire(&route, quiet()).unwrap();
    entered.recv_timeout(LIMIT).unwrap();
    drop(start.handle);
    drop(route);
    assert!(weak.upgrade().is_some());
    let token = router.current_route().unwrap().unwrap();
    let retained = router.retire(&token, quiet()).unwrap();
    assert!(retained.dispatch_error.is_none());
    resume.send(()).unwrap();
    let deadline = Instant::now() + LIMIT;
    while !matches!(
        router
            .observe_retirement(&retained.handle)
            .unwrap()
            .progress,
        RetirementProgress::Complete(_)
    ) {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert_eq!(weak.upgrade().unwrap().shutdowns.load(Ordering::SeqCst), 1);
    finish(&mut host);
}
