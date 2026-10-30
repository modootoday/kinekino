//! One fully configured SDK before the window; only its original native port activates later.
mod activate;
mod prepare;
mod provider;
use crate::{
    application::{
        api::{ApiError, ApplicationClientHandle},
        playback_session::EpisodePlaybackSessions,
        skip_segments::host::late_routed::LateRoutedInstaller,
    },
    infrastructure::mpv::ParentEngineRouter,
};
pub(crate) use activate::{ActivatedRoutedClient, RoutedActivationFailure};
pub use prepare::prepare_routed_client;
pub use provider::{
    prepare_provider_routed_client, prepare_provider_routed_client_with_transitions,
};
use std::sync::Arc;

#[must_use = "Pass the same client to the original host/execution owner and retain activation"]
pub struct RoutedClientBootstrap {
    pub(super) client: ApplicationClientHandle,
    pub(super) activation: PendingRoutedActivation,
}
impl RoutedClientBootstrap {
    pub fn client(&self) -> ApplicationClientHandle {
        self.client.clone()
    }
    pub(crate) fn into_parts(self) -> (ApplicationClientHandle, PendingRoutedActivation) {
        (self.client, self.activation)
    }
}

#[must_use = "Retain until original checked activation or explicit pending close"]
pub(crate) struct PendingRoutedActivation {
    client: ApplicationClientHandle,
    router: Arc<ParentEngineRouter>,
    sessions: Arc<EpisodePlaybackSessions>,
    installer: LateRoutedInstaller,
}
impl PendingRoutedActivation {
    pub(crate) fn router(&self) -> Arc<ParentEngineRouter> {
        self.router.clone()
    }
    pub(crate) fn close_pending(&self) -> Result<(), ApiError> {
        self.installer.close_pending()
    }
}
