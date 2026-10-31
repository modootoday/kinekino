//! Media source port (ADR 0011): catalog, search, episodes and stream resolution
//! supplied by an out-of-process source plugin. The core stays source-neutral:
//! no site host, path or markup lives here.
pub mod document;
mod ports;
#[cfg(test)]
mod tests;

pub use ports::SourceProvider;

/// Source plugin API family and major version this core speaks.
pub use kinekino_plugin_sdk::SOURCE_API;

/// Opaque reference to an item owned by one provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceRef {
    /// Plugin id from the manifest, e.g. `com.example.source`.
    pub provider: String,
    /// Provider-defined id; the core never parses it.
    pub id: String,
}

/// Optional abilities a provider declares in its manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceCapability {
    Browse,
    Search,
    Series,
    Episode,
    Resolve,
}

/// Static description read from `kinekino-plugin.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceManifest {
    pub id: String,
    pub version: String,
    pub api: String,
    pub capabilities: Vec<SourceCapability>,
    /// Hosts the player may fetch media from for this provider.
    pub media_hosts: Vec<String>,
}

impl SourceManifest {
    /// Whether this manifest targets the API the core speaks.
    pub fn is_compatible(&self) -> bool {
        self.api == SOURCE_API
    }
    pub fn supports(&self, capability: SourceCapability) -> bool {
        self.capabilities.contains(&capability)
    }
    /// Exact, case-insensitive host match against the declared media hosts.
    pub fn allows_media_host(&self, host: &str) -> bool {
        self.media_hosts
            .iter()
            .any(|h| h.eq_ignore_ascii_case(host))
    }
}

/// One listing entry (category, search hit, series or episode).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceItem {
    pub reference: SourceRef,
    pub title: String,
    pub kind: SourceItemKind,
}

/// What selecting a listed item opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SourceItemKind {
    /// Browse it again (provider roots and plugins that do not tag items).
    #[default]
    Container,
    /// `resolve` turns it into streams; a listing's playables form the play queue.
    Playable,
}

/// One page of items with an optional continuation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourcePage {
    pub items: Vec<SourceItem>,
    pub next_page: Option<u32>,
}

/// Neighbours of an episode inside its series.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EpisodeAdjacency {
    pub previous: Option<SourceRef>,
    pub next: Option<SourceRef>,
}

/// A playable stream candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamCandidate {
    pub url: String,
    pub headers: Vec<(String, String)>,
}

/// Port error kinds; providers map transport and parse failures here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceError {
    Unsupported(SourceCapability),
    NotFound,
    Unavailable(String),
    Invalid(String),
}
