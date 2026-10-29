//! Plugin wire protocol `document/1`: page fetch and parsing behind the plugin boundary.
//! The request line also carries `"protocol": "document/1"` ([`crate::PROTOCOL_FIELD`]).
use crate::document::{CatalogError, CatalogPage, EpisodeDocumentData, Provenance, RawDocument};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum DocRequest {
    Revision,
    Fetch {
        location: String,
    },
    Parse {
        raw: RawDocument,
    },
    ParseHtml {
        location: String,
        html: String,
        provenance: Provenance,
    },
    EpisodeDocument {
        html: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocResponse {
    Revision(String),
    Raw(RawDocument),
    Page(CatalogPage),
    Episode(EpisodeDocumentData),
    Error(CatalogError),
}
