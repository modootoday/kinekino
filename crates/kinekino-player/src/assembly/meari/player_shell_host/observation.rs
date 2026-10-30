use crate::presentation::web_ui_adapter::player_shell::{
    RouteHostObservation, RouteTicket, ShellIntent, ShellState,
};
use meari_native::infrastructure::{
    native_host::HostStamp,
    window::{NativeWindowNotice, NativeWindowNoticeKind},
};
use meari_native::{application::window_lifecycle::WindowStamp, domain::graphics::FrameTicket};
mod card;
mod retained;

pub struct StartRouteObservation {
    pub state: ShellState,
    ticket: RouteTicket,
    last_revision: u64,
    committed: bool,
    frame_context: Option<FrameTicket>,
    pending_card: Option<card::CardFrameExpectation>,
    pending_retained: Option<retained::RetainedFrameExpectation>,
    terminal_window: bool,
    observed_document: Option<HostStamp>,
    last_card_commit: Option<HostStamp>,
}
impl StartRouteObservation {
    pub fn new(state: ShellState, ticket: RouteTicket) -> Self {
        Self {
            state,
            ticket,
            last_revision: 0,
            committed: false,
            frame_context: None,
            pending_card: None,
            pending_retained: None,
            terminal_window: false,
            observed_document: None,
            last_card_commit: None,
        }
    }
    /// Only a real checked CPU submission for the current host/document can commit the route.
    /// This is not OS compositor presentation, focus or playback evidence.
    pub(super) fn observe(
        &mut self,
        notice: &NativeWindowNotice,
        current: &HostStamp,
        window: WindowStamp,
    ) {
        if notice.host_id == current.host_id
            && notice.revision > self.last_revision
            && notice.window == Some(window)
            && matches!(
                notice.kind,
                NativeWindowNoticeKind::Closed | NativeWindowNoticeKind::ForcedIncomplete
            )
        {
            self.terminal_window = true;
            self.pending_card = None;
            self.pending_retained = None;
        }
        if (self.committed && self.pending_card.is_none() && self.pending_retained.is_none())
            || self.terminal_window
            || self.state.intent() == Some(ShellIntent::CloseRequested)
            || notice.host_id != current.host_id
            || notice.revision <= self.last_revision
        {
            return;
        }
        self.last_revision = notice.revision;
        if notice.kind != NativeWindowNoticeKind::FrameSubmitted || notice.window != Some(window) {
            return;
        }
        let Some(frame) = &notice.frame else {
            return;
        };
        if frame.submission.width == 0 || frame.submission.height == 0 {
            return;
        }
        if self.pending_retained.is_some() {
            self.observe_retained_submission(frame, current);
            return;
        }
        if &frame.document != current {
            return;
        }
        if self.pending_card.is_some() {
            self.observe_card_submission(current, &frame.submission.ticket);
            return;
        }
        if !self.accept_ticket(&frame.submission.ticket) {
            return;
        }
        let Some(route) = self.state.desired_route().cloned() else {
            return;
        };
        if route.as_str() != "view.start" {
            return;
        }
        self.committed = self
            .state
            .observe_route(&self.ticket, RouteHostObservation::Reported { route })
            .is_ok();
        if self.committed {
            self.observed_document = Some(current.clone());
        }
    }
    #[cfg(test)]
    pub(crate) fn observe_synthetic_notice_for_contract_test(
        &mut self,
        notice: &NativeWindowNotice,
        current: &HostStamp,
        window: WindowStamp,
    ) {
        self.observe(notice, current, window);
    }
    fn accept_ticket(&mut self, ticket: &FrameTicket) -> bool {
        if ticket.context_id.is_empty()
            || ticket.context_generation == 0
            || ticket.scene_generation == 0
        {
            return false;
        }
        if let Some(previous) = &self.frame_context
            && (ticket.context_id != previous.context_id
                || ticket.context_generation != previous.context_generation
                || ticket.scene_generation <= previous.scene_generation)
        {
            return false;
        }
        // First context identity comes only from the observer installed on this preparation.
        self.frame_context = Some(ticket.clone());
        true
    }
}

#[cfg(test)]
mod tests;
