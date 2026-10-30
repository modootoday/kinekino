//! Original frame delivery to the registered SDK child; never constructs a second SDK.
mod observation;
mod retirement;
mod shared;
use crate::{
    application::catalog::CatalogArtifacts,
    assembly::{
        meari::{
            playback_child::routed::{
                RegisteredRoutedChild, RoutedMediaReady, RoutedMediaRegistration,
                RoutedMediaRegistrationFailure, register_routed_media,
            },
            player_shell_host::{
                FrameBindFailure, FrameBinding, FrameHookError, ShellFrameConsumer,
            },
        },
        native_playback::routed::{
            ReadyRoutedNative, RoutedSetupError,
            bootstrap::{ActivatedRoutedClient, PendingRoutedActivation, RoutedActivationFailure},
        },
    },
    infrastructure::mpv::TrustedMpvExecutable,
};
use meari_native::{
    application::window_lifecycle::{WindowLifecycle, WindowPhase},
    infrastructure::{
        native_host::{HostStamp, NativeDocumentHandle},
        window::{NativeSurfaceRegistrar, NativeWindowNotice, NativeWindowNoticeKind},
    },
};
pub(super) use shared::RoutedFrameConsumer;
use std::{rc::Rc, sync::Arc};

struct BoundFrame {
    document: NativeDocumentHandle,
    children: Rc<WindowLifecycle<RegisteredRoutedChild>>,
}

struct RoutedFrameState {
    media: retirement::MediaLifecycle,
    trace: crate::assembly::meari::player_shell_host::registration_redraw::RoutedFrameTrace,
    redraw: Rc<crate::assembly::meari::player_shell_host::registration_redraw::RegistrationRedraw>,
    observation: observation::ReadyEmitter,
    video: String,
    origin: HostStamp,
    binding: Option<BoundFrame>,
    registrar: Option<NativeSurfaceRegistrar<RegisteredRoutedChild>>,
    registration: Option<RoutedMediaRegistration>,
    registration_failure: Option<Box<RoutedMediaRegistrationFailure>>,
    executable: Option<TrustedMpvExecutable>,
    executable_path: std::path::PathBuf,
    executable_sha256: String,
    artifacts: Arc<dyn CatalogArtifacts>,
    pending: Option<PendingRoutedActivation>,
    activated: Option<ActivatedRoutedClient<RoutedMediaReady>>,
    activation_failure: Option<Box<RoutedActivationFailure<RoutedMediaReady>>>,
    first_error: Option<String>,
    revocation_error: Option<FrameHookError>,
}
impl RoutedFrameState {
    fn capture_checkpoint(&mut self) {
        use crate::assembly::meari::{
            playback_child::routed::RoutedSurfaceObservation,
            player_shell_host::registration_redraw::{CheckpointTrace, bounded_error},
        };
        let checkpoint = self
            .registration
            .as_ref()
            .map(|registration| &registration.checkpoint)
            .or_else(|| {
                self.registration_failure
                    .as_ref()
                    .and_then(|failure| failure.checkpoint.as_ref())
            });
        let Some(checkpoint) = checkpoint else {
            return;
        };
        match checkpoint.diagnostic_snapshot() {
            Ok(value) => {
                self.trace.last_checkpoint = Some(CheckpointTrace {
                    surface: match value.surface {
                        RoutedSurfaceObservation::PreparedSurface => "prepared_surface",
                        RoutedSurfaceObservation::RetainedUnpreparedInput => {
                            "retained_unprepared_input"
                        }
                        RoutedSurfaceObservation::RetainedRejectedInput => {
                            "retained_rejected_input"
                        }
                    },
                    original_engine_retained: value.original_engine_retained,
                    retained_route_generation: value.retained_route_generation,
                    admission_revoked: value.admission_revoked,
                    state_revoked: value.state_revoked,
                    cleanup_retained: value.cleanup_retained,
                    first_error: value.first_error.as_ref().map(bounded_error),
                    revocation_error: value.revocation_error.as_ref().map(bounded_error),
                });
                self.trace.checkpoint_read_error = None;
            }
            Err(error) => self.trace.checkpoint_read_error = Some(bounded_error(&error)),
        }
    }
    pub(super) fn new(
        video: String,
        origin: HostStamp,
        executable: TrustedMpvExecutable,
        artifacts: Arc<dyn CatalogArtifacts>,
        pending: PendingRoutedActivation,
        redraw: Rc<
            crate::assembly::meari::player_shell_host::registration_redraw::RegistrationRedraw,
        >,
    ) -> Self {
        Self {
            media: retirement::MediaLifecycle::new(origin.owner.clone()),
            trace: Default::default(),
            redraw,
            observation: observation::ReadyEmitter::default(),
            video,
            origin,
            binding: None,
            registrar: None,
            registration: None,
            registration_failure: None,
            executable_path: executable.path().to_path_buf(),
            executable_sha256: executable.sha256().into(),
            executable: Some(executable),
            artifacts,
            pending: Some(pending),
            activated: None,
            activation_failure: None,
            first_error: None,
            revocation_error: None,
        }
    }

    fn invalidate(&self) -> Result<(), FrameHookError> {
        if let Some(registration) = &self.registration {
            registration.checkpoint.invalidate_original().map_err(|_| {
                FrameHookError("original routed admission revocation failed; owner retained")
            })?;
        }
        if let Some(checkpoint) = self
            .registration_failure
            .as_ref()
            .and_then(|f| f.checkpoint.as_ref())
        {
            checkpoint.invalidate_original().map_err(|_| {
                FrameHookError("original partial routed admission revocation failed")
            })?;
        }
        if let Some(pending) = &self.pending {
            pending
                .close_pending()
                .map_err(|_| FrameHookError("original pending native port close failed"))?;
        }
        if let Some(failure) = &self.activation_failure {
            failure
                .activation
                .close_pending()
                .map_err(|_| FrameHookError("failed original activation port close failed"))?;
        }
        Ok(())
    }

    fn submitted(&mut self, notice: &NativeWindowNotice) -> Result<bool, String> {
        let binding = self
            .binding
            .as_ref()
            .ok_or("original frame binding missing")?;
        let stamp = binding.document.stamp().map_err(|e| format!("{e:?}"))?;
        let frame = notice.frame.as_ref().ok_or("original frame missing")?;
        let window = binding.children.stamp().map_err(|e| format!("{e:?}"))?;
        if binding.children.phase().map_err(|e| format!("{e:?}"))? != WindowPhase::Ready
            || notice.host_id != stamp.host_id
            || notice.window != Some(window)
            || frame.document != stamp
            || stamp.host_id != self.origin.host_id
            || stamp.owner != self.origin.owner
            || stamp.definition_revision != self.origin.definition_revision
        {
            return Err("original routed document/window identity changed".into());
        }
        if self.registration.is_none() {
            self.trace.registration_attempted = true;
            let reservation = self
                .registrar
                .as_ref()
                .ok_or("original registrar missing")?
                .reserve_video(&stamp, &self.video)
                .map_err(|e| format!("{e:?}"))?;
            // The registrar moves to the registration/failure carrier. The binding keeps only
            // the original document and coordinator; no parent lease is duplicated.
            let registrar = self.registrar.take().ok_or("original registrar missing")?;
            let pending = self.pending.as_ref().ok_or("original activation missing")?;
            match register_routed_media(registrar, reservation, pending.router()) {
                Ok(registration) => {
                    self.registration = Some(registration);
                    self.trace.registered = true;
                    self.capture_checkpoint();
                    self.redraw
                        .queue(&stamp)
                        .map_err(|error| error.to_string())?;
                    return Ok(true);
                }
                Err(failure) => {
                    self.registration_failure = Some(failure);
                    self.capture_checkpoint();
                    return Err(
                        "original routed child registration failed; partial retained".into(),
                    );
                }
            }
        }
        self.capture_checkpoint();
        let registration = self.registration.as_ref().ok_or("original child missing")?;
        if self.activated.is_some() {
            registration
                .checkpoint
                .refresh_current_frame()
                .map_err(|e| format!("{e:?}"))?;
            registration
                .checkpoint
                .revalidate_active_frame()
                .map_err(|e| format!("{e:?}"))?;
            return Ok(false);
        }
        self.trace.activation_attempted = true;
        let executable = self
            .executable
            .take()
            .ok_or("original executable already consumed")?;
        let ready = registration
            .checkpoint
            .activate(executable, self.artifacts.clone())
            .map_err(|e| format!("{e:?}"))?;
        // activate returned after releasing State's RefMut. The bootstrap reacquires fresh
        // checked authority itself, so an owned ready notification is never used as proof.
        let pending = self
            .pending
            .take()
            .ok_or("original activation already consumed")?;
        match pending.activate(ready) {
            Ok(activated) => {
                self.activated = Some(activated);
                self.trace.activated = true;
            }
            Err(failure) => {
                let error = match &failure.error {
                    RoutedSetupError::Ready(error) => error.to_string(),
                    RoutedSetupError::Api(error) => error.message.clone(),
                };
                self.activation_failure = Some(failure);
                self.capture_checkpoint();
                return Err(error);
            }
        }
        self.trace.capture_attempted = true;
        self.observation.capture(
            &self
                .activated
                .as_ref()
                .ok_or("Original activation missing after success")?
                .ready,
            &stamp,
            window,
            frame,
            &registration.lease,
        )?;
        self.trace.ready_captured = self.observation.snapshot().ready.is_some();
        self.capture_checkpoint();
        Ok(true)
    }
}
impl ShellFrameConsumer<RegisteredRoutedChild> for RoutedFrameState {
    fn routed_observation(
        &self,
    ) -> Option<crate::assembly::meari::player_shell_host::RoutedFrameObservation> {
        let mut snapshot = self.observation.snapshot();
        let mut trace = self.trace.clone();
        let (queued, requested, cancelled) = self.redraw.snapshot();
        trace.redraw_queued = queued;
        trace.redraw_requested = requested;
        trace.redraw_cancelled = cancelled;
        trace.error(
            self.first_error.as_deref(),
            self.revocation_error.as_ref().map(|error| error.0),
        );
        snapshot.trace = Some(Arc::new(trace));
        Some(snapshot)
    }
    fn bind(
        &mut self,
        binding: FrameBinding<RegisteredRoutedChild>,
    ) -> Result<(), Box<FrameBindFailure<RegisteredRoutedChild>>> {
        if self.binding.is_some() || binding.document.stamp().ok().as_ref() != Some(&self.origin) {
            return Err(Box::new(FrameBindFailure {
                error: FrameHookError("original routed frame binding changed or already installed"),
                original: binding,
            }));
        }
        self.registrar = Some(binding.registrar);
        self.binding = Some(BoundFrame {
            document: binding.document,
            children: binding.children,
        });
        self.trace.bound = true;
        Ok(())
    }

    fn observe(&mut self, notice: &NativeWindowNotice) -> Result<bool, FrameHookError> {
        if matches!(
            notice.kind,
            NativeWindowNoticeKind::Closed | NativeWindowNoticeKind::ForcedIncomplete
        ) {
            self.redraw.cancel();
            if let Err(error) = self.invalidate() {
                self.revocation_error = Some(error);
                self.capture_checkpoint();
                return Err(FrameHookError(
                    "Original routed close revocation failed; retained",
                ));
            }
            self.capture_checkpoint();
            return Ok(false);
        }
        if notice.kind == NativeWindowNoticeKind::FrameSubmitted {
            self.trace.frame_callbacks = self.trace.frame_callbacks.saturating_add(1);
            if let Some(frame) = &notice.frame {
                let ticket = &frame.submission.ticket;
                self.trace.last_scene_generation = Some(ticket.scene_generation);
                self.trace.last_context_generation = Some(ticket.context_generation);
                self.trace.last_context_id = Some(ticket.context_id.chars().take(128).collect());
                self.trace.frame_identity_truncated |= ticket.context_id.chars().count() > 128;
            }
        }
        if self.first_error.is_some() {
            return Err(FrameHookError(
                "original routed frame failed; retained for cleanup",
            ));
        }
        if self.media.retiring.is_some() {
            self.drive_retirement()
                .map_err(|_| FrameHookError("Original route media retirement retained an error"))?;
            return Ok(false);
        }
        if self.media.retired {
            return Ok(false);
        }
        if notice.kind != NativeWindowNoticeKind::FrameSubmitted {
            return Ok(false);
        }
        match self.submitted(notice) {
            Ok(wake) => {
                if self.activated.is_some()
                    && let Some(ticket) = self.media.viewing.clone()
                {
                    if let Err(error) = self.resume_original_transitions() {
                        self.media
                            .policy
                            .fail_viewing(&ticket, error)
                            .map_err(|_| {
                                FrameHookError("Original Viewing failure retention failed")
                            })?;
                        return Err(FrameHookError(
                            "Original transition rearm failed; owner retained",
                        ));
                    }
                    self.media
                        .policy
                        .complete_viewing(&ticket)
                        .map_err(|_| FrameHookError("Original fresh Viewing ticket differs"))?;
                    self.media.viewing = None;
                    self.media.retained_client = None;
                }
                Ok(wake)
            }
            Err(error) => {
                if let Some(ticket) = &self.media.viewing {
                    let failure = retirement::native_error(
                        "Original fresh Viewing registration failed; owners retained",
                    );
                    let _ = self.media.policy.fail_viewing(ticket, failure);
                }
                self.first_error = Some(error);
                self.revocation_error = self.invalidate().err();
                self.capture_checkpoint();
                Err(FrameHookError(
                    "original routed frame failed; original owners retained",
                ))
            }
        }
    }
}
