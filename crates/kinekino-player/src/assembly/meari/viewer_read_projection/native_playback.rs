//! Accepted original-session observations and typed control inputs; no optimistic player state.
use crate::{
    application::{
        playback_control::ControlReceipt,
        playback_session::{ControlEpisodePlayback, EpisodePlaybackObservation},
    },
    domain::{api_values::DecimalU64, playback_control::PlaybackControl},
};
use meari::{
    application::plugins::{PluginError, PluginErrorCode, PluginTicket},
    domain::Value,
};
use std::collections::BTreeMap;

pub(crate) struct NativePlaybackProjection {
    observation: Option<EpisodePlaybackObservation>,
    pending: Vec<(PluginTicket, u64)>,
    diagnostic: Option<String>,
    seek_draft: String,
    volume_draft: String,
    range_draft: Option<RangeDraft>,
    failure: Option<FailureNotice>,
}
struct FailureNotice {
    message: String,
    uncertain: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct RangeContext {
    instance: u64,
    artifact: String,
    duration: Option<u64>,
}
struct RangeDraft {
    context: RangeContext,
    seek: bool,
    value: u64,
}
impl Default for NativePlaybackProjection {
    fn default() -> Self {
        Self {
            observation: None,
            pending: Vec::new(),
            diagnostic: None,
            seek_draft: String::new(),
            volume_draft: "100".into(),
            range_draft: None,
            failure: None,
        }
    }
}
impl NativePlaybackProjection {
    pub(crate) fn permits_controls_idle(&self) -> bool {
        self.failure.is_none()
            && self.pending.is_empty()
            && self.observation.as_ref().is_some_and(|actual| {
                actual.playback.connected
                    && actual.playback.paused == Some(false)
                    && actual.playback.idle == Some(false)
                    && !actual.playback.has_error
            })
    }
    fn range_context(&self) -> Result<RangeContext, PluginError> {
        let observation = self.settled_observation().ok_or_else(|| {
            PluginError::new(
                PluginErrorCode::MissingCapability,
                "Original range observation unavailable",
            )
        })?;
        Ok(RangeContext {
            instance: observation.engine_instance_id.0,
            artifact: observation.resolution_artifact_id.clone(),
            duration: observation.playback.duration_us.map(|value| value.0),
        })
    }
    pub(crate) fn set_range_draft(&mut self, seek: bool, text: &str) -> Result<(), PluginError> {
        use crate::presentation::playback_range::*;
        let invalid = |message| PluginError::new(PluginErrorCode::InvalidPayload, message);
        let context = self.range_context()?;
        let value = coordinate(text, if seek { SEEK_MAX } else { VOLUME_MAX }).map_err(invalid)?;
        if seek {
            seek_us(value, context.duration.unwrap_or(0)).map_err(invalid)?;
        }
        self.range_draft = Some(RangeDraft {
            context,
            seek,
            value,
        });
        Ok(())
    }
    pub(crate) fn range_control(
        &self,
        seek: bool,
        text: &str,
    ) -> Result<ControlEpisodePlayback, PluginError> {
        use crate::presentation::playback_range::*;
        let context = self.range_context()?;
        let invalid = |message| PluginError::new(PluginErrorCode::InvalidPayload, message);
        let value = coordinate(text, if seek { SEEK_MAX } else { VOLUME_MAX }).map_err(invalid)?;
        let draft = self
            .range_draft
            .as_ref()
            .filter(|draft| draft.context == context && draft.seek == seek && draft.value == value)
            .ok_or_else(|| {
                PluginError::new(
                    PluginErrorCode::Stale,
                    "Original range gesture context changed",
                )
            })?;
        let control = if seek {
            PlaybackControl::Seek {
                position_us: DecimalU64(
                    seek_us(draft.value, context.duration.unwrap_or(0)).map_err(invalid)?,
                ),
            }
        } else {
            PlaybackControl::SetVolume {
                volume_percent: draft.value as f64 / 10.0,
            }
        };
        self.control(control)
    }
    pub(crate) fn initial_volume(&self) -> Result<f64, PluginError> {
        let volume = self.volume_draft.parse::<f64>().map_err(|_| {
            PluginError::new(
                PluginErrorCode::InvalidPayload,
                "재생 시작 음량을 입력하세요",
            )
        })?;
        PlaybackControl::SetVolume {
            volume_percent: volume,
        }
        .validate()
        .map_err(|message| PluginError::new(PluginErrorCode::InvalidPayload, message))?;
        Ok(volume)
    }
    pub(crate) fn observation(&self) -> Option<&EpisodePlaybackObservation> {
        self.observation.as_ref()
    }
    pub(crate) fn settled_observation(&self) -> Option<&EpisodePlaybackObservation> {
        if !self.pending.is_empty() {
            return None;
        }
        self.observation
            .as_ref()
            .filter(|actual| actual.playback.connected)
    }
    pub(crate) fn set_draft(&mut self, seek: bool, value: String) {
        if seek {
            self.seek_draft = value;
        } else {
            self.volume_draft = value;
        }
    }
    pub(crate) fn observe(&mut self, observation: EpisodePlaybackObservation) {
        if self.observation.as_ref().is_none_or(|previous| {
            previous.engine_instance_id != observation.engine_instance_id
                || previous.resolution_artifact_id != observation.resolution_artifact_id
        }) {
            self.seek_draft.clear();
        }
        self.observation = Some(observation);
        if self.pending.is_empty() {
            self.failure = None;
        }
        if self.range_draft.as_ref().is_some_and(|draft| {
            !self
                .range_context()
                .is_ok_and(|context| context == draft.context)
        }) {
            self.range_draft = None;
        }
        self.diagnostic = None;
    }
    pub(crate) fn unavailable(&mut self, diagnostic: String) {
        self.range_draft = None;
        self.seek_draft.clear();
        self.observation = None;
        self.diagnostic = Some(diagnostic);
    }
    pub(crate) fn control(
        &self,
        control: PlaybackControl,
    ) -> Result<ControlEpisodePlayback, PluginError> {
        if !self.pending.is_empty() {
            return Err(PluginError::new(
                PluginErrorCode::Busy,
                "Original native control ticket still requires acceptance",
            ));
        }
        let observation = self
            .observation
            .as_ref()
            .filter(|o| o.playback.connected)
            .ok_or_else(|| {
                PluginError::new(
                    PluginErrorCode::MissingCapability,
                    "Original native session observation is unavailable",
                )
            })?;
        control
            .validate()
            .map_err(|message| PluginError::new(PluginErrorCode::InvalidPayload, message))?;
        Ok(ControlEpisodePlayback {
            engine_instance_id: observation.engine_instance_id,
            control,
        })
    }
    pub(crate) fn requested(&mut self, ticket: &PluginTicket) {
        if ticket.capability() == "playback.engine" || super::is_native_control(ticket.capability())
        {
            self.range_draft = None;
        }
        if ticket.capability() == "playback.engine" {
            self.observation = None;
        } else if super::is_native_control(ticket.capability())
            && let Some(observation) = &self.observation
        {
            self.pending
                .retain(|(old, _)| old.capability() != ticket.capability());
            self.pending
                .push((ticket.clone(), observation.engine_instance_id.0));
        }
    }
    pub(crate) fn receipt(
        &mut self,
        ticket: &PluginTicket,
        receipt: ControlReceipt,
    ) -> Result<(), String> {
        let expected = self
            .pending
            .iter()
            .find(|(pending, _)| pending == ticket)
            .map(|(_, id)| *id)
            .ok_or("Original native control ticket is missing")?;
        if receipt.engine_instance_id.0 != expected {
            return Err("Native control receipt belongs to a different engine instance".into());
        }
        self.pending.retain(|(pending, _)| pending != ticket);
        self.failure = None;
        self.diagnostic = Some(
            if self
                .observation
                .as_ref()
                .is_some_and(|o| o.engine_instance_id.0 != expected)
            {
                "이전 재생 대상에 보낸 명령이 접수되었습니다. 현재 재생 관측값은 유지합니다.".into()
            } else {
                "명령을 접수했습니다. 엔진 관측 갱신으로 적용 결과를 확인하세요.".into()
            },
        );
        // Queue admission is not completion. Refreshing playback.engine provides subsequent
        // position/paused/volume; a receipt never changes those observed values.
        Ok(())
    }
    pub(crate) fn retire_accepted_terminal(&mut self, ticket: &PluginTicket) {
        self.pending.retain(|(pending, _)| pending != ticket);
    }
    pub(crate) fn control_failed(&mut self, diagnostic: String, uncertain: bool) {
        let failure = self.failure.get_or_insert_with(|| FailureNotice {
            message: diagnostic.clone(),
            uncertain,
        });
        failure.uncertain |= uncertain;
        self.diagnostic = Some(diagnostic);
    }
    pub(crate) fn props(&self) -> BTreeMap<String, Value> {
        let observation = self.observation.as_ref();
        let playback = observation.map(|o| &o.playback);
        let present = observation.is_some();
        let connected = playback.is_some_and(|p| p.connected);
        let mut props = BTreeMap::from([
            (
                "nativeVolumeLabel".into(),
                Value::String(
                    playback
                        .filter(|value| value.connected)
                        .and_then(|value| value.volume_percent)
                        .map(|value| format!("{value}%"))
                        .unwrap_or_else(|| "미확인".into()),
                ),
            ),
            (
                "nativePositionLabel".into(),
                Value::String(crate::presentation::playback_time::display_time(
                    playback
                        .filter(|value| value.connected)
                        .and_then(|value| value.position_us)
                        .map(|value| value.0),
                )),
            ),
            (
                "nativeDurationLabel".into(),
                Value::String(crate::presentation::playback_time::display_time(
                    playback
                        .filter(|value| value.connected)
                        .and_then(|value| value.duration_us)
                        .map(|value| value.0),
                )),
            ),
            (
                "nativeSeekDraft".into(),
                Value::String(self.seek_draft.clone()),
            ),
            (
                "nativeVolumeDraft".into(),
                Value::String(self.volume_draft.clone()),
            ),
            (
                "nativeControlsDisabled".into(),
                Value::Bool(!connected || !self.pending.is_empty()),
            ),
            (
                "nativeInstanceId".into(),
                Value::String(
                    observation
                        .map(|o| o.engine_instance_id.0.to_string())
                        .unwrap_or_default(),
                ),
            ),
            (
                "nativeResolutionId".into(),
                Value::String(
                    observation
                        .map(|o| o.resolution_artifact_id.clone())
                        .unwrap_or_default(),
                ),
            ),
            (
                "nativePausedLabel".into(),
                Value::String(
                    match playback.and_then(|p| p.paused) {
                        Some(true) => "일시 정지",
                        Some(false) => "재생 중",
                        None => "재생 상태 관측 없음",
                    }
                    .into(),
                ),
            ),
            ("nativeObservationPresent".into(), Value::Bool(present)),
            (
                "nativePlaybackDiagnostic".into(),
                Value::String(self.diagnostic.clone().unwrap_or_default()),
            ),
        ]);
        self.range_props(&mut props);
        self.status_props(&mut props);
        for (key, value) in [
            (
                "nativePositionUs",
                playback
                    .and_then(|p| p.position_us)
                    .map(|v| v.0.to_string()),
            ),
            (
                "nativeDurationUs",
                playback
                    .and_then(|p| p.duration_us)
                    .map(|v| v.0.to_string()),
            ),
            (
                "nativeVolumePercent",
                playback
                    .and_then(|p| p.volume_percent)
                    .map(|v| v.to_string()),
            ),
        ] {
            props.insert(key.into(), Value::String(value.unwrap_or_default()));
        }
        props
    }
    fn range_props(&self, props: &mut BTreeMap<String, Value>) {
        use crate::presentation::{playback_range::*, playback_time::display_time};
        let observation = self.settled_observation();
        let duration = observation
            .and_then(|value| value.playback.duration_us)
            .map(|value| value.0);
        let position = observation
            .and_then(|value| value.playback.position_us)
            .map(|value| value.0);
        let volume = observation
            .and_then(|value| value.playback.volume_percent)
            .and_then(volume_coordinate);
        let draft = self.range_draft.as_ref().filter(|draft| {
            self.range_context()
                .is_ok_and(|context| context == draft.context)
        });
        let seek = draft
            .filter(|draft| draft.seek)
            .map(|draft| draft.value)
            .unwrap_or_else(|| seek_coordinate(position.unwrap_or(0), duration.unwrap_or(0)));
        let volume = draft
            .filter(|draft| !draft.seek)
            .map(|draft| draft.value)
            .or(volume)
            .unwrap_or(0);
        for (key, value) in [
            ("nativeSeekRangeValue", seek.to_string()),
            ("nativeVolumeRangeValue", volume.to_string()),
            (
                "nativeSeekPreviewLabel",
                display_time(
                    draft
                        .filter(|draft| draft.seek)
                        .and_then(|draft| seek_us(draft.value, duration.unwrap_or(0)).ok())
                        .or(position),
                ),
            ),
            (
                "nativeVolumePreviewLabel",
                if observation
                    .and_then(|value| value.playback.volume_percent)
                    .is_some()
                    || draft.is_some_and(|draft| !draft.seek)
                {
                    format!("{:.1}%", volume as f64 / 10.0)
                } else {
                    "미확인".into()
                },
            ),
        ] {
            props.insert(key.into(), Value::String(value));
        }
        props.insert(
            "nativeSeekRangeDisabled".into(),
            Value::Bool(duration.is_none_or(|value| value == 0) || observation.is_none()),
        );
        props.insert(
            "nativeVolumeRangeDisabled".into(),
            Value::Bool(observation.is_none()),
        );
    }
    fn status_props(&self, props: &mut BTreeMap<String, Value>) {
        let engine_error = self
            .observation
            .as_ref()
            .is_some_and(|actual| actual.playback.has_error);
        let summary = if let Some(failure) = &self.failure {
            failure.message.clone()
        } else if engine_error {
            "엔진이 재생 오류를 보고했습니다. 상태를 다시 확인하세요.".into()
        } else if !self.pending.is_empty() {
            "명령 처리 결과를 확인 중입니다. 같은 요청을 다시 보내지 마세요.".into()
        } else if self
            .observation
            .as_ref()
            .is_some_and(|actual| actual.playback.connected)
        {
            if self.diagnostic.is_some() {
                "명령을 접수했습니다. 재생 상태 갱신으로 적용 결과를 확인하세요.".into()
            } else {
                "원래 재생 대상의 상태를 확인했습니다.".into()
            }
        } else {
            "재생 상태가 미확인 또는 연결 해제 상태입니다.".into()
        };
        props.insert("nativeStatusSummary".into(), Value::String(summary));
        props.insert(
            "nativeStatusError".into(),
            Value::Bool(self.failure.is_some() || engine_error),
        );
        props.insert(
            "nativeStatusUncertain".into(),
            Value::Bool(
                self.failure
                    .as_ref()
                    .is_some_and(|failure| failure.uncertain),
            ),
        );
    }
}

pub(crate) fn typed_payload(input: &ControlEpisodePlayback) -> serde_json::Value {
    let mut payload = input.control.payload();
    payload["engine_instance_id"] = serde_json::json!(DecimalU64(input.engine_instance_id.0));
    payload
}
