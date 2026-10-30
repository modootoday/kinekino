//! Read-only engine clock for one exact source and original engine; never source PTS authority.
use super::*;
use crate::{
    application::playback_status::{PlaybackStatus, PlaybackStatusPort, project_native_status},
    domain::{
        api_values::DecimalU64,
        review_source::{MediaReference, ReviewCase},
    },
};
#[derive(Clone)]
pub struct ReviewWorkerStatusReader {
    state: std::sync::Weak<Mutex<ReviewWorkerSnapshot>>,
    case: ReviewCase,
    media: MediaReference,
    instance: u64,
}
impl ReviewWorkerStatusReader {
    pub(super) fn new(
        state: &State,
        case: ReviewCase,
        media: MediaReference,
        instance: u64,
    ) -> Result<Self, ApiError> {
        if case.validate().is_err() || media.validate().is_err() || instance == 0 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Exact valid review source and engine instance required",
            ));
        }
        Ok(Self {
            state: Arc::downgrade(state),
            case,
            media,
            instance,
        })
    }
}
impl PlaybackStatusPort for ReviewWorkerStatusReader {
    fn available(&self) -> bool {
        self.state.upgrade().is_some()
    }
    fn status(&self) -> Result<PlaybackStatus, ApiError> {
        let state = self.state.upgrade().ok_or_else(expired)?;
        let state = state
            .lock()
            .map_err(|_| ApiError::new(ErrorCode::Internal, "Review status owner poisoned"))?;
        if state.physical_closed || state.owner_thread_ended {
            return Err(expired());
        }
        let source = state.source.as_ref().ok_or_else(pending)?;
        if source.case != self.case
            || source.media != self.media
            || state
                .engine_instance_id
                .is_some_and(|id| id != self.instance)
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Review status source or original engine binding differs",
            ));
        }
        if state.closing || state.engine_instance_id.is_none() || !state.playback.connected {
            return Err(pending());
        }
        if state.first_error.is_some() {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Review status original engine has failed",
            ));
        }
        let mut result = project_native_status(&state.playback);
        result.published_age_ms = DecimalU64(
            state
                .published_at
                .elapsed()
                .as_millis()
                .try_into()
                .unwrap_or(u64::MAX),
        );
        Ok(result)
    }
}
fn pending() -> ApiError {
    ApiError::new(
        ErrorCode::NeedsInput,
        "No live observation for the exact review source and engine",
    )
}
fn expired() -> ApiError {
    ApiError::new(
        ErrorCode::NotFound,
        "Original review playback owner has ended",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::review_playback::{ReviewContentClass, ReviewInputIdentity};
    #[test]
    fn source_instance_close_and_owner_expiry_never_return_stale_clock() {
        // Explicit synthetic observation; no actual engine, PTS proof, or quality claim.
        let case = ReviewCase {
            package_sha256: "a".repeat(64),
            item_id: "item".into(),
            source_request_sha256: "b".repeat(64),
        };
        let media = MediaReference {
            asset_id: "source".into(),
            sha256: "c".repeat(64),
        };
        let source = ReviewInputIdentity {
            case: case.clone(),
            media: media.clone(),
            bytes: 32,
            declared_media_type: "video/mp4".into(),
            classification: ReviewContentClass::IsoBmffHeader,
        };
        let state = Arc::new(Mutex::new(ReviewWorkerSnapshot {
            source: Some(source),
            process_created: true,
            pid: Some(1),
            creation_filetime: Some(1),
            engine_instance_id: Some(7),
            playback: NativePlayback {
                connected: true,
                time: Some(17.),
                duration: Some(25.5),
                pause_observed: true,
                paused: true,
                ..Default::default()
            },
            closing: false,
            physical_closed: false,
            child_exit: None,
            family_empty: false,
            forced: false,
            ipc_exited: false,
            first_error: None,
            first_error_code: None,
            owner_thread_ended: false,
            published_at: Instant::now(),
        }));
        let reader = ReviewWorkerStatusReader::new(&state, case.clone(), media.clone(), 7).unwrap();
        let mut foreign_case = case.clone();
        foreign_case.item_id = "other-item".into();
        assert_eq!(
            ReviewWorkerStatusReader::new(&state, foreign_case, media.clone(), 7)
                .unwrap()
                .status()
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(
            reader.status().unwrap().position_us,
            Some(DecimalU64(17_000_000))
        );
        let mut foreign = media.clone();
        foreign.sha256 = "d".repeat(64);
        assert_eq!(
            ReviewWorkerStatusReader::new(&state, case.clone(), foreign, 7)
                .unwrap()
                .status()
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(
            ReviewWorkerStatusReader::new(&state, case, media, 8)
                .unwrap()
                .status()
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        state.lock().unwrap().owner_thread_ended = true;
        assert_eq!(reader.status().unwrap_err().code, ErrorCode::NotFound);
        state.lock().unwrap().owner_thread_ended = false;
        state.lock().unwrap().closing = true;
        assert_eq!(reader.status().unwrap_err().code, ErrorCode::NeedsInput);
        state.lock().unwrap().physical_closed = true;
        assert_eq!(reader.status().unwrap_err().code, ErrorCode::NotFound);
        drop(state);
        assert!(!reader.available());
        assert_eq!(reader.status().unwrap_err().code, ErrorCode::NotFound);
    }
}
