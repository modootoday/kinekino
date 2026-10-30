use super::{PendingRoutedActivation, RoutedClientBootstrap};
use crate::{
    application::{
        api::{ApplicationClientHandle, PlayerSdk},
        episode_resolve::EpisodeResolutionResults,
        frame_capture::FrameCapturePort,
        model_review::ReviewStore,
        playback_control::PlaybackControlPort,
        playback_session::EpisodePlaybackSessions,
        playback_status::PlaybackStatusPort,
        skip_segments::host::late_routed::LateRoutedPort,
        subtitles::storage::SubtitleStorage,
    },
    infrastructure::{mpv::ParentEngineRouter, thread_completion::DispatcherHandle},
};
use std::sync::Arc;

/// Consume the caller's one configured SDK. Existing review/proposal/job/catalog stores are retained.
/// `results` is the caller's already-open shared results port; no store or worker is opened here.
pub fn prepare_routed_client<S, D, P, C, R>(
    base: PlayerSdk<S, D, P, C, R>,
    results: Arc<dyn EpisodeResolutionResults>,
    dispatcher: DispatcherHandle,
) -> RoutedClientBootstrap
where
    S: SubtitleStorage + Send + Sync + 'static,
    D: FrameCapturePort + Send + Sync + 'static,
    P: PlaybackStatusPort + 'static,
    C: PlaybackControlPort + 'static,
    R: ReviewStore + 'static,
{
    let router = Arc::new(ParentEngineRouter::new(dispatcher));
    prepare_using_router(base, results, router)
}

pub(super) fn prepare_using_router<S, D, P, C, R>(
    base: PlayerSdk<S, D, P, C, R>,
    results: Arc<dyn EpisodeResolutionResults>,
    router: Arc<ParentEngineRouter>,
) -> RoutedClientBootstrap
where
    S: SubtitleStorage + Send + Sync + 'static,
    D: FrameCapturePort + Send + Sync + 'static,
    P: PlaybackStatusPort + 'static,
    C: PlaybackControlPort + 'static,
    R: ReviewStore + 'static,
{
    let sessions = Arc::new(EpisodePlaybackSessions::new(results, router.clone()));
    let (port, installer) = LateRoutedPort::prepare(sessions.clone());
    let client: ApplicationClientHandle = Arc::new(
        base.with_shared_episode_playback(sessions.clone())
            .with_skip_host(port),
    );
    let activation = PendingRoutedActivation {
        client: client.clone(),
        router,
        sessions,
        installer,
    };
    RoutedClientBootstrap { client, activation }
}
