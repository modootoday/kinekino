//! Bounded HLS structure and URI policy; no network or player dependency.
mod attributes;
mod parser;
pub use parser::verify;
use serde::{Deserialize, Serialize};
pub const HLS_VALIDATION_PROFILE: &str = "hls-bounded-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HlsPlaylistKind {
    Master,
    Media,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HlsSummary {
    pub kind: HlsPlaylistKind,
    pub variants: u32,
    pub segments: u32,
    pub resource_references: u32,
    pub encrypted: bool,
    pub end_list: bool,
}
#[derive(Debug)]
pub struct VerifiedHlsManifest {
    source_url: String,
    summary: HlsSummary,
    resources: Vec<String>,
}
impl VerifiedHlsManifest {
    pub fn source_url(&self) -> &str {
        &self.source_url
    }
    pub fn summary(&self) -> &HlsSummary {
        &self.summary
    }
    pub fn resources(&self) -> &[String] {
        &self.resources
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HlsError {
    Invalid,
    Limit,
    UnsafeTransport,
    Unsupported,
}
