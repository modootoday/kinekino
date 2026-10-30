#[path = "ports.rs"]
mod ports;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        browser_provider::{BrowserDocument, CaptureKind},
    },
    domain::{api_values::DecimalU64, player_document::PlayerDocumentLocation},
};
use kinekino_core::source::document::{EpisodeDocumentData, PageLocation, ResourceKind};
pub use ports::EpisodeMarkupPort;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
mod inspect;
mod observed_media;
pub(crate) use inspect::inspect as inspect_resolution;
pub use inspect::{EpisodeResolutionInspection, InspectEpisodeResolution};
mod results;
pub use observed_media::{VerifiedObservedManifest, observed_manifest, observed_response_status};
pub(crate) use results::expired as resolution_expired;
pub const RESOLUTION_SCOPE: &str =
    "bounded_hls_manifest_and_private_evidence; decoding_and_playback_unverified";
pub use results::{EpisodeResolutionResults, ResolvedManifestRecord, StoredEpisodeResolution};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResolveEpisode {
    pub episode_url: String,
}
impl ResolveEpisode {
    pub fn location(&self) -> Result<PageLocation, ApiError> {
        let location = PageLocation::parse(&self.episode_url)
            .map_err(|_| invalid("Invalid episode location"))?;
        if location.kind() != ResourceKind::Episode {
            return Err(invalid("An episode URL is required"));
        }
        Ok(location)
    }
}

#[derive(Debug)]
pub struct EpisodePlayerTarget {
    pub episode_id: String,
    pub content_id: DecimalU64,
    pub source_body_sha256: String,
    pub source_document_generation: DecimalU64,
    pub source_pointer: &'static str,
    pub document: PlayerDocumentLocation,
}

pub fn player_target(
    input: &ResolveEpisode,
    document: &BrowserDocument,
    markup: &dyn EpisodeMarkupPort,
) -> Result<EpisodePlayerTarget, ApiError> {
    let requested = input.location()?;
    let observed = PageLocation::parse(&document.final_url)
        .map_err(|_| invalid("Invalid episode document identity"))?;
    if document.kind != CaptureKind::DomSnapshot
        || document.response.is_some()
        || document.document_generation.0 == 0
        || document.body.len() > crate::domain::browser_provider::MAX_CAPTURE_BYTES
        || requested.as_str() != observed.as_str()
    {
        return Err(invalid("Episode document does not match its request"));
    }
    let decoded = markup.read(&document.body)?;
    let page = requested
        .resolve(&decoded.page_url)
        .map_err(|_| invalid("Invalid embedded page identity"))?;
    let playing = requested
        .resolve(&format!("/e/{}", decoded.playing_slug))
        .map_err(|_| invalid("Invalid playing episode identity"))?;
    if page.as_str() != requested.as_str()
        || playing.as_str() != requested.as_str()
        || decoded.content_id == 0
    {
        return Err(invalid(
            "Embedded episode identity does not match its document",
        ));
    }
    if decoded.body_type != "video" || !decoded.iframe {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Episode body is not an observed iframe video target",
        ));
    }
    let target = PlayerDocumentLocation::parse(&decoded.target_url).map_err(|_| {
        ApiError::new(
            ErrorCode::NeedsInput,
            "Episode player document is outside the admitted provider scope",
        )
    })?;
    Ok(EpisodePlayerTarget {
        episode_id: requested.stable_id(),
        content_id: DecimalU64(decoded.content_id),
        source_body_sha256: format!("{:x}", Sha256::digest(&document.body)),
        source_document_generation: document.document_generation,
        source_pointer: "/props/playing/body/url",
        document: target,
    })
}
fn invalid(message: &'static str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(test)]
mod tests;
