use crate::{
    application::{
        api::{ApiError, ErrorCode},
        audio_handoff::intents::{AudioHandoffIntents, RequestedStartupState},
        playback_control::{ControlReceipt, PlaybackControlPort},
    },
    domain::{api_values::DecimalU64, playback_control::PlaybackControl},
    infrastructure::mpv::MpvControlHandle,
};
use std::sync::{Arc, RwLock};
type Binding = Arc<RwLock<Option<MpvControlHandle>>>;
pub struct NativeControlBinder {
    binding: Binding,
    intents: AudioHandoffIntents,
}
#[derive(Clone)]
pub struct NativeControlPort {
    binding: Binding,
    intents: AudioHandoffIntents,
}
/// Give the binder to the native lifecycle owner and the port to the shared SDK.
pub fn native_control_channel() -> (NativeControlBinder, NativeControlPort) {
    let binding = Arc::new(RwLock::new(None));
    let intents = AudioHandoffIntents::default();
    (
        NativeControlBinder {
            binding: binding.clone(),
            intents: intents.clone(),
        },
        NativeControlPort { binding, intents },
    )
}
impl NativeControlBinder {
    pub fn bind(&self, handle: MpvControlHandle) -> Result<(), ApiError> {
        let mut binding = self.binding.write().map_err(|_| unavailable())?;
        self.intents.bind_ready(DecimalU64(handle.instance_id()))?;
        *binding = Some(handle);
        Ok(())
    }
    pub fn bind_for_handoff(
        &self,
        handle: MpvControlHandle,
        requested: RequestedStartupState,
    ) -> Result<(), ApiError> {
        let mut binding = self.binding.write().map_err(|_| unavailable())?;
        self.intents
            .bind_for_handoff(DecimalU64(handle.instance_id()), requested)?;
        *binding = Some(handle);
        Ok(())
    }
    pub fn audio_intents(&self) -> AudioHandoffIntents {
        self.intents.clone()
    }
    pub fn clear(&self) -> Result<(), ApiError> {
        let mut binding = self.binding.write().map_err(|_| unavailable())?;
        self.intents.clear()?;
        *binding = None;
        Ok(())
    }
}
impl Drop for NativeControlBinder {
    fn drop(&mut self) {
        let _ = self.clear();
    }
}
impl PlaybackControlPort for NativeControlPort {
    fn audio_handoff_available(&self) -> bool {
        true
    }
    fn audio_handoff_status(&self) -> Result<serde_json::Value, ApiError> {
        self.intents.reported_handoff()
    }
    fn submit(&self, control: PlaybackControl) -> Result<ControlReceipt, ApiError> {
        let binding = self.binding.read().map_err(|_| unavailable())?;
        let handle = binding.as_ref().ok_or_else(|| {
            ApiError::new(
                ErrorCode::NotFound,
                "No native playback control instance bound",
            )
        })?;
        self.intents
            .submit(handle, DecimalU64(handle.instance_id()), control)
    }
}
fn unavailable() -> ApiError {
    ApiError::new(
        ErrorCode::Internal,
        "Native playback control binding unavailable",
    )
}
