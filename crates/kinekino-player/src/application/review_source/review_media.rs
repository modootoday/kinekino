//! Media already held by a verified review package. No path lookup or decoder proof.
use super::review_cases::{RetainedCase, retain_case};
use super::{CaseLease, MediaLease, MediaResolver, conflict, invalid};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        model_review::ReviewStore,
    },
    domain::review_source::{MediaReference, ReviewCase},
};

/// One existing custody adapter; other registered media stores remain separate adapters.
pub struct ReviewPackageMedia<S> {
    store: S,
}
impl<S: ReviewStore> ReviewPackageMedia<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}
struct RetainedMedia {
    case: RetainedCase,
    media: MediaReference,
}
impl MediaLease for RetainedMedia {
    fn identity(&self) -> &MediaReference {
        &self.media
    }
    fn verified_duration_us(&self) -> Option<u64> {
        None
    }
    fn check_current(&self) -> Result<(), ApiError> {
        self.case.check_current()?;
        let package = self.case.session.package();
        let asset = package
            .assets()
            .into_iter()
            .find(|asset| asset.asset_id == self.media.asset_id)
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::NotFound,
                    "Media asset is not in this package custody",
                )
            })?;
        if asset.sha256 != self.media.sha256 {
            return Err(conflict("Media asset content identity differs"));
        }
        // Declared format admission only. Content hashes/bytes are verified by ReviewStore;
        // decode, duration and timeline mapping are deliberately not inferred from MIME.
        if !matches!(
            asset.media_type.as_str(),
            "video/mp4"
                | "video/webm"
                | "video/x-matroska"
                | "video/quicktime"
                | "audio/wav"
                | "audio/x-wav"
                | "audio/mpeg"
                | "audio/flac"
                | "audio/ogg"
                | "audio/mp4"
                | "audio/webm"
        ) {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Asset is not an admitted declared media format",
            ));
        }
        Ok(())
    }
}
impl<S: ReviewStore> MediaResolver for ReviewPackageMedia<S> {
    fn retain(
        &self,
        case: &ReviewCase,
        media: &MediaReference,
    ) -> Result<Box<dyn MediaLease + '_>, ApiError> {
        media
            .validate()
            .map_err(|_| invalid("Invalid media custody identity"))?;
        let lease = RetainedMedia {
            case: retain_case(&self.store, case)?,
            media: media.clone(),
        };
        lease.check_current()?;
        Ok(Box::new(lease))
    }
}
