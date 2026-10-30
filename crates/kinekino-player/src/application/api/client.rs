#[path = "client/ports/mod.rs"]
mod ports;
pub use ports::{
    ApplicationClient, AssetClient, CardClient, ClipClient, EpisodeClient, JobBatchClient,
    JobClient, PlaybackClient, ProjectClient, ProposalClient, ReviewClient,
};
mod episode_transition;
mod skip_segments;
use super::{ApiError, ErrorCode, Request, RequestContext, Response};
use crate::application::subtitles::convert::{ConversionResult, ConvertSubtitles};
pub use episode_transition::EpisodeTransitionClient;
pub use skip_segments::SkipSegmentsClient;
use std::sync::Arc;

/// Shared, transport-neutral client. UI code receives this handle from assembly.
pub type ApplicationClientHandle = Arc<dyn ApplicationClient>;

fn invalid_response() -> ApiError {
    ApiError::new(ErrorCode::Internal, "Invalid application response")
}
fn review_call<T: serde::Serialize, U: serde::de::DeserializeOwned>(
    client: &(impl ApplicationClient + ?Sized),
    context: RequestContext,
    command: &str,
    payload: T,
) -> Result<U, ApiError> {
    let mut request = Request::new(command);
    request.context = context;
    request.payload = serde_json::to_value(payload).map_err(|_| invalid_response())?;
    let response = client.execute(request);
    if let Some(error) = response.error {
        return Err(error);
    }
    if !response.ok {
        return Err(invalid_response());
    }
    serde_json::from_value(response.data.ok_or_else(invalid_response)?)
        .map_err(|_| invalid_response())
}
