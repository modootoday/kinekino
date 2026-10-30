use super::*;
use crate::{
    application::{
        catalog::{CatalogCaptureKind, CatalogRawSource},
        episode_resolve::{EpisodeResolutionResults, StoredEpisodeResolution, resolution_expired},
        jobs::VerifiedJobOutput,
    },
    domain::{api_values::DecimalU64, hls},
};
use serde_json::json;

impl CatalogFiles {
    fn verify_episode_evidence(&self, result: &StoredEpisodeResolution) -> Result<(), ApiError> {
        result.validate()?;
        for reference in &result.raw_artifacts {
            if self.load_raw(&reference.artifact_id)?.reference != *reference {
                return Err(resolution_expired());
            }
        }
        let manifest = self.load_raw(&result.manifest.artifact.artifact_id)?;
        let capture = &manifest.capture;
        let network = capture
            .browser
            .as_ref()
            .and_then(|browser| browser.network.as_ref())
            .ok_or_else(resolution_expired)?;
        if !matches!(
            capture.capture_kind,
            CatalogCaptureKind::BrowserResponseBody
        ) || capture
            .status
            .is_none_or(|status| !(200..300).contains(&status))
            || capture.final_url != result.manifest.url
            || network.response.url != result.manifest.url
            || Some(network.response.status) != capture.status
            || network
                .request_headers
                .as_ref()
                .map(|headers| &headers.referer)
                != Some(&result.manifest.referer)
            || network.scope.url != result.player_document_url
            || network.scope.session != network.session
            || network.request_method.as_deref() != Some("GET")
            || !matches!(
                network.purpose,
                Some(
                    crate::application::browser_network::ResponsePurpose::ManifestCandidate
                        | crate::application::browser_network::ResponsePurpose::MediaProbe
                )
            )
            || hls::verify(&result.manifest.url, &manifest.body)
                .map_err(|_| resolution_expired())?
                .summary()
                != &result.manifest.summary
        {
            return Err(resolution_expired());
        }
        Ok(())
    }
}
impl EpisodeResolutionResults for CatalogFiles {
    fn publish(
        &self,
        result: &StoredEpisodeResolution,
        metadata: &CaptureMetadata,
    ) -> Result<VerifiedJobOutput, ApiError> {
        self.verify_episode_evidence(result)?;
        if !matches!(metadata.capture_kind, CatalogCaptureKind::EpisodeResolution)
            || metadata.status.is_some()
            || metadata.requested_url != result.episode_url
        {
            return Err(resolution_expired());
        }
        let bytes = serde_json::to_vec(result).map_err(|_| resolution_expired())?;
        let reference = self.save(&bytes, metadata)?;
        EpisodeResolutionResults::read(self, &reference.artifact_id)?;
        Ok(VerifiedJobOutput {
            path: self
                .root
                .join(&reference.body_path)
                .to_str()
                .ok_or_else(resolution_expired)?
                .into(),
            sha256: reference.body_sha256.clone(),
            bytes: DecimalU64(bytes.len() as u64),
            receipt: json!({"kind":"episode_resolution","source_kind":"request_descriptor","source_descriptor_sha256":result.source_descriptor_sha256,"resolution_artifact":reference,"raw_artifacts":result.raw_artifacts,"manifest_verified":true,"manifest_validation_profile":result.manifest.validation_profile,"referenced_resources_verified":false,"media_verified":false,"media_decoded":false,"native_playback":false,"live_session_capability":false,"url_expiration_verified":false}),
        })
    }
    fn read(&self, artifact_id: &str) -> Result<StoredEpisodeResolution, ApiError> {
        let loaded = self.load_raw(artifact_id)?;
        if !matches!(
            loaded.capture.capture_kind,
            CatalogCaptureKind::EpisodeResolution
        ) || loaded.capture.status.is_some()
        {
            return Err(resolution_expired());
        }
        let version: serde_json::Value =
            serde_json::from_slice(&loaded.body).map_err(|_| resolution_expired())?;
        if version["schema_version"] != "2" {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Episode resolution schema is unsupported; read the original artifact with catalog.read_raw",
            ));
        }
        let stored: StoredEpisodeResolution =
            serde_json::from_slice(&loaded.body).map_err(|_| resolution_expired())?;
        self.verify_episode_evidence(&stored)?;
        Ok(stored)
    }
}
