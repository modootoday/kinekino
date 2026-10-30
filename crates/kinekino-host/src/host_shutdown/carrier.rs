use kinekino_player::{
    application::api::ApiError,
    infrastructure::local_host::{
        DispatchState, LocalHost, LocalHostShutdownStatus, PendingLocalHost, WorkerJoin,
    },
};

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum JoinView {
    Pending,
    Joined,
    Panicked,
}
#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DispatchView {
    Idle,
    Accepted,
    Running,
    /// SDK invocation returned (including a caught panic), not business operation success.
    Completed,
}
/// A lossless diagnostic view of the original transport receipt, not a capability or success flag.
/// dispatch=None denotes poisoned state; it is never inferred as idle or completed.
#[derive(serde::Serialize)]
pub(crate) struct LocalHostShutdownView {
    pipe: JoinView,
    sdk: JoinView,
    dispatch: Option<DispatchView>,
    dispatch_poisoned: bool,
}
impl From<LocalHostShutdownStatus> for LocalHostShutdownView {
    fn from(status: LocalHostShutdownStatus) -> Self {
        let join = |state| match state {
            WorkerJoin::Pending => JoinView::Pending,
            WorkerJoin::Joined => JoinView::Joined,
            WorkerJoin::Panicked => JoinView::Panicked,
        };
        Self {
            pipe: join(status.pipe),
            sdk: join(status.sdk),
            dispatch: status.dispatch.map(|state| match state {
                DispatchState::Idle => DispatchView::Idle,
                DispatchState::Accepted => DispatchView::Accepted,
                DispatchState::Running => DispatchView::Running,
                DispatchState::Completed => DispatchView::Completed,
            }),
            dispatch_poisoned: status.dispatch.is_none(),
        }
    }
}

/// Holds the original dependency until explicit pipe and SDK joins are observed.
/// Dropping this carrier abandons join proof; callers must retain incomplete outcomes.
#[must_use = "Keep this original shutdown owner until its transport joins are observed"]
pub(crate) struct ClosingHost<D> {
    pending: PendingLocalHost,
    dependency: D,
    first_error: Option<ApiError>,
}
pub(super) enum Readiness<D> {
    Ready(ReadyForDependencies<D>),
    Incomplete(ClosingHost<D>, LocalHostShutdownStatus),
}
impl<D> ClosingHost<D> {
    pub(crate) fn begin(local: LocalHost, dependency: D, first_error: Option<ApiError>) -> Self {
        Self {
            pending: local.begin_shutdown(),
            dependency,
            first_error,
        }
    }
    pub(crate) fn first_error(&self) -> Option<&ApiError> {
        self.first_error.as_ref()
    }
    /// Only this original Pending owner can supply the join evidence for extraction.
    pub(super) fn try_ready(mut self) -> Readiness<D> {
        let transport = self.pending.poll();
        if !transport.is_quiescent() {
            return Readiness::Incomplete(self, transport);
        }
        Readiness::Ready(ReadyForDependencies {
            dependency: self.dependency,
            first_error: self.first_error,
            transport,
        })
    }
}

/// Transport workers joined normally. Dependency cleanup and operation success are separate.
#[must_use = "Recover the original dependency and finish its separately owned cleanup"]
pub(crate) struct ReadyForDependencies<D> {
    dependency: D,
    first_error: Option<ApiError>,
    transport: LocalHostShutdownStatus,
}
impl<D> ReadyForDependencies<D> {
    pub(crate) fn into_parts(self) -> (D, Option<ApiError>, LocalHostShutdownStatus) {
        (self.dependency, self.first_error, self.transport)
    }
}
