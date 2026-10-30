//! Serializable observation is data, never authority to retain or render a media file.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        review_source::MutationControl,
    },
    domain::{clip_document::ClipSourcePin, identifiers::valid_id},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DurationBasis {
    ReferenceStreamTicks,
    ContainerReportedSpan,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipSourceObservation {
    pub source: ClipSourcePin,
    pub reference_stream_index: u32,
    pub duration_basis: DurationBasis,
    pub raw_probe_sha256: String,
    pub probe_sha256: String,
    pub color_transfer: Option<String>,
    pub rotation_degrees: Option<i32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectClipSource {
    pub media_id: String,
}
pub trait ClipSourceInspector: Send + Sync {
    fn inspect(
        &self,
        media_id: &str,
        control: &dyn MutationControl,
    ) -> Result<ClipSourceObservation, ApiError>;
}
pub fn inspect_clip_source(
    inspector: &dyn ClipSourceInspector,
    request: InspectClipSource,
    control: &dyn MutationControl,
) -> Result<ClipSourceObservation, ApiError> {
    if !valid_id(&request.media_id) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Valid catalog media ID required",
        ));
    }
    control.checkpoint()?;
    let observation = inspector.inspect(&request.media_id, control)?;
    control.checkpoint()?;
    if observation.source.media_id != request.media_id {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Source observation belongs to a different media ID",
        ));
    }
    Ok(observation)
}
