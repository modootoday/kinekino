//! Registration requests one real completion redraw; this is not a frame/Ready receipt.
use meari_native::infrastructure::native_host::{HostStamp, HostTurn, HostTurnReason};
use std::cell::{Cell, RefCell};

#[derive(Default)]
pub(crate) struct RegistrationRedraw {
    original: RefCell<Option<HostStamp>>,
    queued: Cell<bool>,
    requested: Cell<bool>,
    cancelled: Cell<bool>,
}
impl RegistrationRedraw {
    /// Called only after the original retirement ticket completes and the same window commits Viewing.
    pub(crate) fn rearm_for_viewing(&self, stamp: &HostStamp) -> Result<(), super::FrameHookError> {
        let mut original = self
            .original
            .try_borrow_mut()
            .map_err(|_| super::FrameHookError("Original registration redraw borrowed"))?;
        let previous = original.as_ref().ok_or(super::FrameHookError(
            "Original registration redraw missing",
        ))?;
        if previous.host_id != stamp.host_id
            || previous.owner != stamp.owner
            || previous.definition_revision != stamp.definition_revision
            || !self.cancelled.get()
        {
            return Err(super::FrameHookError(
                "Original registration redraw scope or retirement differs",
            ));
        }
        *original = Some(stamp.clone());
        self.queued.set(false);
        self.requested.set(false);
        self.cancelled.set(false);
        Ok(())
    }
    pub(crate) fn queue(&self, stamp: &HostStamp) -> Result<(), super::FrameHookError> {
        if self.queued.get() || self.cancelled.get() {
            return Ok(());
        }
        *self
            .original
            .try_borrow_mut()
            .map_err(|_| super::FrameHookError("Original registration redraw borrowed"))? =
            Some(stamp.clone());
        self.queued.set(true);
        Ok(())
    }
    pub(crate) fn cancel(&self) {
        self.cancelled.set(true);
    }
    pub(crate) fn take_for(&self, turn: &HostTurn, open: bool) -> bool {
        if !open {
            self.cancel();
        }
        if self.cancelled.get() || self.requested.get() || turn.reason != HostTurnReason::Completion
        {
            return false;
        }
        let Ok(original) = self.original.try_borrow() else {
            return false;
        };
        let Some(original) = original.as_ref() else {
            return false;
        };
        if original.host_id != turn.stamp.host_id
            || original.owner != turn.stamp.owner
            || original.definition_revision != turn.stamp.definition_revision
        {
            self.cancel();
            return false;
        }
        self.requested.set(true);
        true
    }
    pub(crate) fn snapshot(&self) -> (bool, bool, bool) {
        (
            self.queued.get(),
            self.requested.get(),
            self.cancelled.get(),
        )
    }
}

#[derive(Clone, Default, serde::Serialize)]
pub(crate) struct RoutedFrameTrace {
    pub last_checkpoint: Option<CheckpointTrace>,
    pub checkpoint_read_error: Option<BoundedError>,
    pub bound: bool,
    pub frame_callbacks: u64,
    pub registration_attempted: bool,
    pub registered: bool,
    pub activation_attempted: bool,
    pub activated: bool,
    pub capture_attempted: bool,
    pub ready_captured: bool,
    pub redraw_queued: bool,
    pub redraw_requested: bool,
    pub redraw_cancelled: bool,
    pub last_scene_generation: Option<u64>,
    pub last_context_generation: Option<u64>,
    pub last_context_id: Option<String>,
    pub frame_identity_truncated: bool,
    pub first_functional_error: Option<String>,
    pub revocation_error: Option<String>,
    pub error_text_truncated: bool,
}
#[derive(Clone, serde::Serialize)]
pub(crate) struct CheckpointTrace {
    pub surface: &'static str,
    pub original_engine_retained: bool,
    pub retained_route_generation: Option<u64>,
    pub admission_revoked: bool,
    pub state_revoked: bool,
    pub cleanup_retained: bool,
    pub first_error: Option<BoundedError>,
    pub revocation_error: Option<BoundedError>,
}
#[derive(Clone, serde::Serialize)]
pub(crate) struct BoundedError {
    pub message: String,
    pub truncated: bool,
}
pub(crate) fn bounded_error(error: &impl std::fmt::Display) -> BoundedError {
    struct Text {
        value: BoundedError,
        count: usize,
    }
    impl std::fmt::Write for Text {
        fn write_str(&mut self, value: &str) -> std::fmt::Result {
            for character in value.chars() {
                if self.count < 128 {
                    self.value.message.push(character);
                    self.count += 1;
                } else {
                    self.value.truncated = true;
                }
            }
            Ok(())
        }
    }
    let mut text = Text {
        value: BoundedError {
            message: String::new(),
            truncated: false,
        },
        count: 0,
    };
    let _ = std::fmt::write(&mut text, format_args!("{error}"));
    text.value
}
impl RoutedFrameTrace {
    pub(crate) fn error(&mut self, first: Option<&str>, revocation: Option<&str>) {
        fn bounded(value: Option<&str>, truncated: &mut bool) -> Option<String> {
            value.map(|value| {
                *truncated |= value.chars().count() > 512;
                value.chars().take(512).collect()
            })
        }
        self.first_functional_error = bounded(first, &mut self.error_text_truncated);
        self.revocation_error = bounded(revocation, &mut self.error_text_truncated);
    }
}

#[cfg(test)]
mod route_rearm_tests {
    use super::*;
    fn stamp() -> HostStamp {
        HostStamp {
            host_id: 1,
            owner: serde_json::from_value(serde_json::json!({
                "instance": {"document":"route-redraw","epoch":1,"id":1}, "source_generation":1
            }))
            .unwrap(),
            definition_revision: 1,
            document_revision: 1,
            turn_sequence: 1,
        }
    }
    #[test]
    fn completed_route_registration_rearms_only_same_owner_after_explicit_cancel() {
        let redraw = RegistrationRedraw::default();
        let first = stamp();
        redraw.queue(&first).unwrap();
        assert!(redraw.rearm_for_viewing(&first).is_err());
        redraw.cancel();
        let mut foreign = first.clone();
        foreign.definition_revision += 1;
        assert!(redraw.rearm_for_viewing(&foreign).is_err());
        assert_eq!(redraw.snapshot(), (true, false, true));
        let mut current = first;
        current.document_revision += 1;
        current.turn_sequence += 1;
        redraw.rearm_for_viewing(&current).unwrap();
        assert_eq!(redraw.snapshot(), (false, false, false));
        redraw.queue(&current).unwrap();
        assert_eq!(redraw.snapshot(), (true, false, false));
    }
}
