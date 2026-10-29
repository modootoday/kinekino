//! Out-of-process media source plugins (ADR 0011 M3): manifest discovery and a
//! `SourceProvider` that talks to the plugin executable over one JSON line per call.
mod document;
pub mod document_wire;
mod manifest;
mod process;
#[cfg(test)]
mod tests;
pub mod wire;

pub use document::{DocumentPlugin, DocumentRules, install_fixture_rules};
pub use manifest::{
    Categories, DocumentSpec, MANIFEST_FILE, PluginManifest, default_dir, discover,
};
pub use process::ProcessSource;
