//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait EpisodeMarkupPort: Send + Sync {
    fn read(&self, html: &[u8]) -> Result<EpisodeDocumentData, ApiError>;
}
