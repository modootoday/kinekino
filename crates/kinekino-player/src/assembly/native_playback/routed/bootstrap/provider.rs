//! Bind the original live provider to the caller's configured SDK before native activation.
use super::{RoutedClientBootstrap, prepare_routed_client};
use crate::{
    application::{
        api::PlayerSdk,
        frame_capture::FrameCapturePort,
        model_review::ReviewStore,
        playback_control::PlaybackControlPort,
        playback_session::transition::{EpisodeTransitions, TransitionSlot},
        playback_status::PlaybackStatusPort,
        subtitles::storage::SubtitleStorage,
    },
    infrastructure::{
        browser_provider_jobs::RetainingProviderJobsHost, mpv::ParentEngineRouter,
        thread_completion::DispatcherHandle,
    },
};
use std::sync::Arc;

/// `provider` must remain in the original launch/cleanup owner until retained shutdown completes.
/// Starts no worker, opens no store and creates no transport or standalone native engine.
pub fn prepare_provider_routed_client<S, D, P, C, R>(
    base: PlayerSdk<S, D, P, C, R>,
    provider: &RetainingProviderJobsHost,
    dispatcher: DispatcherHandle,
) -> RoutedClientBootstrap
where
    S: SubtitleStorage + Send + Sync + 'static,
    D: FrameCapturePort + Send + Sync + 'static,
    P: PlaybackStatusPort + 'static,
    C: PlaybackControlPort + 'static,
    R: ReviewStore + 'static,
{
    let base = base
        .with_jobs(provider.jobs())
        .with_catalog(provider.catalog());
    prepare_routed_client(base, provider.files(), dispatcher)
}

/// The caller owns this slot: close_admission with original route revocation before cleanup;
/// call end after native/execution cleanup, then begin provider shutdown. Retain job/error evidence.
pub fn prepare_provider_routed_client_with_transitions<S, D, P, C, R>(
    base: PlayerSdk<S, D, P, C, R>,
    provider: &RetainingProviderJobsHost,
    dispatcher: DispatcherHandle,
    transitions: Arc<TransitionSlot>,
) -> RoutedClientBootstrap
where
    S: SubtitleStorage + Send + Sync + 'static,
    D: FrameCapturePort + Send + Sync + 'static,
    P: PlaybackStatusPort + 'static,
    C: PlaybackControlPort + 'static,
    R: ReviewStore + 'static,
{
    let files = provider.files();
    let jobs = provider.jobs();
    let router = Arc::new(
        ParentEngineRouter::new_with_transitions(dispatcher, transitions.clone())
            .with_transition_jobs(jobs.clone()),
    );
    let service = EpisodeTransitions::new(
        files.clone(),
        files.clone(),
        jobs.clone(),
        router.clone(),
        transitions,
    );
    let base = base
        .with_jobs(jobs)
        .with_catalog(provider.catalog())
        .with_episode_transitions(service);
    super::prepare::prepare_using_router(base, files, router)
}
