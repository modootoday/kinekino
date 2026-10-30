//! Request and historical result data; native error sources retain their original identities.
use super::super::QuarantineError;
use meari_native::{
    application::window_lifecycle::WindowStamp, domain::graphics::FrameTicket,
    infrastructure::native_host::HostStamp,
};
use std::{
    num::{NonZeroU64, NonZeroUsize},
    sync::Arc,
    time::Instant,
};

#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
pub(crate) struct FrameRequest {
    pub id: NonZeroU64,
    pub origin: HostStamp,
    pub video: String,
    pub expires_at: Instant,
    pub frame_budget: NonZeroUsize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
pub(crate) enum FrameRejection {
    Expired,
    BudgetExhausted,
    ForeignFrame,
    SourceChanged,
    WindowChanged,
    DocumentChanged,
    NativeFailure,
    Revoked,
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
pub(crate) enum FrameResultState {
    Pending,
    ObservedDisabled { frame: FrameTicket },
    Rejected(FrameRejection),
}
/// Historical observation only: no lease, snapshot, checked proof, or execution authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FrameOwnedResult {
    pub request: NonZeroU64,
    pub observed: HostStamp,
    pub window: Option<WindowStamp>,
    pub state: FrameResultState,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FrameReceiveError {
    Busy,
    ForeignRequest,
    ForeignScope,
    BeforeObservation,
}
pub(crate) struct FrameHostUpdate {
    pub result: Option<FrameOwnedResult>,
    pub redraw: bool,
}
#[derive(Clone, Debug)]
#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
pub(crate) enum FrameFailure {
    Rejected(FrameRejection),
    Native(Arc<QuarantineError>),
    Access(Arc<meari_native::infrastructure::window::NativeSurfaceAccessError>),
    Host(Arc<meari_native::infrastructure::native_host::HostAccessError>),
    Lifecycle(Arc<meari_native::application::window_lifecycle::LifecycleError>),
}
#[allow(
    dead_code,
    reason = "NoMedia opt-in frame producer is intentionally uninstalled pending production host assignment"
)]
impl FrameFailure {
    pub(super) fn rejection(&self) -> FrameRejection {
        match self {
            Self::Rejected(reason) => *reason,
            _ => FrameRejection::NativeFailure,
        }
    }
}
impl std::fmt::Display for FrameFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NoMedia frame: {self:?}")
    }
}
impl std::error::Error for FrameFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Rejected(_) => None,
            Self::Native(error) => Some(error.as_ref()),
            Self::Access(error) => Some(error.as_ref()),
            Self::Host(error) => Some(error.as_ref()),
            Self::Lifecycle(error) => Some(error.as_ref()),
        }
    }
}
