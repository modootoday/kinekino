//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Implemented by the plugin host adapter; every ability defaults to `Unsupported`.
pub trait SourceProvider: Send + Sync {
    fn manifest(&self) -> &SourceManifest;
    fn browse(&self, _category: &SourceRef, _page: u32) -> Result<SourcePage, SourceError> {
        Err(SourceError::Unsupported(SourceCapability::Browse))
    }
    fn search(&self, _query: &str, _page: u32) -> Result<SourcePage, SourceError> {
        Err(SourceError::Unsupported(SourceCapability::Search))
    }
    fn series(&self, _series: &SourceRef) -> Result<SourcePage, SourceError> {
        Err(SourceError::Unsupported(SourceCapability::Series))
    }
    fn episode(&self, _episode: &SourceRef) -> Result<EpisodeAdjacency, SourceError> {
        Err(SourceError::Unsupported(SourceCapability::Episode))
    }
    fn resolve(&self, _episode: &SourceRef) -> Result<Vec<StreamCandidate>, SourceError> {
        Err(SourceError::Unsupported(SourceCapability::Resolve))
    }
}
