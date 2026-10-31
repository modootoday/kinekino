//! Port behind the 작품 탐색 view: installed source plugins and their listings. The window
//! knows only `kinekino_core::source` types; the app wires the plugin host in.
use kinekino_core::source::{SourceError, SourceItem, SourcePage, SourceRef};

/// Listing rows the browse view shows per page (fixed slots, no remount).
pub const BROWSE_ROWS: usize = 12;

pub trait Catalog {
    /// Installed sources that can browse. Each item's reference is the provider root
    /// (`id` empty); browsing it lists what the provider shows first.
    fn sources(&self) -> Vec<SourceItem>;
    /// One page under `at` (a source root or any listed item).
    fn browse(&self, at: &SourceRef, page: u32) -> Result<SourcePage, SourceError>;
}
