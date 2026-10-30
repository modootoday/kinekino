//! Presentation values only. Missing observations never acquire playback defaults.
use meari::domain::Value;
use std::collections::BTreeMap;
pub fn native_default_props() -> BTreeMap<String, Value> {
    let mut props = BTreeMap::new();
    for key in [
        "nativePositionLabel",
        "nativeDurationLabel",
        "nativeVolumeLabel",
        "nativeSeekPreviewLabel",
        "nativeVolumePreviewLabel",
    ] {
        props.insert(key.into(), Value::String("미확인".into()));
    }
    for key in [
        "nativeInstanceId",
        "nativeResolutionId",
        "nativePlaybackDiagnostic",
        "nativePositionUs",
        "nativeDurationUs",
        "nativeVolumePercent",
        "nativeSeekDraft",
        "nativeVolumeDraft",
        "nativeCurrentTitle",
        "nativeCurrentEpisode",
        "nativeCurrentCaption",
    ] {
        props.insert(key.into(), Value::String(String::new()));
    }
    props.insert(
        "nativePausedLabel".into(),
        Value::String("재생 상태 관측 없음".into()),
    );
    props.insert("nativeControlsDisabled".into(), Value::Bool(true));
    props.insert(
        "nativeStatusSummary".into(),
        Value::String("재생 상태가 미확인 또는 연결 해제 상태입니다.".into()),
    );
    props.insert("nativeStatusError".into(), Value::Bool(false));
    props.insert("nativeStatusUncertain".into(), Value::Bool(false));
    for key in ["nativeSeekRangeValue", "nativeVolumeRangeValue"] {
        props.insert(key.into(), Value::String("0".into()));
    }
    for key in ["nativeSeekRangeDisabled", "nativeVolumeRangeDisabled"] {
        props.insert(key.into(), Value::Bool(true));
    }
    props.insert("nativeObservationPresent".into(), Value::Bool(false));
    props
}
/// Local UI labels are data; no SDK capability or source-supplied ticket is introduced.
pub fn local_action_props() -> BTreeMap<String, Value> {
    BTreeMap::from([
        (
            "retryAckLabel".into(),
            Value::String("수신 처리 재시도".into()),
        ),
        (
            "refreshProjectionLabel".into(),
            Value::String("화면 갱신".into()),
        ),
        (
            "retryAckLabelKey".into(),
            Value::String("viewer.read.retry_ack".into()),
        ),
        (
            "refreshProjectionLabelKey".into(),
            Value::String("viewer.read.refresh_projection".into()),
        ),
        ("capabilitiesRetryDisabled".into(), Value::Bool(true)),
        ("engineRetryDisabled".into(), Value::Bool(true)),
        ("statusRetryDisabled".into(), Value::Bool(true)),
    ])
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadPhase {
    NotRequested,
    Pending,
    Received,
    Failed,
    AckFailed,
}
impl ReadPhase {
    pub fn key(self) -> &'static str {
        match self {
            Self::NotRequested => "viewer.read.not_requested",
            Self::Pending => "viewer.read.pending",
            Self::Received => "viewer.read.received",
            Self::Failed => "viewer.read.failed",
            Self::AckFailed => "viewer.read.ack_failed",
        }
    }
}
#[derive(Clone, Debug)]
pub struct ReadRow {
    pub phase: ReadPhase,
    pub value: Option<String>,
    pub diagnostic: Option<String>,
    pub error_key: Option<&'static str>,
}
impl Default for ReadRow {
    fn default() -> Self {
        Self {
            phase: ReadPhase::NotRequested,
            value: None,
            diagnostic: None,
            error_key: None,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct ReadPanel {
    pub rows: [ReadRow; 3],
    pub projection_error: Option<String>,
}
impl ReadPanel {
    pub fn props(&self) -> BTreeMap<String, Value> {
        let mut props = BTreeMap::new();
        for (name, text) in [
            ("readPanelLabel", "플레이어 상태 조회"),
            ("capabilitiesActionLabel", "지원 기능 조회"),
            ("engineActionLabel", "엔진 조회"),
            ("statusActionLabel", "재생 상태 조회"),
        ] {
            props.insert(name.into(), Value::String(text.into()));
        }
        for (index, prefix) in ["capabilities", "engine", "status"].into_iter().enumerate() {
            let row = &self.rows[index];
            props.insert(
                format!("{prefix}StateCode"),
                Value::String(row.phase.key().into()),
            );
            let label = match row.phase {
                ReadPhase::NotRequested => "조회 전",
                ReadPhase::Pending => "조회 중",
                ReadPhase::Received => "수신 완료",
                ReadPhase::Failed => "조회 오류",
                ReadPhase::AckFailed => "수신 확인 재시도 필요",
            };
            props.insert(format!("{prefix}State"), Value::String(label.into()));
            props.insert(
                format!("{prefix}Value"),
                Value::String(row.value.clone().unwrap_or_else(|| "미관측".into())),
            );
            props.insert(
                format!("{prefix}Error"),
                Value::String(
                    match row.error_key {
                        Some("viewer.read.ack_failed") => "수신 확인을 완료하지 못했습니다.",
                        Some("viewer.read.sdk_failed") => "조회 요청이 실패했습니다.",
                        Some("viewer.read.decode_failed") => "조회 결과를 표시할 수 없습니다.",
                        Some("viewer.read.consumer_failed") => "조회 결과 처리가 보류되었습니다.",
                        _ => "",
                    }
                    .into(),
                ),
            );
            props.insert(
                format!("{prefix}ErrorCode"),
                Value::String(row.error_key.unwrap_or_default().into()),
            );
            props.insert(
                format!("{prefix}Busy"),
                Value::Bool(matches!(
                    row.phase,
                    ReadPhase::Pending | ReadPhase::AckFailed
                )),
            );
        }
        props
    }
}
#[cfg(test)]
#[path = "read_observation_tests.rs"]
mod tests;
