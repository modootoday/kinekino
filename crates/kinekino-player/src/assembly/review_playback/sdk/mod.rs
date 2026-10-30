//! One trusted configuration supplies source storage, evidence and playback.
use super::*;
use crate::{
    application::{
        api::PlayerSdk, frame_capture::FrameCapturePort, model_review::ReviewStore,
        playback_control::PlaybackControlPort, playback_status::PlaybackStatusPort,
        subtitles::storage::SubtitleStorage,
    },
    infrastructure::{
        model_review::ReviewFiles, review_source::open_api,
        timeline_evidence::ReviewTimelineEvidence,
    },
};
#[cfg(test)]
mod tests;

#[must_use = "Retain the original service through transport shutdown and physical completion"]
pub struct ReviewSdkAssembly<S, D, P, C> {
    pub client: PlayerSdk<S, D, P, C, ReviewFiles>,
    pub sessions: Arc<ReviewPlaybackSessions>,
}
/// Replaces the three review adapters together while preserving other supplied
/// SDK capabilities. Admission is trusted Rust input, never inferred from stored
/// Verified data or deserialized from an SDK request.
pub fn assemble_sdk<S, D, P, C, R>(
    base: PlayerSdk<S, D, P, C, R>,
    configuration: LocalReviewSessionConfig,
    writable: bool,
    max_history: usize,
) -> Result<ReviewSdkAssembly<S, D, P, C>, ApiError>
where
    S: SubtitleStorage,
    D: FrameCapturePort,
    P: PlaybackStatusPort,
    C: PlaybackControlPort,
    R: ReviewStore,
{
    let (configuration, admission) = configuration.into_parts();
    let _guards =
        crate::infrastructure::review_playback_sessions::retain_root_layout(&configuration)?;
    let evidence = Arc::new(ReviewTimelineEvidence::new(
        ReviewFiles::open(&configuration.review_root)?,
        admission,
    )?);
    assemble_sdk_with_evidence(base, configuration, evidence, writable, max_history)
}

/// Supplies one live or archive authority to both adapters. The resolver is
/// trusted in-process input, never created from a stored Verified association.
pub fn assemble_sdk_with_evidence<S, D, P, C, R>(
    base: PlayerSdk<S, D, P, C, R>,
    configuration: crate::infrastructure::review_playback_sessions::LocalReviewRuntimeConfig,
    evidence: Arc<
        dyn crate::application::review_source::timeline_evidence::TimelineEvidenceResolver,
    >,
    writable: bool,
    max_history: usize,
) -> Result<ReviewSdkAssembly<S, D, P, C>, ApiError>
where
    S: SubtitleStorage,
    D: FrameCapturePort,
    P: PlaybackStatusPort,
    C: PlaybackControlPort,
    R: ReviewStore,
{
    let review_root = configuration.review_root.clone();
    let user_root = configuration.user_root.clone();
    // Retain all disjoint roots before optional user-store initialization. No
    // media starts here; readonly assembly never creates storage.
    let _root_guards =
        crate::infrastructure::review_playback_sessions::retain_root_layout(&configuration)?;
    let source =
        open_api(&review_root, &user_root, writable)?.with_timeline_evidence(evidence.clone());
    let sessions = assemble_local_with_evidence(configuration, evidence, max_history)?;
    let client = base
        .with_review_store(ReviewFiles::open(&review_root)?)
        .with_review_source(source)
        .with_review_playback_sessions(sessions.clone());
    Ok(ReviewSdkAssembly { client, sessions })
}
