#[cfg(test)]
use crate::application::api::PlaybackClient;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_control::ControlReceipt,
    },
    domain::playback_control::PlaybackControl,
};
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use tokio::sync::mpsc::{Sender, error::TrySendError};
mod options;
pub const COMMAND_CAPACITY: usize = 64;
static NEXT_INSTANCE: AtomicU64 = AtomicU64::new(1);
pub(super) fn guarded_deadline(now: std::time::Instant) -> Result<std::time::Instant, ApiError> {
    now.checked_add(options::GUARDED_OPERATION_TIMEOUT)
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::ResourceExhausted,
                "Native operation clock exhausted",
            )
        })
}
#[derive(Default)]
pub(crate) struct ConnectionState {
    pub(super) guarded: super::guarded_command::GuardedQueue,
    pub(super) native_observation: std::sync::Mutex<super::native_observation::ReceiveState>,
    pub live: AtomicBool,
    pub connected: AtomicBool,
    pub(super) options: std::sync::Mutex<super::options_state::OptionsState>,
}
impl ConnectionState {
    pub fn close(&self) {
        let _ = self.guarded.close();
        self.connected.store(false, Ordering::Release);
        self.live.store(false, Ordering::Release);
    }
}
#[derive(Clone)]
pub struct MpvControlHandle {
    sender: Sender<Value>,
    pub(crate) state: Arc<ConnectionState>,
    instance_id: u64,
}
impl crate::application::playback_control::PlaybackControlPort for MpvControlHandle {
    fn submit(&self, control: PlaybackControl) -> Result<ControlReceipt, ApiError> {
        MpvControlHandle::submit(self, control)
    }
}
impl MpvControlHandle {
    pub(crate) fn native_receive_observation(
        &self,
    ) -> Result<crate::application::playback_session::NativeReceiveObservation, ApiError> {
        let state = self
            .state
            .native_observation
            .try_lock()
            .map_err(|error| match error {
                std::sync::TryLockError::WouldBlock => {
                    let mut error =
                        ApiError::new(ErrorCode::NeedsInput, "Native receive observations busy");
                    error.retryable = true;
                    error
                }
                std::sync::TryLockError::Poisoned(_) => {
                    ApiError::new(ErrorCode::Internal, "Native receive observations poisoned")
                }
            })?;
        let mut report = state.report(std::time::Instant::now());
        if !self.state.live.load(Ordering::Acquire) || !self.state.connected.load(Ordering::Acquire)
        {
            report.connected = false;
            report.position_seconds.value = None;
            report.duration_seconds.value = None;
            report.paused.value = None;
            report.idle.value = None;
            report.seekable.value = None;
            report.partially_seekable.value = None;
        }
        Ok(report)
    }
    pub fn instance_id(&self) -> u64 {
        self.instance_id
    }
    pub(crate) fn new(sender: Sender<Value>) -> anyhow::Result<Self> {
        Self::new_with_scope(sender, super::guarded_command::HostScope::disabled(), true)
    }
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Scoped owner startup is assigned to C B6")
    )]
    pub(super) fn new_scoped(
        sender: Sender<Value>,
        scope: Arc<super::guarded_command::HostScope>,
    ) -> anyhow::Result<Self> {
        Self::new_with_scope(sender, scope, false)
    }
    fn new_with_scope(
        sender: Sender<Value>,
        scope: Arc<super::guarded_command::HostScope>,
        standalone: bool,
    ) -> anyhow::Result<Self> {
        let instance_id = NEXT_INSTANCE
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| anyhow::anyhow!("Native instance identifier exhausted"))?;
        let guarded = if standalone {
            super::guarded_command::GuardedQueue::standalone(scope, instance_id)
        } else {
            super::guarded_command::GuardedQueue::scoped(scope, instance_id)
        }
        .map_err(|error| anyhow::anyhow!("{error:?}"))?;
        Ok(Self {
            sender,
            state: Arc::new(ConnectionState {
                live: AtomicBool::new(true),
                connected: AtomicBool::new(false),
                options: std::sync::Mutex::new(Default::default()),
                native_observation: std::sync::Mutex::new(Default::default()),
                guarded,
            }),
            instance_id,
        })
    }
    pub fn submit(&self, control: PlaybackControl) -> Result<ControlReceipt, ApiError> {
        control
            .validate()
            .map_err(|m| ApiError::new(ErrorCode::InvalidArgument, m))?;
        self.enqueue(wire(&control))
    }
    fn enqueue(&self, command: Value) -> Result<ControlReceipt, ApiError> {
        if !self.state.live.load(Ordering::Acquire) {
            return Err(ApiError::new(
                ErrorCode::NotFound,
                "Native playback instance ended",
            ));
        }
        if self.sender.is_closed() {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Native playback command connection closed",
            ));
        }
        if !self.state.connected.load(Ordering::Acquire) {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Native playback control connection is not ready",
            ));
        }
        self.sender.try_send(command).map_err(queue_error)?;
        Ok(ControlReceipt::enqueued(self.instance_id))
    }
    pub(crate) fn send_legacy(&self, value: Value) -> anyhow::Result<()> {
        self.sender
            .try_send(value)
            .map_err(|_| anyhow::anyhow!("mpv command queue unavailable or full"))
    }
}
pub(crate) fn wire(control: &PlaybackControl) -> Value {
    match control {
        PlaybackControl::Play => json!(["set_property", "pause", false]),
        PlaybackControl::Pause => json!(["set_property", "pause", true]),
        PlaybackControl::Seek { position_us } => {
            json!(["seek", position_us.0 as f64 / 1_000_000., "absolute+exact"])
        }
        PlaybackControl::SetVolume { volume_percent } => {
            json!(["set_property", "volume", volume_percent])
        }
        PlaybackControl::SetMuted { muted } => json!(["set_property", "mute", muted]),
    }
}

#[cfg(test)]
mod mute_wire_tests {
    use super::*;
    #[test]
    fn typed_mute_and_volume_are_independent_fixed_native_properties() {
        assert_eq!(
            wire(&PlaybackControl::SetMuted { muted: true }),
            json!(["set_property", "mute", true])
        );
        assert_eq!(
            wire(&PlaybackControl::SetMuted { muted: false }),
            json!(["set_property", "mute", false])
        );
        assert_eq!(
            wire(&PlaybackControl::SetVolume { volume_percent: 0. }),
            json!(["set_property", "volume", 0.])
        );
    }
}
fn queue_error(error: TrySendError<Value>) -> ApiError {
    match error {
        TrySendError::Full(_) => ApiError::new(
            ErrorCode::ResourceExhausted,
            "Native command queue is full; request was not accepted",
        ),
        TrySendError::Closed(_) => ApiError::new(
            ErrorCode::EngineFailed,
            "Native command connection closed; request was not accepted",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        application::{
            api::{PlayerSdk, RequestContext},
            playback_control::{ControlCompletion, PlaybackControlPort},
        },
        domain::api_values::DecimalU64,
        infrastructure::native_playback_control::native_control_channel,
    };
    fn fixture() -> (MpvControlHandle, tokio::sync::mpsc::Receiver<Value>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(COMMAND_CAPACITY);
        (MpvControlHandle::new(sender).unwrap(), receiver)
    }
    #[test]
    fn native_adapter_sdk_maps_typed_actions_and_only_acknowledges_enqueue() {
        let (binder, port) = native_control_channel();
        let (handle, mut receiver) = fixture();
        binder.bind(handle.clone()).unwrap();
        let sdk = PlayerSdk::default().with_playback_control(port);
        assert_eq!(
            sdk.playback_control(RequestContext::default(), PlaybackControl::Play)
                .unwrap_err()
                .code,
            ErrorCode::NeedsInput
        );
        assert!(receiver.try_recv().is_err());
        handle.state.connected.store(true, Ordering::Release);
        for (action, expected) in [
            (
                PlaybackControl::Play,
                json!(["set_property", "pause", false]),
            ),
            (
                PlaybackControl::Pause,
                json!(["set_property", "pause", true]),
            ),
            (
                PlaybackControl::Seek {
                    position_us: DecimalU64(500_000),
                },
                json!(["seek", 0.5, "absolute+exact"]),
            ),
            (
                PlaybackControl::SetVolume {
                    volume_percent: 37.5,
                },
                json!(["set_property", "volume", 37.5]),
            ),
        ] {
            let receipt = sdk
                .playback_control(RequestContext::default(), action)
                .unwrap();
            assert_eq!(receipt.engine_instance_id.0, handle.instance_id);
            assert_eq!(receipt.completion, ControlCompletion::Unobserved);
            assert_eq!(receiver.try_recv().unwrap(), expected);
        }
    }
    #[test]
    fn queue_pressure_closed_connection_and_ended_instance_never_accept() {
        let (handle, mut receiver) = fixture();
        handle.state.connected.store(true, Ordering::Release);
        for _ in 0..COMMAND_CAPACITY {
            handle.submit(PlaybackControl::Play).unwrap();
        }
        assert_eq!(
            handle.submit(PlaybackControl::Pause).unwrap_err().code,
            ErrorCode::ResourceExhausted
        );
        for _ in 0..COMMAND_CAPACITY {
            assert_eq!(
                receiver.try_recv().unwrap(),
                json!(["set_property", "pause", false])
            );
        }
        assert!(receiver.try_recv().is_err());
        drop(receiver);
        assert_eq!(
            handle.submit(PlaybackControl::Play).unwrap_err().code,
            ErrorCode::EngineFailed
        );
        handle.state.close();
        assert_eq!(
            handle.submit(PlaybackControl::Play).unwrap_err().code,
            ErrorCode::NotFound
        );
    }
    #[test]
    fn replacement_and_binder_drop_do_not_retarget_old_handles() {
        let (binder, port) = native_control_channel();
        assert!(port.available());
        assert_eq!(
            port.submit(PlaybackControl::Play).unwrap_err().code,
            ErrorCode::NotFound
        );
        let (old, mut old_rx) = fixture();
        let (new, mut new_rx) = fixture();
        old.state.connected.store(true, Ordering::Release);
        new.state.connected.store(true, Ordering::Release);
        binder.bind(old.clone()).unwrap();
        let before = port.submit(PlaybackControl::Play).unwrap();
        binder.clear().unwrap();
        assert_eq!(
            port.submit(PlaybackControl::Play).unwrap_err().code,
            ErrorCode::NotFound
        );
        binder.bind(new).unwrap();
        let after = port.submit(PlaybackControl::Pause).unwrap();
        assert_ne!(before.engine_instance_id, after.engine_instance_id);
        old.submit(PlaybackControl::SetVolume { volume_percent: 0. })
            .unwrap();
        assert_eq!(
            old_rx.try_recv().unwrap(),
            json!(["set_property", "pause", false])
        );
        assert_eq!(
            old_rx.try_recv().unwrap(),
            json!(["set_property", "volume", 0.])
        );
        assert_eq!(
            new_rx.try_recv().unwrap(),
            json!(["set_property", "pause", true])
        );
        assert!(new_rx.try_recv().is_err());
        drop(binder);
        assert_eq!(
            port.submit(PlaybackControl::Play).unwrap_err().code,
            ErrorCode::NotFound
        );
    }

    #[test]
    fn concurrent_rebinding_receipts_always_match_the_receiving_instance() {
        let (binder, port) = native_control_channel();
        let (old, mut old_rx) = fixture();
        let (new, mut new_rx) = fixture();
        old.state.connected.store(true, Ordering::Release);
        new.state.connected.store(true, Ordering::Release);
        let old_id = old.instance_id;
        let new_id = new.instance_id;
        binder.bind(old.clone()).unwrap();
        let worker = std::thread::spawn(move || {
            for _ in 0..200 {
                binder.bind(new.clone()).unwrap();
                binder.clear().unwrap();
                binder.bind(old.clone()).unwrap();
            }
        });
        let mut accepted_old = 0;
        let mut accepted_new = 0;
        for _ in 0..200 {
            match port.submit(PlaybackControl::Play) {
                Ok(r) if r.engine_instance_id.0 == old_id => accepted_old += 1,
                Ok(r) if r.engine_instance_id.0 == new_id => accepted_new += 1,
                Ok(_) => panic!("unknown instance receipt"),
                Err(e) => assert!(matches!(
                    e.code,
                    ErrorCode::NotFound | ErrorCode::ResourceExhausted
                )),
            }
        }
        worker.join().unwrap();
        let mut queued_old = 0;
        let mut queued_new = 0;
        while old_rx.try_recv().is_ok() {
            queued_old += 1;
        }
        while new_rx.try_recv().is_ok() {
            queued_new += 1;
        }
        assert_eq!((accepted_old, accepted_new), (queued_old, queued_new));
    }

    #[test]
    fn accepted_control_does_not_fabricate_a_status_observation() {
        use crate::application::{
            native_playback::NativePlayback, playback_status::native_status_channel,
        };
        let (binder, port) = native_control_channel();
        let (handle, _receiver) = fixture();
        handle.state.connected.store(true, Ordering::Release);
        binder.bind(handle).unwrap();
        let (publisher, reader) = native_status_channel();
        publisher
            .publish(&NativePlayback {
                connected: true,
                paused: true,
                pause_observed: true,
                time: Some(2.),
                ..Default::default()
            })
            .unwrap();
        let sdk = PlayerSdk::default()
            .with_playback_control(port)
            .with_playback_status(reader);
        let receipt = sdk
            .playback_control(RequestContext::default(), PlaybackControl::Play)
            .unwrap();
        assert_eq!(receipt.completion, ControlCompletion::Unobserved);
        let status = sdk.playback_status(RequestContext::default()).unwrap();
        assert_eq!(status.paused, Some(true));
        assert_eq!(status.position_us, Some(DecimalU64(2_000_000)));
    }
}
