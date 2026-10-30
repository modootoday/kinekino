//! Reuses the selected case's original ReviewSession through all evidence reads.
use super::{
    CaseLease, conflict, invalid,
    review_cases::{RetainedCase, retain_case},
};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        model_review::ReviewStore,
    },
    domain::review_source::{EvidenceReference, ReviewCase, valid_review_object_id},
};

pub struct ReviewCaseEvidence<S> {
    store: S,
}
impl<S: ReviewStore> ReviewCaseEvidence<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
    pub fn retain(&self, case: &ReviewCase) -> Result<RetainedReviewEvidence, ApiError> {
        Ok(RetainedReviewEvidence {
            case: retain_case(&self.store, case)?,
        })
    }
}
/// This is custody of input evidence, not a Verified timeline authority token.
pub struct RetainedReviewEvidence {
    case: RetainedCase,
}
impl RetainedReviewEvidence {
    pub(crate) fn package_snapshot(&self) -> crate::domain::model_review::ReviewPackage {
        self.case.session.package().clone()
    }
    pub fn case(&self) -> &ReviewCase {
        self.case.case()
    }
    pub fn check_current(&self) -> Result<(), ApiError> {
        self.case.check_current()
    }
    pub fn read(&self, reference: &EvidenceReference, max_bytes: u64) -> Result<Vec<u8>, ApiError> {
        self.inspect(reference)?;
        let bytes = self.case.session.read_evidence_asset(
            &reference.asset_id,
            &reference.sha256,
            max_bytes,
        )?;
        self.check_current()?;
        Ok(bytes)
    }
    /// Metadata remains backed by this handle's original package/file guards.
    pub fn inspect(
        &self,
        reference: &EvidenceReference,
    ) -> Result<crate::domain::model_review::EvidenceAsset, ApiError> {
        self.check_current()?;
        if !valid_review_object_id(&reference.asset_id)
            || !crate::domain::identifiers::valid_sha256(&reference.sha256)
        {
            return Err(invalid("Invalid case evidence reference"));
        }
        let item = self
            .case
            .session
            .package()
            .items
            .iter()
            .find(|item| item.item_id == self.case.case().item_id)
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Review item is unavailable"))?;
        let asset = item
            .assets()
            .into_iter()
            .find(|asset| asset.asset_id == reference.asset_id)
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::NotFound,
                    "Evidence is not referenced by this review item",
                )
            })?;
        if asset.sha256 != reference.sha256 {
            return Err(conflict("Review item evidence content identity differs"));
        }
        Ok(asset.clone())
    }
    pub fn read_prefix(
        &self,
        reference: &EvidenceReference,
        max_prefix: u64,
    ) -> Result<Vec<u8>, ApiError> {
        self.inspect(reference)?;
        let bytes = self.case.session.read_evidence_prefix(
            &reference.asset_id,
            &reference.sha256,
            max_prefix,
        )?;
        self.check_current()?;
        Ok(bytes)
    }
}
