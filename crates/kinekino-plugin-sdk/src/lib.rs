//! Public contract for kinekino source plugins (ADR 0011 decision 5): protocol constants,
//! `source/1` and `document/1` wire types, the document values they carry, and the
//! `plugin.json` schema.
//! Depends on `serde` and `serde_json` only; never on kinekino internals.
pub mod document;
#[cfg(feature = "fixture")]
pub mod fixture;
#[cfg(feature = "fixture")]
pub mod golden;
pub mod manifest;
pub mod protocol;
pub mod serve;
pub mod wire;

pub use manifest::{MANIFEST_FILE, Manifest};
pub use protocol::{DOCUMENT_PROTOCOL, PROTOCOL_FIELD, SOURCE_API};
pub use serve::{SourcePlugin, handle_line, serve, unsupported};
