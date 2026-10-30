use super::*;
use meari::application::host_callbacks::{HostCallbacks, LocalActionKey};
use meari::domain::host_callbacks::{CallbackArguments, CallbackSignature};
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone)]
pub(super) enum Command {
    Front(String),
    Back(String),
    Save,
    Accept,
    Retry,
    Consume,
    Discard,
    Revert,
    ReturnViewing,
}
pub(super) struct Inputs {
    caption_source: Option<
        std::rc::Weak<
            RefCell<
                crate::presentation::web_ui_adapter::player_shell::window_caption::CaptionState,
            >,
        >,
    >,
    callbacks: HostCallbacks,
    keys: BTreeMap<String, LocalActionKey>,
    pub(super) queue: VecDeque<Command>,
    barrier: bool,
    stopped: bool,
    revoked: bool,
}
impl Inputs {
    pub(super) fn new(
        scope: meari::application::plugins::PluginScope,
    ) -> Result<Self, ViewerHostError> {
        let mut callbacks = HostCallbacks::new(scope);
        let mut keys = BTreeMap::new();
        for (name, signature) in Self::signatures() {
            keys.insert(
                name.into(),
                callbacks
                    .bind_local(name.into(), signature)
                    .map_err(plugin)?,
            );
        }
        Ok(Self {
            caption_source: None,
            callbacks,
            keys,
            queue: VecDeque::new(),
            barrier: false,
            stopped: false,
            revoked: false,
        })
    }
    fn signatures() -> Vec<(&'static str, CallbackSignature)> {
        Self::editor_signatures()
            .into_iter()
            .chain(
                crate::presentation::web_ui_adapter::player_shell::window_caption::BINDINGS
                    .into_iter()
                    .map(|(name, _)| (name, CallbackSignature::Action)),
            )
            .collect()
    }
    fn editor_signatures() -> Vec<(&'static str, CallbackSignature)> {
        use CallbackSignature::{Action, Text};
        vec![
            ("onFrontChange", Text),
            ("onBackChange", Text),
            ("onSave", Action),
            ("onAccept", Action),
            ("onCaptureRetry", Action),
            ("onCaptureConsume", Action),
            ("onCaptureDiscard", Action),
            ("onCaptureRevert", Action),
            ("onCaptureReturnViewing", Action),
        ]
    }
    pub(super) fn bindings(
        &self,
    ) -> Result<meari::application::host_callbacks::CallbackBindings, ViewerHostError> {
        self.callbacks.export_bindings().map_err(plugin)
    }
    pub(super) fn props(&self) -> std::collections::BTreeMap<String, meari::domain::Value> {
        use meari::domain::{Value, host_callbacks::CallbackReference};
        Self::editor_signatures()
            .into_iter()
            .map(|(name, signature)| {
                (
                    name.into(),
                    Value::CallbackReference(CallbackReference {
                        binding: name.into(),
                        signature,
                    }),
                )
            })
            .collect()
    }
    pub(super) fn admit(
        &mut self,
        stamp: &HostStamp,
        pending: &PendingCallback,
        busy: bool,
    ) -> IntentAdmission<ViewerHostError> {
        if self.stopped {
            return IntentAdmission::Capacity;
        }
        let invocation = match self.callbacks.dispatch_local(
            &pending.handle,
            pending.invocation.arguments.clone(),
            &stamp.owner,
        ) {
            Ok(invocation) => invocation,
            Err(_) => return IntentAdmission::Rejected(ViewerHostError::ForeignCheckpoint),
        };
        let Some((name, _)) = self
            .keys
            .iter()
            .find(|(_, key)| invocation.matches_key(key))
        else {
            return IntentAdmission::Rejected(ViewerHostError::UnsupportedCallback);
        };
        match crate::presentation::web_ui_adapter::player_shell::window_caption::decode(
            &invocation,
            |name| self.keys.get(name).cloned(),
        ) {
            Ok(Some(intent)) => {
                let Some(source) = self
                    .caption_source
                    .as_ref()
                    .and_then(std::rc::Weak::upgrade)
                else {
                    return IntentAdmission::Rejected(ViewerHostError::UnsupportedCallback);
                };
                let Ok(mut source) = source.try_borrow_mut() else {
                    return IntentAdmission::Capacity;
                };
                return match source.request(intent) {
                    Ok(()) => IntentAdmission::Accepted,
                    Err(error)
                        if error.code == meari::application::plugins::PluginErrorCode::Busy =>
                    {
                        IntentAdmission::Capacity
                    }
                    Err(error) => IntentAdmission::Rejected(plugin(error)),
                };
            }
            Ok(None) => (),
            Err(error) => return IntentAdmission::Rejected(plugin(error)),
        }
        if self.barrier || self.queue.len() >= 8 {
            return IntentAdmission::Capacity;
        }
        let command = match (name.as_str(), invocation.arguments()) {
            ("onFrontChange", CallbackArguments::Text(text)) if !busy && text.len() <= 8192 => {
                Command::Front(text.clone())
            }
            ("onBackChange", CallbackArguments::Text(text)) if !busy && text.len() <= 8192 => {
                Command::Back(text.clone())
            }
            ("onSave", CallbackArguments::Action) if !busy => Command::Save,
            ("onAccept", CallbackArguments::Action) if !busy => Command::Accept,
            ("onCaptureRetry", CallbackArguments::Action) => Command::Retry,
            ("onCaptureConsume", CallbackArguments::Action) => Command::Consume,
            ("onCaptureDiscard", CallbackArguments::Action) => Command::Discard,
            ("onCaptureRevert", CallbackArguments::Action) if !busy => Command::Revert,
            ("onCaptureReturnViewing", CallbackArguments::Action) => Command::ReturnViewing,
            _ => return IntentAdmission::Rejected(ViewerHostError::UnsupportedCallback),
        };
        self.barrier = !matches!(command, Command::Front(_) | Command::Back(_));
        self.queue.push_back(command);
        IntentAdmission::Accepted
    }
    pub(super) fn drained(&mut self) {
        self.barrier = false;
    }
    pub(super) fn set_caption_source(
        &mut self,
        source: std::rc::Weak<
            RefCell<
                crate::presentation::web_ui_adapter::player_shell::window_caption::CaptionState,
            >,
        >,
    ) {
        self.caption_source = Some(source);
    }
    pub(super) fn stop(&mut self) -> Result<(), meari::application::plugins::PluginError> {
        self.stopped = true;
        if self.revoked {
            return Ok(());
        }
        self.callbacks.invalidate()?;
        self.revoked = true;
        Ok(())
    }
}
