//! Observed-clock review window policy. No native process or proof ownership here.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_status::PlaybackStatus,
    },
    domain::{playback_control::PlaybackControl, review_source::PlaybackWindow},
};
pub struct ReviewWindow {
    window: PlaybackWindow,
}
impl ReviewWindow {
    pub fn new(start_us: u64, end_us: u64) -> Result<Self, ApiError> {
        Ok(Self {
            window: PlaybackWindow::new(start_us, end_us).map_err(|_| {
                ApiError::new(ErrorCode::InvalidArgument, "Invalid retained review window")
            })?,
        })
    }
    pub fn check_control(
        &self,
        intent: &PlaybackControl,
        status: &PlaybackStatus,
    ) -> Result<(), ApiError> {
        intent
            .validate()
            .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
        match intent {
            PlaybackControl::Seek { position_us } if !self.window.contains_seek(position_us.0) => {
                Err(needs_input())
            }
            PlaybackControl::Play => {
                if !status.connected
                    || status.paused.is_none()
                    || !status
                        .position_us
                        .is_some_and(|position| self.window.contains_play(position.0))
                {
                    Err(needs_input())
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
    /// Poll-observed pause request, never a hard sample/frame stop guarantee.
    pub fn should_pause(&self, status: &PlaybackStatus) -> Result<bool, ApiError> {
        if !status.connected || status.paused != Some(false) {
            return Ok(false);
        }
        let position = status.position_us.ok_or_else(needs_input)?;
        Ok(!self.window.contains_play(position.0))
    }
}
fn needs_input() -> ApiError {
    ApiError::new(
        ErrorCode::NeedsInput,
        "Observed source clock must be inside the retained review window",
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        application::playback_status::PlaybackStatusOrigin, domain::api_values::DecimalU64,
    };
    fn status(position: Option<u64>, paused: Option<bool>) -> PlaybackStatus {
        PlaybackStatus {
            origin: PlaybackStatusOrigin::NativeObservation,
            published_age_ms: DecimalU64(0),
            connected: true,
            paused,
            idle: Some(false),
            position_us: position.map(DecimalU64),
            duration_us: Some(DecimalU64(600_000_000)),
            volume_percent: None,
            muted: None,
            video_size: None,
            audio_channels: None,
            has_error: false,
        }
    }
    #[test]
    fn play_requires_observed_inner_clock_and_seek_keeps_both_observed_endpoints() {
        let window = ReviewWindow::new(17_000_000, 25_500_000).unwrap();
        for position in [17_000_000, 25_500_000] {
            assert!(
                window
                    .check_control(
                        &PlaybackControl::Seek {
                            position_us: DecimalU64(position)
                        },
                        &status(None, None)
                    )
                    .is_ok()
            );
        }
        for position in [16_999_999, 25_500_001] {
            assert_eq!(
                window
                    .check_control(
                        &PlaybackControl::Seek {
                            position_us: DecimalU64(position)
                        },
                        &status(None, None)
                    )
                    .unwrap_err()
                    .code,
                ErrorCode::NeedsInput
            );
        }
        assert!(
            window
                .check_control(
                    &PlaybackControl::Play,
                    &status(Some(17_000_000), Some(true))
                )
                .is_ok()
        );
        for clock in [
            status(None, Some(true)),
            status(Some(0), Some(true)),
            status(Some(25_500_000), Some(true)),
            status(Some(17_000_000), None),
        ] {
            assert_eq!(
                window
                    .check_control(&PlaybackControl::Play, &clock)
                    .unwrap_err()
                    .code,
                ErrorCode::NeedsInput
            );
        }
    }
    #[test]
    fn moving_beyond_window_requests_pause_without_expanding_to_media_duration() {
        let window = ReviewWindow::new(17_000_000, 25_500_000).unwrap();
        assert!(
            !window
                .should_pause(&status(Some(20_000_000), Some(false)))
                .unwrap()
        );
        assert!(
            window
                .should_pause(&status(Some(25_500_000), Some(false)))
                .unwrap()
        );
        assert!(
            window
                .should_pause(&status(Some(16_999_999), Some(false)))
                .unwrap()
        );
        assert!(
            !window
                .should_pause(&status(Some(25_600_000), Some(true)))
                .unwrap()
        );
        assert_eq!(
            window
                .should_pause(&status(None, Some(false)))
                .unwrap_err()
                .code,
            ErrorCode::NeedsInput
        );
    }
}
