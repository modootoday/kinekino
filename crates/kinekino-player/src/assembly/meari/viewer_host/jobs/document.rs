//! Same-document JOB role owner. No SDK worker, endpoint, store or backend configuration.
#[cfg(test)]
pub(super) mod caption_contract_tests;
use super::{
    composition::{self, JobsFrameInput},
    handoff::*,
};
use crate::{
    application::{
        api::RequestContext,
        jobs::{JobId, JobRecord},
        sdk_execution::ExecutionHandle,
    },
    presentation::web_ui_adapter::{
        player_shell::document::product_frame::{ProductFrameBlueprint, WorkspaceRoute},
        studio::{
            inspector_state::{InspectorLayout, InspectorViewState},
            jobs::{list::JobsListProjection, selection::JobsSelection},
        },
    },
};
use meari::{
    application::{host_callbacks::PendingCallback, plugins::*},
    domain::{Value, host_callbacks::CallbackArguments},
};
use meari_native::infrastructure::native_host::{
    HostCommitReceipt, HostStamp, HostUpdateError, NativeDocumentHandle,
};
use std::{collections::BTreeMap, num::NonZeroU64};

pub(super) struct JobsMountInput {
    pub document: NativeDocumentHandle,
    pub frame: ProductFrameBlueprint,
    pub execution: ExecutionHandle,
    pub context: RequestContext,
    pub origin: HostStamp,
    pub generation: NonZeroU64,
    pub product_name: String,
    pub context_label: String,
}
#[must_use = "Parent retains partial handoff, current owner and original document through close"]
pub(super) struct JobsDocument {
    caption_source: Option<
        std::rc::Weak<
            std::cell::RefCell<
                crate::presentation::web_ui_adapter::player_shell::window_caption::CaptionState,
            >,
        >,
    >,
    caption_committed:
        crate::presentation::web_ui_adapter::player_shell::window_caption::CaptionView,
    document: NativeDocumentHandle,
    frame: ProductFrameBlueprint,
    execution: ExecutionHandle,
    context: RequestContext,
    origin: HostStamp,
    generation: NonZeroU64,
    product_name: String,
    context_label: String,
    list: JobsListProjection,
    selection: JobsSelection,
    detail: Option<BTreeMap<String, Value>>,
    record: Option<JobRecord>,
    inspector: InspectorViewState,
    active: Option<ReadyConsumer>,
    previous: Option<ReadyConsumer>,
    transfer: Option<QueryHandoff>,
    recovery: Option<RecoveryExport>,
    target: QueryTarget,
    queued: Option<PendingCallback>,
    original_intent: Option<PendingCallback>,
    first_error: Option<PluginError>,
    recovery_error: Option<PluginError>,
    document_error: Option<HostUpdateError<String>>,
    receipt: Option<HostCommitReceipt>,
    pending_effect: Option<HostCommitReceipt>,
    closing: bool,
    return_requested: bool,
    initial_read_attempted: bool,
    prepared_export_live: bool,
    read_error: Option<String>,
    forced_dirty: bool,
    continuation: bool,
}
impl JobsDocument {
    pub(super) fn new(input: JobsMountInput) -> Self {
        let transfer = QueryHandoff::new(
            input.execution.clone(),
            input.context.clone(),
            input.origin.clone(),
            input.generation,
            QueryTarget::List,
            None,
        );
        Self {
            caption_source: None,
            caption_committed: Default::default(),
            document: input.document,
            frame: input.frame,
            execution: input.execution,
            context: input.context,
            origin: input.origin,
            generation: input.generation,
            product_name: input.product_name,
            context_label: input.context_label,
            list: JobsListProjection::new(input.generation),
            selection: JobsSelection::new(input.generation),
            detail: None,
            record: None,
            inspector: InspectorViewState::new(InspectorLayout::Wide, false),
            active: None,
            previous: None,
            transfer: Some(transfer),
            recovery: None,
            target: QueryTarget::List,
            queued: None,
            original_intent: None,
            first_error: None,
            recovery_error: None,
            document_error: None,
            receipt: None,
            pending_effect: None,
            closing: false,
            return_requested: false,
            initial_read_attempted: false,
            prepared_export_live: false,
            read_error: None,
            forced_dirty: false,
            continuation: false,
        }
    }
    pub(super) fn receipt(&self) -> Option<&HostCommitReceipt> {
        self.receipt.as_ref()
    }
    pub(super) fn with_caption_source(
        mut self,
        source: std::rc::Weak<
            std::cell::RefCell<
                crate::presentation::web_ui_adapter::player_shell::window_caption::CaptionState,
            >,
        >,
    ) -> Self {
        self.caption_source = Some(source);
        self
    }
    fn caption_view(
        &self,
    ) -> crate::presentation::web_ui_adapter::player_shell::window_caption::CaptionView {
        self.caption_source
            .as_ref()
            .and_then(std::rc::Weak::upgrade)
            .and_then(|source| source.try_borrow().ok().map(|source| source.view.clone()))
            .unwrap_or_default()
    }
    pub(super) fn has_pending_effect(&self) -> bool {
        self.pending_effect.is_some()
    }
    pub(super) fn take_continuation(&mut self) -> bool {
        std::mem::take(&mut self.continuation)
    }
    pub(super) fn take_effect(&mut self) -> meari_native::infrastructure::native_host::HostEffects {
        let receipt = self.pending_effect.take();
        meari_native::infrastructure::native_host::HostEffects {
            redraw: receipt.is_some(),
            committed: receipt,
        }
    }
    pub(super) fn return_requested(&self) -> bool {
        self.return_requested
    }
    pub(super) fn has_entry_commit(&self) -> bool {
        self.active.is_some() && self.transfer.is_none()
    }
    #[cfg(test)]
    pub(super) fn detail_result_ready(&self, job: &JobId) -> bool {
        self.transfer.is_none()
            && self.active.as_ref().is_some_and(|active| {
                matches!(&active.owner, QueryOwner::Detail(owner)
                    if owner.pending().is_none()
                        && owner.view.record().is_some_and(|record| &record.job_id == job))
            })
            && self
                .record
                .as_ref()
                .is_some_and(|record| &record.job_id == job)
    }
    pub(super) fn has_root_export(&self) -> bool {
        self.prepared_export_live || self.recovery.is_some() || self.active.is_some()
    }
    pub(super) fn has_queued(&self) -> bool {
        self.queued.is_some()
    }
    fn check_stamp(&self, stamp: &HostStamp) -> Result<(), PluginError> {
        let current = self
            .document
            .stamp()
            .map_err(|error| PluginError::new(PluginErrorCode::Stale, format!("{error:?}")))?;
        if self.closing
            || &current != stamp
            || current.host_id != self.origin.host_id
            || current.owner != self.origin.owner
        {
            return Err(PluginError::new(
                PluginErrorCode::Stale,
                "Foreign JOB document checkpoint",
            ));
        }
        Ok(())
    }
    fn arguments(
        &self,
        pending: &PendingCallback,
        scope: &PluginScope,
    ) -> Result<CallbackArguments, PluginError> {
        if !self.prepared_export_live
            && let Some(recovery) = &self.recovery
        {
            return recovery.admit(pending, scope);
        }
        let owner = if self.prepared_export_live {
            self.transfer.as_ref().and_then(QueryHandoff::prepared)
        } else {
            self.active.as_ref()
        }
        .ok_or_else(|| {
            PluginError::new(PluginErrorCode::Busy, "JOB transition has no active export")
        })?;
        let invocation = owner
            .owner
            .local(pending, scope)
            .map_err(|failure| match failure {
                meari::application::host_callbacks::LocalDispatchError::Invalid(error) => error,
                meari::application::host_callbacks::LocalDispatchError::NotLocal => {
                    PluginError::new(PluginErrorCode::Stale, "JOB target changed")
                }
            })?;
        if !owner
            .keys
            .get(&pending.invocation.prop)
            .is_some_and(|key| invocation.matches_key(key))
        {
            return Err(PluginError::new(
                PluginErrorCode::Stale,
                "Foreign JOB local key",
            ));
        }
        Ok(invocation.arguments().clone())
    }
    pub(super) fn admit(
        &mut self,
        stamp: &HostStamp,
        pending: &PendingCallback,
    ) -> Result<(), PluginError> {
        self.check_stamp(stamp)?;
        let arguments = self.arguments(pending, &stamp.owner)?;
        if let Some((_, intent)) =
            crate::presentation::web_ui_adapter::player_shell::window_caption::BINDINGS
                .iter()
                .find(|(name, _)| *name == pending.invocation.prop)
        {
            if !matches!(arguments, CallbackArguments::Action) {
                return Err(PluginError::new(
                    PluginErrorCode::InvalidPayload,
                    "Window intent requires Action",
                ));
            }
            let source = self
                .caption_source
                .as_ref()
                .and_then(std::rc::Weak::upgrade)
                .ok_or_else(|| {
                    PluginError::new(
                        PluginErrorCode::Closing,
                        "Original JOB window source closed",
                    )
                })?;
            source
                .try_borrow_mut()
                .map_err(|_| {
                    PluginError::new(PluginErrorCode::Busy, "Original window caption borrowed")
                })?
                .request(*intent)?;
            self.forced_dirty = true;
            return Ok(());
        }
        if self.queued.is_some() {
            return Err(PluginError::new(
                PluginErrorCode::Busy,
                "Original JOB intent remains",
            ));
        }
        self.queued = Some(PendingCallback {
            handle: pending.handle.clone(),
            invocation: pending.invocation.clone(),
        });
        Ok(())
    }
    pub(super) fn admit_route_recovery(
        &mut self,
        stamp: &HostStamp,
        pending: &PendingCallback,
    ) -> Result<(), PluginError> {
        self.check_stamp(stamp)?;
        let arguments = self.arguments(pending, &stamp.owner)?;
        let name = pending.invocation.prop.as_str();
        let caption = crate::presentation::web_ui_adapter::player_shell::window_caption::BINDINGS
            .iter()
            .any(|(binding, _)| *binding == name);
        let allowed = caption
            || matches!((name, &arguments), ("onNavigate", CallbackArguments::Text(target)) if target == "VIEW")
            || matches!(
                name,
                "onRetryJobsResult" | "onRetryJobResult" | "onRetryJobsConnection"
            );
        if !allowed {
            return Err(PluginError::new(
                PluginErrorCode::Closing,
                "Only original query recovery is available during return",
            ));
        }
        self.admit(stamp, pending)
    }
    fn snapshots(&mut self) {
        if let Some(current) = &self.active {
            match &current.owner {
                QueryOwner::List(owner) => self.list = owner.view.clone(),
                QueryOwner::Detail(owner) => {
                    if self.selection.active().is_some_and(|selected| {
                        self.selection.accepts_result(selected)
                            && selected.job() == Some(owner.view.selected())
                    }) {
                        self.detail = Some(owner.view.props());
                        self.record = owner.view.record().cloned();
                    }
                }
            }
        }
    }
    fn project(
        &mut self,
        bindings: meari::application::host_callbacks::CallbackBindings,
        selection: &JobsSelection,
        detail: Option<&BTreeMap<String, Value>>,
        blocked: bool,
        retry: bool,
    ) -> Result<HostCommitReceipt, PluginError> {
        let before = self
            .document
            .stamp()
            .map_err(|error| PluginError::new(PluginErrorCode::Stale, format!("{error:?}")))?;
        let caption = self.caption_view();
        let result = self.document.try_reproject(&before, |document| {
            let retry_kind = if retry {
                self.transfer
                    .as_ref()
                    .and_then(QueryHandoff::blocked_kind)
                    .or_else(|| {
                        self.active
                            .as_ref()
                            .filter(|owner| owner.owner.blocked())
                            .map(|owner| owner.owner.kind())
                    })
            } else {
                None
            };
            let props = composition::props(
                document,
                JobsFrameInput {
                    caption: &caption,
                    product_name: &self.product_name,
                    context_label: &self.context_label,
                    route: self.generation,
                    list: &self.list,
                    selection,
                    detail,
                    inspector: self.inspector,
                    blocked,
                    retry_list: retry_kind == Some(QueryKind::List),
                    retry_detail: retry_kind == Some(QueryKind::Detail),
                    connection_retry: !self.return_requested
                        && self.transfer.as_ref().is_some_and(QueryHandoff::failed),
                    diagnostic: self.read_error.as_deref(),
                    status: if self.read_error.is_some() {
                        None
                    } else if self.return_requested {
                        Some("시청 화면으로 돌아갈 준비를 하고 있습니다.")
                    } else if self.transfer.is_some() {
                        Some("작업 정보 연결을 준비하고 있습니다.")
                    } else {
                        None
                    },
                },
            )?;
            let props = self.frame.workspace_props(
                document,
                WorkspaceRoute::Jobs,
                self.generation,
                props,
            )?;
            document.update_module_props_with_callbacks(props, bindings)
        });
        match result {
            Ok(((), receipt)) => {
                self.caption_committed = caption;
                self.receipt = Some(receipt.clone());
                self.pending_effect = Some(receipt.clone());
                Ok(receipt)
            }
            Err(error) => {
                let diagnostic = format!("{error:?}");
                self.document_error.get_or_insert(error);
                Err(PluginError::new(PluginErrorCode::Adapter, diagnostic))
            }
        }
    }
    pub(super) fn prepare_entry(&mut self) -> Result<bool, PluginError> {
        let result = self.finish_transfer();
        if let Err(error) = result {
            let primary = self.record_transfer_failure(error);
            if self.pending_effect.is_some() {
                self.continuation = true;
            }
            if self.has_root_export()
                && self.pending_effect.is_none()
                && let Err(error) = self.paint_recovery()
            {
                self.recovery_error.get_or_insert(error);
            }
            return Err(primary);
        }
        result
    }
    fn paint_recovery(&mut self) -> Result<(), PluginError> {
        let recovery = RecoveryExport::new(self.origin.owner.clone())?;
        let selection = self.selection.clone();
        let detail = self.detail.clone();
        self.project(
            recovery.bindings()?,
            &selection,
            detail.as_ref(),
            true,
            true,
        )?;
        self.recovery = Some(recovery);
        self.prepared_export_live = false;
        Ok(())
    }
    fn finish_transfer(&mut self) -> Result<bool, PluginError> {
        let transfer = self
            .transfer
            .as_mut()
            .ok_or_else(|| PluginError::new(PluginErrorCode::Stale, "No JOB handoff"))?;
        if !transfer.prepare()? {
            return Ok(false);
        }
        let mut selection = self.selection.clone();
        if let Some(pending) = selection.pending().cloned() {
            selection
                .commit(&pending)
                .map_err(|error| PluginError::new(PluginErrorCode::Stale, format!("{error:?}")))?;
        }
        let prepared = transfer.prepared_mut().expect("prepared consumer");
        let detail = match &mut prepared.owner {
            QueryOwner::List(owner) => {
                owner.view = self.list.clone();
                self.detail.clone()
            }
            QueryOwner::Detail(owner) => {
                if let Some(record) = &self.record
                    && &record.job_id == owner.view.selected()
                {
                    owner.view.received(record.clone()).map_err(|message| {
                        PluginError::new(PluginErrorCode::InvalidPayload, message)
                    })?;
                }
                Some(owner.view.props())
            }
        };
        let bindings = prepared.owner.bindings()?;
        let before = self
            .document
            .stamp()
            .map_err(|error| PluginError::new(PluginErrorCode::Stale, format!("{error:?}")))?;
        transfer.stage_commit(&before)?;
        let receipt = match self.project(bindings, &selection, detail.as_ref(), false, false) {
            Ok(receipt) => receipt,
            Err(error) => {
                self.transfer
                    .as_mut()
                    .unwrap()
                    .retain_failure(error.clone());
                return Err(error);
            }
        };
        self.prepared_export_live = true;
        let current = self
            .document
            .stamp()
            .map_err(|error| PluginError::new(PluginErrorCode::Stale, format!("{error:?}")))?;
        let committed = self.transfer.as_mut().unwrap().commit(receipt, &current)?;
        if let Some(error) = committed.first_error {
            self.first_error.get_or_insert(error);
        }
        if let Some(error) = committed.cleanup_error {
            self.recovery_error.get_or_insert(error);
        }
        self.receipt = Some(committed.receipt);
        self.active = Some(committed.current);
        self.previous = committed.previous;
        self.selection = selection;
        self.detail = detail;
        self.transfer = None;
        self.recovery = None;
        self.prepared_export_live = false;
        Ok(true)
    }
    fn begin_transfer(&mut self, target: QueryTarget) -> Result<(), PluginError> {
        let recovery = RecoveryExport::new(self.origin.owner.clone())?;
        let bindings = recovery.bindings()?;
        let selection = self.selection.clone();
        let detail = self.detail.clone();
        self.project(bindings, &selection, detail.as_ref(), true, false)?;
        self.recovery = Some(recovery);
        self.prepared_export_live = false;
        let current = self
            .document
            .stamp()
            .map_err(|error| PluginError::new(PluginErrorCode::Stale, format!("{error:?}")))?;
        self.transfer = Some(QueryHandoff::new(
            self.execution.clone(),
            self.context.clone(),
            current,
            self.generation,
            target.clone(),
            self.active.take(),
        ));
        self.target = target;
        Ok(())
    }
    pub(super) fn retry(&mut self) -> Result<bool, PluginError> {
        let recovery = RecoveryExport::new(self.origin.owner.clone())?;
        let selection = self.selection.clone();
        let detail = self.detail.clone();
        self.project(
            recovery.bindings()?,
            &selection,
            detail.as_ref(),
            true,
            true,
        )?;
        self.recovery = Some(recovery);
        self.prepared_export_live = false;
        let transfer = self
            .transfer
            .as_mut()
            .ok_or_else(|| PluginError::new(PluginErrorCode::Stale, "No failed JOB handoff"))?;
        transfer.retry()
    }
    fn record_failure(&mut self, error: PluginError) -> PluginError {
        self.first_error.get_or_insert_with(|| error.clone());
        self.read_error = Some(error.message.clone());
        error
    }
    fn record_transfer_failure(&mut self, error: PluginError) -> PluginError {
        if let Some(transfer) = &mut self.transfer {
            transfer.retain_failure(error.clone());
        }
        self.record_failure(error)
    }
    fn select(&mut self, job: JobId) -> Result<(), PluginError> {
        let selected = self
            .selection
            .request(&self.list, Some(job.clone()))
            .map_err(|error| PluginError::new(PluginErrorCode::Stale, format!("{error:?}")))?;
        if let Err(error) = self.begin_transfer(QueryTarget::Detail(job)) {
            let _ = self.selection.abort(&selected);
            return Err(error);
        }
        Ok(())
    }
    fn execute(&mut self, name: &str, arguments: CallbackArguments) -> Result<(), PluginError> {
        match (name, arguments) {
            ("onNavigate", CallbackArguments::Text(target)) if target == "VIEW" => {
                self.return_requested = true;
                Ok(())
            }
            ("onNavigate", CallbackArguments::Text(target)) if target == "JOB" => {
                if self
                    .transfer
                    .as_ref()
                    .is_some_and(|transfer| transfer.failed() || transfer.blocked())
                {
                    self.retry()?;
                }
                Ok(())
            }
            ("onInspectorOpenChange", CallbackArguments::Boolean(open)) => {
                let _focus_request = self.inspector.apply_open_change(open);
                Ok(())
            }
            ("onSelectJob", CallbackArguments::Text(job)) => {
                self.select(JobId::try_from(job).map_err(|message| {
                    PluginError::new(PluginErrorCode::InvalidPayload, message)
                })?)
            }
            ("onRefreshJobs", CallbackArguments::Action) => {
                if self.transfer.is_some() {
                    return Err(PluginError::new(
                        PluginErrorCode::Busy,
                        "Original handoff remains",
                    ));
                }
                if let Some(ReadyConsumer {
                    owner: QueryOwner::List(owner),
                    ..
                }) = &mut self.active
                {
                    owner.request_from_host()
                } else {
                    self.begin_transfer(QueryTarget::List)
                }
            }
            ("onRefreshJob", CallbackArguments::Action) => {
                let selected =
                    self.selection.active().cloned().ok_or_else(|| {
                        PluginError::new(PluginErrorCode::Stale, "No selected job")
                    })?;
                if self.selection.accepts_result(&selected)
                    && let Some(ReadyConsumer {
                        owner: QueryOwner::Detail(owner),
                        ..
                    }) = &mut self.active
                {
                    return owner.request_from_host();
                }
                self.select(selected.job().cloned().ok_or_else(|| {
                    PluginError::new(PluginErrorCode::Stale, "Selection is empty")
                })?)
            }
            (
                "onRetryJobsResult" | "onRetryJobResult" | "onRetryJobsConnection" | "onOpenJobs",
                CallbackArguments::Action,
            ) => {
                if self.transfer.is_some() {
                    self.retry()?;
                    Ok(())
                } else {
                    self.active
                        .as_mut()
                        .ok_or_else(|| {
                            PluginError::new(PluginErrorCode::Stale, "No original query")
                        })?
                        .owner
                        .retry_retained()
                }
            }
            _ => Err(PluginError::new(
                PluginErrorCode::MissingCapability,
                "이 화면에서는 해당 동작을 아직 제공하지 않습니다.",
            )),
        }
    }
    /// Parent calls only after the same original callback batch has fully drained.
    pub(super) fn advance(&mut self, remaining_callbacks: usize) -> Result<bool, PluginError> {
        if remaining_callbacks != 0 {
            return Ok(false);
        }
        if self.closing {
            return Err(PluginError::new(
                PluginErrorCode::Closing,
                "JOB document is closing",
            ));
        }
        let mut changed = std::mem::take(&mut self.forced_dirty);
        changed |= self.caption_view() != self.caption_committed;
        if let Some(pending) = &self.queued {
            let scope = self
                .document
                .stamp()
                .map_err(|error| PluginError::new(PluginErrorCode::Stale, format!("{error:?}")))?
                .owner;
            let arguments = self.arguments(pending, &scope);
            self.original_intent = self.queued.take();
            let arguments = arguments.map_err(|error| self.record_failure(error))?;
            let name = self
                .original_intent
                .as_ref()
                .unwrap()
                .invocation
                .prop
                .clone();
            self.read_error = None;
            if let Err(error) = self.execute(&name, arguments) {
                self.record_failure(error);
            }
            changed = true;
        }
        // An input may have committed the recovery export. Deliver that exact receipt
        // before preparing/committing the next owner in a later native host turn.
        if self.pending_effect.is_some() {
            self.continuation = true;
            return Ok(true);
        }
        if self.return_requested {
            return Ok(changed);
        }
        if self.transfer.is_some() {
            if self.transfer.as_ref().is_some_and(QueryHandoff::failed) {
                if changed {
                    self.paint_recovery()?;
                }
                return Ok(changed);
            }
            let ready = match self.finish_transfer() {
                Ok(ready) => ready,
                Err(error) => {
                    self.record_transfer_failure(error);
                    if self.pending_effect.is_none() {
                        self.paint_recovery()?;
                    } else {
                        self.forced_dirty = true;
                        self.continuation = true;
                    }
                    return Ok(true);
                }
            };
            if !ready {
                if self.transfer.as_ref().is_some_and(QueryHandoff::blocked)
                    && self.read_error.is_none()
                {
                    self.read_error =
                        Some("원래 조회 결과 확인이 남아 있습니다. 다시 확인해 주세요.".into());
                    let selection = self.selection.clone();
                    let detail = self.detail.clone();
                    let bindings = self
                        .recovery
                        .as_ref()
                        .ok_or_else(|| {
                            PluginError::new(PluginErrorCode::Stale, "Recovery export absent")
                        })?
                        .bindings()?;
                    self.project(bindings, &selection, detail.as_ref(), true, true)?;
                    changed = true;
                }
                return Ok(changed);
            }
            self.forced_dirty = true;
            self.continuation = true;
            self.active
                .as_mut()
                .unwrap()
                .owner
                .request_from_host()
                .map_err(|error| self.record_failure(error))?;
            return Ok(true);
        }
        if let Some(active) = &mut self.active {
            let result = active.owner.poll();
            match result {
                Ok(polled) => {
                    changed |= polled;
                    if polled {
                        self.read_error = active.owner.blocked_message();
                    }
                }
                Err(error) => {
                    self.read_error = Some(error.message.clone());
                    self.first_error.get_or_insert(error);
                    changed = true;
                }
            }
        }
        self.snapshots();
        if changed {
            let bindings = self
                .active
                .as_ref()
                .ok_or_else(|| PluginError::new(PluginErrorCode::Busy, "JOB consumer absent"))?
                .owner
                .bindings()?;
            let blocked = self
                .active
                .as_ref()
                .is_some_and(|owner| owner.owner.pending());
            let retry = self
                .active
                .as_ref()
                .is_some_and(|owner| owner.owner.blocked());
            let selection = self.selection.clone();
            let detail = self.detail.clone();
            self.project(bindings, &selection, detail.as_ref(), blocked, retry)
                .map_err(|error| self.record_failure(error))?;
        }
        Ok(changed)
    }
    pub(super) fn initial_read(&mut self) -> Result<(), PluginError> {
        if self.initial_read_attempted {
            return Ok(());
        }
        self.initial_read_attempted = true;
        self.forced_dirty = true;
        self.active
            .as_mut()
            .ok_or_else(|| PluginError::new(PluginErrorCode::Busy, "JOB entry not committed"))?
            .owner
            .request_from_host()
    }
    pub(super) fn close(&mut self) -> Result<bool, PluginError> {
        self.closing = true;
        if let Some(recovery) = &mut self.recovery {
            recovery.invalidate()?;
        }
        if let Some(transfer) = &mut self.transfer
            && !transfer.close()?
        {
            return Ok(false);
        }
        if let Some(current) = &mut self.active
            && !current.owner.close()?
        {
            return Ok(false);
        }
        if let Some(previous) = &mut self.previous
            && !previous.owner.close()?
        {
            return Ok(false);
        }
        Ok(true)
    }
    pub(super) fn prepare_route_release(&mut self) -> Result<bool, PluginError> {
        if self.recovery.is_none() || self.prepared_export_live {
            let recovery = RecoveryExport::new(self.origin.owner.clone())?;
            let selection = self.selection.clone();
            let detail = self.detail.clone();
            self.project(
                recovery.bindings()?,
                &selection,
                detail.as_ref(),
                true,
                false,
            )?;
            self.recovery = Some(recovery);
            self.prepared_export_live = false;
            self.continuation = true;
            return Ok(false);
        }
        let result: Result<bool, PluginError> = (|| {
            if let Some(transfer) = &mut self.transfer
                && !transfer.close()?
            {
                return Ok(false);
            }
            if let Some(active) = &mut self.active
                && !active.owner.close()?
            {
                return Ok(false);
            }
            if let Some(previous) = &mut self.previous
                && !previous.owner.close()?
            {
                return Ok(false);
            }
            Ok(true)
        })();
        let blocked = self.transfer.as_ref().is_some_and(QueryHandoff::blocked)
            || self
                .active
                .as_ref()
                .is_some_and(|owner| owner.owner.blocked());
        if result.is_err() || (matches!(result, Ok(false)) && blocked && self.read_error.is_none())
        {
            if let Err(error) = &result {
                self.read_error = Some(error.message.clone());
                self.first_error.get_or_insert_with(|| error.clone());
            } else {
                self.read_error =
                    Some("원래 조회 결과 확인이 남아 있습니다. 다시 확인해 주세요.".into());
            }
            let selection = self.selection.clone();
            let detail = self.detail.clone();
            let bindings = self.recovery.as_ref().unwrap().bindings()?;
            self.project(bindings, &selection, detail.as_ref(), true, true)?;
        }
        result
    }
    pub(super) fn process_route_recovery(&mut self) -> Result<(), PluginError> {
        let Some(pending) = self.queued.as_ref() else {
            return Ok(());
        };
        let scope = self
            .document
            .stamp()
            .map_err(|error| PluginError::new(PluginErrorCode::Stale, format!("{error:?}")))?
            .owner;
        let arguments = self.arguments(pending, &scope)?;
        let name = pending.invocation.prop.as_str();
        if matches!((name, &arguments), ("onNavigate", CallbackArguments::Text(target)) if target == "VIEW")
            || matches!(
                name,
                "onRetryJobsResult" | "onRetryJobResult" | "onRetryJobsConnection"
            )
        {
            self.retry_retained_for_close()?;
            self.original_intent = self.queued.take();
            self.read_error = None;
            return Ok(());
        }
        self.original_intent = self.queued.take();
        Err(PluginError::new(
            PluginErrorCode::Closing,
            "Only original query recovery is available during return",
        ))
    }
    pub(super) fn retry_retained_for_close(&mut self) -> Result<(), PluginError> {
        if let Some(transfer) = &mut self.transfer {
            transfer.retry_retained_for_close()?;
        }
        for current in [&mut self.active, &mut self.previous].into_iter().flatten() {
            if current.owner.blocked() {
                current.owner.retry_retained()?;
            }
        }
        Ok(())
    }
}
