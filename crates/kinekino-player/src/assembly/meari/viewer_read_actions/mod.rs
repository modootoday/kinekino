//! Rust-owned local keys and a bounded queue of original callbacks, never durable grants.
use meari::{
    application::{
        host_callbacks::{HostCallbacks, LocalActionInvocation, LocalActionKey, PendingCallback},
        plugins::*,
    },
    domain::host_callbacks::CallbackSignature,
};
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChooserIntent {
    List,
    Next,
    Document(String),
    Item(String),
}
pub(crate) struct ChooserActions {
    keys: Vec<LocalActionKey>,
}
impl ChooserActions {
    pub(crate) fn bind(callbacks: &mut HostCallbacks) -> Result<Self, PluginError> {
        let mut keys = Vec::new();
        for (prop, signature) in [
            ("onChooserList", CallbackSignature::Action),
            ("onChooserNext", CallbackSignature::Action),
            ("onChooserDocument", CallbackSignature::Text),
            ("onChooserItem", CallbackSignature::Text),
        ] {
            keys.push(callbacks.bind_local(prop.into(), signature)?);
        }
        Ok(Self { keys })
    }
    pub(crate) fn intent(&self, invocation: &LocalActionInvocation) -> Option<ChooserIntent> {
        use meari::domain::host_callbacks::CallbackArguments;
        let index = self
            .keys
            .iter()
            .position(|key| invocation.matches_key(key))?;
        match (index, invocation.arguments()) {
            (0, CallbackArguments::Action) => Some(ChooserIntent::List),
            (1, CallbackArguments::Action) => Some(ChooserIntent::Next),
            (2, CallbackArguments::Text(key)) if key.len() <= 96 => {
                Some(ChooserIntent::Document(key.clone()))
            }
            (3, CallbackArguments::Text(key)) if key.len() <= 96 => {
                Some(ChooserIntent::Item(key.clone()))
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadKind {
    Capabilities,
    Engine,
    Status,
}
impl ReadKind {
    pub fn command(self) -> &'static str {
        match self {
            Self::Capabilities => "capabilities",
            Self::Engine => "playback.engine",
            Self::Status => "playback.status",
        }
    }
    pub fn index(self) -> usize {
        match self {
            Self::Capabilities => 0,
            Self::Engine => 1,
            Self::Status => 2,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadIntent {
    RetryAck(ReadKind),
    RetryNativeAck,
    RetryCatalogAck,
    ReleaseCatalog,
    SetNativeSeekDraft,
    SetNativeVolumeDraft,
    SetNativeSeekRangeDraft,
    SetNativeVolumeRangeDraft,
    WindowMinimize,
    WindowToggleMaximize,
    WindowUserClose,
    ToggleFullscreen,
    ToggleEpisodes,
    ToggleDiagnostics,
    RefreshProjection,
}
#[derive(Default)]
pub struct LocalDrain {
    pub refreshed: bool,
    pub resumed: bool,
}
pub const ACTIONS: [(&str, ReadIntent); 7] = [
    (
        "onRetryCapabilitiesAck",
        ReadIntent::RetryAck(ReadKind::Capabilities),
    ),
    ("onRetryEngineAck", ReadIntent::RetryAck(ReadKind::Engine)),
    ("onRetryStatusAck", ReadIntent::RetryAck(ReadKind::Status)),
    ("onRefreshReadProjection", ReadIntent::RefreshProjection),
    ("onRetryNativeAck", ReadIntent::RetryNativeAck),
    ("onRetryCatalogAck", ReadIntent::RetryCatalogAck),
    ("onReleaseCatalog", ReadIntent::ReleaseCatalog),
];
#[derive(Clone, Debug)]
pub struct AckClaim {
    pub ticket: PluginTicket,
    pub generation: u64,
}
#[derive(Debug)]
pub struct QueuedReadAction {
    pub callback: PendingCallback,
    pub intent: ReadIntent,
    pub claim: Option<AckClaim>,
}
pub struct ReadActions {
    keys: Vec<(LocalActionKey, ReadIntent)>,
    queue: VecDeque<QueuedReadAction>,
}
fn busy() -> PluginError {
    PluginError::new(PluginErrorCode::Busy, "Local read queue is full")
}
impl ReadActions {
    pub fn bind(callbacks: &mut HostCallbacks) -> Result<Self, PluginError> {
        let mut keys = ACTIONS
            .into_iter()
            .map(|(prop, intent)| {
                callbacks
                    .bind_local(prop.into(), CallbackSignature::Action)
                    .map(|key| (key, intent))
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (prop, intent) in [
            ("onToggleFullscreen", ReadIntent::ToggleFullscreen),
            ("onToggleEpisodes", ReadIntent::ToggleEpisodes),
            ("onToggleDiagnostics", ReadIntent::ToggleDiagnostics),
            ("onWindowMinimize", ReadIntent::WindowMinimize),
            ("onWindowToggleMaximize", ReadIntent::WindowToggleMaximize),
            ("onWindowUserClose", ReadIntent::WindowUserClose),
        ] {
            keys.push((
                callbacks.bind_local(prop.into(), CallbackSignature::Action)?,
                intent,
            ));
        }
        for (prop, intent) in [
            ("onNativeSeekDraft", ReadIntent::SetNativeSeekDraft),
            ("onNativeVolumeDraft", ReadIntent::SetNativeVolumeDraft),
            (
                "onNativeSeekRangeDraft",
                ReadIntent::SetNativeSeekRangeDraft,
            ),
            (
                "onNativeVolumeRangeDraft",
                ReadIntent::SetNativeVolumeRangeDraft,
            ),
        ] {
            keys.push((
                callbacks.bind_local(prop.into(), CallbackSignature::Text)?,
                intent,
            ));
        }
        Ok(Self {
            keys,
            queue: VecDeque::new(),
        })
    }
    pub fn intent(&self, invocation: &LocalActionInvocation) -> Option<ReadIntent> {
        self.keys
            .iter()
            .find(|(key, _)| invocation.matches_key(key))
            .map(|(_, intent)| *intent)
    }
    pub fn enqueue(
        &mut self,
        callback: &PendingCallback,
        intent: ReadIntent,
        claim: Option<AckClaim>,
    ) -> Result<(), PluginError> {
        if self.queue.len() >= 4 {
            return Err(busy());
        }
        self.queue.push_back(QueuedReadAction {
            callback: PendingCallback {
                handle: callback.handle.clone(),
                invocation: callback.invocation.clone(),
            },
            intent,
            claim,
        });
        Ok(())
    }
    pub fn pop(&mut self) -> Option<QueuedReadAction> {
        self.queue.pop_front()
    }
    pub fn restore(&mut self, action: QueuedReadAction) -> Result<(), Box<QueuedReadAction>> {
        if self.queue.len() >= 4 {
            return Err(Box::new(action));
        }
        self.queue.push_front(action);
        Ok(())
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}
#[cfg(test)]
mod tests;
