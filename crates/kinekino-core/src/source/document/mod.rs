//! Source-neutral document types shared by hosts and source adapters (ADR 0011 M1b).
//! Wire values live in `kinekino-plugin-sdk`; host-side location rules and ports stay here.
mod location;
mod ports;
pub use kinekino_plugin_sdk::document::{
    CatalogError, CatalogPage, DiagnosticEdge, DiagnosticHint, Entry, EpisodeDocumentData,
    ErrorKind, ErrorNode, Provenance, RawDocument, ReqwestFlags, TransportDiagnostic,
    TransportPhase,
};
pub use location::{Category, LocationRules, PageLocation, ResourceKind, install_rules};
pub use ports::{CatalogFetchPort, CatalogReadPort};
