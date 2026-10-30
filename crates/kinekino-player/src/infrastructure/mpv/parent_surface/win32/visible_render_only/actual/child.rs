//! Test child delegates original cleanup authority, release observation and coordinator ownership.
use super::super::*;
use super::{Evidence, explain};
use crate::infrastructure::mpv::parent_surface::SurfaceInputFailure;
use meari_native::application::window_lifecycle::{CleanupProgress, CompletionSignal, NativeChild};

pub(super) enum Owned {
    Ready(Box<VisibleRenderOnlyChild>),
    Rejected(Box<SurfaceInputFailure>),
    Prepare(Box<NativeSurfaceInput>),
}
pub(super) struct State {
    pub(super) owned: Owned,
    pub(super) first_error: Option<Arc<VisibleError>>,
    pub(super) request: Option<CleanupRequest>,
    signal: Option<CompletionSignal>,
    evidence: Rc<RefCell<Evidence>>,
}
impl State {
    pub(super) fn prepare(
        input: std::result::Result<NativeSurfaceInput, Box<SurfaceInputFailure>>,
        evidence: Rc<RefCell<Evidence>>,
    ) -> Self {
        let (owned, error) = match input {
            Ok(input) => match VisibleRenderOnlyChild::prepare(input) {
                Ok(owner) => (Owned::Ready(Box::new(owner)), None),
                Err(failure) => {
                    let PrepareFailure { error, input } = *failure;
                    (Owned::Prepare(Box::new(input)), Some(Arc::new(error)))
                }
            },
            Err(failure) => {
                let error = Arc::new(VisibleError::Input(failure.error.clone()));
                (Owned::Rejected(failure), Some(error))
            }
        };
        Self {
            owned,
            first_error: error,
            request: None,
            signal: None,
            evidence,
        }
    }
    pub(super) fn ready_mut(&mut self) -> SharedResult<&mut VisibleRenderOnlyChild> {
        match &mut self.owned {
            Owned::Ready(owner) => Ok(owner),
            _ => Err(self
                .first_error
                .clone()
                .unwrap_or_else(|| Arc::new(VisibleError::Phase))),
        }
    }
    fn identity(
        &self,
    ) -> (
        meari_native::application::window_lifecycle::WindowStamp,
        u64,
    ) {
        match &self.owned {
            Owned::Ready(owner) => (owner.input.window(), owner.input.child()),
            Owned::Rejected(failure) => (failure.parent.window(), failure.parent.child()),
            Owned::Prepare(input) => (input.window(), input.child()),
        }
    }
    fn cleanup(&mut self, request: CleanupRequest, begin: bool) -> SharedResult<CleanupProgress> {
        if self.identity() != (request.stamp.window(), request.stamp.child()) {
            return Err(Arc::new(VisibleError::Identity));
        }
        if let Some(current) = self.request {
            if current.stamp != request.stamp || current.reason != request.reason {
                return Err(Arc::new(VisibleError::Identity));
            }
        } else if !begin {
            return Err(Arc::new(VisibleError::Phase));
        }
        let start = begin && self.request.is_none();
        match &self.owned {
            Owned::Ready(owner) => owner
                .input
                .with_cleanup_parent(request, |parent| {
                    require_parent(&owner.native, &owner.input, parent)
                })
                .map_err(VisibleError::Input)
                .and_then(|v| v)
                .map_err(Arc::new)?,
            Owned::Rejected(failure) => failure
                .slot
                .with_cleanup_surface(request, |_| ())
                .map_err(|e| {
                    Arc::new(VisibleError::Input(SurfaceInputError::CleanupAccess(
                        Arc::new(e),
                    )))
                })?,
            Owned::Prepare(input) => input
                .with_cleanup_parent(request, |_| ())
                .map_err(|e| Arc::new(VisibleError::Input(e)))?,
        }
        self.request = Some(request);
        let result = match &mut self.owned {
            Owned::Ready(owner) => {
                if self.first_error.is_none() {
                    self.first_error = owner.first_error();
                }
                if start {
                    match owner.revoke(request) {
                        Ok(()) => self.evidence.borrow_mut().hide_observed = true,
                        Err(error) => {
                            self.first_error.get_or_insert(error);
                        }
                    }
                    owner.release(request)
                } else {
                    owner
                        .input
                        .with_cleanup_parent(request, |parent| {
                            require_parent(&owner.native, &owner.input, parent)?;
                            owner
                                .native
                                .observe_quarantine_release()
                                .map_err(VisibleError::Child)
                        })
                        .map_err(VisibleError::Input)
                        .and_then(|v| v)
                        .map_err(Arc::new)
                }
            }
            Owned::Rejected(failure) => failure
                .slot
                .with_cleanup_surface(request, |_| ProbeRelease::NotCreated)
                .map_err(|e| {
                    Arc::new(VisibleError::Input(SurfaceInputError::CleanupAccess(
                        Arc::new(e),
                    )))
                }),
            Owned::Prepare(input) => input
                .with_cleanup_parent(request, |_| ProbeRelease::NotCreated)
                .map_err(|e| Arc::new(VisibleError::Input(e))),
        };
        match result {
            Ok(ProbeRelease::Complete { .. }) => {
                self.evidence.borrow_mut().destroy_ack_and_class_release = true;
                Ok(CleanupProgress::Complete)
            }
            Ok(ProbeRelease::NotCreated) => Ok(CleanupProgress::Complete),
            Ok(ProbeRelease::Unobserved) => Ok(CleanupProgress::ObservationUnavailable),
            Err(error) => {
                self.first_error.get_or_insert_with(|| error.clone());
                self.evidence
                    .borrow_mut()
                    .cleanup_error
                    .get_or_insert_with(|| explain(&error));
                Ok(CleanupProgress::ObservationUnavailable)
            }
        }
    }
}
pub(crate) struct Child {
    pub(super) state: Rc<RefCell<State>>,
    diagnostic: Option<Arc<VisibleError>>,
}
impl Child {
    pub(super) fn new(state: Rc<RefCell<State>>) -> Self {
        Self {
            state,
            diagnostic: None,
        }
    }
}
impl NativeChild for Child {
    type Error = Arc<VisibleError>;
    fn parent_lost(&mut self) {
        let mut state = self.state.borrow_mut();
        match &mut state.owned {
            Owned::Ready(owner) => {
                owner.revoked = true;
                owner.input.revoke();
            }
            Owned::Rejected(failure) => failure.slot.revoke(),
            Owned::Prepare(input) => input.revoke(),
        }
    }
    fn begin_cleanup(
        &mut self,
        request: CleanupRequest,
        signal: CompletionSignal,
    ) -> SharedResult<CleanupProgress> {
        if signal.stamp() != request.stamp {
            return Err(Arc::new(VisibleError::Identity));
        }
        let mut state = self
            .state
            .try_borrow_mut()
            .map_err(|_| Arc::new(VisibleError::Phase))?;
        let result = state.cleanup(request, true);
        if state
            .request
            .is_some_and(|r| r.stamp == request.stamp && r.reason == request.reason)
        {
            state.signal = Some(signal);
        }
        self.diagnostic = state.first_error.clone();
        result
    }
    fn observe_cleanup(&mut self, request: CleanupRequest) -> SharedResult<CleanupProgress> {
        let Ok(mut state) = self.state.try_borrow_mut() else {
            return Ok(CleanupProgress::ObservationUnavailable);
        };
        let result = state.cleanup(request, false);
        self.diagnostic = state.first_error.clone();
        result
    }
    fn observation_error(&self) -> Option<&Self::Error> {
        self.diagnostic.as_ref()
    }
}
