//! UI argument adapters only; common SDK retains control validation and instance policy.
use crate::{
    assembly::meari::viewer_read_projection::native_playback::{
        NativePlaybackProjection, typed_payload,
    },
    domain::{api_values::DecimalU64, playback_control::PlaybackControl},
};
use meari::{
    application::{host_callbacks::HostCallbacks, plugins::*},
    domain::host_callbacks::{CallbackArguments, CallbackSignature},
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub(super) const CONTROLS: [(&str, &str); 6] = [
    ("onNativePlay", "playback.play"),
    ("onNativePause", "playback.pause"),
    ("onNativeSeek", "playback.seek"),
    ("onNativeVolume", "playback.set_volume"),
    ("onNativeSeekCommit", "playback.seek"),
    ("onNativeVolumeCommit", "playback.set_volume"),
];
pub(super) fn bind(
    callbacks: &mut HostCallbacks,
    registration: RegistrationHandle,
    manifest: &PluginManifest,
    state: Rc<RefCell<NativePlaybackProjection>>,
    admission: Rc<Cell<bool>>,
) -> Result<(), PluginError> {
    for (prop, command) in CONTROLS {
        let signature = if matches!(command, "playback.play" | "playback.pause") {
            CallbackSignature::Action
        } else {
            CallbackSignature::Text
        };
        let schema = manifest.capabilities[command].request_schema.clone();
        let state = state.clone();
        let admission = admission.clone();
        callbacks.bind(
            prop.into(),
            signature,
            registration.clone(),
            command.into(),
            move |arguments| {
                if !admission.get() {
                    return Err(PluginError::new(
                        PluginErrorCode::MissingCapability,
                        "Refresh the original native engine observation before controlling it",
                    ));
                }
                let invalid = || {
                    PluginError::new(
                        PluginErrorCode::InvalidPayload,
                        "Invalid typed native control input",
                    )
                };
                if matches!(prop, "onNativeSeekCommit" | "onNativeVolumeCommit") {
                    let CallbackArguments::Text(value) = arguments else {
                        return Err(invalid());
                    };
                    let state = state.try_borrow().map_err(|_| {
                        PluginError::new(
                            PluginErrorCode::Busy,
                            "Native original-session projection borrowed",
                        )
                    })?;
                    let input = state.range_control(prop == "onNativeSeekCommit", value)?;
                    return PluginPayload::encode(schema.clone(), &typed_payload(&input));
                }
                let control = match (command, arguments) {
                    ("playback.play", CallbackArguments::Action) => PlaybackControl::Play,
                    ("playback.pause", CallbackArguments::Action) => PlaybackControl::Pause,
                    ("playback.seek", CallbackArguments::Text(value)) => PlaybackControl::Seek {
                        position_us: DecimalU64(
                            crate::presentation::playback_time::parse_seek_time(value)
                                .map_err(|_| invalid())?,
                        ),
                    },
                    ("playback.set_volume", CallbackArguments::Text(value)) => {
                        PlaybackControl::SetVolume {
                            volume_percent: value.parse().map_err(|_| invalid())?,
                        }
                    }
                    _ => return Err(invalid()),
                };
                let state = state.try_borrow().map_err(|_| {
                    PluginError::new(
                        PluginErrorCode::Busy,
                        "Native original-session projection borrowed",
                    )
                })?;
                let input = state.control(control)?;
                PluginPayload::encode(schema.clone(), &typed_payload(&input))
            },
        )?;
    }
    Ok(())
}
