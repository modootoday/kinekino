mod assets;
mod bundle;
pub use bundle::CardBundleFiles;
mod frame;
mod inspection;
mod render;
use crate::application::{
    api::ApiError,
    proposals::card_export::{
        CardImagePort, ExportCardReviewPng, PreparedReviewPng, SelectedAcceptedCard,
    },
};
pub struct PngCardReviewExport;
impl CardImagePort for PngCardReviewExport {
    fn render(
        &self,
        selection: SelectedAcceptedCard<'_>,
        input: &ExportCardReviewPng,
    ) -> Result<PreparedReviewPng, ApiError> {
        render::render(selection, input)
    }
    fn publish_new(&self, path: &str, bytes: &[u8]) -> Result<(), ApiError> {
        frame::validate_output(path)?;
        kinekino_core::file_publication::publish(path, bytes)
    }
}
