//! Shared engine-time/status projection. No publication, receive-clock update, or authority.
use super::{PlaybackStatus, PlaybackStatusOrigin, VideoSize};
use crate::{application::native_playback::NativePlayback, domain::api_values::DecimalU64};

pub(super) fn engine_micros(value: Option<f64>) -> Option<DecimalU64> {
    let scaled = value? * 1_000_000.;
    (scaled.is_finite() && scaled >= 0. && scaled < u64::MAX as f64)
        .then(|| DecimalU64(scaled.floor() as u64))
}
pub(crate) fn project_native_status(state: &NativePlayback) -> PlaybackStatus {
    PlaybackStatus {
        origin: PlaybackStatusOrigin::NativeObservation,
        published_age_ms: DecimalU64(0),
        connected: state.connected,
        paused: state.pause_observed.then_some(state.paused),
        idle: state.idle_observed.then_some(state.idle),
        position_us: engine_micros(state.time),
        duration_us: engine_micros(state.duration),
        volume_percent: state.volume.filter(|v| v.is_finite() && *v >= 0.),
        muted: state.muted,
        video_size: state
            .width
            .zip(state.height)
            .and_then(|(w, h)| {
                Some(VideoSize {
                    width: w.try_into().ok()?,
                    height: h.try_into().ok()?,
                })
            })
            .filter(|v| v.width > 0 && v.height > 0),
        audio_channels: state
            .audio_channels
            .and_then(|v| u32::try_from(v).ok())
            .filter(|v| *v > 0),
        has_error: state.error.is_some(),
    }
}
