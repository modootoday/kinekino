use super::*;
use crate::infrastructure::mpv::ProbeRelease;
use meari_native::application::window_lifecycle::{CleanupProgress, NativeChild};

pub(super) fn progress(release: ProbeRelease) -> CleanupProgress {
    match release {
        ProbeRelease::Complete { .. } | ProbeRelease::NotCreated => CleanupProgress::Complete,
        ProbeRelease::Unobserved => CleanupProgress::ObservationUnavailable,
    }
}
pub(super) fn release_failure(
    error: QuarantineError,
    class_retry_ready: bool,
) -> Result<CleanupProgress, QuarantineError> {
    if class_retry_ready {
        Err(error)
    } else {
        Ok(CleanupProgress::ObservationUnavailable)
    }
}
pub(super) fn may_retry_release(
    begin: bool,
    new_attempt: bool,
    entered: bool,
    class_ready: bool,
) -> bool {
    (begin && new_attempt && !entered) || (entered && class_ready)
}
pub(super) fn starts_new_attempt(
    previous: Option<CleanupRequest>,
    request: CleanupRequest,
) -> Result<bool, QuarantineError> {
    match previous {
        Some(current) if current.stamp == request.stamp => {
            if current.reason != request.reason {
                Err(QuarantineError::Identity)
            } else {
                Ok(false)
            }
        }
        _ => Ok(true),
    }
}
impl OwnerState {
    fn cleanup(
        &mut self,
        request: CleanupRequest,
        begin: bool,
    ) -> Result<CleanupProgress, QuarantineError> {
        let identity = match &self.input {
            OriginalInput::Ready(input) => (input.window(), input.child()),
            OriginalInput::Rejected(failure) => (failure.parent.window(), failure.parent.child()),
        };
        if identity != (request.stamp.window(), request.stamp.child()) {
            return Err(QuarantineError::Identity);
        }
        let new_attempt = starts_new_attempt(self.request, request)?;
        if !begin
            && !self.request.is_some_and(|current| {
                current.stamp == request.stamp && current.reason == request.reason
            })
        {
            return Err(QuarantineError::Identity);
        }
        // Duplicate begin of the same attempt observes only; a new explicit attempt can retry.
        let start = begin && new_attempt;
        let mut entered = false;
        let result = match &self.input {
            OriginalInput::Ready(input) => input
                .with_cleanup_parent(request, |_| {
                    entered = true;
                    match self.probe.as_mut() {
                        Some(probe) if start => probe.release(),
                        Some(probe) => probe.observe_quarantine_release(),
                        None => Ok(ProbeRelease::NotCreated),
                    }
                })
                .map_err(|error| QuarantineError::Input(Arc::new(error)))
                .and_then(|result| result.map_err(|error| QuarantineError::Probe(Arc::new(error)))),
            // Rejected metadata never reached OS creation. Still require original cleanup slot access.
            OriginalInput::Rejected(failure) => failure
                .slot
                .with_cleanup_surface(request, |_| {
                    entered = true;
                    ProbeRelease::NotCreated
                })
                .map_err(|error| {
                    QuarantineError::Input(Arc::new(
                        crate::infrastructure::mpv::SurfaceInputError::CleanupAccess(Arc::new(
                            error,
                        )),
                    ))
                }),
        };
        if entered {
            self.revoked = true;
            self.request = Some(request);
        }
        match result {
            Ok(release) => Ok(progress(release)),
            Err(error) => {
                self.first_error.get_or_insert_with(|| error.clone());
                // Actual coordinator retries Failed entries only. A confirmed destroyed child
                // with a remaining registered class has no Destroy work in flight.
                // An uncertain OS return is not a terminal failure or completion acknowledgement.
                release_failure(
                    error,
                    may_retry_release(
                        begin,
                        new_attempt,
                        entered,
                        self.probe
                            .as_ref()
                            .is_some_and(HiddenSurfaceProbe::quarantine_class_retry_ready),
                    ),
                )
            }
        }
    }
}
impl NativeChild for QuarantineChild {
    type Error = QuarantineError;
    fn parent_lost(&mut self) {
        let mut state = self.owner.borrow_mut();
        state.revoked = true;
        match &state.input {
            OriginalInput::Ready(input) => input.revoke(),
            OriginalInput::Rejected(failure) => failure.slot.revoke(),
        }
    }
    fn begin_cleanup(
        &mut self,
        request: CleanupRequest,
        signal: CompletionSignal,
    ) -> Result<CleanupProgress, Self::Error> {
        let Ok(mut state) = self.owner.try_borrow_mut() else {
            self.diagnostic.get_or_insert(QuarantineError::Unavailable);
            // No cleanup request/signal was installed and no teardown started.
            return Err(QuarantineError::Unavailable);
        };
        if signal.stamp() != request.stamp {
            return Err(QuarantineError::Identity);
        }
        let result = state.cleanup(request, true);
        if self.diagnostic.is_none() {
            self.diagnostic = state.first_error.clone();
        }
        if state.request.is_some_and(|current| {
            current.stamp == request.stamp && current.reason == request.reason
        }) {
            state.signal = Some(signal);
        }
        result
    }
    fn observe_cleanup(&mut self, request: CleanupRequest) -> Result<CleanupProgress, Self::Error> {
        let Ok(mut state) = self.owner.try_borrow_mut() else {
            self.diagnostic.get_or_insert(QuarantineError::Unavailable);
            return Ok(CleanupProgress::ObservationUnavailable);
        };
        let result = state.cleanup(request, false);
        if self.diagnostic.is_none() {
            self.diagnostic = state.first_error.clone();
        }
        result
    }
    fn observation_error(&self) -> Option<&Self::Error> {
        self.diagnostic.as_ref()
    }
}
