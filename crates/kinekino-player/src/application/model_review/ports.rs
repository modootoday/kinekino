//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait ReviewStore: Send + Sync {
    fn available(&self) -> bool {
        true
    }
    /// Session retains verified package/assets and rooted-directory guards through publication.
    fn open(&self, package_sha256: &str) -> Result<Box<dyn ReviewSession>, ApiError>;
}
