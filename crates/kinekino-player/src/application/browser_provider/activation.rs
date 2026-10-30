use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeActivationStatus {
    Enqueued,
    InvocationObserved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeActivationObservation {
    pub document_generation: DecimalU64,
    pub native_navigation_id: DecimalU64,
    pub status: EpisodeActivationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MainNavigationDiagnostic {
    pub observed_unix_ms: DecimalU64,
    pub document_generation: DecimalU64,
    pub native_navigation_id: DecimalU64,
    pub http_status_code: Option<i32>,
    pub is_success: bool,
    pub web_error_status: i32,
}

/// Requested resource only; neither a response nor a verified media result.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaRequestObservation {
    pub document_url: String,
    pub document_generation: DecimalU64,
    pub native_navigation_id: DecimalU64,
    pub url: String,
    pub referer: Option<String>,
    pub request_method: String,
    pub observed_unix_ms: DecimalU64,
}

/// A native frame navigation candidate, never an admitted media source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerDocumentObservationKind {
    FrameNavigation,
    PopupIntent,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerFrameObservation {
    pub kind: PlayerDocumentObservationKind,
    pub owner_document_generation_at_observation: DecimalU64,
    pub owner_navigation_id_at_observation: DecimalU64,
    pub frame_navigation_id: Option<DecimalU64>,
    pub frame_id: Option<DecimalU64>,
    pub parent_frame_id: Option<DecimalU64>,
    pub request_source_kind: Option<String>,
    pub lineage_verified: bool,
    pub url: String,
    pub observed_unix_ms: DecimalU64,
    pub request_method: Option<String>,
    pub referer: Option<String>,
}
