use super::*;
use crate::application::api::PlaybackClient;
use crate::{
    application::api::{ApiError, ErrorCode, RequestContext},
    domain::{api_values::DecimalU64, playback_control::PlaybackControl},
};

fn seek_control(seconds: f64) -> Result<PlaybackControl, ApiError> {
    let micros = seconds * 1_000_000.;
    if !micros.is_finite()
        || micros < 0.
        || micros > crate::domain::playback_control::MAX_SEEK_US as f64
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid seek position",
        ));
    }
    Ok(PlaybackControl::Seek {
        position_us: DecimalU64(micros.round() as u64),
    })
}

impl PlayerApp {
    pub fn with_native_control(
        mut self,
        binder: Option<crate::infrastructure::native_playback_control::NativeControlBinder>,
    ) -> Self {
        if let Some(binder) = &binder {
            self.audio_intents = binder.audio_intents();
        }
        self.native_control = binder;
        self.clear_native_control();
        self
    }

    pub(super) fn clear_native_control(&mut self) {
        if let Some(binder) = &self.native_control
            && let Err(error) = binder.clear()
        {
            self.native_control_error =
                Some(format!("재생 제어 연결 해제 실패: {}", error.message));
        }
    }

    pub(super) fn install_native(
        &mut self,
        player: Mpv,
        prepared: crate::application::audio_handoff::coordinator::PreparedAudioHandoff,
    ) -> bool {
        let handle = player.control_handle();
        let engine = DecimalU64(handle.instance_id());
        let requested = prepared.requested();
        self.mpv = Some(player);
        if let Some(binder) = &self.native_control
            && let Err(error) = binder.bind_for_handoff(handle.clone(), requested)
        {
            self.native_control_error = Some(format!("재생 제어 연결 실패: {}", error.message));
            self.clear_native_control();
            self.stop_native();
            return false;
        }
        if self.native_control.is_none()
            && let Err(error) = self.audio_intents.bind_for_handoff(engine, requested)
        {
            self.native_control_error = Some(format!("오디오 전환 연결 실패: {}", error.message));
            self.stop_native();
            return false;
        }
        match prepared.bind(engine, self.audio_intents.clone()) {
            Ok(handoff) => self.native_audio = Some(handoff),
            Err(error) => {
                self.native_control_error =
                    Some(format!("오디오 전환 준비 실패: {}", error.message));
                self.stop_native();
                return false;
            }
        }
        self.native_control_error = None;
        true
    }

    pub(super) fn submit_native(&mut self, control: PlaybackControl) {
        let Some(player) = &self.mpv else {
            self.native_control_error = Some("제어할 네이티브 재생이 없습니다.".into());
            return;
        };
        let requested = control.clone();
        let shared_provider = self.native_control.is_some();
        let result = if shared_provider {
            self.client
                .playback_control(RequestContext::default(), control)
        } else {
            // Compatibility entry points have no shared provider injected.
            let handle = player.control_handle();
            self.audio_intents
                .submit(&handle, DecimalU64(handle.instance_id()), control)
        };
        match result {
            Ok(receipt) => {
                self.native_control_error = None;
                diagnostics::record(
                    "native-control-receipt",
                    json!({
                        "requested": requested,
                        "shared_provider": shared_provider,
                        "receipt": receipt,
                    }),
                );
            }
            Err(error) => {
                diagnostics::record(
                    "native-control-rejected",
                    json!({
                        "requested": requested,
                        "shared_provider": shared_provider,
                        "error": error,
                    }),
                );
                self.native_control_error = Some(format!("재생 제어 요청 실패: {}", error.message));
            }
        }
    }

    pub(super) fn seek_native(&mut self, seconds: f64) {
        match seek_control(seconds) {
            Ok(control) => self.submit_native(control),
            Err(error) => {
                self.native_control_error = Some(format!("이동 요청 실패: {}", error.message))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gui_seek_converts_seconds_without_saturating_invalid_values() {
        assert_eq!(
            seek_control(1.234567).unwrap(),
            PlaybackControl::Seek {
                position_us: DecimalU64(1_234_567)
            }
        );
        assert_eq!(
            seek_control(0.).unwrap(),
            PlaybackControl::Seek {
                position_us: DecimalU64(0)
            }
        );
        for invalid in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            -0.1,
            9_007_199_255.,
        ] {
            assert_eq!(
                seek_control(invalid).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
        }
    }
}
