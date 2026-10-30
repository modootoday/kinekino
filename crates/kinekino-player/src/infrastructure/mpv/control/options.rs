use super::MpvControlHandle;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_control::ControlReceipt,
        playback_session::{PlaybackOptionsObservation, SelectEpisodeTrack, TrackControlReceipt},
    },
    domain::playback_options::{PlaybackRate, TrackCatalogError, TrackKind, TrackSelection},
};
use serde_json::json;

// Trusted SDK operation budget, never derived from web messages or event freshness.
pub(super) const GUARDED_OPERATION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

impl MpvControlHandle {
    pub(crate) fn options_observation(&self) -> Result<PlaybackOptionsObservation, ApiError> {
        Ok(self.state.options.lock().map_err(|_| unavailable())?.view())
    }
    pub(crate) fn submit_rate(&self, rate: PlaybackRate) -> Result<ControlReceipt, ApiError> {
        self.enqueue(json!(["set_property", "speed", rate.get()]))
    }
    pub(crate) fn submit_track(
        &self,
        input: SelectEpisodeTrack,
    ) -> Result<TrackControlReceipt, ApiError> {
        input.validate()?;
        if input.engine_instance_id.0 != self.instance_id {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native playback instance changed",
            ));
        }
        let options = self.state.options.lock().map_err(|_| unavailable())?;
        options
            .catalog
            .authorize(
                input.expected_tracks_revision,
                input.track_kind,
                input.selection,
            )
            .map_err(|reason| {
                ApiError::new(
                    match reason {
                        TrackCatalogError::StaleRevision => ErrorCode::RevisionConflict,
                        TrackCatalogError::UnknownTrack => ErrorCode::NotFound,
                        _ => ErrorCode::NeedsInput,
                    },
                    "Native track selection does not match the latest observed catalog",
                )
            })?;
        let property = match input.track_kind {
            TrackKind::Audio => "aid",
            TrackKind::Video => "vid",
            TrackKind::Subtitle => "sid",
        };
        let selection = match input.selection {
            TrackSelection::Disabled => json!("no"),
            TrackSelection::Id { track_id } => json!(track_id.get()),
        };
        let receipt = self.enqueue(json!(["set_property", property, selection]))?;
        Ok(TrackControlReceipt::enqueued(
            receipt,
            input.expected_tracks_revision,
        ))
    }
}
fn unavailable() -> ApiError {
    ApiError::new(
        ErrorCode::Internal,
        "Native option observations unavailable",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        api_values::DecimalU64,
        playback_options::{NativeTrack, TrackId},
    };
    use std::sync::atomic::Ordering;
    #[test]
    fn missing_kind_stale_revision_and_closed_instance_never_enqueue_track_commands() {
        let (sender, mut receiver) = tokio::sync::mpsc::channel(8);
        let handle = MpvControlHandle::new(sender).unwrap();
        handle.state.connected.store(true, Ordering::Release);
        handle
            .state
            .options
            .lock()
            .unwrap()
            .catalog
            .observe(vec![NativeTrack {
                id: TrackId::new(1).unwrap(),
                kind: TrackKind::Audio,
                language: None,
                codec: None,
                selected: Some(true),
                external: None,
            }])
            .unwrap();
        let mut input = SelectEpisodeTrack {
            engine_instance_id: DecimalU64(handle.instance_id),
            expected_tracks_revision: DecimalU64(1),
            track_kind: TrackKind::Video,
            selection: TrackSelection::Id {
                track_id: TrackId::new(1).unwrap(),
            },
        };
        assert_eq!(
            handle.submit_track(input.clone()).unwrap_err().code,
            ErrorCode::NotFound
        );
        assert!(receiver.try_recv().is_err());
        input.track_kind = TrackKind::Audio;
        let receipt = handle.submit_track(input.clone()).unwrap();
        assert_eq!(receipt.validated_tracks_revision.0, 1);
        assert_eq!(
            receiver.try_recv().unwrap(),
            json!(["set_property", "aid", 1])
        );
        handle
            .state
            .options
            .lock()
            .unwrap()
            .catalog
            .observe(vec![])
            .unwrap();
        assert_eq!(
            handle.submit_track(input.clone()).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        assert!(receiver.try_recv().is_err());
        input.expected_tracks_revision = DecimalU64(2);
        input.selection = TrackSelection::Disabled;
        handle.state.close();
        assert_eq!(
            handle.submit_track(input).unwrap_err().code,
            ErrorCode::NotFound
        );
        assert!(receiver.try_recv().is_err());
    }
}
