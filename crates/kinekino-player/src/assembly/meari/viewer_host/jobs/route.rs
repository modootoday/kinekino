//! Jobs route tickets and checked host/frame hooks; SDK policies stay outside presentation.
use super::super::*;
use super::{
    document::{JobsDocument, JobsMountInput},
    handoff::RecoveryExport,
};
use crate::{
    assembly::meari::viewer_read_actions::ReadKind,
    presentation::web_ui_adapter::player_shell::{NavigationGuards, RouteCatalog, RouteTicket},
};
use meari::{
    application::{
        host_callbacks::CallbackBindings,
        plugins::{PluginError, PluginErrorCode},
    },
    domain::Value,
};
use std::{collections::BTreeMap, num::NonZeroU64};

#[derive(Clone, Copy, PartialEq, Eq)]
enum EntryPhase {
    Requested,
    Draining,
    Committing,
    Armed,
}
struct Entry {
    ticket: Option<RouteTicket>,
    origin: HostStamp,
    _intent: PendingCallback,
    phase: EntryPhase,
    recovery: Option<RecoveryExport>,
    recovery_live: bool,
    queued: Option<PendingCallback>,
    error: Option<ViewerHostError>,
    first_error: Option<ViewerHostError>,
    recovery_error: Option<ViewerHostError>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReturnPhase {
    Releasing,
    Exporting,
    Committing,
    CommittedAwaitingArm,
    Armed,
    Observed,
}
struct Returning {
    ticket: RouteTicket,
    phase: ReturnPhase,
    bindings: Option<CallbackBindings>,
    generation: Option<NonZeroU64>,
    error: Option<ViewerHostError>,
    first_error: Option<ViewerHostError>,
    committed: Option<(HostCommitReceipt, HostStamp)>,
    proof: Option<meari_native::infrastructure::native_host::RetainedCommitProof>,
    proof_error: Option<Arc<HostAccessError>>,
    first_proof_error: Option<Arc<HostAccessError>>,
    retry_intent: Option<PendingCallback>,
}
impl Returning {
    fn proof_failure(&mut self, error: HostAccessError) -> ViewerHostError {
        let error = Arc::new(error);
        let previous = self.proof_error.replace(error.clone());
        self.first_proof_error
            .get_or_insert_with(|| previous.unwrap_or(error));
        stale()
    }
}
#[derive(Default)]
pub(in crate::assembly::meari::viewer_host) struct JobsLedger {
    sequence: u64,
    entry: Option<Entry>,
    document: Option<JobsDocument>,
    completed: Option<JobsDocument>,
    returning: Option<Returning>,
    effect: Option<HostCommitReceipt>,
    redraw: bool,
    completion: Option<Arc<dyn crate::application::sdk_execution::CompletionNotifier>>,
    continuation: bool,
    wake_error: Option<Arc<crate::application::api::ApiError>>,
}
impl JobsLedger {
    pub(in crate::assembly::meari::viewer_host) fn occupied(&self) -> bool {
        self.entry.is_some() || self.document.is_some() || self.returning.is_some()
    }
    pub(in crate::assembly::meari::viewer_host) fn set_completion(
        &mut self,
        completion: Arc<dyn crate::application::sdk_execution::CompletionNotifier>,
    ) {
        self.completion = Some(completion);
    }
    pub(in crate::assembly::meari::viewer_host) fn poll_close(
        &mut self,
    ) -> Result<bool, PluginError> {
        if let Some(entry) = &mut self.entry
            && let Some(recovery) = &mut entry.recovery
        {
            recovery.invalidate()?;
        }
        if let Some(document) = &mut self.document
            && !document.close()?
        {
            return Ok(false);
        }
        if let Some(completed) = &mut self.completed
            && !completed.close()?
        {
            return Ok(false);
        }
        Ok(true)
    }
    fn effects(&mut self) -> HostEffects {
        if let Some(document) = &mut self.document {
            self.continuation |= document.take_continuation();
            let effect = document.take_effect();
            if let Some(receipt) = effect.committed
                && self.effect.as_ref().is_none_or(|current| {
                    receipt.after().document_revision > current.after().document_revision
                })
            {
                self.effect = Some(receipt);
            }
        }
        let receipt = self.effect.take();
        if std::mem::take(&mut self.continuation) {
            let result = self.completion.as_ref().map_or_else(
                || {
                    Err(crate::application::api::ApiError::new(
                        crate::application::api::ErrorCode::Unsupported,
                        "Original Jobs continuation notifier is not configured",
                    ))
                },
                |completion| completion.notify(),
            );
            if let Err(error) = result {
                self.wake_error.get_or_insert_with(|| Arc::new(error));
            }
        }
        let redraw = std::mem::take(&mut self.redraw);
        HostEffects {
            redraw: receipt.is_some() || redraw,
            committed: receipt,
        }
    }
}
fn callback(error: PluginError) -> ViewerHostError {
    ViewerHostError::Callback(Arc::new(error))
}
fn stale() -> ViewerHostError {
    ViewerHostError::ForeignCheckpoint
}

impl ViewerHost {
    fn admit_original_jobs_local(
        &self,
        stamp: &HostStamp,
        pending: &PendingCallback,
        armed: bool,
    ) -> IntentAdmission<ViewerHostError> {
        if pending.invocation.prop != "onOpenJobs" {
            return IntentAdmission::Rejected(callback(PluginError::new(
                PluginErrorCode::Closing,
                "Only the current Viewing route intent is available",
            )));
        }
        let Ok(mut owner) = self.callbacks.try_borrow_mut() else {
            return IntentAdmission::Capacity;
        };
        if let Err(error) = owner.verify_jobs_entry(&stamp.owner, pending) {
            return IntentAdmission::Rejected(callback(error));
        }
        if armed || owner.has_jobs_entry() {
            return IntentAdmission::Accepted;
        }
        match owner.queue_jobs_entry(&stamp.owner, pending) {
            Ok(true) => IntentAdmission::Accepted,
            Ok(false) => IntentAdmission::Rejected(stale()),
            Err(error) => IntentAdmission::Rejected(callback(local_error(error))),
        }
    }
    pub(in crate::assembly::meari::viewer_host) fn admit_jobs(
        &mut self,
        stamp: &HostStamp,
        pending: &PendingCallback,
    ) -> Option<IntentAdmission<ViewerHostError>> {
        if pending.invocation.prop == "onOpenJobs" {
            let live = match self.cards.try_borrow() {
                Ok(cards) => cards.live_opening.is_some(),
                Err(_) => return Some(IntentAdmission::Capacity),
            };
            let queued = match self.callbacks.try_borrow() {
                Ok(owner) => owner.has_live_entry(),
                Err(_) => return Some(IntentAdmission::Capacity),
            };
            if live || queued {
                return Some(IntentAdmission::Rejected(callback(PluginError::new(
                    PluginErrorCode::Closing,
                    "Original live proposal entry is retained",
                ))));
            }
        }
        let cell = self.jobs.clone();
        let mut guard = match cell.try_borrow_mut() {
            Ok(jobs) => jobs,
            Err(_) => return Some(IntentAdmission::Capacity),
        };
        let jobs = &mut *guard;
        if jobs.entry.is_none()
            && jobs.document.is_none()
            && pending.invocation.prop != "onOpenJobs"
        {
            return None;
        }
        if self.document.as_ref().is_none_or(|document| {
            document.lifecycle() != HostLifecycle::Open
                || document.stamp().ok().as_ref() != Some(stamp)
        }) {
            return Some(IntentAdmission::Rejected(stale()));
        }
        let result = if let Some(entry) = &mut jobs.entry {
            if entry.phase == EntryPhase::Armed {
                return Some(IntentAdmission::Rejected(callback(PluginError::new(
                    PluginErrorCode::Closing,
                    "Jobs frame observation is still pending",
                ))));
            }
            if let Some(document) = &mut jobs.document
                && document.has_root_export()
            {
                document.admit(stamp, pending)
            } else if entry.recovery_live {
                match entry
                    .recovery
                    .as_ref()
                    .expect("live recovery")
                    .admit(pending, &stamp.owner)
                {
                    Ok(_) if entry.queued.is_none() => {
                        entry.queued = Some(PendingCallback {
                            handle: pending.handle.clone(),
                            invocation: pending.invocation.clone(),
                        });
                        Ok(())
                    }
                    Ok(_)
                        if pending.invocation.prop == "onOpenJobs"
                            && entry
                                .queued
                                .as_ref()
                                .is_some_and(|queued| queued.invocation.prop == "onOpenJobs") =>
                    {
                        Ok(())
                    }
                    Ok(_) => Err(PluginError::new(
                        PluginErrorCode::Closing,
                        "A different original route intent is retained",
                    )),
                    Err(error) => Err(error),
                }
            } else {
                if pending.invocation.prop != "onOpenJobs" {
                    return Some(IntentAdmission::Rejected(callback(PluginError::new(
                        PluginErrorCode::Closing,
                        "Only the original Jobs route intent is available",
                    ))));
                }
                self.callbacks
                    .try_borrow_mut()
                    .map_err(|_| PluginError::new(PluginErrorCode::Busy, "Viewing owner borrowed"))
                    .and_then(|mut owner| {
                        owner
                            .queue_jobs_entry(&stamp.owner, pending)
                            .map(|_| ())
                            .map_err(local_error)
                    })
            }
        } else if let Some(document) = &mut jobs.document {
            if jobs
                .returning
                .as_ref()
                .is_some_and(|request| request.phase == ReturnPhase::Armed)
            {
                return Some(self.admit_original_jobs_local(stamp, pending, true));
            }
            if jobs.returning.as_ref().is_some_and(|request| {
                matches!(
                    request.phase,
                    ReturnPhase::CommittedAwaitingArm | ReturnPhase::Observed
                )
            }) {
                return Some(self.admit_original_jobs_local(stamp, pending, false));
            }
            if jobs.returning.is_some() {
                document.admit_route_recovery(stamp, pending)
            } else {
                document.admit(stamp, pending)
            }
        } else if self.product_frame.is_none() {
            Err(PluginError::new(
                PluginErrorCode::MissingCapability,
                "Product Jobs graph is not installed",
            ))
        } else {
            return Some(self.admit_original_jobs_local(stamp, pending, false));
        };
        Some(match result {
            Ok(()) => IntentAdmission::Accepted,
            Err(error) if error.code == PluginErrorCode::Busy => IntentAdmission::Capacity,
            Err(error) => IntentAdmission::Rejected(callback(error)),
        })
    }
    fn viewing_props(&self) -> Result<BTreeMap<String, Value>, ViewerHostError> {
        let owner = self.callbacks.try_borrow().map_err(|_| stale())?;
        let mut props = self.base_props.clone();
        props.extend(
            owner
                .projection
                .try_borrow()
                .map_err(|_| stale())?
                .panel
                .props(),
        );
        props.extend(owner.local_props());
        Ok(props)
    }
    fn commit_viewing_props(
        &self,
        bindings: CallbackBindings,
        props: BTreeMap<String, Value>,
        generation: NonZeroU64,
    ) -> Result<HostCommitReceipt, ViewerHostError> {
        let document = self.document.as_ref().ok_or_else(stale)?;
        let before = document.stamp().map_err(|_| stale())?;
        let frame = self
            .product_frame
            .as_ref()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        document.try_reproject(&before, |document| {
            let props = frame.workspace_props(document,
                crate::presentation::web_ui_adapter::player_shell::document::product_frame::WorkspaceRoute::Viewing,
                generation, props)?;
            document.update_module_props_with_callbacks(props, bindings)
        }).map(|(_, receipt)| receipt).map_err(|error| callback(PluginError::new(PluginErrorCode::Adapter, format!("{error:?}"))))
    }
    fn recovery_view(
        &self,
        recovery: &RecoveryExport,
        failed: bool,
    ) -> Result<HostCommitReceipt, ViewerHostError> {
        let mut props = self.viewing_props()?;
        let owner = self.callbacks.try_borrow().map_err(|_| stale())?;
        for field in ["capabilitiesBusy", "engineBusy", "statusBusy"] {
            props.insert(field.into(), Value::Bool(true));
        }
        for (kind, field) in [
            (ReadKind::Capabilities, "capabilitiesRetryDisabled"),
            (ReadKind::Engine, "engineRetryDisabled"),
            (ReadKind::Status, "statusRetryDisabled"),
        ] {
            props.insert(field.into(), Value::Bool(owner.ack_claim(kind).is_err()));
        }
        let ack_retry = [ReadKind::Capabilities, ReadKind::Engine, ReadKind::Status]
            .into_iter()
            .any(|kind| owner.ack_claim(kind).is_ok());
        props.insert(
            "jobsEntryDisabled".into(),
            Value::Bool(!failed && !ack_retry),
        );
        props.insert(
            "jobsEntryStatus".into(),
            Value::String(
                if failed {
                    "작업 대기열을 열지 못했습니다. 다시 시도해 주세요."
                } else {
                    "작업 대기열을 준비하고 있습니다."
                }
                .into(),
            ),
        );
        drop(owner);
        let receipt = self.commit_viewing_props(
            recovery.bindings().map_err(callback)?,
            props,
            self.viewing_generation,
        )?;
        self.callbacks
            .try_borrow()
            .map_err(|_| stale())?
            .projection
            .try_borrow_mut()
            .map_err(|_| stale())?
            .committed();
        Ok(receipt)
    }
    pub(in crate::assembly::meari::viewer_host) fn advance_jobs(
        &mut self,
        turn: &HostTurn,
    ) -> Result<Option<HostEffects>, ViewerHostError> {
        let result = self.advance_jobs_inner(turn);
        if let Ok(jobs) = self.jobs.try_borrow()
            && let Some(error) = &jobs.wake_error
        {
            self.last_error = Some(ViewerHostError::Sdk(error.clone()));
        }
        if let Err(error) = result {
            let cell = self.jobs.clone();
            let mut jobs = cell.try_borrow_mut().map_err(|_| stale())?;
            let effects = jobs.effects();
            if effects.committed.is_some() {
                self.last_error = Some(error);
                return Ok(Some(effects));
            }
            return Err(error);
        }
        result
    }
    fn advance_jobs_inner(
        &mut self,
        turn: &HostTurn,
    ) -> Result<Option<HostEffects>, ViewerHostError> {
        let cell = self.jobs.clone();
        let mut jobs = cell.try_borrow_mut().map_err(|_| stale())?;
        if turn.remaining_callbacks != 0 {
            let queued = self
                .callbacks
                .try_borrow()
                .is_ok_and(|owner| owner.has_jobs_entry());
            return Ok((queued || jobs.entry.is_some() || jobs.document.is_some())
                .then(HostEffects::default));
        }
        if jobs.entry.is_none() && jobs.document.is_none() {
            let mut owner = self.callbacks.try_borrow_mut().map_err(|_| stale())?;
            if !owner.has_jobs_entry() {
                return Ok(None);
            }
            if owner
                .card_selection_queue()
                .is_some_and(|queue| queue.try_borrow().is_ok_and(|queue| !queue.is_empty()))
            {
                return Ok(None);
            }
            owner.drain_local(&turn.stamp.owner).map_err(callback)?;
            let intent = owner
                .take_jobs_entry(&turn.stamp.owner)
                .map_err(callback)?
                .ok_or_else(stale)?;
            drop(owner);
            jobs.entry = Some(Entry {
                ticket: None,
                origin: turn.stamp.clone(),
                _intent: intent,
                phase: EntryPhase::Requested,
                recovery: None,
                recovery_live: false,
                queued: None,
                error: None,
                first_error: None,
                recovery_error: None,
            });
        }
        if let Some(mut entry) = jobs.entry.take() {
            let result = self.advance_jobs_entry(&mut jobs, &mut entry, turn);
            let observed = entry.phase == EntryPhase::Armed
                && self
                    .route_observation
                    .as_ref()
                    .and_then(|observer| observer.try_borrow().ok())
                    .is_some_and(|observer| {
                        observer
                            .state
                            .observed_route()
                            .is_some_and(|route| route.as_str() == "studio.jobs")
                    });
            if let Err(error) = &result {
                if matches!(error, ViewerHostError::ReadConsumerRetry(_)) {
                    jobs.entry = Some(entry);
                    return Err(error.clone());
                }
                self.last_error = Some(error.clone());
                if entry.recovery_live
                    && jobs.effect.is_none()
                    && !jobs
                        .document
                        .as_ref()
                        .is_some_and(JobsDocument::has_root_export)
                {
                    match self.recovery_view(entry.recovery.as_ref().unwrap(), true) {
                        Ok(receipt) => jobs.effect = Some(receipt),
                        Err(secondary) => {
                            entry.recovery_error.get_or_insert(secondary);
                        }
                    }
                }
            }
            if !observed {
                jobs.entry = Some(entry);
            }
            if observed {
                let document = jobs.document.as_mut().ok_or_else(stale)?;
                document.initial_read().map_err(callback)?;
                document.advance(0).map_err(callback)?;
            }
            return Ok(Some(jobs.effects()));
        }
        if jobs.returning.is_some() {
            self.advance_jobs_return(&mut jobs)?;
            return Ok(Some(jobs.effects()));
        }
        if let Some(document) = &mut jobs.document {
            document.advance(0).map_err(callback)?;
            if document.return_requested() {
                self.request_jobs_return(&mut jobs)?;
                self.advance_jobs_return(&mut jobs)?;
            }
            return Ok(Some(jobs.effects()));
        }
        Ok(None)
    }
    fn request_jobs_return(&mut self, jobs: &mut JobsLedger) -> Result<(), ViewerHostError> {
        if jobs.returning.is_some()
            || !jobs
                .document
                .as_ref()
                .is_some_and(JobsDocument::return_requested)
        {
            return Err(stale());
        }
        let mut observer = self
            .route_observation
            .as_ref()
            .ok_or_else(stale)?
            .try_borrow_mut()
            .map_err(|_| stale())?;
        if observer
            .state
            .observed_route()
            .is_none_or(|route| route.as_str() != "studio.jobs")
        {
            return Err(stale());
        }
        let ticket = observer
            .state
            .request_route(
                &RouteCatalog::bundled().map_err(|_| stale())?,
                "view.start",
                NavigationGuards::default(),
            )
            .map_err(ViewerHostError::Navigation)?;
        jobs.returning = Some(Returning {
            ticket,
            phase: ReturnPhase::Releasing,
            bindings: None,
            generation: None,
            error: None,
            first_error: None,
            committed: None,
            proof: None,
            proof_error: None,
            first_proof_error: None,
            retry_intent: None,
        });
        Ok(())
    }
    fn advance_jobs_entry(
        &mut self,
        jobs: &mut JobsLedger,
        entry: &mut Entry,
        turn: &HostTurn,
    ) -> Result<(), ViewerHostError> {
        let result = (|| {
            if let Some(pending) = entry.queued.as_ref() {
                entry
                    .recovery
                    .as_ref()
                    .ok_or_else(stale)?
                    .admit(pending, &turn.stamp.owner)
                    .map_err(callback)?;
                let name = pending.invocation.prop.as_str();
                if let Some(kind) = match name {
                    "onRetryCapabilitiesAck" => Some(ReadKind::Capabilities),
                    "onRetryEngineAck" => Some(ReadKind::Engine),
                    "onRetryStatusAck" => Some(ReadKind::Status),
                    _ => None,
                } {
                    let mut owner = self.callbacks.try_borrow_mut().map_err(|_| stale())?;
                    let claim = owner.ack_claim(kind).map_err(callback)?;
                    owner
                        .retry_read_acceptance(&claim.ticket)
                        .map_err(callback)?;
                } else if name != "onOpenJobs" {
                    return Err(ViewerHostError::UnsupportedCallback);
                }
                if name == "onOpenJobs" {
                    let mut owner = self.callbacks.try_borrow_mut().map_err(|_| stale())?;
                    for kind in [ReadKind::Capabilities, ReadKind::Engine, ReadKind::Status] {
                        if let Ok(claim) = owner.ack_claim(kind) {
                            owner
                                .retry_read_acceptance(&claim.ticket)
                                .map_err(callback)?;
                        }
                    }
                }
                entry.queued = None;
                entry.error = None;
                if let Some(document) = &mut jobs.document
                    && !document.has_entry_commit()
                {
                    document.retry().map_err(callback)?;
                }
            } else if entry.error.is_some() && !entry.recovery_live {
                let mut owner = self.callbacks.try_borrow_mut().map_err(|_| stale())?;
                if owner
                    .take_jobs_entry(&turn.stamp.owner)
                    .map_err(callback)?
                    .is_some()
                {
                    entry.error = None;
                }
            }
            if entry.error.is_some() {
                if let Some(document) = &mut jobs.document
                    && document.has_queued()
                {
                    document.advance(0).map_err(callback)?;
                    entry.error = None;
                } else {
                    return Ok(());
                }
            }
            if entry.phase == EntryPhase::Requested {
                if self.product_frame.is_none()
                    || self.document.as_ref().is_none_or(|document| {
                        document.stamp().ok().is_none_or(|current| {
                            current.host_id != entry.origin.host_id
                                || current.owner != entry.origin.owner
                                || current.definition_revision != entry.origin.definition_revision
                        })
                    })
                {
                    return Err(stale());
                }
                self.callbacks
                    .try_borrow()
                    .map_err(|_| stale())?
                    .verify_jobs_entry(&turn.stamp.owner, &entry._intent)
                    .map_err(callback)?;
                let mut observer = self
                    .route_observation
                    .as_ref()
                    .ok_or_else(stale)?
                    .try_borrow_mut()
                    .map_err(|_| stale())?;
                if observer
                    .state
                    .observed_route()
                    .is_none_or(|route| route.as_str() != "view.start")
                {
                    return Err(stale());
                }
                let cards = self.cards.try_borrow().map_err(|_| stale())?;
                if cards.active.is_some()
                    || cards.capture.is_some()
                    || cards.inspection.is_some()
                    || cards.opening.is_some()
                    || cards.capture_entry.is_some()
                {
                    return Err(stale());
                }
                if entry.ticket.is_none() {
                    entry.ticket = Some(
                        observer
                            .state
                            .request_route(
                                &RouteCatalog::bundled().map_err(|_| stale())?,
                                "studio.jobs",
                                NavigationGuards::default(),
                            )
                            .map_err(ViewerHostError::Navigation)?,
                    );
                }
                drop(cards);
                drop(observer);
                if entry.recovery.is_none() {
                    entry.recovery = Some(
                        RecoveryExport::viewing(entry.origin.owner.clone(), &self.viewing_props()?)
                            .map_err(callback)?,
                    );
                }
                jobs.effect = Some(self.recovery_view(entry.recovery.as_ref().unwrap(), false)?);
                entry.recovery_live = true;
                entry.phase = EntryPhase::Draining;
                jobs.continuation = true;
                return Ok(());
            }
            if entry.phase == EntryPhase::Draining {
                let mut owner = self.callbacks.try_borrow_mut().map_err(|_| stale())?;
                if owner
                    .prepare_route_release(&entry.origin.owner)
                    .map_err(callback)?
                    .is_some()
                {
                    owner.consume_reads().map_err(callback)?;
                    if owner
                        .prepare_route_release(&entry.origin.owner)
                        .map_err(callback)?
                        .is_none()
                    {
                        // Continue to the same-handle attach below in this original host turn.
                    } else {
                        let dirty = owner.projection.try_borrow().map_err(|_| stale())?.dirty;
                        let delay = owner.next_consumer_retry();
                        drop(owner);
                        if dirty {
                            jobs.effect =
                                Some(self.recovery_view(entry.recovery.as_ref().unwrap(), false)?);
                            jobs.continuation = true;
                            return Ok(());
                        }
                        if let Some(delay) = delay {
                            return Err(ViewerHostError::ReadConsumerRetry(delay));
                        }
                        return Ok(());
                    }
                }
                let execution = owner.original_execution();
                let context = owner.original_context();
                drop(owner);
                if jobs.document.is_none() {
                    let next = jobs
                        .sequence
                        .checked_add(1)
                        .and_then(NonZeroU64::new)
                        .ok_or_else(stale)?;
                    let product_name = match self.base_props.get("productName") {
                        Some(Value::String(value)) => value.clone(),
                        _ => return Err(stale()),
                    };
                    jobs.document = Some(
                        JobsDocument::new(JobsMountInput {
                            document: self.document.as_ref().ok_or_else(stale)?.clone(),
                            frame: self.product_frame.as_ref().ok_or_else(stale)?.clone(),
                            execution,
                            context,
                            origin: self
                                .document
                                .as_ref()
                                .unwrap()
                                .stamp()
                                .map_err(|_| stale())?,
                            generation: next,
                            product_name,
                            context_label: "작업 대기열".into(),
                        })
                        .with_caption_source(Rc::downgrade(&self.window_ui.captions)),
                    );
                    jobs.sequence = next.get();
                }
                entry.phase = EntryPhase::Committing;
            }
            if entry.phase == EntryPhase::Committing {
                let document = jobs.document.as_mut().ok_or_else(stale)?;
                if !document.has_entry_commit() && !document.prepare_entry().map_err(callback)? {
                    return Ok(());
                }
                let receipt = document.receipt().ok_or_else(stale)?;
                let current = self
                    .document
                    .as_ref()
                    .ok_or_else(stale)?
                    .stamp()
                    .map_err(|_| stale())?;
                self.route_observation
                    .as_ref()
                    .ok_or_else(stale)?
                    .try_borrow_mut()
                    .map_err(|_| stale())?
                    .arm_jobs_route(
                        entry.ticket.as_ref().ok_or_else(stale)?.clone(),
                        receipt,
                        &current,
                    )
                    .map_err(ViewerHostError::Navigation)?;
                entry.phase = EntryPhase::Armed;
            }
            Ok(())
        })();
        if let Err(error) = &result
            && !matches!(error, ViewerHostError::ReadConsumerRetry(_))
        {
            entry.first_error.get_or_insert_with(|| error.clone());
            entry.error = Some(error.clone());
        }
        result
    }
    fn advance_jobs_return(&mut self, jobs: &mut JobsLedger) -> Result<(), ViewerHostError> {
        let mut request = jobs.returning.take().ok_or_else(stale)?;
        let result = (|| {
            let document = jobs.document.as_mut().ok_or_else(stale)?;
            if matches!(
                request.phase,
                ReturnPhase::CommittedAwaitingArm | ReturnPhase::Observed
            ) {
                let mut owner = self.callbacks.try_borrow_mut().map_err(|_| stale())?;
                if owner.has_jobs_entry() {
                    let scope = self
                        .document
                        .as_ref()
                        .ok_or_else(stale)?
                        .stamp()
                        .map_err(|_| stale())?
                        .owner;
                    request.retry_intent = owner.take_jobs_entry(&scope).map_err(callback)?;
                    if request.retry_intent.is_none() {
                        return Err(stale());
                    }
                    request.error = None;
                }
            } else if document.has_queued() {
                document.process_route_recovery().map_err(callback)?;
                request.error = None;
            }
            if request.error.is_some() {
                return Ok(false);
            }
            let current = self
                .document
                .as_ref()
                .ok_or_else(stale)?
                .stamp()
                .map_err(|_| stale())?;
            if request.phase == ReturnPhase::Releasing {
                let released = document.prepare_route_release().map_err(callback)?;
                if released {
                    request.phase = ReturnPhase::Exporting;
                }
                if !released || document.has_pending_effect() {
                    return Ok(false);
                }
            }
            if request.phase == ReturnPhase::Exporting {
                let mut owner = self.callbacks.try_borrow_mut().map_err(|_| stale())?;
                if !owner.is_released()
                    && owner
                        .prepare_route_release(&current.owner)
                        .map_err(callback)?
                        .is_some()
                {
                    owner.consume_reads().map_err(callback)?;
                    return Ok(false);
                }
                request.bindings = Some(
                    owner
                        .resume_released_route(&current.owner)
                        .map_err(callback)?,
                );
                request.phase = ReturnPhase::Committing;
            }
            if request.phase == ReturnPhase::Committing {
                let generation = match request.generation {
                    Some(value) => value,
                    None => {
                        let value = self
                            .viewing_generation
                            .get()
                            .checked_add(1)
                            .and_then(NonZeroU64::new)
                            .ok_or_else(stale)?;
                        request.generation = Some(value);
                        value
                    }
                };
                let bindings = match request.bindings.take() {
                    Some(value) => value,
                    None => self
                        .callbacks
                        .try_borrow()
                        .map_err(|_| stale())?
                        .bindings()
                        .map_err(callback)?,
                };
                let mut props = self.viewing_props()?;
                props.insert("jobsEntryDisabled".into(), Value::Bool(false));
                props.insert(
                    "jobsEntryLabel".into(),
                    Value::String("복귀 연결 다시 확인".into()),
                );
                props.insert(
                    "jobsEntryStatus".into(),
                    Value::String(
                        "복귀 프레임 연결을 확인하고 있습니다. 연결에 실패하면 다시 확인해 주세요."
                            .into(),
                    ),
                );
                for name in [
                    "capabilitiesBusy",
                    "engineBusy",
                    "statusBusy",
                    "capabilitiesRetryDisabled",
                    "engineRetryDisabled",
                    "statusRetryDisabled",
                ] {
                    props.insert(name.into(), Value::Bool(true));
                }
                let receipt = self.commit_viewing_props(bindings, props, generation)?;
                self.viewing_generation = generation;
                let committed = receipt.after().clone();
                request.committed = Some((receipt.clone(), committed));
                request.phase = ReturnPhase::CommittedAwaitingArm;
                jobs.effect = Some(receipt.clone());
                let proof = self
                    .document
                    .as_ref()
                    .ok_or_else(stale)?
                    .retain_commit(&receipt)
                    .map_err(|error| request.proof_failure(error))?;
                request.proof = Some(proof);
                #[cfg(test)]
                if let Some(probe) = crate::assembly::meari::viewer_native_probe::current() {
                    probe.jobs_commit(
                        self.document.as_ref().unwrap(),
                        request.proof.as_ref().unwrap(),
                    );
                }
            }
            if request.phase == ReturnPhase::CommittedAwaitingArm {
                let (receipt, committed) = request.committed.as_ref().ok_or_else(stale)?;
                let proof = request.proof.as_ref().ok_or_else(stale)?.clone();
                if proof.receipt() != receipt || proof.receipt().after() != committed {
                    return Err(stale());
                }
                let handle = self.document.as_ref().ok_or_else(stale)?.clone();
                handle
                    .validate_retained_commit(&proof)
                    .map_err(|error| request.proof_failure(error))?;
                self.route_observation
                    .as_ref()
                    .ok_or_else(stale)?
                    .try_borrow_mut()
                    .map_err(|_| stale())?
                    .arm_retained_viewing_route(request.ticket.clone(), handle, proof)
                    .map_err(ViewerHostError::Navigation)?;
                request.proof_error = None;
                request.phase = ReturnPhase::Armed;
                jobs.continuation = true;
                jobs.redraw = true;
            }
            let observed = request.phase == ReturnPhase::Armed
                && self
                    .route_observation
                    .as_ref()
                    .and_then(|observer| observer.try_borrow().ok())
                    .is_some_and(|observer| {
                        observer
                            .state
                            .observed_route()
                            .is_some_and(|route| route.as_str() == "view.start")
                    });
            if observed {
                request.phase = ReturnPhase::Observed;
            }
            if request.phase == ReturnPhase::Observed {
                if !document.close().map_err(callback)? {
                    return Ok(false);
                }
                // Only after checked frame observation, restore normal Viewing controls.
                let bindings = self
                    .callbacks
                    .try_borrow()
                    .map_err(|_| stale())?
                    .bindings()
                    .map_err(callback)?;
                let receipt = self.commit_viewing_props(
                    bindings,
                    self.viewing_props()?,
                    self.viewing_generation,
                )?;
                jobs.effect = Some(receipt);
                return Ok(true);
            }
            Ok(false)
        })();
        if let Err(error) = &result {
            request.first_error.get_or_insert_with(|| error.clone());
            request.error = Some(error.clone());
        }
        if matches!(result, Ok(true)) {
            jobs.completed = jobs.document.take();
            #[cfg(test)]
            if let Some(probe) = crate::assembly::meari::viewer_native_probe::current() {
                probe.jobs_owners_released();
            }
        } else {
            jobs.returning = Some(request);
        }
        // Retain errors with their original owners, while still delivering any actual
        // commit receipt produced before arm failed. Recovery requires a fresh intent.
        if let Err(error) = result {
            self.last_error = Some(error);
        }
        Ok(())
    }
}
fn local_error(error: meari::application::host_callbacks::LocalDispatchError) -> PluginError {
    match error {
        meari::application::host_callbacks::LocalDispatchError::Invalid(error) => error,
        _ => PluginError::new(PluginErrorCode::Stale, "Jobs entry is not local"),
    }
}

#[cfg(test)]
#[path = "tests/product_route.rs"]
mod product_route_tests;
