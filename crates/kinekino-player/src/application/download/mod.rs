//! Source preparation only; provider issuance and CLI registration remain separate.
mod job_executor;
mod owned_session;
mod ports;
mod preparation;
mod request;
mod transfer;

pub(crate) use job_executor::DownloadJobExecutor;
pub use ports::*;
pub use preparation::{
    DownloadPreparation, PreparedDownload, PreparedDownloadResources, SourceCustody, SourceOpener,
};
pub use request::{
    DOWNLOAD_OPERATION, DownloadRequest, DownloadSourceSelection, MAX_DOWNLOAD_BYTES,
    MAX_DOWNLOAD_REQUEST_BYTES,
};
pub use transfer::transfer;

#[cfg(test)]
mod tests;
