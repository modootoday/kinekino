//! One configured client before project reads; original native activation after checked frames.
mod dispatcher_cleanup;
mod observation;
mod provider_cleanup;
mod setup;
mod transition_lifetime;
use super::{ProductViewerOutcome, SelectedViewerProject, prepare_product_input};
use crate::{
    application::{
        api::{ApiError, ApplicationClientHandle, ErrorCode, PlayerSdk},
        frame_capture::FrameCapturePort,
        launch::browser_provider::BrowserProviderConfiguration,
        model_review::ReviewStore,
        playback_control::PlaybackControlPort,
        playback_status::PlaybackStatusPort,
        subtitles::storage::SubtitleStorage,
    },
    assembly::{
        meari::{
            native_cleanup_driver::{CleanupDriverOutcome, drive_cleanup},
            playback_child::routed::RegisteredRoutedChild,
            viewer_cleanup::ViewerCleanupLimits,
            viewer_native_launch::{
                finish_typed_viewer,
                routed::{RoutedFailureRetention, RoutedViewerInput, prepare_routed_viewer},
            },
        },
        native_playback::routed::bootstrap::prepare_provider_routed_client_with_transitions,
    },
    infrastructure::{
        browser_provider_jobs::RetainingProviderJobsHost, local_host::LocalHost,
        mpv::TrustedMpvExecutable, thread_completion::DispatcherHost,
    },
};
use dispatcher_cleanup::{DispatcherCleanup, DispatcherCleanupStatus};
pub use setup::ProductPlaybackSetupFailure;
use std::{num::NonZeroUsize, path::Path, sync::Arc, time::Duration};
use transition_lifetime::TransitionLifetime;

pub struct ProductPlaybackConfiguration<'a> {
    pub provider: &'a BrowserProviderConfiguration,
    pub executable: &'a Path,
    pub sha256: &'a str,
}
#[must_use = "Retain the original dispatcher and unique pending activation"]
pub struct ProductPlaybackBootstrap {
    client: ApplicationClientHandle,
    routed: RoutedViewerInput,
    dispatcher: DispatcherHost,
    provider: RetainingProviderJobsHost,
    transitions: TransitionLifetime,
}
pub fn prepare_product_playback<S, D, P, C, R>(
    base: PlayerSdk<S, D, P, C, R>,
    configuration: ProductPlaybackConfiguration<'_>,
) -> Result<ProductPlaybackBootstrap, Box<ProductPlaybackSetupFailure>>
where
    S: SubtitleStorage + Send + Sync + 'static,
    D: FrameCapturePort + Send + Sync + 'static,
    P: PlaybackStatusPort + 'static,
    C: PlaybackControlPort + 'static,
    R: ReviewStore + 'static,
{
    let executable = TrustedMpvExecutable::open(configuration.executable, configuration.sha256)
        .map_err(ProductPlaybackSetupFailure::before_start)?;
    let provider = RetainingProviderJobsHost::start_retaining(
        &configuration.provider.job_root,
        &configuration.provider.catalog_root,
        &configuration.provider.profile_root,
    )
    .map_err(ProductPlaybackSetupFailure::provider_start)?;
    let files = provider.files();
    let dispatcher =
        match DispatcherHost::start(NonZeroUsize::new(2).expect("native dispatcher capacity")) {
            Ok(dispatcher) => dispatcher,
            Err(error) => {
                return Err(ProductPlaybackSetupFailure::dispatcher_start(
                    ApiError::new(ErrorCode::Internal, format!("Native dispatcher: {error}")),
                    provider,
                ));
            }
        };
    let transitions = TransitionLifetime::new();
    let bootstrap = prepare_provider_routed_client_with_transitions(
        base,
        &provider,
        dispatcher.handle(),
        transitions.slot(),
    );
    let (client, activation) = bootstrap.into_parts();
    Ok(ProductPlaybackBootstrap {
        client,
        routed: RoutedViewerInput {
            executable,
            artifacts: files,
            activation,
            transitions: transitions.slot(),
        },
        dispatcher,
        provider,
        transitions,
    })
}
pub fn configured_client<S, D, P, C, R>(
    base: PlayerSdk<S, D, P, C, R>,
    configuration: Option<
        &crate::application::launch::native_playback::NativePlaybackConfiguration,
    >,
    provider: Option<&BrowserProviderConfiguration>,
) -> Result<
    (ApplicationClientHandle, Option<ProductPlaybackBootstrap>),
    Box<ProductPlaybackSetupFailure>,
>
where
    S: SubtitleStorage + Send + Sync + 'static,
    D: FrameCapturePort + Send + Sync + 'static,
    P: PlaybackStatusPort + 'static,
    C: PlaybackControlPort + 'static,
    R: ReviewStore + 'static,
{
    match (configuration, provider) {
        (Some(configuration), Some(provider)) => {
            let bootstrap = prepare_product_playback(
                base,
                ProductPlaybackConfiguration {
                    provider,
                    executable: &configuration.executable,
                    sha256: &configuration.sha256,
                },
            )?;
            Ok((bootstrap.client(), Some(bootstrap)))
        }
        (None, None) => Ok((Arc::new(base), None)),
        _ => Err(ProductPlaybackSetupFailure::before_start(ApiError::new(
            ErrorCode::NeedsInput,
            "Live native playback requires explicit mpv/catalog, job storage and browser profile configuration",
        ))),
    }
}
impl ProductPlaybackBootstrap {
    pub fn client(&self) -> ApplicationClientHandle {
        self.client.clone()
    }
    pub fn run(
        self,
        endpoint: LocalHost,
        project: Option<SelectedViewerProject>,
    ) -> ProductPlaybackOutcome {
        let Self {
            client,
            routed,
            dispatcher,
            provider,
            mut transitions,
        } = self;
        let input = match prepare_product_input(client, endpoint) {
            Ok(input) => input,
            Err(failure) => {
                transitions.close_admission();
                let pending_close_error = routed
                    .activation
                    .close_pending()
                    .err()
                    .map(|error| error.message);
                return ProductPlaybackOutcome {
                    viewer: ProductViewerOutcome::StartFailure(failure),
                    retained: Some(RoutedFailureRetention {
                        _retained_media_error: None,
                        diagnostic: "original execution startup failed".into(),
                        routed: Some(routed),
                        consumer: None,
                    }),
                    dispatcher: Some(dispatcher),
                    dispatcher_cleanup: None,
                    pending_close_error,
                    provider: Some(provider),
                    provider_cleanup: None,
                    transitions,
                };
            }
        };
        let (owners, retained) = match prepare_routed_viewer(input, routed, project) {
            Ok(prepared) => (prepared.run(), None),
            Err(failure) => {
                let (owners, retention) = failure.into_cleanup_parts();
                (*owners, Some(retention))
            }
        };
        let pending_close_error = retained
            .as_ref()
            .and_then(RoutedFailureRetention::close_pending);
        transitions.close_admission();
        let viewer = ProductViewerOutcome::Finished(Box::new(finish_typed_viewer(
            owners,
            ViewerCleanupLimits::new(Duration::from_secs(2), 500, Duration::from_millis(2))
                .expect("fixed Viewer cleanup limits"),
        )));
        if !viewer.succeeded() {
            return ProductPlaybackOutcome {
                viewer,
                retained,
                dispatcher: Some(dispatcher),
                dispatcher_cleanup: None,
                pending_close_error,
                provider: Some(provider),
                provider_cleanup: None,
                transitions,
            };
        }
        transitions.finish_once();
        if !transitions.ended_successfully() {
            return ProductPlaybackOutcome {
                viewer,
                retained,
                dispatcher: Some(dispatcher),
                dispatcher_cleanup: None,
                pending_close_error,
                provider: Some(provider),
                provider_cleanup: None,
                transitions,
            };
        }
        let provider_cleanup = Some(provider_cleanup::close(provider));
        let dispatcher_cleanup = Some(drive_cleanup(
            DispatcherCleanup::new(dispatcher),
            Duration::from_millis(2),
            std::thread::sleep,
        ));
        ProductPlaybackOutcome {
            viewer,
            retained,
            dispatcher: None,
            dispatcher_cleanup,
            pending_close_error,
            provider: None,
            provider_cleanup,
            transitions,
        }
    }
}
#[must_use = "Original dispatcher/failed setup actors remain owned; Drop is not a cleanup receipt"]
pub struct ProductPlaybackOutcome {
    viewer: ProductViewerOutcome<RegisteredRoutedChild>,
    retained: Option<RoutedFailureRetention>,
    dispatcher: Option<DispatcherHost>,
    dispatcher_cleanup: Option<CleanupDriverOutcome<DispatcherCleanup>>,
    pending_close_error: Option<String>,
    provider: Option<RetainingProviderJobsHost>,
    provider_cleanup: Option<CleanupDriverOutcome<provider_cleanup::ProviderCleanup>>,
    transitions: TransitionLifetime,
}
impl ProductPlaybackOutcome {
    pub fn end_review(self) -> Result<(), Box<Self>> {
        if !self.succeeded() {
            return Err(Box::new(self));
        }
        // Called only for full product exit, after original child/execution and dispatcher
        // completion. Panel collapse never calls this consuming lifetime boundary.
        drop(self);
        Ok(())
    }
    pub fn succeeded(&self) -> bool {
        self.viewer.succeeded()
            && self.retained.is_none()
            && self.pending_close_error.is_none()
            && self.transitions.ended_successfully()
            && self
                .provider_cleanup
                .as_ref()
                .is_some_and(|p| p.status == provider_cleanup::Status::Complete)
            && self
                .dispatcher_cleanup
                .as_ref()
                .is_some_and(|c| c.status == DispatcherCleanupStatus::Joined)
    }
    pub fn report(&self) -> serde_json::Value {
        let mut report = self.viewer.report();
        report["routed_setup_error"] =
            serde_json::json!(self.retained.as_ref().map(|r| &r.diagnostic));
        report["pending_close_error"] = serde_json::json!(self.pending_close_error);
        report["provider_retained"] = serde_json::json!(self.provider.is_some());
        report["provider_cleanup"] =
            serde_json::json!(self.provider_cleanup.as_ref().map(|p| &p.evidence));
        report["retained_pending_setup"] =
            serde_json::json!(self.retained.as_ref().is_some_and(|r| r.routed.is_some()));
        report["retained_rejected_consumer"] =
            serde_json::json!(self.retained.as_ref().is_some_and(|r| r.consumer.is_some()));
        report["dispatcher"] = serde_json::json!(
            self.dispatcher
                .as_ref()
                .map(|d| format!("{:?}", d.handle().status()))
        );
        report["dispatcher_cleanup"] = serde_json::json!(
            self.dispatcher_cleanup
                .as_ref()
                .map(|c| format!("{:?}", c.evidence))
        );
        report["media_started"] = serde_json::Value::Null;
        report["dispatcher_joined"] = serde_json::Value::Bool(
            self.dispatcher_cleanup
                .as_ref()
                .is_some_and(|c| c.status == DispatcherCleanupStatus::Joined),
        );
        report["success"] = serde_json::Value::Bool(self.succeeded());
        report["transition_admission_closed"] =
            serde_json::json!(self.transitions.admission_closed());
        report["transition_end"] = serde_json::json!(self.transitions.end_result());
        report
    }
    pub(crate) fn original_transition_lifetime(&self) -> &TransitionLifetime {
        &self.transitions
    }
    pub(crate) fn original_provider_shutdown(
        &self,
    ) -> Option<
        &crate::infrastructure::browser_provider_jobs::ProviderShutdownOutcome<
            kinekino_jobs::infrastructure::RetainingWorkerHost,
        >,
    > {
        self.provider_cleanup
            .as_ref()
            .map(|cleanup| &cleanup.owners)
    }
    pub(crate) fn original_dispatcher_shutdown(
        &self,
    ) -> Option<&crate::infrastructure::thread_completion::DispatcherShutdown> {
        self.dispatcher_cleanup
            .as_ref()
            .map(|cleanup| &cleanup.evidence)
    }
    pub(crate) fn original_viewer_outcome(&self) -> &ProductViewerOutcome<RegisteredRoutedChild> {
        &self.viewer
    }
}
