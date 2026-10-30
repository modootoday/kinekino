//! Bounded read-only observations of a current native child document.
use super::{BrowserSnapshot, BrowserWorkState, EventDisposition, FrameDocumentIdentity};
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::api_values::DecimalU64,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

pub const MAX_RUNTIME_ENVELOPE_BYTES: usize = 96 * 1024;
pub const MAX_RUNTIME_SAMPLES: u8 = 8;
const SAMPLE_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentReadyState {
    Loading,
    Interactive,
    Complete,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentVisibility {
    Hidden,
    Visible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlVisibility {
    pub present: bool,
    pub displayed: bool,
    pub visible: bool,
    pub has_layout: bool,
    pub center_in_viewport: bool,
    pub center_hit: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VideoRuntime {
    pub ready_state: u8,
    pub network_state: u8,
    pub paused: bool,
    pub ended: bool,
    pub muted: bool,
    pub current_time_seconds: Option<f64>,
    pub duration_seconds: Option<f64>,
    pub error_code: Option<u8>,
    pub current_src: String,
    pub video_width: u32,
    pub video_height: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameRuntimeSnapshot {
    pub document_url: String,
    pub document_ready_state: DocumentReadyState,
    pub document_visibility: DocumentVisibility,
    pub fire_player_defined: bool,
    pub fireload_defined: bool,
    pub jwplayer_defined: bool,
    /// Site-owned flag only; never promoted to player readiness.
    pub player_loaded_flag: Option<bool>,
    pub play_overlay: ControlVisibility,
    pub player_root: ControlVisibility,
    pub popup_overlay: ControlVisibility,
    pub video_count: u32,
    pub videos: Vec<VideoRuntime>,
}

#[derive(Clone)]
pub struct FrameRuntimeRequest {
    identity: FrameDocumentIdentity,
    sample: u8,
}
impl FrameRuntimeRequest {
    pub fn identity(&self) -> &FrameDocumentIdentity {
        &self.identity
    }
    pub fn sample(&self) -> u8 {
        self.sample
    }
}

#[derive(Clone, Serialize)]
pub struct FrameRuntimeObservation {
    pub sample: u8,
    pub observed_unix_ms: DecimalU64,
    pub snapshot: FrameRuntimeSnapshot,
}

#[derive(Clone, Default)]
pub struct FrameRuntimeState {
    observations: Vec<FrameRuntimeObservation>,
    pending: Option<FrameRuntimeRequest>,
    attempts: u8,
    next_sample_at: Option<Instant>,
}
impl FrameRuntimeState {
    pub fn reserve(
        &mut self,
        identity: &FrameDocumentIdentity,
        now: Instant,
    ) -> Option<FrameRuntimeRequest> {
        if self.pending.is_some()
            || self.attempts >= MAX_RUNTIME_SAMPLES
            || self.next_sample_at.is_some_and(|next| now < next)
        {
            return None;
        }
        self.attempts += 1;
        let request = FrameRuntimeRequest {
            identity: identity.clone(),
            sample: self.attempts,
        };
        self.pending = Some(request.clone());
        Some(request)
    }
    pub fn observations(&self) -> &[FrameRuntimeObservation] {
        &self.observations
    }
    pub fn finished(&self) -> bool {
        self.attempts == MAX_RUNTIME_SAMPLES && self.pending.is_none()
    }
    pub fn pending(&self) -> bool {
        self.pending.is_some()
    }
    pub fn accept(
        &mut self,
        request: &FrameRuntimeRequest,
        json: &str,
        observed_unix_ms: DecimalU64,
        now: Instant,
    ) -> Result<bool, ApiError> {
        // Stale/duplicate payloads are discarded before parsing untrusted page data.
        if self.pending.as_ref().is_none_or(|pending| {
            pending.sample != request.sample || pending.identity != request.identity
        }) {
            return Ok(false);
        }
        let snapshot = decode(json)?;
        if snapshot.document_url != request.identity.url {
            return Err(invalid("Child runtime observation changed its document"));
        }
        self.observations.push(FrameRuntimeObservation {
            sample: request.sample,
            observed_unix_ms,
            snapshot,
        });
        self.pending = None;
        self.next_sample_at = Some(now + SAMPLE_INTERVAL);
        Ok(true)
    }
}

pub(super) fn reserve_current(
    snapshot: &mut BrowserSnapshot,
    asset_observed: bool,
    now: Instant,
) -> Option<FrameRuntimeRequest> {
    if !asset_observed || snapshot.state != BrowserWorkState::Running || snapshot.navigation_pending
    {
        return None;
    }
    for frame in &mut snapshot.frame_documents {
        if frame.destroyed
            || frame.superseded
            || frame.completion.is_none()
            || frame.document.is_none()
            || frame.identity.owner_document_generation != snapshot.document_generation
            || Some(frame.identity.owner_native_navigation_id) != snapshot.native_navigation_id
            || crate::domain::player_document::PlayerDocumentLocation::parse(&frame.identity.url)
                .is_err()
        {
            continue;
        }
        if let Some(request) = frame.runtime.reserve(&frame.identity, now) {
            return Some(request);
        }
    }
    None
}

pub(super) fn apply_current(
    snapshot: &mut BrowserSnapshot,
    request: &FrameRuntimeRequest,
    json: &str,
    observed_unix_ms: DecimalU64,
    now: Instant,
) -> Result<EventDisposition, ApiError> {
    if snapshot.state != BrowserWorkState::Running
        || snapshot.navigation_pending
        || snapshot.document_generation != request.identity.owner_document_generation
        || snapshot.native_navigation_id != Some(request.identity.owner_native_navigation_id)
    {
        return Ok(EventDisposition::Stale);
    }
    let Some(frame) = snapshot
        .frame_documents
        .iter_mut()
        .find(|frame| frame.identity == request.identity && !frame.destroyed && !frame.superseded)
    else {
        return Ok(EventDisposition::Stale);
    };
    if frame.runtime.accept(request, json, observed_unix_ms, now)? {
        Ok(EventDisposition::Applied)
    } else {
        Ok(EventDisposition::Stale)
    }
}

pub(super) fn current_finished(snapshot: &BrowserSnapshot, expected_url: &str) -> bool {
    snapshot.state == BrowserWorkState::Running
        && !snapshot.navigation_pending
        && snapshot.frame_documents.iter().any(|frame| {
            !frame.destroyed
                && !frame.superseded
                && frame.identity.owner_document_generation == snapshot.document_generation
                && Some(frame.identity.owner_native_navigation_id) == snapshot.native_navigation_id
                && frame.identity.url == expected_url
                && frame.runtime.finished()
        })
}

fn decode(json: &str) -> Result<FrameRuntimeSnapshot, ApiError> {
    if json.len() > MAX_RUNTIME_ENVELOPE_BYTES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Child runtime envelope exceeds limit",
        ));
    }
    let snapshot: FrameRuntimeSnapshot =
        serde_json::from_str(json).map_err(|_| invalid("Invalid child runtime observation"))?;
    if !bounded_url(&snapshot.document_url)
        || snapshot.video_count > 64
        || snapshot.videos.len() != (snapshot.video_count as usize).min(8)
        || snapshot.videos.iter().any(|video| {
            video.ready_state > 4
                || video.network_state > 3
                || video
                    .error_code
                    .is_some_and(|code| !(1..=4).contains(&code))
                || !bounded_url(&video.current_src)
                || video
                    .current_time_seconds
                    .is_some_and(|value| !value.is_finite() || value < 0.0)
                || video
                    .duration_seconds
                    .is_some_and(|value| !value.is_finite() || value < 0.0)
        })
    {
        return Err(invalid("Invalid bounded child runtime values"));
    }
    for overlay in [
        &snapshot.play_overlay,
        &snapshot.player_root,
        &snapshot.popup_overlay,
    ] {
        if !overlay.present
            && (overlay.displayed
                || overlay.visible
                || overlay.has_layout
                || overlay.center_in_viewport
                || overlay.center_hit)
        {
            return Err(invalid("Invalid absent child control observation"));
        }
    }
    Ok(snapshot)
}
fn bounded_url(value: &str) -> bool {
    value.len() <= 8192 && !value.chars().any(char::is_control)
}
fn invalid(message: &'static str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn identity() -> FrameDocumentIdentity {
        FrameDocumentIdentity {
            owner_document_generation: DecimalU64(2),
            owner_native_navigation_id: DecimalU64(5),
            frame_id: DecimalU64(0),
            document_generation: DecimalU64(1),
            native_navigation_id: DecimalU64(6),
            url: "https://media.example.test/video/0123456789abcdef0123456789abcdef".into(),
        }
    }
    fn payload() -> serde_json::Value {
        let control = json!({"present":true,"displayed":false,"visible":true,"has_layout":false,"center_in_viewport":false,"center_hit":false});
        json!({"document_url":identity().url,"document_ready_state":"complete","document_visibility":"hidden","fire_player_defined":true,"fireload_defined":true,"jwplayer_defined":true,"player_loaded_flag":true,"play_overlay":control,"player_root":control,"popup_overlay":control,"video_count":0,"videos":[]})
    }
    #[test]
    fn pending_interval_and_sample_budget_bound_observation_without_promoting_site_flag() {
        let start = Instant::now();
        let mut state = FrameRuntimeState::default();
        for sample in 1..=MAX_RUNTIME_SAMPLES {
            let now = start + SAMPLE_INTERVAL * u32::from(sample);
            let request = state.reserve(&identity(), now).unwrap();
            assert_eq!(request.sample(), sample);
            assert!(state.reserve(&identity(), now).is_none());
            assert!(
                state
                    .accept(&request, &payload().to_string(), DecimalU64(1), now)
                    .unwrap()
            );
            assert!(state.reserve(&identity(), now).is_none());
            assert!(
                !state
                    .accept(&request, "malformed duplicate", DecimalU64(1), now)
                    .unwrap()
            );
        }
        assert!(state.finished());
        assert!(
            state
                .reserve(&identity(), start + Duration::from_secs(10))
                .is_none()
        );
        assert_eq!(state.observations().len(), 8);
        assert!(state.observations()[0].snapshot.videos.is_empty());
        assert_eq!(
            state.observations()[0].snapshot.player_loaded_flag,
            Some(true)
        );
    }
    #[test]
    fn foreign_identity_is_rejected_before_decode_and_changed_url_is_not_accepted() {
        let now = Instant::now();
        let mut state = FrameRuntimeState::default();
        let request = state.reserve(&identity(), now).unwrap();
        let mut foreign = request.clone();
        foreign.identity.native_navigation_id = DecimalU64(7);
        assert!(
            !state
                .accept(&foreign, "malformed stale payload", DecimalU64(1), now)
                .unwrap()
        );
        let mut wrong = payload();
        wrong["document_url"] = json!("https://example.invalid/private");
        let error = state
            .accept(&request, &wrong.to_string(), DecimalU64(1), now)
            .unwrap_err();
        assert!(!error.message.contains("private"));
        assert!(state.pending());
    }
    #[test]
    fn media_bounds_and_excessive_source_values_are_not_trusted() {
        let mut wrong = payload();
        wrong["video_count"] = json!(1);
        wrong["videos"] = json!([{"ready_state":5,"network_state":0,"paused":false,"ended":false,"muted":false,"current_time_seconds":null,"duration_seconds":null,"error_code":null,"current_src":"","video_width":0,"video_height":0}]);
        assert!(decode(&wrong.to_string()).is_err());
        wrong["videos"][0]["ready_state"] = json!(4);
        wrong["videos"][0]["current_src"] = json!("x".repeat(8193));
        assert!(decode(&wrong.to_string()).is_err());
        assert!(decode(&"x".repeat(MAX_RUNTIME_ENVELOPE_BYTES + 1)).is_err());
    }
}
