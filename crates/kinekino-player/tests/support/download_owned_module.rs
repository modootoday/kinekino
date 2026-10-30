//! Exact checkpoint source mounts. HTTP/browser source remains a separate closure.
#[path = "../../src/application/download/owned_session.rs"]
pub(crate) mod owned_session;
#[path = "../../src/application/download/ports.rs"]
mod ports;
#[path = "../../src/application/download/preparation.rs"]
mod preparation;
pub use preparation::{
    DownloadPreparation, PreparedDownload, PreparedDownloadResources, SourceCustody, SourceOpener,
};
#[path = "../../src/application/download/transfer.rs"]
mod transfer;
pub use ports::*;
pub use transfer::transfer;
