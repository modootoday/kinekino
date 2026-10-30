//! Shared display and one pending window intent; original route callback owners validate inputs.
#[cfg(test)]
#[path = "window_caption/contract_fixture.rs"]
pub(crate) mod contract_fixture;
use meari::{
    application::{
        host_callbacks::{LocalActionInvocation, LocalActionKey},
        plugins::{PluginError, PluginErrorCode},
    },
    domain::{
        Value,
        host_callbacks::{CallbackArguments, CallbackReference, CallbackSignature},
    },
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CaptionIntent {
    Minimize,
    ToggleMaximize,
    UserClose,
}
pub(crate) const BINDINGS: [(&str, CaptionIntent); 3] = [
    ("onWindowMinimize", CaptionIntent::Minimize),
    ("onWindowToggleMaximize", CaptionIntent::ToggleMaximize),
    ("onWindowUserClose", CaptionIntent::UserClose),
];
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CaptionView {
    pub minimize_disabled: bool,
    pub maximize_disabled: bool,
    pub close_disabled: bool,
    pub maximize_label: String,
    pub maximize_icon: String,
    pub status: String,
    pub error: bool,
}
impl Default for CaptionView {
    fn default() -> Self {
        Self {
            minimize_disabled: true,
            maximize_disabled: true,
            close_disabled: true,
            maximize_label: "최대화 상태 미확인".into(),
            maximize_icon: "aw-icon aw-icon-maximize".into(),
            status: String::new(),
            error: false,
        }
    }
}
impl CaptionView {
    pub(crate) fn props(&self) -> BTreeMap<String, Value> {
        let mut props = BTreeMap::from([
            (
                "captionMinimizeDisabled".into(),
                Value::Bool(self.minimize_disabled),
            ),
            (
                "captionMaximizeDisabled".into(),
                Value::Bool(self.maximize_disabled),
            ),
            (
                "captionCloseDisabled".into(),
                Value::Bool(self.close_disabled),
            ),
            (
                "captionMaximizeLabel".into(),
                Value::String(self.maximize_label.clone()),
            ),
            (
                "captionMaximizeIconClass".into(),
                Value::String(self.maximize_icon.clone()),
            ),
            ("captionStatus".into(), Value::String(self.status.clone())),
            ("captionStatusError".into(), Value::Bool(self.error)),
        ]);
        for (name, _) in BINDINGS {
            props.insert(
                name.into(),
                Value::CallbackReference(CallbackReference {
                    binding: name.into(),
                    signature: CallbackSignature::Action,
                }),
            );
        }
        props
    }
}
#[derive(Default)]
pub(crate) struct CaptionState {
    pub view: CaptionView,
    pending: Option<CaptionIntent>,
    closed: bool,
}
impl CaptionState {
    pub(crate) fn request(&mut self, intent: CaptionIntent) -> Result<(), PluginError> {
        if self.closed {
            return Err(PluginError::new(
                PluginErrorCode::Closing,
                "Original window caption owner closed",
            ));
        }
        if self.pending.is_some() {
            return Err(PluginError::new(
                PluginErrorCode::Busy,
                "Original window intent pending",
            ));
        }
        self.pending = Some(intent);
        Ok(())
    }
    pub(crate) fn take(&mut self) -> Option<CaptionIntent> {
        self.pending.take()
    }
    pub(crate) fn close(&mut self) {
        self.closed = true;
        self.pending = None;
        self.view = CaptionView::default();
    }
}
/// Only accepts an invocation produced by the caller's original HostCallbacks dispatcher.
pub(crate) fn decode(
    invocation: &LocalActionInvocation,
    key: impl Fn(&str) -> Option<LocalActionKey>,
) -> Result<Option<CaptionIntent>, PluginError> {
    for (name, intent) in BINDINGS {
        if key(name)
            .as_ref()
            .is_some_and(|key| invocation.matches_key(key))
        {
            if !matches!(invocation.arguments(), CallbackArguments::Action) {
                return Err(PluginError::new(
                    PluginErrorCode::InvalidPayload,
                    "Window intent requires Action",
                ));
            }
            return Ok(Some(intent));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_observation_keeps_all_caption_controls_disabled() {
        let props = CaptionView::default().props();
        for name in [
            "captionMinimizeDisabled",
            "captionMaximizeDisabled",
            "captionCloseDisabled",
        ] {
            assert_eq!(props.get(name), Some(&Value::Bool(true)));
        }
        for (name, _) in BINDINGS {
            assert_eq!(
                props.get(name),
                Some(&Value::CallbackReference(CallbackReference {
                    binding: name.into(),
                    signature: CallbackSignature::Action,
                }))
            );
        }
    }

    #[test]
    fn caption_intents_remain_bounded_and_close_retires_pending_intent() {
        let mut state = CaptionState::default();
        assert!(state.request(CaptionIntent::Minimize).is_ok());
        assert!(state.request(CaptionIntent::ToggleMaximize).is_err());
        assert_eq!(state.take(), Some(CaptionIntent::Minimize));
        assert!(state.request(CaptionIntent::UserClose).is_ok());
        state.close();
        assert_eq!(state.take(), None);
        assert!(state.request(CaptionIntent::UserClose).is_err());
        assert_eq!(state.view, CaptionView::default());
    }
}
