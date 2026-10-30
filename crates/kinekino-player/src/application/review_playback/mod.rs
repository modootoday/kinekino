//! Local review input and runtime admission. This module never launches an engine.
pub mod api;
pub mod schema;
pub mod sessions;
pub mod window;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        model_review::ReviewStore,
        review_source::{RetainedReviewEvidence, ReviewCaseEvidence},
    },
    domain::review_source::{EvidenceReference, MediaReference, ReviewCase},
};
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewContentClass {
    IsoBmffHeader,
    MatroskaHeader,
}
pub trait ReviewContentClassifier {
    fn classify(&self, prefix: &[u8]) -> Result<ReviewContentClass, ApiError>;
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ReviewInputIdentity {
    pub case: ReviewCase,
    pub media: MediaReference,
    pub bytes: u64,
    pub declared_media_type: String,
    pub classification: ReviewContentClass,
}
/// A full-content-verified session plus independently classified raw prefix; not decode proof.
pub struct ReviewMediaInput {
    identity: ReviewInputIdentity,
    input: RetainedReviewEvidence,
}
impl ReviewMediaInput {
    pub fn identity(&self) -> &ReviewInputIdentity {
        &self.identity
    }
    pub fn check_current(&self) -> Result<(), ApiError> {
        self.input.check_current()
    }
}
pub fn inspect_review_input<S: ReviewStore>(
    reader: &ReviewCaseEvidence<S>,
    case: &ReviewCase,
    media: &MediaReference,
    classifier: &dyn ReviewContentClassifier,
) -> Result<ReviewMediaInput, ApiError> {
    media.validate().map_err(|_| invalid())?;
    let input = reader.retain(case)?;
    let reference = EvidenceReference {
        asset_id: media.asset_id.clone(),
        sha256: media.sha256.clone(),
    };
    let metadata = input.inspect(&reference)?;
    if !matches!(
        metadata.media_type.as_str(),
        "video/mp4" | "video/x-matroska"
    ) {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Local review input classification is not yet supported for this declared media type",
        ));
    }
    let classification = classifier.classify(&input.read_prefix(&reference, 65536)?)?;
    if !matches!(
        (metadata.media_type.as_str(), &classification),
        ("video/mp4", ReviewContentClass::IsoBmffHeader)
            | ("video/x-matroska", ReviewContentClass::MatroskaHeader)
    ) {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Declared media type and raw review header differ",
        ));
    }
    input.check_current()?;
    Ok(ReviewMediaInput {
        identity: ReviewInputIdentity {
            case: case.clone(),
            media: media.clone(),
            bytes: metadata.bytes.0,
            declared_media_type: metadata.media_type,
            classification,
        },
        input,
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReviewRuntimeBackend {
    OwnedLocalMpv,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReviewChildOwnership {
    OwnedJobUntilPhysicalExit,
}
/// Required fields, no Default/Deserialize. A descriptor is not a resource permit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewRuntimeProfile {
    pub backend: ReviewRuntimeBackend,
    pub engine_sha256: String,
    pub max_source_bytes: u64,
    /// Windows Job committed-memory limit; does not claim a measured RSS ceiling.
    pub max_job_committed_bytes: u64,
    pub cpu_threads: u32,
    pub startup_timeout_ms: u64,
    pub child_ownership: ReviewChildOwnership,
}
pub trait ReviewRuntimeLease {
    fn profile(&self) -> &ReviewRuntimeProfile;
    fn check_current(&self) -> Result<(), ApiError>;
}
impl<T: ReviewRuntimeLease + ?Sized> ReviewRuntimeLease for Box<T> {
    fn profile(&self) -> &ReviewRuntimeProfile {
        (**self).profile()
    }
    fn check_current(&self) -> Result<(), ApiError> {
        (**self).check_current()
    }
}
/// Concrete implementations must reserve resources and retain the tool/job ownership.
pub trait ReviewRuntimeAdmission {
    fn reserve(
        &self,
        input: &ReviewInputIdentity,
    ) -> Result<Box<dyn ReviewRuntimeLease + '_>, ApiError>;
}
pub struct UnavailableReviewRuntime;
impl ReviewRuntimeAdmission for UnavailableReviewRuntime {
    fn reserve(
        &self,
        _input: &ReviewInputIdentity,
    ) -> Result<Box<dyn ReviewRuntimeLease + '_>, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Owned local review playback runtime is not admitted",
        ))
    }
}
pub type AdmittedReviewInput<'a> = ReviewInputWithRuntime<Box<dyn ReviewRuntimeLease + 'a>>;
pub struct ReviewInputWithRuntime<L: ReviewRuntimeLease> {
    source: ReviewMediaInput,
    runtime: L,
    snapshot: ReviewRuntimeProfile,
}
impl<L: ReviewRuntimeLease> ReviewInputWithRuntime<L> {
    pub(crate) fn into_parts(self) -> (ReviewMediaInput, L) {
        (self.source, self.runtime)
    }
    pub fn identity(&self) -> &ReviewInputIdentity {
        self.source.identity()
    }
    pub fn check_current(&self) -> Result<(), ApiError> {
        self.source.check_current()?;
        self.runtime.check_current()?;
        if self.runtime.profile() != &self.snapshot {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Review runtime grant changed",
            ));
        }
        Ok(())
    }
}
pub fn reserve_review_runtime<'a>(
    source: ReviewMediaInput,
    runtime: &'a dyn ReviewRuntimeAdmission,
) -> Result<AdmittedReviewInput<'a>, ApiError> {
    source.check_current()?;
    let permit = runtime.reserve(source.identity())?;
    admit_review_runtime(source, permit)
}
/// Preserve the concrete resource owner for typed backend assembly.
pub fn admit_review_runtime<L: ReviewRuntimeLease>(
    source: ReviewMediaInput,
    permit: L,
) -> Result<ReviewInputWithRuntime<L>, ApiError> {
    source.check_current()?;
    permit.check_current()?;
    let profile = permit.profile().clone();
    if !crate::domain::identifiers::valid_sha256(&profile.engine_sha256)
        || profile.max_source_bytes == 0
        || profile.max_job_committed_bytes == 0
        || profile.cpu_threads == 0
        || profile.startup_timeout_ms == 0
    {
        return Err(invalid());
    }
    if source.identity.bytes > profile.max_source_bytes {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Review source exceeds reserved runtime budget",
        ));
    }
    let result = ReviewInputWithRuntime {
        source,
        runtime: permit,
        snapshot: profile,
    };
    result.check_current()?;
    Ok(result)
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid explicit review input or runtime grant",
    )
}
pub fn prepare_review_input<'a, S: ReviewStore>(
    reader: &ReviewCaseEvidence<S>,
    case: &ReviewCase,
    media: &MediaReference,
    classifier: &dyn ReviewContentClassifier,
    runtime: &'a dyn ReviewRuntimeAdmission,
) -> Result<AdmittedReviewInput<'a>, ApiError> {
    reserve_review_runtime(
        inspect_review_input(reader, case, media, classifier)?,
        runtime,
    )
}
#[cfg(test)]
mod tests;
