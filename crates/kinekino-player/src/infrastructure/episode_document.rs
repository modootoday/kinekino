use crate::application::{
    api::{ApiError, ErrorCode},
    episode_resolve::EpisodeMarkupPort,
};
use kinekino_core::source::document::EpisodeDocumentData;

pub struct HtmlEpisodeMarkup;
impl EpisodeMarkupPort for HtmlEpisodeMarkup {
    fn read(&self, html: &[u8]) -> Result<EpisodeDocumentData, ApiError> {
        crate::infrastructure::catalog::document_plugin()
            .and_then(|plugin| plugin.episode_document(html))
            .map_err(|error| ApiError::new(ErrorCode::NeedsInput, error.message))
    }
}
