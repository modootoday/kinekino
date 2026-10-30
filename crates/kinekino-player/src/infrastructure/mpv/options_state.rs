//! Prepared owned-instance option observations. No polling or implicit command execution.
use crate::{
    application::playback_session::PlaybackOptionsObservation,
    domain::playback_options::{TrackCatalog, TrackCatalogError},
};
use serde_json::Value;

#[derive(Default)]
pub(super) struct OptionsState {
    pub catalog: TrackCatalog,
    observation: PlaybackOptionsObservation,
}
impl OptionsState {
    pub fn view(&self) -> PlaybackOptionsObservation {
        self.observation.clone()
    }
    pub fn apply(&mut self, message: &Value) {
        if matches!(
            message["event"].as_str(),
            Some("start-file" | "end-file" | "shutdown")
        ) {
            self.invalidate_tracks();
            self.observation.audio_selection = None;
            self.observation.video_selection = None;
            self.observation.subtitle_selection = None;
            return;
        }
        if message["event"] != "property-change" {
            return;
        }
        let value = &message["data"];
        match message["name"].as_str() {
            Some("speed") => self.observation.rate = super::option_projection::rate(value),
            Some("aid") => {
                self.observation.audio_selection = super::option_projection::choice(value)
            }
            Some("vid") => {
                self.observation.video_selection = super::option_projection::choice(value)
            }
            Some("sid") => {
                self.observation.subtitle_selection = super::option_projection::choice(value)
            }
            Some("track-list") if value.is_null() => self.invalidate_tracks(),
            Some("track-list") => {
                self.observation.tracks_unavailable_reason =
                    super::track_projection::apply(&mut self.catalog, value).err();
                self.observation.tracks = self.catalog.view();
            }
            _ => {}
        }
    }
    fn invalidate_tracks(&mut self) {
        self.observation.tracks_unavailable_reason = Some(
            self.catalog
                .invalidate()
                .err()
                .unwrap_or(TrackCatalogError::Unobserved),
        );
        self.observation.tracks = self.catalog.view();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        api_values::DecimalU64,
        playback_options::{TrackId, TrackKind, TrackSelection},
    };
    use serde_json::json;
    #[test]
    fn file_lifetime_invalidates_catalog_and_effective_choices_but_not_global_rate() {
        let mut state = OptionsState::default();
        state.apply(&json!({"event":"property-change","name":"speed","data":1.25}));
        state.apply(&json!({"event":"property-change","name":"aid","data":1}));
        state.apply(&json!({"event":"property-change","name":"track-list","data":[{"id":1,"type":"audio","selected":true}]}));
        let revision = state.view().tracks.unwrap().revision;
        assert!(
            state
                .catalog
                .authorize(
                    revision,
                    TrackKind::Audio,
                    TrackSelection::Id {
                        track_id: TrackId::new(1).unwrap()
                    }
                )
                .is_ok()
        );
        state.apply(&json!({"event":"end-file"}));
        let ended = state.view();
        assert_eq!(ended.rate, Some(1.25));
        assert!(ended.audio_selection.is_none());
        assert!(ended.tracks.is_none());
        assert_eq!(
            state
                .catalog
                .authorize(revision, TrackKind::Audio, TrackSelection::Disabled),
            Err(TrackCatalogError::Unobserved)
        );
        state.apply(&json!({"event":"property-change","name":"track-list","data":[]}));
        assert_ne!(state.view().tracks.unwrap().revision, revision);
        assert!(
            state
                .catalog
                .authorize(DecimalU64(0), TrackKind::Audio, TrackSelection::Disabled)
                .is_err()
        );
    }
}
