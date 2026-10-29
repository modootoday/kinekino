//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// HTTP error responses remain available as raw evidence before parsing.
pub trait CatalogFetchPort: Send + Sync {
    fn fetch(&self, location: &PageLocation) -> Result<RawDocument, CatalogError>;
}

/// Implementations own transport and parsing. No GUI or browser automation dependency.
pub trait CatalogReadPort: Send + Sync {
    fn read(&self, location: &PageLocation) -> Result<CatalogPage, CatalogError>;
}
