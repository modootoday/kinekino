//! Move-only Completion reader and bounded owned result storage.
use super::{
    FrameHostUpdate, FrameOwnedResult, FrameReceiveError, FrameRejection, FrameResultState,
};
use meari_native::infrastructure::native_host::{HostStamp, HostTurn, HostTurnReason};
use std::{cell::RefCell, num::NonZeroU64, rc::Rc, time::Instant};

pub(super) struct Mailbox {
    pub(super) request: NonZeroU64,
    pub(super) origin: HostStamp,
    pub(super) expires_at: Instant,
    pub(super) pending: Option<FrameOwnedResult>,
    pub(super) redraw: bool,
}
pub(crate) struct FrameReceiver {
    pub(super) mailbox: Rc<RefCell<Mailbox>>,
}
pub(super) fn same_source(a: &HostStamp, b: &HostStamp) -> bool {
    a.host_id == b.host_id && a.owner == b.owner && a.definition_revision == b.definition_revision
}
impl FrameReceiver {
    /// A later Completion reflects owned data; it never reopens a native input or renews a request.
    pub fn receive(&self, turn: &HostTurn) -> Result<FrameHostUpdate, FrameReceiveError> {
        self.receive_at(turn, Instant::now())
    }
    pub(super) fn receive_at(
        &self,
        turn: &HostTurn,
        now: Instant,
    ) -> Result<FrameHostUpdate, FrameReceiveError> {
        if turn.reason != HostTurnReason::Completion {
            return Ok(FrameHostUpdate {
                result: None,
                redraw: false,
            });
        }
        let mut mailbox = self
            .mailbox
            .try_borrow_mut()
            .map_err(|_| FrameReceiveError::Busy)?;
        if let Some(result) = &mailbox.pending {
            if result.request != mailbox.request {
                return Err(FrameReceiveError::ForeignRequest);
            }
            if !same_source(&mailbox.origin, &turn.stamp)
                || !same_source(&result.observed, &turn.stamp)
                || result.observed.document_revision != turn.stamp.document_revision
            {
                return Err(FrameReceiveError::ForeignScope);
            }
            if turn.stamp.turn_sequence < result.observed.turn_sequence {
                return Err(FrameReceiveError::BeforeObservation);
            }
        }
        // Completed observations are historical data. Pending redraw is still new work.
        if now >= mailbox.expires_at
            && mailbox
                .pending
                .as_ref()
                .is_some_and(|result| matches!(result.state, FrameResultState::Pending))
        {
            if let Some(result) = mailbox.pending.as_mut() {
                result.state = FrameResultState::Rejected(FrameRejection::Expired);
            }
            mailbox.redraw = false;
        }
        Ok(FrameHostUpdate {
            result: mailbox.pending.take(),
            redraw: std::mem::take(&mut mailbox.redraw),
        })
    }
}
