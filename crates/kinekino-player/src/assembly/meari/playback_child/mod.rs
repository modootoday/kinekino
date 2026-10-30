//! UI ownership of the native parent; cleanup workers receive no window authority.
//!
//! Explicit retry retains the router's original retirement task and typed evidence.
//! Lease/broker association is a trusted assembly contract, not an OS validation.
mod cleanup;
pub(crate) mod routed;

use crate::{
    application::api::ApiError,
    infrastructure::mpv::{ParentEngineRouter, ParentRouteHandle},
};
use cleanup::CleanupMachine;
pub use cleanup::PlaybackCleanupError;
use meari_native::{
    application::window_lifecycle::{
        CleanupProgress, CleanupRequest, CompletionSignal, NativeChild,
    },
    infrastructure::window::NativeParentLease,
};
use std::sync::Arc;

/// Creation failure retains the actual lease and original route/engine owner.
pub struct NativePlaybackChildFailure {
    pub error: ApiError,
    pub parent: NativeParentLease,
    pub router: Arc<ParentEngineRouter>,
    pub route: ParentRouteHandle,
}

/// Retain on the UI thread until the coordinator releases this child.
pub struct NativePlaybackChild {
    _parent: NativeParentLease,
    cleanup: CleanupMachine,
}

impl NativePlaybackChild {
    /// Trusted assembly must pair the lease with its router generation. This constructor
    /// stores both identities; it does not authenticate an OS parent/broker association.
    pub fn new(
        parent: NativeParentLease,
        router: Arc<ParentEngineRouter>,
        route: ParentRouteHandle,
    ) -> Result<Self, Box<NativePlaybackChildFailure>> {
        let cleanup = match CleanupMachine::new(
            parent.window(),
            parent.child(),
            router.clone(),
            route.clone(),
        ) {
            Ok(cleanup) => cleanup,
            Err(error) => {
                return Err(Box::new(NativePlaybackChildFailure {
                    error,
                    parent,
                    router,
                    route,
                }));
            }
        };
        Ok(Self {
            _parent: parent,
            cleanup,
        })
    }
}

impl NativeChild for NativePlaybackChild {
    type Error = PlaybackCleanupError;
    fn parent_lost(&mut self) {
        self.cleanup.parent_lost();
    }
    fn begin_cleanup(
        &mut self,
        request: CleanupRequest,
        signal: CompletionSignal,
    ) -> Result<CleanupProgress, Self::Error> {
        self.cleanup.begin_cleanup(request, signal)
    }
    fn observe_cleanup(&mut self, request: CleanupRequest) -> Result<CleanupProgress, Self::Error> {
        self.cleanup.observe_cleanup(request)
    }
    fn observation_error(&self) -> Option<&Self::Error> {
        self.cleanup.observation_error()
    }
}

#[cfg(test)]
mod tests;
