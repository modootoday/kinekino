#[path = "ports.rs"]
mod ports;
pub use ports::BrowserProviderPort;
mod activation;
mod catalog_jobs;
mod document_owner;
mod document_session;
pub use document_owner::BrowserDocumentSourceOwner;
pub use document_session::BrowserDocumentSession;
mod episode_jobs;
mod evidence_archive;
mod frame_documents;
mod frame_runtime;
mod network_contract;
mod replay;
pub use activation::{
    EpisodeActivationObservation, EpisodeActivationStatus, MainNavigationDiagnostic,
    MediaRequestObservation, PlayerDocumentObservationKind, PlayerFrameObservation,
};
pub use episode_jobs::BrowserEpisodeResolveJobs;
pub use evidence_archive::BrowserEvidenceClock;
pub use frame_documents::{
    FrameDocumentEvent, FrameDocumentIdentity, FrameDocumentState, FrameNavigationDiagnostic,
};
pub use frame_runtime::{
    ControlVisibility, DocumentReadyState, DocumentVisibility, FrameRuntimeObservation,
    FrameRuntimeRequest, FrameRuntimeSnapshot, FrameRuntimeState, MAX_RUNTIME_ENVELOPE_BYTES,
    VideoRuntime,
};
pub use network_contract::{OwnedMediaStream, OwnedResponseCapture};
#[cfg(all(test, feature = "source-adapter-test-support"))]
pub(crate) use replay::fixture::fixture_work_for_request;
#[cfg(feature = "source-adapter-test-support")]
pub use replay::fixture::{fixture_grant_for_request, fixture_stream_for_request};
pub use replay::{MediaReplayPolicy, OwnedMediaReplay};
mod state;
pub use catalog_jobs::BrowserCatalogJobs;
#[cfg(test)]
mod activation_tests;
#[cfg(test)]
mod media_stream_tests;
#[cfg(test)]
mod network_tests;
#[cfg(test)]
mod runtime_tests;
#[cfg(test)]
mod tests;
pub use crate::domain::browser_provider::*;
use crate::{
    application::api::ApiError,
    domain::{api_values::DecimalU64, media::MediaSource},
};
use serde::{Deserialize, Serialize};
pub use state::{BrowserEvent, BrowserWork, EventDisposition};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserNavigationObservation {
    pub url: String,
    pub document_generation: DecimalU64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureKind {
    NetworkResponse,
    DomSnapshot,
}
#[derive(Debug, Clone)]
pub struct NetworkResponseMetadata {
    pub status: u16,
    pub content_type: String,
    pub content_encoding: Option<String>,
}
/// Internal owned evidence, never serialized inline by the RPC dispatcher.
#[derive(Clone)]
pub struct BrowserDocument {
    pub kind: CaptureKind,
    pub final_url: String,
    pub document_generation: DecimalU64,
    pub observed_unix_ms: DecimalU64,
    pub body: Arc<[u8]>,
    pub response: Option<NetworkResponseMetadata>,
}
#[derive(Clone)]
pub struct BrowserMedia {
    pub document_url: String,
    pub document_generation: DecimalU64,
    pub observed_unix_ms: DecimalU64,
    pub source: MediaSource,
    pub manifest_prefix: Option<Arc<[u8]>>,
}
#[derive(Clone)]
pub enum BrowserResult {
    Document(BrowserDocument),
    Media(BrowserMedia),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserWorkState {
    Queued,
    Running,
    Cancelling,
    Succeeded,
    Failed,
    Cancelled,
}
impl BrowserWorkState {
    pub fn terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }
}
#[derive(Clone)]
pub struct BrowserSnapshot {
    pub ticket: BrowserTicket,
    pub state: BrowserWorkState,
    pub current_url: Option<String>,
    pub document_generation: DecimalU64,
    pub navigation_pending: bool,
    pub navigation_history: Vec<BrowserNavigationObservation>,
    pub native_navigation_id: Option<DecimalU64>,
    pub navigation_diagnostic: Option<MainNavigationDiagnostic>,
    pub activation: Option<EpisodeActivationObservation>,
    pub activation_history: Vec<EpisodeActivationObservation>,
    pub player_frames: Vec<PlayerFrameObservation>,
    pub media_requests: Vec<MediaRequestObservation>,
    pub frame_documents: Vec<FrameDocumentState>,
    pub network: Option<crate::application::browser_network::NetworkEvidence>,
    pub engine_version: Option<String>,
    pub blocking_requested: bool,
    pub blocking_installed: bool,
    pub requested_count: DecimalU64,
    pub blocked_count: DecimalU64,
    pub result: Option<Arc<BrowserResult>>,
    pub document: Option<Arc<BrowserDocument>>,
    pub failure: Option<Arc<ApiError>>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelDisposition {
    Requested,
    AlreadyRequested,
    AlreadyTerminal,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserHostState {
    Starting,
    Ready,
    Stopping,
    Stopped,
    Failed,
}
#[derive(Clone)]
pub struct BrowserHostSnapshot {
    pub state: BrowserHostState,
    pub worker_exited: bool,
    pub failure: Option<Arc<ApiError>>,
}
pub trait BrowserProviderOwner: Send + Sync {
    fn request_shutdown(&self) -> Result<(), ApiError>;
    fn shutdown_snapshot(&self) -> Result<BrowserHostSnapshot, ApiError>;
}
