use super::super::cleanup::CleanupMachine;
use super::admission::OriginalAdmission;
use super::errors::RoutedChildError;
use crate::infrastructure::mpv::{
    NativeEpisodeEngine, NativeSurfaceInput, ParentEngineRouter, ParentMediaSurface,
    ParentRouteHandle, SurfaceInputFailure,
};
use meari_native::application::window_lifecycle::WindowStamp;
use std::{rc::Rc, sync::Arc};
pub(super) enum SurfaceOwner {
    Ready(Box<ParentMediaSurface>),
    Rejected(Box<SurfaceInputFailure>),
    Prepare(Box<NativeSurfaceInput>),
}
pub(super) struct State {
    pub(super) admission: Rc<OriginalAdmission>,
    pub(super) surface: SurfaceOwner,
    pub(super) router: Arc<ParentEngineRouter>,
    pub(super) route: Option<ParentRouteHandle>,
    pub(super) original_engine: Option<Arc<NativeEpisodeEngine>>,
    pub(super) cleanup: Option<CleanupMachine>,
    pub(super) unbound_cleanup: Option<super::unbound_cleanup::UnboundCleanup>,
    pub(super) revoked: bool,
    pub(super) first_error: Option<RoutedChildError>,
}
impl State {
    pub(super) fn identity(&self) -> (WindowStamp, u64) {
        match &self.surface {
            SurfaceOwner::Ready(surface) => surface.identity(),
            SurfaceOwner::Rejected(failure) => (failure.parent.window(), failure.parent.child()),
            SurfaceOwner::Prepare(input) => (input.window(), input.child()),
        }
    }
    pub(super) fn remember(&mut self, error: RoutedChildError) -> RoutedChildError {
        self.first_error.get_or_insert_with(|| error.clone());
        error
    }
}
