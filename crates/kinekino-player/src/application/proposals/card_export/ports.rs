//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait CardImagePort: Send + Sync {
    fn render(
        &self,
        selection: SelectedAcceptedCard<'_>,
        input: &ExportCardReviewPng,
    ) -> Result<PreparedReviewPng, ApiError>;
    fn publish_new(&self, path: &str, bytes: &[u8]) -> Result<(), ApiError>;
}
