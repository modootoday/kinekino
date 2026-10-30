#[path = "ports.rs"]
mod ports;
use crate::{
    application::api::{ApiError, ErrorCode, Request},
    domain::{api_values::DecimalU64, playback_control::PlaybackControl},
};
pub use ports::PlaybackControlPort;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
mod native_lifetime;
pub use native_lifetime::StoppedNativeEngine;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlAcceptance {
    Accepted,
    DeferredUntilHandoff,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlCompletion {
    Unobserved,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Accepted means native queue admission; deferred_until_handoff means coalesced
/// startup intent. Neither value proves native application or completion.
pub struct ControlReceipt {
    pub acceptance: ControlAcceptance,
    pub completion: ControlCompletion,
    pub engine_instance_id: DecimalU64,
}
impl ControlReceipt {
    pub fn deferred_until_handoff(engine_instance_id: u64) -> Self {
        Self {
            acceptance: ControlAcceptance::DeferredUntilHandoff,
            completion: ControlCompletion::Unobserved,
            engine_instance_id: DecimalU64(engine_instance_id),
        }
    }
    pub fn enqueued(engine_instance_id: u64) -> Self {
        Self {
            acceptance: ControlAcceptance::Accepted,
            completion: ControlCompletion::Unobserved,
            engine_instance_id: DecimalU64(engine_instance_id),
        }
    }
}
pub struct UnavailablePlaybackControl;
impl PlaybackControlPort for UnavailablePlaybackControl {
    fn available(&self) -> bool {
        false
    }
    fn submit(&self, _control: PlaybackControl) -> Result<ControlReceipt, ApiError> {
        Err(ApiError::new(
            ErrorCode::Unsupported,
            "Native playback control adapter unavailable",
        ))
    }
}
pub fn is_command(command: &str) -> bool {
    matches!(
        command,
        "playback.play"
            | "playback.pause"
            | "playback.seek"
            | "playback.set_volume"
            | "playback.set_muted"
    )
}
pub fn parse(command: &str, payload: &Value) -> Result<PlaybackControl, ApiError> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Empty {}
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Seek {
        position_us: DecimalU64,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Volume {
        volume_percent: f64,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Muted {
        muted: bool,
    }
    let invalid = || {
        ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid native playback control payload",
        )
    };
    let control = match command {
        "playback.play" | "playback.pause" => {
            let _: Empty = serde_json::from_value(payload.clone()).map_err(|_| invalid())?;
            if command == "playback.play" {
                PlaybackControl::Play
            } else {
                PlaybackControl::Pause
            }
        }
        "playback.seek" => {
            let p: Seek = serde_json::from_value(payload.clone()).map_err(|_| invalid())?;
            PlaybackControl::Seek {
                position_us: p.position_us,
            }
        }
        "playback.set_volume" => {
            let p: Volume = serde_json::from_value(payload.clone()).map_err(|_| invalid())?;
            PlaybackControl::SetVolume {
                volume_percent: p.volume_percent,
            }
        }
        "playback.set_muted" => {
            let p: Muted = serde_json::from_value(payload.clone()).map_err(|_| invalid())?;
            PlaybackControl::SetMuted { muted: p.muted }
        }
        _ => return Err(invalid()),
    };
    control
        .validate()
        .map_err(|m| ApiError::new(ErrorCode::InvalidArgument, m))?;
    Ok(control)
}
pub fn execute(request: &Request, port: &impl PlaybackControlPort) -> Result<Value, ApiError> {
    let control = parse(&request.command, &request.payload)?;
    if request.context.expected_revision.is_some()
        || request.context.idempotency_key.is_some()
        || request.context.timeout_ms.is_some()
    {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Native control revision, idempotency and completion timeout are unavailable",
        ));
    }
    serde_json::to_value(port.submit(control)?)
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Invalid control receipt"))
}
pub fn payload_schema(command: &str) -> Value {
    match command {
        "playback.seek" => {
            json!({"type":"object","additionalProperties":false,"required":["position_us"],"properties":{"position_us":{"type":"string","pattern":"^[0-9]+$","description":"Absolute engine playback microseconds; maximum 9007199254740991, validated by application; native seek accuracy is not guaranteed"}}})
        }
        "playback.set_volume" => {
            json!({"type":"object","additionalProperties":false,"required":["volume_percent"],"properties":{"volume_percent":{"type":"number","minimum":0,"maximum":100}}})
        }
        "playback.set_muted" => {
            json!({"type":"object","additionalProperties":false,"required":["muted"],"properties":{"muted":{"type":"boolean"}}})
        }
        _ => json!({"type":"object","maxProperties":0,"additionalProperties":false}),
    }
}
pub fn response_schema() -> Value {
    json!({"type":"object","additionalProperties":false,"required":["acceptance","completion","engine_instance_id"],"properties":{"acceptance":{"enum":["accepted","deferred_until_handoff"]},"completion":{"const":"unobserved"},"engine_instance_id":{"type":"string","pattern":"^[0-9]+$"}}})
}

#[cfg(test)]
mod mute_tests {
    use super::*;
    #[test]
    fn mute_payload_is_typed_strict_and_not_inferred_from_volume() {
        assert_eq!(
            parse("playback.set_muted", &json!({"muted":true})).unwrap(),
            PlaybackControl::SetMuted { muted: true }
        );
        assert_eq!(
            parse("playback.set_muted", &json!({"muted":false})).unwrap(),
            PlaybackControl::SetMuted { muted: false }
        );
        for payload in [
            json!({}),
            json!({"muted":null}),
            json!({"muted":"false"}),
            json!({"muted":0}),
            json!({"muted":false,"volume_percent":0}),
            json!({"muted":false,"command":"unsafe"}),
        ] {
            assert_eq!(
                parse("playback.set_muted", &payload).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
        }
    }
}
