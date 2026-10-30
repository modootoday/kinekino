//! Startup failure keeps each original owner; no anyhow conversion can discard a partial host.
use super::{
    ProductPlaybackBootstrap,
    dispatcher_cleanup::{DispatcherCleanup, DispatcherCleanupStatus},
    provider_cleanup::{self, ProviderCleanup},
    transition_lifetime::TransitionLifetime,
};
use crate::{
    application::api::{ApiError, ApplicationClientHandle},
    assembly::meari::{
        native_cleanup_driver::{CleanupDriverOutcome, drive_cleanup},
        viewer_native_launch::routed::RoutedViewerInput,
    },
    infrastructure::{
        browser_provider_jobs::{ProviderStartFailure, RetainingProviderJobsHost},
        thread_completion::DispatcherHost,
    },
};
use kinekino_jobs::infrastructure::RetainingWorkerHost;
use std::time::Duration;

#[must_use = "Keep the original setup error and all incomplete cleanup owners to process exit"]
pub struct ProductPlaybackSetupFailure {
    error: ApiError,
    provider: Option<CleanupDriverOutcome<ProviderCleanup>>,
    dispatcher: Option<CleanupDriverOutcome<DispatcherCleanup>>,
    pending_close_error: Option<String>,
    routed: Option<RoutedViewerInput>,
    client: Option<ApplicationClientHandle>,
    provider_retained: Option<RetainingProviderJobsHost>,
    dispatcher_retained: Option<DispatcherHost>,
    transitions: Option<TransitionLifetime>,
}
impl ProductPlaybackSetupFailure {
    pub(super) fn before_start(error: ApiError) -> Box<Self> {
        Box::new(Self {
            error,
            provider: None,
            dispatcher: None,
            pending_close_error: None,
            routed: None,
            client: None,
            provider_retained: None,
            dispatcher_retained: None,
            transitions: None,
        })
    }
    pub(super) fn provider_start(failure: ProviderStartFailure<RetainingWorkerHost>) -> Box<Self> {
        let (error, pending) = failure.into_shutdown(provider_cleanup::original_deadline());
        Box::new(Self {
            error,
            provider: pending.map(provider_cleanup::observe),
            dispatcher: None,
            pending_close_error: None,
            routed: None,
            client: None,
            provider_retained: None,
            dispatcher_retained: None,
            transitions: None,
        })
    }
    pub(super) fn dispatcher_start(
        error: ApiError,
        provider: RetainingProviderJobsHost,
    ) -> Box<Self> {
        Box::new(Self {
            error,
            provider: Some(provider_cleanup::close(provider)),
            dispatcher: None,
            pending_close_error: None,
            routed: None,
            client: None,
            provider_retained: None,
            dispatcher_retained: None,
            transitions: None,
        })
    }
    pub fn report(&self) -> serde_json::Value {
        serde_json::json!({"success":false,"stage":"routed_setup","first_error":self.error,
            "pending_close_error":self.pending_close_error,"pending_activation_retained":self.routed.is_some(),"original_client_retained":self.client.is_some(),
            "provider_cleanup":self.provider.as_ref().map(|p| &p.evidence),
            "dispatcher_joined":self.dispatcher.as_ref().is_some_and(|d| d.status==DispatcherCleanupStatus::Joined),
            "provider_retained":self.provider_retained.is_some(),"dispatcher_retained":self.dispatcher_retained.is_some(),
            "transition_admission_closed":self.transitions.as_ref().map(TransitionLifetime::admission_closed),
            "transition_end":self.transitions.as_ref().and_then(TransitionLifetime::end_result)})
    }
}
impl ProductPlaybackBootstrap {
    pub fn abort_setup(self, error: ApiError) -> Box<ProductPlaybackSetupFailure> {
        let Self {
            client,
            routed,
            dispatcher,
            provider,
            mut transitions,
        } = self;
        transitions.close_admission();
        let pending_close_error = routed
            .activation
            .close_pending()
            .err()
            .map(|error| error.message);
        // This path runs before a window/registered child/execution owner was started.
        transitions.finish_once();
        if !transitions.ended_successfully() {
            return Box::new(ProductPlaybackSetupFailure {
                error,
                provider: None,
                dispatcher: None,
                pending_close_error,
                routed: Some(routed),
                client: Some(client),
                provider_retained: Some(provider),
                dispatcher_retained: Some(dispatcher),
                transitions: Some(transitions),
            });
        }
        let provider = provider_cleanup::close(provider);
        let dispatcher = drive_cleanup(
            DispatcherCleanup::new(dispatcher),
            Duration::from_millis(2),
            std::thread::sleep,
        );
        Box::new(ProductPlaybackSetupFailure {
            error,
            provider: Some(provider),
            dispatcher: Some(dispatcher),
            pending_close_error,
            routed: Some(routed),
            client: Some(client),
            provider_retained: None,
            dispatcher_retained: None,
            transitions: Some(transitions),
        })
    }
}
