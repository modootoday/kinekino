use super::*;
use crate::{
    application::{
        catalog::CaptureMetadata, episode_resolve::StoredEpisodeResolution, jobs::VerifiedJobOutput,
    },
    domain::api_values::DecimalU64,
};
use std::sync::Mutex;

#[derive(Default)]
struct ExpiredResults(Mutex<Vec<String>>);
impl EpisodeResolutionResults for ExpiredResults {
    fn publish(
        &self,
        _: &StoredEpisodeResolution,
        _: &CaptureMetadata,
    ) -> Result<VerifiedJobOutput, ApiError> {
        panic!("Playback must not publish or repair a result")
    }
    fn read(&self, id: &str) -> Result<StoredEpisodeResolution, ApiError> {
        self.0.lock().unwrap().push(id.into());
        Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Synthetic source identity mismatch",
        ))
    }
}
struct UncalledEngine;
impl EpisodePlaybackEngine for UncalledEngine {
    fn capture_frame(&self, _: CaptureEpisodeFrame) -> Result<CapturedEpisodeFrame, ApiError> {
        panic!("Invalid capture must not reach an engine");
    }
    fn open(
        &self,
        _: EpisodePlaybackSource,
        _: bool,
        _: f64,
    ) -> Result<OpenedEpisodePlayback, ApiError> {
        panic!("Invalid or expired evidence must not start an engine")
    }
    fn close(&self, _: CloseEpisodePlayback) -> Result<ClosedEpisodePlayback, ApiError> {
        panic!("Invalid identity must not close an engine")
    }
    fn observe(&self) -> Result<EpisodePlaybackObservation, ApiError> {
        panic!("Opening must not invent a native observation")
    }
    fn control(
        &self,
        _: ControlEpisodePlayback,
    ) -> Result<crate::application::playback_control::ControlReceipt, ApiError> {
        panic!("Invalid identity must not control an engine")
    }
}

#[test]
fn native_capture_rejects_remote_paths_streams_unknown_fields_and_missing_instance() {
    let sessions = EpisodePlaybackSessions::new(
        Arc::new(ExpiredResults::default()),
        Arc::new(UncalledEngine),
    );
    for path in [
        "relative.png",
        "https://example.invalid/frame.png",
        r"\\host\frame.png",
        "C:/frame.png:stream",
        "C:/frame.jpg",
    ] {
        assert_eq!(
            sessions
                .capture_frame(CaptureEpisodeFrame {
                    engine_instance_id: DecimalU64(1),
                    output_path: path.into()
                })
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
    }
    let local = std::env::temp_dir()
        .join("synthetic-native.png")
        .to_string_lossy()
        .into_owned();
    assert_eq!(
        sessions
            .capture_frame(CaptureEpisodeFrame {
                engine_instance_id: DecimalU64(0),
                output_path: local
            })
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert!(serde_json::from_value::<CaptureEpisodeFrame>(serde_json::json!({"engine_instance_id":"1","output_path":"C:/frame.png","source_path":"untrusted"})).is_err());
}

#[test]
fn invalid_inputs_do_not_read_sources_and_expired_evidence_never_reaches_engine() {
    let results = Arc::new(ExpiredResults::default());
    let sessions = EpisodePlaybackSessions::new(results.clone(), Arc::new(UncalledEngine));
    let id = "a".repeat(64);
    for (artifact_id, volume) in [
        ("invalid".into(), 50.),
        (id.clone(), f64::NAN),
        (id.clone(), 101.),
    ] {
        assert_eq!(
            sessions
                .open(OpenEpisodePlayback {
                    resolution_artifact_id: artifact_id,
                    paused: false,
                    volume_percent: volume,
                })
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
    }
    assert!(results.0.lock().unwrap().is_empty());
    assert_eq!(
        sessions
            .open(OpenEpisodePlayback {
                resolution_artifact_id: id.clone(),
                paused: true,
                volume_percent: 0.,
            })
            .unwrap_err()
            .code,
        ErrorCode::SourceExpired
    );
    assert_eq!(*results.0.lock().unwrap(), vec![id]);
    assert_eq!(
        sessions
            .close(CloseEpisodePlayback {
                engine_instance_id: DecimalU64(0),
            })
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
}

#[test]
fn external_requests_cannot_supply_urls_headers_or_engine_options() {
    for field in ["url", "referer", "cookies", "executable", "arguments"] {
        let mut value = serde_json::json!({
            "resolution_artifact_id":"a".repeat(64),
            "paused":true,
            "volume_percent":0.,
        });
        value[field] = serde_json::json!("synthetic-untrusted-value");
        assert!(
            serde_json::from_value::<OpenEpisodePlayback>(value).is_err(),
            "{field}"
        );
    }
}
