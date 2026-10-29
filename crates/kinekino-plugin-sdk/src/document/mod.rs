//! `document/1` values: catalog pages, raw documents, provenance and typed errors.
mod episode_document;
mod transport_diagnostic;
pub use episode_document::EpisodeDocumentData;
pub use transport_diagnostic::{
    DiagnosticEdge, DiagnosticHint, ErrorNode, ReqwestFlags, TransportDiagnostic, TransportPhase,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    InvalidLocation,
    Transport,
    Timeout,
    RedirectLimit,
    HttpStatus,
    BodyTooLarge,
    InvalidEncoding,
    UnsupportedContent,
    AccessRequired,
    UnrecognizedPage,
    InvalidDocument,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogError {
    pub kind: ErrorKind,
    pub message: String,
    pub provenance: Option<Box<Provenance>>,
    pub transport: Option<Box<TransportDiagnostic>>,
}
impl CatalogError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            provenance: None,
            transport: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub requested_url: String,
    pub final_url: String,
    pub redirects: Vec<String>,
    pub sha256: String,
    pub bytes: usize,
    pub observed_unix_ms: u64,
    pub parser_revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogPage {
    pub title: String,
    pub series: Option<Entry>,
    pub entries: Vec<Entry>,
    pub next_page: Option<String>,
    pub provenance: Provenance,
}

/// Exact bounded response bytes, persisted separately from RPC metadata by the caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawDocument {
    pub status: u16,
    pub content_type: String,
    pub content_encoding: Option<String>,
    pub body: Vec<u8>,
    pub provenance: Provenance,
}
