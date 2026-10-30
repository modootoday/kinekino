#[derive(Debug, Clone)]
pub struct BrowserMessage {
    pub generation: u64,
    pub event: BrowserEvent,
}
impl BrowserMessage {
    pub fn into_current(self, generation: u64) -> Option<BrowserEvent> {
        (self.generation != 0 && self.generation == generation).then_some(self.event)
    }
}

#[derive(Debug, Clone)]
pub enum BrowserEvent {
    InputActivity,
    Episodes(crate::domain::episodes::EpisodePage),
    Navigated(String),
    OpenRequested(String),
    Media(crate::domain::media::MediaSource),
    Blocked,
    Loaded,
    Requested,
    Playback(crate::domain::playback::WebPlayback),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obsolete_manifest_callback_cannot_restore_previous_media() {
        let source = crate::domain::media::MediaSource {
            url: "https://cdn.example/old.m3u8".into(),
            referer: "https://media.example.test/".into(),
            hls: true,
        };
        let mut session = Session::default();
        let old = BrowserMessage {
            generation: 1,
            event: BrowserEvent::Media(source.clone()),
        };
        if let Some(event) = old.into_current(2) {
            session.observe(event);
        }
        assert!(session.media.is_none());
        let current = BrowserMessage {
            generation: 2,
            event: BrowserEvent::Media(source),
        };
        if let Some(event) = current.into_current(2) {
            session.observe(event);
        }
        assert!(session.media.is_some());
    }

    #[test]
    fn navigation_discards_previous_media_without_requesting_reload() {
        let mut session = Session::default();
        session.observe(BrowserEvent::Media(crate::domain::media::MediaSource {
            url: "https://cdn.example/a.m3u8".into(),
            referer: "https://media.example.test/".into(),
            hls: true,
        }));
        assert!(session.media.is_some());
        assert!(
            session
                .observe(BrowserEvent::Navigated(
                    "https://catalog.example.test/".into()
                ))
                .is_none()
        );
        assert!(session.media.is_none());
        assert_eq!(
            session.observe(BrowserEvent::OpenRequested(
                "https://frame.example.test/".into()
            )),
            Some("https://frame.example.test/".into())
        );
    }
}

#[derive(Default)]
pub struct Session {
    pub blocked: u64,
    pub media: Option<crate::domain::media::MediaSource>,
    pub status: String,
    pub loaded: bool,
    pub requested: u64,
    pub playback: Option<crate::domain::playback::WebPlayback>,
    playback_document_generation: Option<u64>,
    playback_sequence: u64,
}

impl Session {
    pub fn reset_page(&mut self) {
        self.media = None;
        self.loaded = false;
        self.playback = None;
        self.playback_document_generation = None;
        self.playback_sequence = 0;
        self.status = "Web compatibility mode".into();
    }

    pub fn observe(&mut self, event: BrowserEvent) -> Option<String> {
        self.observe_bound(event, None)
    }

    /// Retain the accepted document identity with its observed playback revision.
    pub fn observe_message(
        &mut self,
        message: BrowserMessage,
        current_generation: u64,
    ) -> Option<String> {
        if current_generation == 0 {
            return None;
        }
        let event = message.into_current(current_generation)?;
        self.observe_bound(event, Some(current_generation))
    }

    pub fn audio_handoff_origin(&self) -> Option<crate::application::audio_handoff::AudioOrigin> {
        let generation = self.playback_document_generation?;
        let playback = self.playback.as_ref()?;
        if self.playback_sequence == 0 || playback.duration <= 0. {
            return None;
        }
        Some(crate::application::audio_handoff::AudioOrigin::Web {
            document_generation: crate::domain::api_values::DecimalU64(generation),
            observed_sequence: crate::domain::api_values::DecimalU64(self.playback_sequence),
            expected_duration_seconds: playback.duration,
        })
    }

    pub fn web_silence_observation(
        &self,
    ) -> Option<crate::application::audio_handoff::WebSilenceObservation> {
        let generation = self.playback_document_generation?;
        let playback = self.playback.as_ref()?;
        if self.playback_sequence == 0 {
            return None;
        }
        Some(crate::application::audio_handoff::WebSilenceObservation {
            document_generation: crate::domain::api_values::DecimalU64(generation),
            observation_sequence: crate::domain::api_values::DecimalU64(self.playback_sequence),
            paused: Some(playback.paused),
            muted: Some(playback.muted),
            view_muted: None,
            confirmed_for_engine_instance_id: None,
        })
    }

    fn observe_bound(&mut self, event: BrowserEvent, generation: Option<u64>) -> Option<String> {
        match event {
            BrowserEvent::Episodes(_) | BrowserEvent::InputActivity => None,
            BrowserEvent::Navigated(_) => {
                self.reset_page();
                None
            }
            BrowserEvent::OpenRequested(url) => {
                self.reset_page();
                Some(url)
            }
            BrowserEvent::Media(source) => {
                self.media = Some(source);
                None
            }
            BrowserEvent::Blocked => {
                self.blocked += 1;
                None
            }
            BrowserEvent::Loaded => {
                self.loaded = true;
                None
            }
            BrowserEvent::Requested => {
                self.requested += 1;
                None
            }
            BrowserEvent::Playback(observation) => {
                if !observation.valid() {
                    return None;
                }
                let Some(sequence) = self.playback_sequence.checked_add(1) else {
                    self.playback = None;
                    self.playback_document_generation = None;
                    return None;
                };
                self.playback_sequence = sequence;
                self.playback_document_generation = generation;
                self.playback = Some(observation);
                None
            }
        }
    }
}

#[cfg(test)]
mod audio_evidence_tests {
    use super::*;
    fn playback(paused: bool) -> crate::domain::playback::WebPlayback {
        crate::domain::playback::WebPlayback {
            host: "catalog.example.test".into(),
            time: 2.,
            duration: 1420.,
            paused,
            muted: false,
            volume: 0.8,
            ready: 4,
            width: 1920,
            height: 1080,
            error: 0,
            hidden_ads: 0,
            visible_ads: 0,
            audio_bytes: 1,
            decoded_frames: 1,
        }
    }
    fn message(generation: u64, paused: bool) -> BrowserMessage {
        BrowserMessage {
            generation,
            event: BrowserEvent::Playback(playback(paused)),
        }
    }
    #[test]
    fn bound_observations_keep_original_generation_and_checked_monotonic_sequence() {
        let mut session = Session::default();
        session.observe_message(message(11, false), 11);
        let first = session.web_silence_observation().unwrap();
        assert_eq!(first.document_generation.0, 11);
        assert_eq!(first.observation_sequence.0, 1);
        session.observe_message(message(10, true), 11);
        let stale = session.web_silence_observation().unwrap();
        assert_eq!(stale.observation_sequence, first.observation_sequence);
        assert_eq!(stale.paused, Some(false));
        session.observe_message(message(11, true), 11);
        let fresh = session.web_silence_observation().unwrap();
        assert_eq!(fresh.observation_sequence.0, 2);
        assert_eq!(fresh.paused, Some(true));
        session.reset_page();
        assert!(session.web_silence_observation().is_none());
        assert!(session.audio_handoff_origin().is_none());
    }
    #[test]
    fn generic_unbound_playback_is_ui_state_not_audio_silence_evidence() {
        let mut session = Session::default();
        session.observe(BrowserEvent::Playback(playback(true)));
        assert!(session.playback.is_some());
        assert!(session.web_silence_observation().is_none());
        assert!(session.audio_handoff_origin().is_none());
    }
    #[test]
    fn invalid_or_exhausted_playback_never_fabricates_fresh_silence() {
        let mut session = Session::default();
        let mut invalid = playback(true);
        invalid.volume = f64::NAN;
        session.observe_message(
            BrowserMessage {
                generation: 11,
                event: BrowserEvent::Playback(invalid),
            },
            11,
        );
        assert!(session.web_silence_observation().is_none());
        session.playback_sequence = u64::MAX;
        session.observe_message(message(11, true), 11);
        assert!(session.playback.is_none());
        assert!(session.web_silence_observation().is_none());
        assert_eq!(session.playback_sequence, u64::MAX);
    }
}
