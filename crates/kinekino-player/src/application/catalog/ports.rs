//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait CatalogApi: Send + Sync {
    fn episode_resolution_read_available(&self) -> bool {
        false
    }
    fn inspect_episode_resolution(
        &self,
        _: crate::application::episode_resolve::InspectEpisodeResolution,
    ) -> Result<crate::application::episode_resolve::EpisodeResolutionInspection, ApiError> {
        Err(unavailable())
    }
    fn available(&self) -> bool {
        false
    }
    fn live_available(&self) -> bool {
        self.available()
    }
    fn result_read_available(&self) -> bool {
        false
    }
    fn raw_read_available(&self) -> bool {
        false
    }
    fn read_raw(&self, _: ReadCatalogRaw) -> Result<CatalogRawChunk, ApiError> {
        Err(unavailable())
    }
    fn read_result(&self, _: &str, _: ReadCatalogResult) -> Result<CatalogResult, ApiError> {
        Err(unavailable())
    }
    fn search(&self, _: SearchCatalog) -> Result<CatalogResult, ApiError> {
        Err(unavailable())
    }
    fn episodes(&self, _: ListEpisodes) -> Result<CatalogResult, ApiError> {
        Err(unavailable())
    }
}
