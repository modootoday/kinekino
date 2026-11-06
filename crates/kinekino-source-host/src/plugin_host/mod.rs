//! Out-of-process media source plugins (ADR 0011 M3): manifest discovery and a
//! long-running `SourceProvider` worker over JSON lines (ADR 20261106051332).
mod document;
pub mod document_wire;
mod manifest;
mod process;
mod stdio;
#[cfg(test)]
mod tests;
pub mod wire;
mod worker;

pub use document::{DocumentPlugin, DocumentRules, install_fixture_rules};
pub use manifest::{
    Categories, DocumentSpec, MANIFEST_FILE, PluginManifest, default_dir, discover,
};
pub use process::ProcessSource;
