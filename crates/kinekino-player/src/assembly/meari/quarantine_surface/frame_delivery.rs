//! One opt-in NoMedia consumer; policy, mailbox and native authority stay in separate modules.
mod contract;
mod flow;
mod mailbox;
mod native_backend;
#[cfg(test)]
use super::ConstructionStatus;
use super::{QuarantineChild, QuarantineRegistrationFailure};
use crate::assembly::meari::player_shell_host::{
    FrameBindFailure, FrameBinding, FrameHookError, ShellFrameConsumer,
};
pub(crate) use contract::{
    FrameFailure, FrameHostUpdate, FrameOwnedResult, FrameReceiveError, FrameRejection,
    FrameRequest, FrameResultState,
};
use flow::{Flow, Phase};
#[cfg(test)]
use flow::{FrameBackend, FrameContext, Progress};
pub(crate) use mailbox::FrameReceiver;
use mailbox::Mailbox;
use meari_native::infrastructure::window::NativeWindowNotice;
#[cfg(test)]
use meari_native::{
    application::window_lifecycle::WindowStamp,
    domain::graphics::FrameTicket,
    infrastructure::{
        native_host::{HostStamp, HostTurn, HostTurnReason, NativeDocumentHandle},
        window::NativeWindowNoticeKind,
    },
};
use native_backend::NativeBackend;
#[cfg(test)]
use native_backend::consume_checked;
#[cfg(test)]
use std::num::{NonZeroU64, NonZeroUsize};
use std::{cell::RefCell, rc::Rc, time::Instant};

#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
pub(crate) struct NoMediaFrameConsumer {
    flow: Flow<NativeBackend>,
}
#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
impl NoMediaFrameConsumer {
    pub fn new(request: FrameRequest) -> (Self, FrameReceiver) {
        let mailbox = Rc::new(RefCell::new(Mailbox {
            request: request.id,
            origin: request.origin.clone(),
            expires_at: request.expires_at,
            pending: None,
            redraw: false,
        }));
        let left = request.frame_budget.get();
        (
            Self {
                flow: Flow {
                    request,
                    backend: NativeBackend::default(),
                    origin: None,
                    last_scope: None,
                    window: None,
                    last_frame: None,
                    left,
                    phase: Phase::Waiting,
                    first_error: None,
                    buffered: None,
                    buffered_redraw: false,
                    dirty: false,
                    redraw_requested: false,
                    mailbox: mailbox.clone(),
                },
            },
            FrameReceiver { mailbox },
        )
    }
    pub fn first_error(&self) -> Option<&FrameFailure> {
        self.flow.first_error.as_ref()
    }
    pub fn registration_failure(&self) -> Option<&QuarantineRegistrationFailure> {
        self.flow.backend.failure.as_deref()
    }
}
impl ShellFrameConsumer<QuarantineChild> for NoMediaFrameConsumer {
    fn bind(
        &mut self,
        binding: FrameBinding<QuarantineChild>,
    ) -> Result<(), Box<FrameBindFailure<QuarantineChild>>> {
        if self.flow.origin.is_some() {
            return Err(Box::new(FrameBindFailure {
                error: FrameHookError("frame consumer already bound"),
                original: binding,
            }));
        }
        let stamp = match binding.document.stamp() {
            Ok(stamp) => stamp,
            Err(_) => {
                return Err(Box::new(FrameBindFailure {
                    error: FrameHookError("frame document unavailable during bind"),
                    original: binding,
                }));
            }
        };
        if stamp != self.flow.request.origin {
            return Err(Box::new(FrameBindFailure {
                error: FrameHookError("original frame request document changed before bind"),
                original: binding,
            }));
        }
        self.flow.origin = Some(stamp.clone());
        self.flow.last_scope = Some(stamp);
        self.flow.backend.document = Some(binding.document);
        self.flow.backend.children = Some(binding.children);
        self.flow.backend.registrar = Some(binding.registrar);
        Ok(())
    }
    fn observe(&mut self, notice: &NativeWindowNotice) -> Result<bool, FrameHookError> {
        self.flow.observe(notice, &Instant::now)
    }
}

#[cfg(test)]
pub(crate) fn reflection_channel_for_test(
    request: NonZeroU64,
    origin: &HostStamp,
    expires_at: Instant,
) -> (FrameReceiver, impl Fn(FrameOwnedResult, bool) + use<>) {
    let mailbox = Rc::new(RefCell::new(Mailbox {
        request,
        origin: origin.clone(),
        expires_at,
        pending: None,
        redraw: false,
    }));
    let receiver = FrameReceiver {
        mailbox: mailbox.clone(),
    };
    (receiver, move |result, redraw| {
        let mut mailbox = mailbox.borrow_mut();
        mailbox.pending = Some(result);
        mailbox.redraw |= redraw;
    })
}
#[cfg(test)]
mod tests;
