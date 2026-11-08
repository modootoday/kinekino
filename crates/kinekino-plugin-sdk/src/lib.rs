//! Public contract for kinekino source plugins (ADR 0011 decision 5): protocol constants,
//! `source/1` and `document/1` wire types, the document values they carry, and the
//! `plugin.json` schema.
//! Depends on `serde` and `serde_json` only; never on kinekino internals.
//!
//! # Writing a plugin
//! Implement [`SourcePlugin`] for the capabilities the manifest declares (the rest answer
//! `unsupported`) and call [`serve_loop`] from `main`. The host keeps one process per
//! plugin and writes one request line at a time, waiting for its answer before the next;
//! the loop ends cleanly on stdin EOF. [`serve`] answers a single line and exits: the host
//! then starts the process again on the next call, so it still works but is not
//! recommended. Mark each listed [`Item`](wire::source::Item) `container` (list it again)
//! or `playable` (resolve it to streams); how a site groups its works stays in the plugin.
//!
//! ```
//! use kinekino_plugin_sdk::wire::source::{Item, ItemKind, Ref, Response};
//! use kinekino_plugin_sdk::{SourcePlugin, handle_line};
//!
//! struct Demo;
//! impl SourcePlugin for Demo {
//!     fn browse(&mut self, category: &Ref, _page: u32) -> Response {
//!         let reference = Ref { provider: category.provider.clone(), id: "a".into() };
//!         let item = Item { reference, title: "A".into(), kind: Some(ItemKind::Playable) };
//!         Response::Page { items: vec![item], next_page: None }
//!     }
//! }
//! // `fn main() -> std::io::Result<()> { kinekino_plugin_sdk::serve_loop(&mut Demo) }`
//! let line = r#"{"method":"browse","category":{"provider":"p","id":""},"page":1}"#;
//! let answer: serde_json::Value = serde_json::from_str(&handle_line(&mut Demo, line)).unwrap();
//! assert_eq!(answer["page"]["items"][0]["kind"], "playable");
//! let other = r#"{"method":"resolve","episode":{"provider":"p","id":"a"}}"#;
//! assert!(handle_line(&mut Demo, other).contains("unsupported"));
//! ```
//!
//! # Versions
//! Crate semver and protocol majors ([`SOURCE_API`], [`DOCUMENT_PROTOCOL`]) move
//! separately; new wire fields are optional. The crate is not on crates.io
//! (`publish = false`) and no release tag exists yet, so depend on it by path or git rev.
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
pub use serve::{SourcePlugin, handle_line, serve, serve_loop, serve_loop_io, unsupported};
