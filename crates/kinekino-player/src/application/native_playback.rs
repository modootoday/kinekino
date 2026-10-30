use serde::Serialize;

#[derive(Clone, Debug, Default, Serialize)]
pub struct NativePlayback {
    pub connected: bool,
    pub time: Option<f64>,
    pub duration: Option<f64>,
    pub paused: bool,
    /// Legacy bool defaults are not evidence that a property was observed.
    pub pause_observed: bool,
    pub idle: bool,
    pub idle_observed: bool,
    pub volume: Option<f64>,
    pub muted: Option<bool>,
    pub width: Option<u64>,
    pub height: Option<u64>,
    pub audio_channels: Option<u64>,
    pub acknowledgements: u64,
    pub fullscreen: bool,
    pub error: Option<String>,
}

pub fn matches_video(state: &NativePlayback, expected: f64) -> bool {
    state.connected
        && !state.idle
        && state.error.is_none()
        && state.time.is_some_and(|time| time > 0.)
        && state
            .duration
            .is_some_and(|duration| (duration - expected).abs() < 2.)
        && state.width.is_some_and(|width| width > 0)
}

/// Preserves the requested pause state until the startup handoff has applied it.
#[derive(Default)]
pub struct NativePauseIntent {
    requested: bool,
    applied: bool,
}
impl NativePauseIntent {
    pub fn reset(&mut self, paused: bool) {
        self.requested = paused;
        self.applied = false;
    }
    pub fn requested(&self) -> bool {
        self.requested
    }
    pub fn observe(&mut self, confirmed: bool, paused: bool) {
        if confirmed && paused == self.requested {
            self.applied = true;
        }
    }
    pub fn intent(&self, observed: bool) -> bool {
        if self.applied {
            observed
        } else {
            self.requested
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pending_pause_survives_selection_before_startup_acknowledgement() {
        let mut intent = NativePauseIntent::default();
        intent.reset(true);
        assert!(intent.intent(false));
        intent.observe(false, false);
        assert!(intent.intent(false));
        intent.observe(true, false);
        assert!(intent.intent(false));
        intent.observe(true, true);
        assert!(!intent.intent(false));
    }
    #[test]
    fn does_not_confirm_individual_segments_as_a_full_episode() {
        let mut state = NativePlayback {
            connected: true,
            time: Some(1.),
            duration: Some(3.),
            width: Some(1920),
            ..Default::default()
        };
        assert!(!matches_video(&state, 1420.));
        state.duration = Some(1420.);
        assert!(matches_video(&state, 1420.));
    }
}
