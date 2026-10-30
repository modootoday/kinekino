//! Render and forward the shared application handoff state; no audio policy or OS calls.
use super::*;
use crate::infrastructure::browser::LegacyAudioHandoffPorts;

impl PlayerApp {
    pub(super) fn advance_native_audio(&mut self) {
        let Some(mut handoff) = self.native_audio.take() else {
            return;
        };
        let was_ready = self.native_started;
        {
            let mut ports = LegacyAudioHandoffPorts::new(&mut self.mpv, &self.browser);
            handoff.drive(&mut ports, Instant::now());
        }
        self.native_started = handoff.is_ready();
        if !was_ready && self.native_started {
            if let Some(paused) = handoff.requested_pause() {
                self.native_pause.reset(paused);
            }
            match self.browser.set_visible(false) {
                Ok(()) => {
                    self.browser_open = false;
                    self.session.status = "네이티브 재생 · 앱 내부 영상".into();
                }
                Err(error) => {
                    self.native_control_error =
                        Some(format!("오디오 준비 완료 · 웹 화면 숨김 실패: {error}"));
                }
            }
        }
        if let Some(error) = handoff.failure() {
            self.native_control_error = Some(format!("오디오 전환 실패: {}", error.message));
            self.session.status =
                "네이티브 오디오 전환 실패 · 정리 상태를 확인한 뒤 다시 열어 주세요".into();
            if handoff.cleanup_complete() && !self.local_media {
                match self.browser.set_visible(true) {
                    Ok(()) => self.browser_open = true,
                    Err(error) => {
                        self.native_control_error =
                            Some(format!("정리 후 웹 화면 표시 실패: {error}"))
                    }
                }
            }
        }
        if self.mpv.is_none() {
            self.clear_native_control();
            self.clear_native_status();
            self.native_connected = false;
        }
        if let Ok(observation) = serde_json::to_value(handoff.observation())
            && self.native_audio_logged.as_ref() != Some(&observation)
        {
            diagnostics::record("native-audio-handoff", &observation);
            self.native_audio_logged = Some(observation);
        }
        self.native_audio = Some(handoff);
    }

    pub(super) fn stop_native(&mut self) -> bool {
        if let Some(mut handoff) = self.native_audio.take() {
            {
                let mut ports = LegacyAudioHandoffPorts::new(&mut self.mpv, &self.browser);
                handoff.close(&mut ports);
            }
            diagnostics::record("native-audio-close", handoff.observation());
            if !handoff.cleanup_complete() {
                self.native_control_error = Some(handoff.failure().map_or_else(
                    || "네이티브 종료 확인이 필요합니다".into(),
                    |error| format!("네이티브 정리 실패: {}", error.message),
                ));
                self.native_started = false;
                self.native_audio = Some(handoff);
                return false;
            }
        } else if let Some(player) = self.mpv.as_mut() {
            let stopped = match player.shutdown_observed() {
                Ok(stopped) => stopped,
                Err(error) => {
                    self.native_control_error =
                        Some(format!("네이티브 종료 실패: {}", error.message));
                    return false;
                }
            };
            if let Err(error) = self
                .audio_intents
                .clear_expected(stopped.engine_instance_id())
            {
                self.native_control_error =
                    Some(format!("오디오 소유권 정리 실패: {}", error.message));
                return false;
            }
            self.mpv.take();
        }
        self.clear_native_control();
        self.clear_native_status();
        self.native_started = false;
        self.native_connected = false;
        self.scrub = None;
        if let Err(error) = self.browser.set_visible(self.browser_open) {
            self.native_control_error = Some(format!("웹 화면 표시 상태 적용 실패: {error}"));
            return false;
        }
        true
    }
}
