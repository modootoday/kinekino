//! Port behind the 작품 탐색 view: installed source plugins and their listings. The window
//! knows only `kinekino_core::source` types; the app wires the plugin host in.
use kinekino_core::source::{
    SourceCapability, SourceError, SourceItem, SourcePage, SourceRef, StreamCandidate,
};

/// Row slots of the browse list (fixed, no remount). Pages append into them and the
/// list scrolls; past this many rows "더 보기" hides (ADR 20261031103033).
pub const BROWSE_ROWS: usize = 60;

/// Called from worker threads (see `jobs`), so implementations are shared across them.
pub trait Catalog: Send + Sync {
    /// Installed sources that can browse. Each item's reference is the provider root
    /// (`id` empty); browsing it lists what the provider shows first.
    fn sources(&self) -> Vec<SourceItem>;
    /// One page under `at` (a source root or any listed item).
    fn browse(&self, at: &SourceRef, page: u32) -> Result<SourcePage, SourceError>;
    /// One page of `provider`'s answers to `query` (its own interpretation).
    fn search(&self, _provider: &str, _query: &str, _page: u32) -> Result<SourcePage, SourceError> {
        Err(SourceError::Unsupported(SourceCapability::Search))
    }
    /// Stream candidates for a playable item, already checked against the provider's
    /// declared media hosts.
    fn resolve(&self, _playable: &SourceRef) -> Result<Vec<StreamCandidate>, SourceError> {
        Err(SourceError::Unsupported(SourceCapability::Resolve))
    }
}

/// Request headers a plugin may set on a stream; anything else is dropped so a plugin
/// cannot steer mpv options or leak cookies across hosts.
pub const STREAM_HEADERS: [&str; 3] = ["Referer", "User-Agent", "Origin"];
