//! Reuses the review store's verified, retained package and evidence custody.
use super::{CaseLease, CaseResolver, conflict, invalid};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        model_review::{ReviewSession, ReviewStore},
    },
    domain::review_source::ReviewCase,
};

pub struct ReviewCases<S> {
    store: S,
}
impl<S: ReviewStore> ReviewCases<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}
pub(super) struct RetainedCase {
    case: ReviewCase,
    pub(super) session: Box<dyn ReviewSession>,
}
impl CaseLease for RetainedCase {
    fn case(&self) -> &ReviewCase {
        &self.case
    }
    fn check_current(&self) -> Result<(), ApiError> {
        // ReviewSession owns immutable package/evidence guards until this lease drops.
        let package = self.session.package();
        package
            .validate()
            .map_err(|_| invalid("Invalid retained review package"))?;
        let item = package
            .items
            .iter()
            .find(|item| item.item_id == self.case.item_id)
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Review item not found"))?;
        if item.source.sha256 != self.case.source_request_sha256 {
            return Err(conflict(
                "Review source request differs from the selected item",
            ));
        }
        Ok(())
    }
}
impl<S: ReviewStore> CaseResolver for ReviewCases<S> {
    fn retain(&self, case: &ReviewCase) -> Result<Box<dyn CaseLease + '_>, ApiError> {
        Ok(Box::new(retain_case(&self.store, case)?))
    }
}
pub(super) fn retain_case(
    store: &impl ReviewStore,
    case: &ReviewCase,
) -> Result<RetainedCase, ApiError> {
    case.validate()
        .map_err(|_| invalid("Invalid review case identity"))?;
    let session = store.open(&case.package_sha256)?;
    let lease = RetainedCase {
        case: case.clone(),
        session,
    };
    lease.check_current()?;
    Ok(lease)
}
