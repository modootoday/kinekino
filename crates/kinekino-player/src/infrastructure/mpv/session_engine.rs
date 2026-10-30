use super::{Mpv, TrustedMpvExecutable};
mod replacement;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        catalog::CatalogArtifacts,
        playback_control::{ControlCompletion, ControlReceipt},
        playback_session::{
            CaptureEpisodeFrame, CapturedEpisodeFrame, CloseEpisodePlayback, ClosedEpisodePlayback,
            ControlEpisodePlayback, EpisodePlaybackEngine, EpisodePlaybackObservation,
            EpisodePlaybackSource, NativeEngineDiagnostics, OpenedEpisodePlayback,
            SelectEpisodeTrack, SetEpisodeRate, TrackControlReceipt,
        },
        playback_status::{
            NativeStatusPublisher, NativeStatusReader, PlaybackStatusPort, native_status_channel,
        },
    },
    domain::api_values::DecimalU64,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

struct Active {
    mpv: Mpv,
    artifact_id: String,
    publisher: NativeStatusPublisher,
    reader: NativeStatusReader,
    source_url: String,
    diagnostics_store: Arc<dyn CatalogArtifacts>,
    diagnostics: Option<NativeEngineDiagnostics>,
}
impl Active {
    fn from_mpv(
        mpv: Mpv,
        source: &EpisodePlaybackSource,
        diagnostics_store: Arc<dyn CatalogArtifacts>,
    ) -> Self {
        let (publisher, reader) = native_status_channel();
        Self {
            mpv,
            artifact_id: source.artifact_id().into(),
            publisher,
            reader,
            source_url: source.url().into(),
            diagnostics_store,
            diagnostics: None,
        }
    }
    fn id(&self) -> u64 {
        self.mpv.control_handle().instance_id()
    }
    fn require_id(&self, expected: u64) -> Result<(), ApiError> {
        if self.id() != expected {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native playback instance changed",
            ));
        }
        Ok(())
    }
    fn close(&mut self) -> Result<ClosedEpisodePlayback, ApiError> {
        self.mpv.shutdown().map_err(|_| {
            ApiError::new(
                ErrorCode::EngineFailed,
                "Owned native shutdown could not be confirmed",
            )
        })?;
        self.publisher.clear()?;
        let diagnostics = super::diagnostics::record(
            &self.mpv,
            &self.artifact_id,
            &self.source_url,
            self.diagnostics_store.as_ref(),
            true,
        )?;
        Ok(ClosedEpisodePlayback {
            engine_instance_id: DecimalU64(self.id()),
            engine_exited: true,
            ipc_worker_exited: true,
            stderr_worker_exited: true,
            stdout_worker_exited: true,
            diagnostics,
        })
    }
}

/// Clones address one owned lifetime. No GUI process discovery or implicit engine startup.
#[derive(Clone)]
pub struct NativeEpisodeEngine {
    scope: Arc<super::guarded_command::HostScope>,
    executable: Arc<TrustedMpvExecutable>,
    parent: Option<u32>,
    active: Arc<Mutex<Option<Active>>>,
    closed: Arc<AtomicBool>,
    diagnostics_store: Arc<dyn CatalogArtifacts>,
}
impl NativeEpisodeEngine {
    /// Observation only: never extends the original host lifetime or grants admission.
    #[cfg(feature = "validation-export-delivery-loss")]
    pub fn validation_export_counter_probe(
        &self,
    ) -> impl Fn() -> Result<(u64, u64), ApiError> + Send + Sync + 'static {
        let original = Arc::downgrade(&self.scope);
        move || {
            let scope = original.upgrade().ok_or_else(|| {
                ApiError::new(
                    ErrorCode::SourceExpired,
                    "Original validation counter owner expired",
                )
            })?;
            scope.registry.validation_export_counts()
        }
    }
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "C5 binds the original weak scope before route activation"
        )
    )]
    pub(super) fn guard_scope(&self) -> std::sync::Weak<super::guarded_command::HostScope> {
        Arc::downgrade(&self.scope)
    }
    fn retain_startup_failure(
        &self,
        active: &mut Option<Active>,
        source: &EpisodePlaybackSource,
        failure: super::owned_startup::OwnedStartupFailure,
    ) -> ApiError {
        let (mut error, evidence) = failure.into_parts();
        let Some(mpv) = evidence else {
            return error;
        };
        let cleanup_confirmed = error.details["cleanup_confirmed"] == true;
        let recorded = super::diagnostics::record(
            &mpv,
            source.artifact_id(),
            source.url(),
            self.diagnostics_store.as_ref(),
            cleanup_confirmed,
        );
        let retained = !cleanup_confirmed || recorded.is_err();
        let diagnostics = match recorded {
            Ok(capture) => {
                error.details["diagnostic_capture_confirmed"] = serde_json::json!(true);
                error.details["diagnostics"] = serde_json::json!(&capture);
                Some(capture)
            }
            Err(capture_error) => {
                error.details["diagnostic_capture_confirmed"] = serde_json::json!(false);
                error.details["diagnostic_capture_error_code"] =
                    serde_json::json!(capture_error.code);
                None
            }
        };
        error.details["owned_failure_retained"] = serde_json::json!(retained);
        if retained {
            let mut failed = Active::from_mpv(*mpv, source, self.diagnostics_store.clone());
            failed.diagnostics = diagnostics;
            *active = Some(failed);
        }
        error
    }
    pub fn new(
        executable: TrustedMpvExecutable,
        parent: Option<u32>,
        diagnostics_store: Arc<dyn CatalogArtifacts>,
    ) -> Result<Self, ApiError> {
        Self::new_with_scope(
            executable,
            parent,
            diagnostics_store,
            super::guarded_command::HostScope::disabled(),
        )
    }
    pub(super) fn new_with_scope(
        executable: TrustedMpvExecutable,
        parent: Option<u32>,
        diagnostics_store: Arc<dyn CatalogArtifacts>,
        scope: Arc<super::guarded_command::HostScope>,
    ) -> Result<Self, ApiError> {
        if parent == Some(0) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid native parent window",
            ));
        }
        Ok(Self {
            scope,
            executable: Arc::new(executable),
            parent,
            active: Arc::new(Mutex::new(None)),
            closed: Arc::new(AtomicBool::new(false)),
            diagnostics_store,
        })
    }
    pub(super) fn new_trusted_standalone(
        executable: TrustedMpvExecutable,
        diagnostics_store: Arc<dyn CatalogArtifacts>,
    ) -> Result<Self, ApiError> {
        let scope = super::guarded_command::HostScope::for_verified_standalone(&executable);
        Self::new_with_scope(executable, None, diagnostics_store, scope)
    }
    /// Trusted host EOF path; unlike an external request it already owns this entire lifetime.
    pub fn shutdown(&self) -> Result<Option<ClosedEpisodePlayback>, ApiError> {
        self.closed.store(true, Ordering::Release);
        // Admission closes before waiting for the ownership lock. A callback
        // may use this same word without waiting for registry or process work.
        self.scope.word.close();
        let terminal = self
            .scope
            .registry
            .close(super::guarded_command::Failure::text(
                super::guarded_command::FailureStage::Connection,
                "Native host lifetime ended",
            ));
        let mut active = self.active.lock().map_err(|_| unavailable())?;
        let cleanup = match active.as_mut() {
            Some(current) => current.close().map(Some),
            None => Ok(None),
        };
        if cleanup.is_ok() {
            *active = None;
        }
        drop(active);
        // Preserve recovery on failed cleanup; never claim a successful drain.
        let drain = if cleanup.is_ok() {
            self.scope.word.close_and_drain()
        } else {
            Ok(())
        };
        if let Err(mut error) = terminal {
            if let Err(cleanup_error) = cleanup {
                error.details["cleanup_error"] = serde_json::json!(cleanup_error);
            }
            if let Err(drain_error) = drain {
                error.details["drain_error"] = serde_json::json!(drain_error);
            }
            return Err(error);
        }
        let receipt = cleanup?;
        drain?;
        Ok(receipt)
    }
}
impl EpisodePlaybackEngine for NativeEpisodeEngine {
    fn native_export_issuer(
        &self,
    ) -> Result<crate::application::playback_session::v1::NativeExportIssuer, ApiError> {
        self.scope.require_standalone_export()?;
        self.scope.export_issuer()
    }
    fn observe_native_guard_for_export(
        &self,
        issuer: &crate::application::playback_session::v1::NativeExportIssuer,
    ) -> Result<crate::application::playback_session::NativeGuardObservation, ApiError> {
        self.scope.require_enabled()?;
        issuer.require_issuer(&Arc::downgrade(&self.scope))?;
        self.observe_native_guard()
    }
    fn submit_exported_guarded_seek_v1(
        &self,
        input: crate::application::playback_session::v1::ExportGuardedSeekV1,
    ) -> Result<crate::application::playback_session::v1::ExportedGuardedSeekV1, ApiError> {
        if input.seek.guard.has_routed_export_owner() {
            self.scope.require_enabled()?;
        } else {
            self.scope.require_standalone_export()?;
        }
        input
            .seek
            .guard
            .require_issuer(&Arc::downgrade(&self.scope))?;
        if self.closed.load(Ordering::Acquire) || self.scope.word.is_closed() {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Original native host admission closed",
            ));
        }
        input
            .seek
            .guard
            .submit_exported_native_v1(input.seek.position_us, input.export)
    }
    fn observe_native_guard(
        &self,
    ) -> Result<crate::application::playback_session::NativeGuardObservation, ApiError> {
        let active = self.active.try_lock().map_err(|error| match error {
            std::sync::TryLockError::WouldBlock => {
                let mut error =
                    ApiError::new(ErrorCode::NeedsInput, "Native episode observations busy");
                error.retryable = true;
                error
            }
            std::sync::TryLockError::Poisoned(_) => unavailable(),
        })?;
        let current = active.as_ref().ok_or_else(missing)?;
        let control = current.mpv.control_handle();
        let mut observation = crate::application::playback_session::NativeGuardObservation {
            engine_instance_id: DecimalU64(current.id()),
            resolution_artifact_id: current.artifact_id.clone(),
            native: control.native_receive_observation()?,
            observation_authorized: false,
            guard: None,
        };
        // The production constructor remains diagnostic-only until C5 activation.
        if self.scope.require_enabled().is_err() {
            return Ok(observation);
        }
        if self.closed.load(Ordering::Acquire) {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Native host lifetime ended",
            ));
        }
        if !observation.native.connected || observation.native.counter_exhausted {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Original native receive state unavailable",
            ));
        }
        let has_error = current
            .mpv
            .updates
            .try_lock()
            .map_err(|error| match error {
                std::sync::TryLockError::WouldBlock => {
                    ApiError::new(ErrorCode::NeedsInput, "Native playback observations busy")
                }
                std::sync::TryLockError::Poisoned(_) => unavailable(),
            })?
            .error
            .is_some();
        let identity = super::guarded_command::Identity {
            engine: current.id(),
            resolution: resolution_identity(&current.artifact_id)?,
            connection: observation.native.connection_generation,
            lifecycle: observation.native.lifecycle_generation,
            entry: None,
        };
        let waiter = control.state.guarded.observe(
            identity,
            has_error,
            super::control::guarded_deadline(std::time::Instant::now())?,
        )?;
        drop(active);
        observation.guard = Some(waiter.wait()?);
        observation.observation_authorized = true;
        Ok(observation)
    }
    fn submit_exported_guarded_seek(
        &self,
        input: crate::application::playback_session::ExportGuardedSeek,
    ) -> Result<crate::application::playback_session::ExportedGuardedSeekSubmission, ApiError> {
        if input.seek.guard.has_routed_export_owner() {
            self.scope.require_enabled()?;
        } else {
            self.scope.require_standalone_export()?;
        }
        input
            .seek
            .guard
            .require_issuer(&Arc::downgrade(&self.scope))?;
        if self.closed.load(Ordering::Acquire) || self.scope.word.is_closed() {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Original native host admission closed",
            ));
        }
        input
            .seek
            .guard
            .submit_exported_native(input.seek.position_us, input.export)
    }
    fn exported_seek_references(
        &self,
        provenance: crate::application::playback_session::NativeResultProvenance,
    ) -> Result<Vec<crate::application::playback_session::NativeResultReference>, ApiError> {
        self.scope.require_standalone_export()?;
        self.scope.registry.exported_references(provenance)
    }
    fn exported_seek_result(
        &self,
        input: crate::application::playback_session::NativeExportResultQuery,
    ) -> Result<crate::application::playback_session::GuardedSeekResult, ApiError> {
        self.scope.require_enabled()?;
        input
            .reference
            .ticket
            .require_issuer(&Arc::downgrade(&self.scope))?;
        self.scope
            .registry
            .read_export(&input.reference, input.consume_terminal)
    }
    fn submit_guarded_seek(
        &self,
        input: crate::application::playback_session::GuardedSeek,
    ) -> Result<crate::application::playback_session::GuardedSeekSubmission, ApiError> {
        self.scope.require_enabled()?;
        input.guard.require_issuer(&Arc::downgrade(&self.scope))?;
        if self.closed.load(Ordering::Acquire) || self.scope.word.is_closed() {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Original native host admission closed",
            ));
        }
        input.guard.submit_native(input.position_us)
    }
    fn guarded_seek_result(
        &self,
        input: crate::application::playback_session::GuardedSeekResultQuery,
    ) -> Result<crate::application::playback_session::GuardedSeekResult, ApiError> {
        self.scope.require_enabled()?;
        input.ticket.require_issuer(&Arc::downgrade(&self.scope))?;
        input.ticket.read_original(input.consume_terminal)
    }
    fn options_available(&self) -> bool {
        true
    }
    fn set_rate(&self, input: SetEpisodeRate) -> Result<ControlReceipt, ApiError> {
        input.validate()?;
        let active = self.active.lock().map_err(|_| unavailable())?;
        let current = active.as_ref().ok_or_else(missing)?;
        current.require_id(input.engine_instance_id.0)?;
        current.mpv.control_handle().submit_rate(input.rate)
    }
    fn select_track(&self, input: SelectEpisodeTrack) -> Result<TrackControlReceipt, ApiError> {
        input.validate()?;
        let active = self.active.lock().map_err(|_| unavailable())?;
        let current = active.as_ref().ok_or_else(missing)?;
        current.require_id(input.engine_instance_id.0)?;
        current.mpv.control_handle().submit_track(input)
    }
    fn capture_frame_available(&self) -> bool {
        true
    }
    fn capture_frame(&self, input: CaptureEpisodeFrame) -> Result<CapturedEpisodeFrame, ApiError> {
        let mut active = self.active.lock().map_err(|_| unavailable())?;
        let current = active.as_mut().ok_or_else(missing)?;
        current.require_id(input.engine_instance_id.0)?;
        current
            .mpv
            .capture_episode_frame(&current.artifact_id, input)
    }
    fn open(
        &self,
        source: EpisodePlaybackSource,
        paused: bool,
        volume_percent: f64,
    ) -> Result<OpenedEpisodePlayback, ApiError> {
        let mut active = self.active.lock().map_err(|_| unavailable())?;
        if self.closed.load(Ordering::Acquire) {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Native host lifetime has ended",
            ));
        }
        if active.is_some() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Close the owned native instance before opening another episode",
            ));
        }
        let mpv = match Mpv::launch_episode_owned_scoped(
            &self.executable,
            &source,
            self.parent,
            paused,
            volume_percent,
            self.scope.clone(),
        ) {
            Ok(mpv) => mpv,
            Err(failure) => {
                return Err(self.retain_startup_failure(&mut active, &source, *failure));
            }
        };
        let receipt = OpenedEpisodePlayback {
            engine_instance_id: DecimalU64(mpv.control_handle().instance_id()),
            resolution_artifact_id: source.artifact_id().into(),
            completion: ControlCompletion::Unobserved,
        };
        *active = Some(Active::from_mpv(
            mpv,
            &source,
            self.diagnostics_store.clone(),
        ));
        Ok(receipt)
    }
    fn close(&self, input: CloseEpisodePlayback) -> Result<ClosedEpisodePlayback, ApiError> {
        let mut active = self.active.lock().map_err(|_| unavailable())?;
        let current = active.as_mut().ok_or_else(missing)?;
        current.require_id(input.engine_instance_id.0)?;
        let receipt = current.close()?;
        *active = None;
        Ok(receipt)
    }
    fn observe(&self) -> Result<EpisodePlaybackObservation, ApiError> {
        let mut active = self.active.lock().map_err(|_| unavailable())?;
        let current = active.as_mut().ok_or_else(missing)?;
        current.publisher.publish(current.mpv.snapshot())?;
        let playback = current.reader.status()?;
        if playback.has_error && current.diagnostics.is_none() {
            current.diagnostics = Some(super::diagnostics::record(
                &current.mpv,
                &current.artifact_id,
                &current.source_url,
                current.diagnostics_store.as_ref(),
                false,
            )?);
        }
        Ok(EpisodePlaybackObservation {
            engine_instance_id: DecimalU64(current.id()),
            resolution_artifact_id: current.artifact_id.clone(),
            playback,
            diagnostics: current.diagnostics.clone(),
            options: Some(current.mpv.control_handle().options_observation()?),
        })
    }
    fn control(&self, input: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
        let active = self.active.lock().map_err(|_| unavailable())?;
        let current = active.as_ref().ok_or_else(missing)?;
        current.require_id(input.engine_instance_id.0)?;
        current.mpv.control_handle().submit(input.control)
    }
}
fn missing() -> ApiError {
    ApiError::new(
        ErrorCode::NotFound,
        "No native episode instance is owned by this host",
    )
}
fn resolution_identity(artifact: &str) -> Result<[u8; 32], ApiError> {
    let invalid = || {
        ApiError::new(
            ErrorCode::RevisionConflict,
            "Original resolution artifact must be a 32-byte hex identity",
        )
    };
    if artifact.len() != 64 || !artifact.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid());
    }
    let mut identity = [0; 32];
    for (index, byte) in identity.iter_mut().enumerate() {
        *byte =
            u8::from_str_radix(&artifact[index * 2..index * 2 + 2], 16).map_err(|_| invalid())?;
    }
    Ok(identity)
}
#[cfg(test)]
#[path = "guarded_command/lifecycle_tests.rs"]
mod lifecycle_tests;

fn unavailable() -> ApiError {
    ApiError::new(
        ErrorCode::Internal,
        "Native episode lifetime is unavailable",
    )
}

#[cfg(test)]
mod standalone_export_tests {
    use super::*;
    #[cfg(feature = "validation-export-delivery-loss")]
    #[test]
    fn validation_probe_is_weak_and_expiry_is_not_a_zero_counter() {
        let probe = {
            let fixture = Fixture::new(true, None);
            let probe = fixture.engine().validation_export_counter_probe();
            assert_eq!(probe().unwrap(), (0, 0));
            probe
        };
        assert_eq!(probe().unwrap_err().code, ErrorCode::SourceExpired);
    }
    use sha2::{Digest, Sha256};
    struct Fixture {
        engine: Option<NativeEpisodeEngine>,
        root: std::path::PathBuf,
    }
    impl Fixture {
        fn new(trusted: bool, parent: Option<u32>) -> Self {
            static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let root = std::env::temp_dir().join(format!(
                "player-standalone-export-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            std::fs::create_dir(&root).unwrap();
            let path = root.join("mpv.exe");
            let bytes = b"Synthetic verified identity; never executed";
            std::fs::write(&path, bytes).unwrap();
            let executable =
                TrustedMpvExecutable::open(&path, &format!("{:x}", Sha256::digest(bytes))).unwrap();
            let catalog = root.join("catalog");
            std::fs::create_dir(&catalog).unwrap();
            let store =
                Arc::new(crate::infrastructure::catalog::CatalogFiles::open(&catalog).unwrap());
            let engine = if trusted {
                NativeEpisodeEngine::new_trusted_standalone(executable, store)
            } else {
                NativeEpisodeEngine::new(executable, parent, store)
            }
            .unwrap();
            Self {
                engine: Some(engine),
                root,
            }
        }
        fn engine(&self) -> &NativeEpisodeEngine {
            self.engine.as_ref().unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            drop(self.engine.take());
            std::fs::remove_dir_all(&self.root).unwrap();
        }
    }
    fn provenance() -> crate::application::playback_session::NativeResultProvenance {
        crate::application::playback_session::NativeResultProvenance {
            registry_generation: "g".into(),
            session: "s".into(),
            pass: 1,
            segment: "t".into(),
            expected_revision: 1,
        }
    }
    #[test]
    fn verified_standalone_explicitly_enables_metadata_reads_without_active_media() {
        let fixture = Fixture::new(true, None);
        let engine = fixture.engine();
        assert!(engine.active.lock().unwrap().is_none());
        assert!(
            engine
                .exported_seek_references(provenance())
                .unwrap()
                .is_empty()
        );
        engine.shutdown().unwrap();
        assert!(
            engine
                .exported_seek_references(provenance())
                .unwrap()
                .is_empty()
        );
    }
    fn wire_origin(
        revision: u64,
    ) -> crate::application::playback_session::v1::NativeExportOriginV1 {
        use crate::application::playback_session::v1::*;
        NativeExportOriginV1::try_from(NativeExportOriginInputV1 {
            registry_generation: "g".into(),
            session: "s".into(),
            pass: DecimalU64(1),
            segment: "t".into(),
            request_revision: DecimalU64(revision),
            engine_instance_id: DecimalU64(7),
            resolution_artifact_id: "03".repeat(32),
        })
        .unwrap()
    }
    fn seed_wire(engine: &NativeEpisodeEngine) {
        engine.scope.set_issuer_for_test(Ok(
            crate::application::playback_session::v1::NativeIssuerScopeV1::from_original_nonce(
                [3; 16],
            )
            .unwrap(),
        ));
    }
    #[test]
    fn foreign_wire_issuer_is_rejected_before_active_observation() {
        let a = Fixture::new(true, None);
        let b = Fixture::new(true, None);
        seed_wire(a.engine());
        seed_wire(b.engine());
        let issuer = a.engine().native_export_issuer().unwrap();
        let _locked = b.engine().active.lock().unwrap();
        assert_eq!(
            b.engine()
                .observe_native_guard_for_export(&issuer)
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
    }
    #[test]
    fn failed_wire_identity_is_cached_before_observation() {
        let fixture = Fixture::new(true, None);
        let engine = fixture.engine();
        engine.scope.set_issuer_for_test(Err(()));
        let _locked = engine.active.lock().unwrap();
        for _ in 0..2 {
            assert_eq!(
                engine.native_export_issuer().unwrap_err().code,
                ErrorCode::EngineFailed
            );
        }
    }
    #[test]
    fn routed_expected_issuer_observe_is_enabled_without_general_standalone_capture() {
        let mut fixture = Fixture::new(false, Some(1));
        fixture.engine.as_mut().unwrap().scope =
            super::super::guarded_command::HostScope::testing();
        let engine = fixture.engine();
        seed_wire(engine);
        assert_eq!(
            engine.native_export_issuer().unwrap_err().code,
            ErrorCode::Unsupported
        );
        let issuer = engine.scope.export_issuer().unwrap();
        let foreign = Fixture::new(true, None);
        seed_wire(foreign.engine());
        let foreign_issuer = foreign.engine().native_export_issuer().unwrap();
        let _locked = engine.active.lock().unwrap();
        assert_eq!(
            engine
                .observe_native_guard_for_export(&foreign_issuer)
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(
            engine
                .observe_native_guard_for_export(&issuer)
                .unwrap_err()
                .code,
            ErrorCode::NeedsInput
        );
    }
    #[test]
    fn wire_target_origin_pending_and_terminal_consume_preserve_original_row() {
        use crate::application::playback_session::{GuardedSeek, v1::*};
        let fixture = Fixture::new(true, None);
        let engine = fixture.engine();
        seed_wire(engine);
        let issuer = engine.native_export_issuer().unwrap();
        let (guard, _link, mut receiver) =
            super::super::guarded_command::standalone_export_guard_for_test(&engine.scope);
        let mut annotation = NativeExportAnnotationInputV1 {
            annotation_revision: DecimalU64(4),
            verified_revision: DecimalU64(4),
            timebase: crate::domain::skip_segments::SegmentTimebase::EnginePlayback,
        };
        let output = engine
            .submit_exported_guarded_seek_v1(ExportGuardedSeekV1 {
                seek: GuardedSeek {
                    guard,
                    position_us: DecimalU64(8_000_000),
                },
                export: PreparedNativeResultExportV1::new(
                    wire_origin(1),
                    NativeExportAnnotationV1::try_from(annotation).unwrap(),
                ),
            })
            .unwrap();
        annotation.annotation_revision = DecimalU64(9);
        annotation.verified_revision = DecimalU64(9);
        assert_eq!(
            NativeExportAnnotationV1::try_from(annotation)
                .unwrap()
                .annotation_revision(),
            DecimalU64(9)
        );
        assert!(receiver.try_recv().is_ok());
        assert_eq!(output.metadata().actual_target_us(), DecimalU64(8_000_000));
        assert_eq!(
            output.metadata().annotation().annotation_revision(),
            DecimalU64(4)
        );
        let reference = output.reference();
        let listed = issuer.list(&wire_origin(1)).unwrap();
        assert_eq!(listed.items().len(), 1);
        assert_eq!(listed.items()[0].reference(), reference);
        assert_eq!(
            issuer.consume(reference, &wire_origin(2)).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(
            issuer.consume(reference, &wire_origin(1)).unwrap_err().code,
            ErrorCode::NeedsInput
        );
        issuer.peek(reference, &wire_origin(1)).unwrap();
        engine.shutdown().unwrap();
        issuer.peek(reference, &wire_origin(1)).unwrap();
        let first = issuer.clone();
        let second = issuer.clone();
        let outcomes = std::thread::scope(|scope| {
            let a = scope.spawn(move || first.consume(reference, &wire_origin(1)));
            let b = scope.spawn(move || second.consume(reference, &wire_origin(1)));
            [a.join().unwrap(), b.join().unwrap()]
        });
        assert_eq!(outcomes.iter().filter(|value| value.is_ok()).count(), 1);
        assert!(
            outcomes
                .iter()
                .filter_map(|value| value.as_ref().err())
                .all(|error| error.code == ErrorCode::NotFound)
        );
        assert!(issuer.list(&wire_origin(1)).unwrap().items().is_empty());
    }
    #[test]
    fn legacy_parented_and_unparented_engines_keep_export_unsupported() {
        for parent in [None, Some(1)] {
            let fixture = Fixture::new(false, parent);
            assert_eq!(
                fixture
                    .engine()
                    .exported_seek_references(provenance())
                    .unwrap_err()
                    .code,
                ErrorCode::Unsupported
            );
        }
    }
    #[test]
    fn original_engine_export_read_uses_issuer_and_survives_close_without_active_reselection() {
        use crate::application::playback_session::{
            ExportGuardedSeek, GuardedSeek, NativeExportResultQuery, PreparedNativeResultExport,
        };
        let fixture = Fixture::new(true, None);
        let engine = fixture.engine();
        let (guard, _link, mut receiver) =
            super::super::guarded_command::standalone_export_guard_for_test(&engine.scope);
        let output = engine
            .submit_exported_guarded_seek(ExportGuardedSeek {
                seek: GuardedSeek {
                    guard,
                    position_us: DecimalU64(8_000_000),
                },
                export: PreparedNativeResultExport {
                    provenance: provenance(),
                },
            })
            .unwrap();
        assert!(receiver.try_recv().is_ok());
        assert!(engine.active.lock().unwrap().is_none());
        let references = engine.exported_seek_references(provenance()).unwrap();
        assert!(references[0].same_request(&output.reference));
        let foreign = Fixture::new(true, None);
        assert_eq!(
            foreign
                .engine()
                .exported_seek_result(NativeExportResultQuery {
                    reference: output.reference.clone(),
                    consume_terminal: false
                })
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(
            engine
                .exported_seek_result(NativeExportResultQuery {
                    reference: output.reference.clone(),
                    consume_terminal: true
                })
                .unwrap_err()
                .code,
            ErrorCode::NeedsInput
        );
        engine.shutdown().unwrap();
        assert!(
            engine
                .exported_seek_result(NativeExportResultQuery {
                    reference: output.reference.clone(),
                    consume_terminal: true
                })
                .unwrap()
                .state()
                .is_terminal()
        );
        assert_eq!(
            engine
                .exported_seek_result(NativeExportResultQuery {
                    reference: output.reference,
                    consume_terminal: false
                })
                .unwrap_err()
                .code,
            ErrorCode::NotFound
        );
    }
}
