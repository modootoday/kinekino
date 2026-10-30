use super::EpisodeResolutionResults;
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::{hls::HlsSummary, identifiers::valid_sha256, request_referer::RequestReferer},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectEpisodeResolution {
    pub resolution_artifact_id: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeResolutionInspection {
    pub resolution_artifact_id: String,
    pub source_descriptor_sha256: String,
    pub manifest_artifact_id: String,
    pub validation_profile: String,
    pub manifest: HlsSummary,
    pub referer_kind: String,
    pub manifest_verified: bool,
    pub referenced_resources_verified: bool,
    pub media_decoded: bool,
    pub native_playback: bool,
    pub live_session_capability: bool,
    pub url_expiration_verified: bool,
    pub source_uris_redacted: bool,
}
pub fn inspect(
    results: &dyn EpisodeResolutionResults,
    input: InspectEpisodeResolution,
) -> Result<EpisodeResolutionInspection, ApiError> {
    if !valid_sha256(&input.resolution_artifact_id) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid episode resolution artifact ID",
        ));
    }
    let stored = results.read(&input.resolution_artifact_id)?;
    stored.validate()?;
    let referer_kind = match stored.manifest.referer {
        RequestReferer::Absent => "absent",
        RequestReferer::Empty => "empty",
        RequestReferer::Url(_) => "url",
    };
    Ok(EpisodeResolutionInspection {
        resolution_artifact_id: input.resolution_artifact_id,
        source_descriptor_sha256: stored.source_descriptor_sha256,
        manifest_artifact_id: stored.manifest.artifact.artifact_id,
        validation_profile: stored.manifest.validation_profile,
        manifest: stored.manifest.summary,
        referer_kind: referer_kind.into(),
        manifest_verified: true,
        referenced_resources_verified: false,
        media_decoded: false,
        native_playback: false,
        live_session_capability: false,
        url_expiration_verified: false,
        source_uris_redacted: true,
    })
}
