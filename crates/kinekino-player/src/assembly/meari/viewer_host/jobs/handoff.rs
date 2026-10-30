//! One callback export per phase; no merge of unrelated callback owners or SDK attachments.
use super::ListReadOwner;
use crate::application::{
    api::RequestContext,
    jobs::{GetJob, JobId},
    sdk_execution::{ExecutionAttachment, ExecutionHandle},
};
use crate::assembly::meari::job_details::JobDetailsOwner;
use meari::{
    application::{
        host_callbacks::{
            CallbackBindings, HostCallbacks, LocalActionInvocation, LocalActionKey,
            LocalDispatchError, PendingCallback,
        },
        plugins::*,
    },
    domain::host_callbacks::{CallbackArguments, CallbackSignature},
};
use meari_native::infrastructure::native_host::{HostCommitReceipt, HostStamp};
use std::collections::BTreeMap;
use std::{num::NonZeroU64, sync::Arc};

const JOB_LOCAL: [(&str, CallbackSignature); 11] = [
    ("onWindowMinimize", CallbackSignature::Action),
    ("onWindowToggleMaximize", CallbackSignature::Action),
    ("onWindowUserClose", CallbackSignature::Action),
    ("onRetryJobsConnection", CallbackSignature::Action),
    ("onNavigate", CallbackSignature::Text),
    ("onInspectorOpenChange", CallbackSignature::Boolean),
    ("onSelectJob", CallbackSignature::Text),
    ("onRefreshJobs", CallbackSignature::Action),
    ("onRetryJobsResult", CallbackSignature::Action),
    ("onRefreshJob", CallbackSignature::Action),
    ("onRetryJobResult", CallbackSignature::Action),
];
pub(super) enum QueryOwner {
    List(Box<ListReadOwner>),
    Detail(Box<JobDetailsOwner>),
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum QueryKind {
    List,
    Detail,
}
impl QueryOwner {
    pub(super) fn kind(&self) -> QueryKind {
        match self {
            Self::List(_) => QueryKind::List,
            Self::Detail(_) => QueryKind::Detail,
        }
    }
    pub(super) fn blocked(&self) -> bool {
        match self {
            Self::List(owner) => owner.consumer.blocked(),
            Self::Detail(owner) => owner.blocked(),
        }
    }
    pub(super) fn pending(&self) -> bool {
        match self {
            Self::List(owner) => owner.consumer.pending().is_some(),
            Self::Detail(owner) => owner.pending().is_some(),
        }
    }
    pub(super) fn blocked_message(&self) -> Option<String> {
        if !self.blocked() {
            return None;
        }
        Some(match self {
            Self::List(owner) => owner.diagnostic.clone(),
            Self::Detail(owner) => owner.view.diagnostic.clone(),
        })
    }
    pub(super) fn root_keys(&mut self) -> Result<BTreeMap<String, LocalActionKey>, PluginError> {
        let mut result = BTreeMap::new();
        for (name, signature) in JOB_LOCAL {
            let key = match self {
                Self::List(owner) => owner.bind_local(name, signature)?,
                Self::Detail(owner) => owner.bind_local(name, signature)?,
            };
            result.insert(name.into(), key);
        }
        Ok(result)
    }
    pub(super) fn bindings(&self) -> Result<CallbackBindings, PluginError> {
        match self {
            Self::List(owner) => owner.bindings(),
            Self::Detail(owner) => owner.bindings(),
        }
    }
    pub(super) fn local(
        &self,
        pending: &PendingCallback,
        scope: &PluginScope,
    ) -> Result<LocalActionInvocation, LocalDispatchError> {
        match self {
            Self::List(owner) => owner.local(pending, scope),
            Self::Detail(owner) => owner.local(pending, scope),
        }
    }
    pub(super) fn poll(&mut self) -> Result<bool, PluginError> {
        match self {
            Self::List(owner) => owner.poll(),
            Self::Detail(owner) => owner.poll(),
        }
    }
    pub(super) fn request_from_host(&mut self) -> Result<(), PluginError> {
        match self {
            Self::List(owner) => owner.request_from_host(),
            Self::Detail(owner) => owner.request_from_host(),
        }
    }
    pub(super) fn close(&mut self) -> Result<bool, PluginError> {
        match self {
            Self::List(owner) => owner.close(),
            Self::Detail(owner) => owner.close(),
        }
    }
    pub(super) fn retry_retained(&mut self) -> Result<(), PluginError> {
        match self {
            Self::List(owner) => owner.retry_retained(),
            Self::Detail(owner) => owner.retry_retained(),
        }
    }
}

#[derive(Clone)]
pub(super) enum QueryTarget {
    List,
    Detail(JobId),
}
pub(super) struct ReadyConsumer {
    pub owner: QueryOwner,
    pub keys: BTreeMap<String, LocalActionKey>,
}
pub(super) struct CommittedHandoff {
    pub current: ReadyConsumer,
    pub previous: Option<ReadyConsumer>,
    pub first_error: Option<PluginError>,
    pub cleanup_error: Option<PluginError>,
    pub receipt: HostCommitReceipt,
}
/// SDK attachment transfer only. The parent still owns ExecutionOwner, route ticket and document.
#[must_use = "Retain old/prepared owner and attachment on every partial handoff failure"]
pub(super) struct QueryHandoff {
    execution: ExecutionHandle,
    context: RequestContext,
    scope: PluginScope,
    origin: HostStamp,
    expected_commit: Option<HostStamp>,
    route: NonZeroU64,
    target: QueryTarget,
    old: Option<ReadyConsumer>,
    attachment: Option<Arc<ExecutionAttachment>>,
    prepared: Option<ReadyConsumer>,
    keys_ready: bool,
    first_error: Option<PluginError>,
    current_error: Option<PluginError>,
    cleanup_error: Option<PluginError>,
    closing: bool,
}
impl QueryHandoff {
    pub(super) fn new(
        execution: ExecutionHandle,
        context: RequestContext,
        origin: HostStamp,
        route: NonZeroU64,
        target: QueryTarget,
        old: Option<ReadyConsumer>,
    ) -> Self {
        Self {
            execution,
            context,
            scope: origin.owner.clone(),
            origin,
            expected_commit: None,
            route,
            target,
            old,
            attachment: None,
            prepared: None,
            keys_ready: false,
            first_error: None,
            current_error: None,
            cleanup_error: None,
            closing: false,
        }
    }
    pub(super) fn prepare(&mut self) -> Result<bool, PluginError> {
        if self.closing {
            return Err(PluginError::new(
                PluginErrorCode::Closing,
                "Handoff is closing",
            ));
        }
        if let Some(error) = &self.current_error {
            return Err(error.clone());
        }
        let result = self.prepare_inner();
        if let Err(error) = &result {
            self.first_error.get_or_insert_with(|| error.clone());
            self.current_error = Some(error.clone());
        }
        result
    }
    fn prepare_inner(&mut self) -> Result<bool, PluginError> {
        if let Some(old) = &mut self.old
            && !old.owner.close()?
        {
            return Ok(false);
        }
        // attach itself checks the original worker's exclusive consumer and all entries.
        if self.attachment.is_none() {
            self.attachment =
                Some(Arc::new(self.execution.attach().map_err(|error| {
                    PluginError::new(PluginErrorCode::Busy, error.message)
                })?));
        }
        if self.prepared.is_none() {
            let attachment = self.attachment.as_ref().expect("stored attachment").clone();
            let owner = match &self.target {
                QueryTarget::List => QueryOwner::List(Box::new(ListReadOwner::new(
                    attachment,
                    self.scope.clone(),
                    self.route,
                    self.context.clone(),
                )?)),
                QueryTarget::Detail(job) => {
                    QueryOwner::Detail(Box::new(JobDetailsOwner::new_embedded(
                        attachment,
                        self.scope.clone(),
                        GetJob {
                            job_id: job.clone(),
                        },
                        self.context.clone(),
                    )?))
                }
            };
            // Retain the original new consumer before any local export binding can fail.
            self.prepared = Some(ReadyConsumer {
                owner,
                keys: BTreeMap::new(),
            });
        }
        if !self.keys_ready {
            let prepared = self.prepared.as_mut().expect("stored consumer");
            prepared.keys = prepared.owner.root_keys()?;
            self.keys_ready = true;
        }
        Ok(true)
    }
    pub(super) fn prepared(&self) -> Option<&ReadyConsumer> {
        self.keys_ready.then_some(()).and(self.prepared.as_ref())
    }
    pub(super) fn prepared_mut(&mut self) -> Option<&mut ReadyConsumer> {
        if self.keys_ready {
            self.prepared.as_mut()
        } else {
            None
        }
    }
    pub(super) fn retain_failure(&mut self, error: PluginError) {
        self.first_error.get_or_insert_with(|| error.clone());
        self.current_error = Some(error);
    }
    /// Explicit recovery never resubmits the old query or creates another SDK worker.
    pub(super) fn retry(&mut self) -> Result<bool, PluginError> {
        if self.closing {
            return Err(PluginError::new(
                PluginErrorCode::Closing,
                "Handoff is closing",
            ));
        }
        if self.current_error.is_none() && !self.blocked() {
            return Err(PluginError::new(
                PluginErrorCode::Stale,
                "No handoff failure",
            ));
        }
        if let Some(old) = &mut self.old
            && old.owner.blocked()
        {
            old.owner.retry_retained()?;
        }
        if let Some(prepared) = &mut self.prepared {
            match prepared.owner.close() {
                Ok(false) => return Ok(false),
                Err(error) => {
                    self.cleanup_error.get_or_insert_with(|| error.clone());
                    return Err(error);
                }
                Ok(true) => {}
            }
        }
        if let Some(attachment) = &self.attachment
            && let Err(error) = attachment.release()
        {
            let error = PluginError::new(PluginErrorCode::Busy, error.message);
            self.cleanup_error.get_or_insert_with(|| error.clone());
            return Err(error);
        }
        self.prepared = None;
        self.attachment = None;
        self.keys_ready = false;
        self.expected_commit = None;
        self.current_error = None;
        Ok(true)
    }
    pub(super) fn stage_commit(&mut self, current: &HostStamp) -> Result<(), PluginError> {
        if self.closing
            || !self.keys_ready
            || self.current_error.is_some()
            || current.host_id != self.origin.host_id
            || current.owner != self.origin.owner
            || current.definition_revision != self.origin.definition_revision
            || current.document_revision < self.origin.document_revision
        {
            return Err(PluginError::new(
                PluginErrorCode::Stale,
                "Handoff commit checkpoint is stale",
            ));
        }
        self.expected_commit = Some(current.clone());
        Ok(())
    }
    /// Caller passes the real atomic document receipt; ticket/frame observation stays in ShellState.
    pub(super) fn commit(
        &mut self,
        receipt: HostCommitReceipt,
        current: &HostStamp,
    ) -> Result<CommittedHandoff, PluginError> {
        if self.closing
            || !self.keys_ready
            || self.current_error.is_some()
            || self.expected_commit.as_ref() != Some(receipt.before())
            || receipt.after() != current
            || receipt.before().owner != self.scope
            || receipt.after().owner != self.scope
            || receipt.before().host_id != receipt.after().host_id
            || receipt.before().definition_revision != receipt.after().definition_revision
            || receipt.before().document_revision.checked_add(1)
                != Some(receipt.after().document_revision)
        {
            return Err(PluginError::new(
                PluginErrorCode::Stale,
                "Handoff document scope changed",
            ));
        }
        let current = self
            .prepared
            .take()
            .ok_or_else(|| PluginError::new(PluginErrorCode::Stale, "No prepared consumer"))?;
        self.keys_ready = false;
        self.expected_commit = None;
        // Current consumer keeps its Arc; explicit attachment guard is no longer an active owner.
        self.attachment = None;
        Ok(CommittedHandoff {
            current,
            previous: self.old.take(),
            first_error: self.first_error.clone(),
            cleanup_error: self.cleanup_error.clone(),
            receipt,
        })
    }
    pub(super) fn blocked(&self) -> bool {
        self.old.as_ref().is_some_and(|owner| owner.owner.blocked())
            || self
                .prepared
                .as_ref()
                .is_some_and(|owner| owner.owner.blocked())
    }
    pub(super) fn blocked_kind(&self) -> Option<QueryKind> {
        [&self.old, &self.prepared]
            .into_iter()
            .flatten()
            .find(|owner| owner.owner.blocked())
            .map(|owner| owner.owner.kind())
    }
    pub(super) fn failed(&self) -> bool {
        self.current_error.is_some()
    }
    pub(super) fn close(&mut self) -> Result<bool, PluginError> {
        self.closing = true;
        if let Some(old) = &mut self.old
            && !old.owner.close()?
        {
            return Ok(false);
        }
        if let Some(prepared) = &mut self.prepared
            && !prepared.owner.close()?
        {
            return Ok(false);
        }
        if let Some(attachment) = &self.attachment {
            attachment
                .release()
                .map_err(|error| PluginError::new(PluginErrorCode::Busy, error.message))?;
        }
        Ok(true)
    }
    pub(super) fn retry_retained_for_close(&mut self) -> Result<(), PluginError> {
        for consumer in [&mut self.old, &mut self.prepared].into_iter().flatten() {
            if consumer.owner.blocked() {
                consumer.owner.retry_retained()?;
            }
        }
        Ok(())
    }
}

pub(super) struct RecoveryExport {
    callbacks: HostCallbacks,
    keys: BTreeMap<String, LocalActionKey>,
}
impl RecoveryExport {
    /// Render the retained Viewing graph without admitting its inactive actions.
    /// Only `keys` from `new` can pass recovery admission; these extra local
    /// bindings are an export surface, never SDK capability registrations.
    pub(super) fn viewing(
        scope: PluginScope,
        props: &BTreeMap<String, meari::domain::Value>,
    ) -> Result<Self, PluginError> {
        let mut recovery = Self::new(scope)?;
        let mut signatures = BTreeMap::new();
        for value in props.values() {
            if let meari::domain::Value::CallbackReference(reference) = value
                && let Some(previous) =
                    signatures.insert(reference.binding.clone(), reference.signature)
                && previous != reference.signature
            {
                return Err(PluginError::new(
                    PluginErrorCode::InvalidPayload,
                    "Viewing recovery callback signatures differ",
                ));
            }
        }
        for (name, signature) in signatures {
            if !recovery.keys.contains_key(&name) {
                recovery.callbacks.bind_local(name, signature)?;
            }
        }
        Ok(recovery)
    }
    pub(super) fn new(scope: PluginScope) -> Result<Self, PluginError> {
        let mut callbacks = HostCallbacks::new(scope);
        let mut keys = BTreeMap::new();
        // Old Viewing can remain the last-good frame while its exact query owner drains.
        for (name, signature) in JOB_LOCAL.into_iter().chain([
            ("onOpenJobs", CallbackSignature::Action),
            ("onReadCapabilities", CallbackSignature::Action),
            ("onReadEngine", CallbackSignature::Action),
            ("onReadStatus", CallbackSignature::Action),
            ("onRetryCapabilitiesAck", CallbackSignature::Action),
            ("onRetryEngineAck", CallbackSignature::Action),
            ("onRetryStatusAck", CallbackSignature::Action),
            ("onRefreshReadProjection", CallbackSignature::Action),
            ("onCapturedCardEdit", CallbackSignature::Text),
            ("onCardSelection", CallbackSignature::Text),
        ]) {
            keys.insert(name.into(), callbacks.bind_local(name.into(), signature)?);
        }
        Ok(Self { callbacks, keys })
    }
    pub(super) fn bindings(&self) -> Result<CallbackBindings, PluginError> {
        self.callbacks.export_bindings()
    }
    pub(super) fn admit(
        &self,
        pending: &PendingCallback,
        scope: &PluginScope,
    ) -> Result<CallbackArguments, PluginError> {
        let invocation = self
            .callbacks
            .dispatch_local(&pending.handle, pending.invocation.arguments.clone(), scope)
            .map_err(|error| match error {
                LocalDispatchError::Invalid(error) => error,
                LocalDispatchError::NotLocal => {
                    PluginError::new(PluginErrorCode::Stale, "Recovery target changed")
                }
            })?;
        if !self
            .keys
            .get(&pending.invocation.prop)
            .is_some_and(|key| invocation.matches_key(key))
        {
            return Err(PluginError::new(
                PluginErrorCode::Stale,
                "Foreign recovery callback",
            ));
        }
        Ok(invocation.arguments().clone())
    }
    pub(super) fn invalidate(&mut self) -> Result<(), PluginError> {
        self.callbacks.invalidate()
    }
}
