use super::ResolveEpisode;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        catalog::{CaptureMetadata, RawArtifact},
        jobs::VerifiedJobOutput,
    },
    domain::{
        hls::HlsSummary, identifiers::valid_sha256, navigation,
        player_document::PlayerDocumentLocation,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedManifestRecord {
    pub validation_profile: String,
    pub url: String,
    pub referer: crate::domain::request_referer::RequestReferer,
    pub summary: HlsSummary,
    pub artifact: RawArtifact,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredEpisodeResolution {
    pub schema_version: String,
    pub source_descriptor_sha256: String,
    pub episode_url: String,
    pub player_document_url: String,
    pub manifest: ResolvedManifestRecord,
    pub raw_artifacts: Vec<RawArtifact>,
}
impl StoredEpisodeResolution {
    pub fn validate(&self) -> Result<(), ApiError> {
        let episode = ResolveEpisode {
            episode_url: self.episode_url.clone(),
        };
        episode.location()?;
        let descriptor = serde_json::to_string(&serde_json::json!({"operation":"episodes.resolve","parameters":{"episode_url":self.episode_url},"source_kind":"request_descriptor"})).map_err(|_| expired())?;
        use sha2::{Digest, Sha256};
        if self.schema_version != "2"
            || self.source_descriptor_sha256
                != format!("{:x}", Sha256::digest(descriptor.as_bytes()))
            || PlayerDocumentLocation::parse(&self.player_document_url).is_err()
            || !navigation::media_transport_allowed(&self.manifest.url)
            || self.manifest.validation_profile != crate::domain::hls::HLS_VALIDATION_PROFILE
            || !self.manifest.referer.allowed()
            || self.raw_artifacts.is_empty()
            || self.raw_artifacts.len() > 16
            || !self.raw_artifacts.contains(&self.manifest.artifact)
            || self
                .raw_artifacts
                .iter()
                .any(|raw| !valid_sha256(&raw.artifact_id) || !valid_sha256(&raw.body_sha256))
        {
            return Err(expired());
        }
        Ok(())
    }
}
pub trait EpisodeResolutionResults: Send + Sync {
    fn publish(
        &self,
        result: &StoredEpisodeResolution,
        metadata: &CaptureMetadata,
    ) -> Result<VerifiedJobOutput, ApiError>;
    /// Historical verified bytes; does not assert a still-live browser session or unexpired URL.
    fn read(&self, artifact_id: &str) -> Result<StoredEpisodeResolution, ApiError>;
}
pub(crate) fn expired() -> ApiError {
    ApiError::new(
        ErrorCode::SourceExpired,
        "Episode resolution or referenced evidence identity mismatch",
    )
}
