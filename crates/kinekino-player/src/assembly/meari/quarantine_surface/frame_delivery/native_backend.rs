//! Original registrar, checkpoint and checked proof adapter; no host reentry or new owner.
use super::super::{
    ConstructionStatus, QuarantineChild, QuarantineError, QuarantineRegistration,
    QuarantineRegistrationFailure, register_no_media,
};
use super::{
    FrameFailure, FrameRejection,
    flow::{FrameBackend, FrameContext, Progress},
};
use meari_native::{
    application::window_lifecycle::{WindowLifecycle, WindowPhase},
    infrastructure::{
        native_host::{HostStamp, NativeDocumentHandle},
        window::{NativeSurfaceRegistrar, NativeWindowNotice},
    },
};
use std::{rc::Rc, sync::Arc, time::Instant};

#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
fn within_deadline(expires_at: Instant, clock: &dyn Fn() -> Instant) -> Result<(), FrameFailure> {
    if clock() >= expires_at {
        Err(FrameFailure::Rejected(FrameRejection::Expired))
    } else {
        Ok(())
    }
}
#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
pub(super) fn consume_checked(
    expires_at: Instant,
    clock: &dyn Fn() -> Instant,
    advance: impl FnOnce() -> Result<ConstructionStatus, FrameFailure>,
    verify: impl FnOnce() -> Result<(), FrameFailure>,
) -> Result<Progress, FrameFailure> {
    within_deadline(expires_at, clock)?;
    let status = advance()?;
    within_deadline(expires_at, clock)?;
    match status {
        ConstructionStatus::Pending { request_redraw } => Ok(Progress::Pending {
            redraw: request_redraw,
        }),
        ConstructionStatus::Constructed => {
            verify()?;
            within_deadline(expires_at, clock)?;
            Ok(Progress::Observed)
        }
    }
}
#[derive(Default)]
#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
pub(super) struct NativeBackend {
    pub(super) document: Option<NativeDocumentHandle>,
    pub(super) children: Option<Rc<WindowLifecycle<QuarantineChild>>>,
    pub(super) registrar: Option<NativeSurfaceRegistrar<QuarantineChild>>,
    registration: Option<QuarantineRegistration>,
    pub(super) failure: Option<Box<QuarantineRegistrationFailure>>,
    #[cfg(test)]
    context: std::cell::RefCell<Option<FrameContext>>,
}
impl FrameBackend for NativeBackend {
    fn context(&self, notice: &NativeWindowNotice) -> Result<FrameContext, FrameFailure> {
        let document = self
            .document
            .as_ref()
            .ok_or(FrameFailure::Rejected(FrameRejection::ForeignFrame))?;
        let stamp = document
            .stamp()
            .map_err(|error| FrameFailure::Host(Arc::new(error)))?;
        let children = self
            .children
            .as_ref()
            .ok_or(FrameFailure::Rejected(FrameRejection::ForeignFrame))?;
        if children
            .phase()
            .map_err(|error| FrameFailure::Lifecycle(Arc::new(error)))?
            != WindowPhase::Ready
        {
            return Err(FrameFailure::Rejected(FrameRejection::Revoked));
        }
        let window = children
            .stamp()
            .map_err(|error| FrameFailure::Lifecycle(Arc::new(error)))?;
        let frame = notice
            .frame
            .as_ref()
            .ok_or(FrameFailure::Rejected(FrameRejection::ForeignFrame))?;
        if notice.host_id != stamp.host_id
            || notice.window != Some(window)
            || frame.document != stamp
        {
            return Err(FrameFailure::Rejected(FrameRejection::ForeignFrame));
        }
        #[cfg(test)]
        {
            match self.context.try_borrow_mut() {
                Ok(mut context) => {
                    *context = Some(FrameContext {
                        stamp: stamp.clone(),
                        window,
                        ticket: frame.submission.ticket.clone(),
                    })
                }
                Err(_) => {
                    if let Some(probe) = crate::assembly::meari::viewer_native_probe::current() {
                        probe.fail(crate::assembly::meari::viewer_native_probe::Fault::Reentrant);
                    }
                }
            }
        }
        Ok(FrameContext {
            stamp,
            window,
            ticket: frame.submission.ticket.clone(),
        })
    }
    fn prepare(&mut self, video: &str, stamp: &HostStamp) -> Result<(), FrameFailure> {
        let registrar = self
            .registrar
            .as_ref()
            .ok_or(FrameFailure::Rejected(FrameRejection::NativeFailure))?;
        let reservation = registrar
            .reserve_video(stamp, video)
            .map_err(|error| FrameFailure::Access(Arc::new(error)))?;
        let registrar = self
            .registrar
            .take()
            .ok_or(FrameFailure::Rejected(FrameRejection::NativeFailure))?;
        match register_no_media(registrar, reservation) {
            Ok(registration) => {
                self.registration = Some(registration);
                Ok(())
            }
            Err(failure) => {
                self.failure = Some(failure);
                Err(FrameFailure::Rejected(FrameRejection::NativeFailure))
            }
        }
    }
    fn consume(
        &mut self,
        expires_at: Instant,
        clock: &dyn Fn() -> Instant,
    ) -> Result<Progress, FrameFailure> {
        let registration = self
            .registration
            .as_ref()
            .ok_or(FrameFailure::Rejected(FrameRejection::NativeFailure))?;
        let progress = consume_checked(
            expires_at,
            clock,
            || {
                registration
                    .checkpoint
                    .advance()
                    .map_err(|error| FrameFailure::Native(Arc::new(error)))
            },
            || {
                within_deadline(expires_at, clock)?;
                registration
                    .checkpoint
                    .with_disabled_child(|proof| {
                        within_deadline(expires_at, clock)?;
                        proof.verify().map_err(|error| {
                            FrameFailure::Native(Arc::new(QuarantineError::Child(Arc::new(error))))
                        })?;
                        within_deadline(expires_at, clock)
                    })
                    .map_err(|error| FrameFailure::Native(Arc::new(error)))?
            },
        );
        #[cfg(test)]
        if matches!(progress, Ok(Progress::Observed)) {
            use crate::assembly::meari::viewer_native_probe::{self, Kind};
            if let Some(probe) = viewer_native_probe::current() {
                match self.context.try_borrow() {
                    Ok(context) => {
                        if let Some(context) = context.as_ref() {
                            probe.record(
                                Kind::ProofObserved,
                                &(
                                    viewer_native_probe::stamp(&context.stamp),
                                    (
                                        context.window.owner(),
                                        context.window.epoch(),
                                        context.window.surface_generation(),
                                    ),
                                    &context.ticket,
                                ),
                            );
                        } else {
                            probe.fail(viewer_native_probe::Fault::Observation);
                        }
                    }
                    Err(_) => probe.fail(viewer_native_probe::Fault::Reentrant),
                }
            }
        }
        progress
    }
}
