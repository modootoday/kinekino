use crate::{
    application::{
        api::{ApiError, ErrorCode},
        clip_projects::{ClipProjectStore, ClipPublication, publication_sha256},
        review_source::MutationControl,
    },
    domain::{
        api_values::DecimalU64,
        clip_document::ClipSourcePin,
        identifiers::{valid_id, valid_sha256},
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedClipProjectPin {
    pub document_id: String,
    pub publication_revision: DecimalU64,
    pub publication_sha256: String,
}
/// Trusted adapter owns original media/ancestor guards and derives the full observation.
/// The lease must retain custody until dropped; caller JSON cannot supply a lease.
pub trait ClipSourceLease: Send + Sync {
    fn observation(&self) -> &ClipSourcePin;
}
/// Rust assembly supplies the locator catalog, tools and bounded probing policy.
/// It must not open a caller-provided URL or trust a declaration as an observation.
pub trait ClipSourceAdmission {
    type Lease: ClipSourceLease;
    fn open_source(
        &self,
        expected: &ClipSourcePin,
        control: &dyn MutationControl,
    ) -> Result<Self::Lease, ApiError>;
}
/// Not serializable or cloneable: media authority lasts only with the original leases.
/// Still requires encoder, subtitles/assets, output and Jobs admission before execution.
pub struct PreparedClipMedia<L: ClipSourceLease> {
    project: SavedClipProjectPin,
    publication: ClipPublication,
    sources: Vec<L>,
}
impl<L: ClipSourceLease> PreparedClipMedia<L> {
    pub fn project(&self) -> &SavedClipProjectPin {
        &self.project
    }
    pub fn publication(&self) -> &ClipPublication {
        &self.publication
    }
    pub fn sources(&self) -> &[L] {
        &self.sources
    }
}
pub fn prepare_media<P: ClipSourceAdmission>(
    store: &dyn ClipProjectStore,
    project: SavedClipProjectPin,
    admission: &P,
    control: &dyn MutationControl,
) -> Result<PreparedClipMedia<P::Lease>, ApiError> {
    control.checkpoint()?;
    if !valid_id(&project.document_id)
        || project.publication_revision.0 == 0
        || !valid_sha256(&project.publication_sha256)
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid saved clip project pin",
        ));
    }
    let stored = store.load(&project.document_id, Some(project.publication_revision))?;
    control.checkpoint()?;
    let publication = stored.publication;
    let parent_valid = if publication.publication_revision.0 == 1 {
        publication.parent_publication_sha256.is_none()
    } else {
        publication
            .parent_publication_sha256
            .as_deref()
            .is_some_and(valid_sha256)
    };
    if publication.schema_version != "1"
        || !parent_valid
        || publication.document.document_id != project.document_id
        || publication.publication_revision != project.publication_revision
        || stored.latest_publication_revision.0 < project.publication_revision.0
        || stored.publication_sha256 != project.publication_sha256
        || publication_sha256(&publication)? != project.publication_sha256
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Saved clip publication differs from the pinned revision",
        ));
    }
    publication
        .document
        .validate()
        .map_err(|_| ApiError::new(ErrorCode::NeedsInput, "Stored clip document is invalid"))?;
    let mut sources = Vec::with_capacity(publication.document.sources.len());
    for expected in &publication.document.sources {
        control.checkpoint()?;
        let lease = admission.open_source(expected, control)?;
        // Stream enumeration order is not a media fact. Identity and every track value are.
        let mut observed = lease.observation().clone();
        let mut declared = expected.clone();
        observed.tracks.sort_by_key(|track| track.stream_index);
        declared.tracks.sort_by_key(|track| track.stream_index);
        if observed != declared {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Observed clip source, clock or container tracks differ from the saved pin",
            ));
        }
        control.checkpoint()?;
        sources.push(lease);
    }
    control.checkpoint()?;
    Ok(PreparedClipMedia {
        project,
        publication,
        sources,
    })
}

#[cfg(test)]
mod tests;
