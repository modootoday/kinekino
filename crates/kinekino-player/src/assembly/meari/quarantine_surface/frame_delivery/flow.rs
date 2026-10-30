//! Frame admission, fixed budget and owned publication over a private backend port.
use super::{
    FrameFailure, FrameOwnedResult, FrameRejection, FrameRequest, FrameResultState,
    mailbox::{Mailbox, same_source},
};
use crate::assembly::meari::player_shell_host::FrameHookError;
use meari_native::{
    application::window_lifecycle::WindowStamp,
    domain::graphics::FrameTicket,
    infrastructure::{
        native_host::HostStamp,
        window::{NativeWindowNotice, NativeWindowNoticeKind},
    },
};
use std::{cell::RefCell, rc::Rc, time::Instant};

#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
pub(super) struct FrameContext {
    pub(super) stamp: HostStamp,
    pub(super) window: WindowStamp,
    pub(super) ticket: FrameTicket,
}
#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
pub(super) enum Progress {
    Pending { redraw: bool },
    Observed,
}
#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
pub(super) trait FrameBackend {
    fn context(&self, notice: &NativeWindowNotice) -> Result<FrameContext, FrameFailure>;
    fn prepare(&mut self, video: &str, stamp: &HostStamp) -> Result<(), FrameFailure>;
    fn consume(
        &mut self,
        expires_at: Instant,
        clock: &dyn Fn() -> Instant,
    ) -> Result<Progress, FrameFailure>;
}
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
pub(super) enum Phase {
    Waiting,
    Pending,
    Observed,
    Failed,
}
#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
pub(super) struct Flow<B> {
    pub(super) request: FrameRequest,
    pub(super) backend: B,
    pub(super) origin: Option<HostStamp>,
    pub(super) last_scope: Option<HostStamp>,
    pub(super) window: Option<WindowStamp>,
    pub(super) last_frame: Option<FrameTicket>,
    pub(super) left: usize,
    pub(super) phase: Phase,
    pub(super) first_error: Option<FrameFailure>,
    pub(super) buffered: Option<FrameOwnedResult>,
    pub(super) buffered_redraw: bool,
    pub(super) dirty: bool,
    pub(super) redraw_requested: bool,
    pub(super) mailbox: Rc<RefCell<Mailbox>>,
}
#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
impl<B: FrameBackend> Flow<B> {
    fn publish(
        &mut self,
        stamp: HostStamp,
        state: FrameResultState,
        redraw: bool,
        clock: &dyn Fn() -> Instant,
    ) -> Result<(), FrameHookError> {
        self.buffered = Some(FrameOwnedResult {
            request: self.request.id,
            observed: stamp,
            window: self.window,
            state,
        });
        self.buffered_redraw |= redraw;
        self.dirty = true;
        self.flush(clock)
    }
    fn flush(&mut self, clock: &dyn Fn() -> Instant) -> Result<(), FrameHookError> {
        if !self.dirty {
            return Ok(());
        }
        let mailbox = self.mailbox.clone();
        let mut mailbox = mailbox
            .try_borrow_mut()
            .map_err(|_| FrameHookError("frame mailbox borrowed"))?;
        let mut pending = self.buffered.clone();
        if clock() >= self.request.expires_at
            && pending
                .as_ref()
                .is_some_and(|result| !matches!(result.state, FrameResultState::Rejected(_)))
        {
            self.first_error
                .get_or_insert(FrameFailure::Rejected(FrameRejection::Expired));
            self.phase = Phase::Failed;
            if let Some(result) = pending.as_mut() {
                result.state = FrameResultState::Rejected(FrameRejection::Expired);
            }
            self.buffered = pending.clone();
            self.buffered_redraw = false;
        }
        if pending
            .as_ref()
            .is_some_and(|result| matches!(result.state, FrameResultState::Rejected(_)))
        {
            mailbox.redraw = false;
        }
        mailbox.pending = pending;
        mailbox.redraw |= self.buffered_redraw;
        self.buffered_redraw = false;
        self.dirty = false;
        Ok(())
    }
    fn notify(&self) -> Result<bool, FrameHookError> {
        Ok(self
            .mailbox
            .try_borrow()
            .map_err(|_| FrameHookError("frame mailbox borrowed"))?
            .pending
            .is_some())
    }
    fn fail(
        &mut self,
        clock: &dyn Fn() -> Instant,
        error: FrameFailure,
        stamp: HostStamp,
    ) -> Result<bool, FrameHookError> {
        let reason = error.rejection();
        self.first_error.get_or_insert(error);
        self.phase = Phase::Failed;
        self.publish(stamp, FrameResultState::Rejected(reason), false, clock)?;
        self.notify()
    }
    pub(super) fn observe(
        &mut self,
        notice: &NativeWindowNotice,
        clock: &dyn Fn() -> Instant,
    ) -> Result<bool, FrameHookError> {
        // Retry owned publication only. Never repeat native work after mailbox backpressure.
        if self.dirty {
            self.flush(clock)?;
        }
        let origin = self
            .origin
            .as_ref()
            .ok_or(FrameHookError("frame consumer not bound"))?
            .clone();
        if matches!(
            notice.kind,
            NativeWindowNoticeKind::Revoking
                | NativeWindowNoticeKind::Suspending
                | NativeWindowNoticeKind::Suspended
                | NativeWindowNoticeKind::Closed
                | NativeWindowNoticeKind::ForcedIncomplete
        ) {
            if self.phase != Phase::Failed {
                return self.fail(
                    clock,
                    FrameFailure::Rejected(FrameRejection::Revoked),
                    self.last_scope.clone().unwrap_or(origin),
                );
            }
            return self.notify();
        }
        if notice.kind != NativeWindowNoticeKind::FrameSubmitted {
            return self.notify();
        }
        if self.phase == Phase::Failed {
            return self.notify();
        }
        if notice.frame.as_ref().is_some_and(|frame| {
            self.last_frame.as_ref() == Some(&frame.submission.ticket)
                && self.last_scope.as_ref() == Some(&frame.document)
                && self.window == notice.window
                && notice.host_id == frame.document.host_id
        }) {
            // Already consumed at the original full stamp: replay publishes data only.
            return self.notify();
        }
        let context = match self.backend.context(notice) {
            Ok(context) => context,
            Err(error) => {
                return self.fail(clock, error, self.last_scope.clone().unwrap_or(origin));
            }
        };
        if !same_source(&origin, &context.stamp) {
            return self.fail(
                clock,
                FrameFailure::Rejected(FrameRejection::SourceChanged),
                self.last_scope.clone().unwrap_or(origin),
            );
        }
        if self.window.is_some_and(|window| window != context.window) {
            return self.fail(
                clock,
                FrameFailure::Rejected(FrameRejection::WindowChanged),
                context.stamp,
            );
        }
        if self.phase == Phase::Observed {
            if self
                .last_scope
                .as_ref()
                .is_some_and(|stamp| stamp.document_revision != context.stamp.document_revision)
            {
                return self.fail(
                    clock,
                    FrameFailure::Rejected(FrameRejection::DocumentChanged),
                    context.stamp,
                );
            }
            return self.notify();
        }
        if clock() >= self.request.expires_at {
            return self.fail(
                clock,
                FrameFailure::Rejected(FrameRejection::Expired),
                context.stamp,
            );
        }
        if self.last_frame.as_ref() == Some(&context.ticket) {
            return self.notify();
        }
        if self.left == 0 {
            return self.fail(
                clock,
                FrameFailure::Rejected(FrameRejection::BudgetExhausted),
                context.stamp,
            );
        }
        self.left -= 1;
        self.window = Some(context.window);
        self.last_frame = Some(context.ticket.clone());
        self.last_scope = Some(context.stamp.clone());
        if self.phase == Phase::Waiting {
            if let Err(error) = self.backend.prepare(&self.request.video, &context.stamp) {
                return self.fail(clock, error, context.stamp);
            }
            self.phase = Phase::Pending;
        }
        let progress = self.backend.consume(self.request.expires_at, clock);
        if progress.is_ok() && clock() >= self.request.expires_at {
            return self.fail(
                clock,
                FrameFailure::Rejected(FrameRejection::Expired),
                context.stamp,
            );
        }
        match progress {
            Ok(Progress::Pending { redraw }) => {
                let redraw = redraw && !std::mem::replace(&mut self.redraw_requested, true);
                self.publish(context.stamp, FrameResultState::Pending, redraw, clock)?;
            }
            Ok(Progress::Observed) => {
                self.phase = Phase::Observed;
                self.publish(
                    context.stamp,
                    FrameResultState::ObservedDisabled {
                        frame: context.ticket,
                    },
                    false,
                    clock,
                )?;
            }
            Err(error) => return self.fail(clock, error, context.stamp),
        }
        self.notify()
    }
}
#[cfg(test)]
#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
impl<B: FrameBackend> Flow<B> {
    pub(super) fn observe_at(
        &mut self,
        notice: &NativeWindowNotice,
        now: Instant,
    ) -> Result<bool, FrameHookError> {
        self.observe(notice, &|| now)
    }
}
