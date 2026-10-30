//! Shared trusted composition for CLI and GUI hosts; constructing opens no media.
use crate::{
    application::{api::ApiError, review_playback::sessions::ReviewPlaybackSessions},
    infrastructure::{
        review_host_identity::fresh_scope,
        review_playback_sessions::{LocalReviewSessionConfig, LocalReviewSessionLauncher},
    },
};
use std::sync::Arc;
mod preparation;
mod sdk;
mod stdio;
mod transport;
pub use preparation::{
    IncompleteTimelineCleanup, TimelinePreparation, TimelinePreparationTools, prepare_timeline,
};
pub use sdk::{ReviewSdkAssembly, assemble_sdk, assemble_sdk_with_evidence};
pub use stdio::{ReviewStdioCompletion, ReviewStdioOutcome, serve_stdio};
pub use transport::{
    PendingReviewTransport, ReviewTransportCompletion, ReviewTransportHost,
    ReviewTransportShutdownObservation, ReviewTransportStartFailure, start_transport,
};

/// Inject this single service into the host's common PlayerSdk. A new host calls
/// this again; persisted or caller-supplied scope identities are forbidden.
/// Configuration remains trusted Rust input and enables no download or GUI.
pub fn assemble_local(
    configuration: LocalReviewSessionConfig,
    max_history: usize,
) -> Result<Arc<ReviewPlaybackSessions>, ApiError> {
    let scope = fresh_scope()?;
    let launcher = LocalReviewSessionLauncher::new(configuration)?;
    Ok(Arc::new(ReviewPlaybackSessions::new(
        scope,
        max_history,
        launcher,
    )?))
}

/// Uses the same retained authority supplied to the SDK's source service.
pub fn assemble_local_with_evidence(
    configuration: crate::infrastructure::review_playback_sessions::LocalReviewRuntimeConfig,
    evidence: Arc<
        dyn crate::application::review_source::timeline_evidence::TimelineEvidenceResolver,
    >,
    max_history: usize,
) -> Result<Arc<ReviewPlaybackSessions>, ApiError> {
    let scope = fresh_scope()?;
    let launcher = LocalReviewSessionLauncher::with_evidence(configuration, evidence)?;
    Ok(Arc::new(ReviewPlaybackSessions::new(
        scope,
        max_history,
        launcher,
    )?))
}
