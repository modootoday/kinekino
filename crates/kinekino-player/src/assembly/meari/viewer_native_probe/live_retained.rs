//! Opt-in checked-window fixture. Input is technical source dispatch, never OS input.
use super::*;
mod commit_journey;
use crate::application::{api::ApiError, sdk_execution::CompletionNotifier};
use meari_native::infrastructure::{
    native_host::{HostAccessError, HostCommitReceipt, NativeDocumentHandle, RetainedCommitProof},
    window::NativeWindowNotice,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) enum Case {
    Card,
    Highlight,
    Cue,
    Document,
}
impl Case {
    pub(crate) fn route(self) -> &'static str {
        match self {
            Self::Card => "studio.cards",
            Self::Highlight => "studio.shorts.recommendations",
            _ => "studio.subtitles.correction",
        }
    }
    fn binding(self) -> &'static str {
        match self {
            Self::Card => "onFrontChange",
            Self::Highlight => "onTitleChange",
            _ => "onDraftChange",
        }
    }
    fn original(self) -> &'static str {
        match self {
            Self::Card | Self::Highlight => "second",
            _ => "둘째 수정",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) enum Mode {
    Positive,
    SourceReturn,
    ArmBusy,
    WakeFailure,
    EntryProofModel,
    ReturnProofModel,
    DefaultChooser,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Initial,
    Opening,
    AwaitingActivation,
    Editing,
    Restoring,
    Saving,
    ReturnReady,
    Returning,
    ViewingObserved,
    Complete,
    Abort,
    ChooserDocuments,
    ChooserItems,
    ChooserOpen,
}
pub(super) struct LiveTrace {
    save_snapshot: [Option<(Phase, SaveSnapshot)>; 2],
    save_snapshots: usize,
    save_observation_pending: bool,
    save_observation_woken: bool,
    source_action_sent: bool,
    source_redraw_sent: bool,
    model_cancel_attempted: bool,
    model_cancel_recorded: bool,
    model_cancel_error: Option<crate::assembly::meari::viewer_host::ViewerHostError>,
    case: Case,
    mode: Mode,
    phase: Phase,
    entry: Option<(NativeDocumentHandle, RetainedCommitProof)>,
    returning: Option<(NativeDocumentHandle, RetainedCommitProof)>,
    active: bool,
    dirty: bool,
    pending: bool,
    arm_fault_taken: bool,
    wake_error_seen: bool,
    arm_recovered: bool,
    notifier_installed: bool,
    retry_wake_sent: bool,
    counts: Arc<NotifyCounts>,
    chooser_cycle: u8,
    chooser_old_keys: Vec<String>,
    chooser_active_recorded: bool,
    chooser_viewing: Option<HostStamp>,
    chooser_fresh_redraw_pending: bool,
    chooser_viewing_ticket: Option<String>,
    chooser_actual_item_ref: Option<crate::domain::proposals::DocumentTextRef>,
    persisted: Option<crate::domain::proposals::StoredProposal>,
    input: Option<CurrentInput>,
    journey: Option<commit_journey::CommitJourney>,
}
struct CurrentInput {
    expected: crate::domain::proposals::StoredProposal,
    progress: InputProgress,
    first_text_started: bool,
    diagnostic_seen: u16,
}
#[derive(Clone, Copy)]
enum InputProgress {
    Initial,
    Changed(usize),
    Restored(usize),
    Saving,
    Saved,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SaveSnapshot {
    pub stage: &'static str,
    pub active: bool,
    pub dirty: Option<bool>,
    pub pending: Option<bool>,
    pub recovery: Option<bool>,
    pub transport: Option<bool>,
    pub token: Option<bool>,
    pub confirmed: Option<bool>,
    pub projection: Option<bool>,
    pub revision: Option<u64>,
    pub remaining: usize,
    pub reason: meari_native::infrastructure::native_host::HostTurnReason,
}
#[derive(Default)]
struct NotifyCounts {
    calls: AtomicUsize,
    delegated: AtomicUsize,
    successful: AtomicUsize,
    errors: Mutex<Vec<NotificationErrorSnapshot>>,
}
/// Evidence copy only. The original owned ApiError is returned to the original host caller.
#[derive(Serialize)]
struct NotificationErrorSnapshot {
    code: crate::application::api::ErrorCode,
    message: String,
    retryable: bool,
    details: serde_json::Value,
    remediation: String,
    request_id: String,
}
impl From<&ApiError> for NotificationErrorSnapshot {
    fn from(error: &ApiError) -> Self {
        Self {
            code: error.code,
            message: error.message.clone(),
            retryable: error.retryable,
            details: error.details.clone(),
            remediation: error.remediation.clone(),
            request_id: error.request_id.clone(),
        }
    }
}
struct OriginalNotifier {
    original: Arc<dyn CompletionNotifier>,
    counts: Arc<NotifyCounts>,
    fail_once: AtomicBool,
}
impl CompletionNotifier for OriginalNotifier {
    fn notify(&self) -> Result<(), ApiError> {
        self.counts.calls.fetch_add(1, Ordering::AcqRel);
        if self.fail_once.swap(false, Ordering::AcqRel) {
            let error = ApiError::new(
                crate::application::api::ErrorCode::Internal,
                "fixture original route notifier fail-once",
            );
            if let Ok(mut errors) = self.counts.errors.lock() {
                errors.push(NotificationErrorSnapshot::from(&error));
            }
            return Err(error);
        }
        self.counts.delegated.fetch_add(1, Ordering::AcqRel);
        match self.original.notify() {
            Ok(()) => {
                self.counts.successful.fetch_add(1, Ordering::AcqRel);
                Ok(())
            }
            Err(error) => {
                if let Ok(mut errors) = self.counts.errors.lock() {
                    errors.push(NotificationErrorSnapshot::from(&error));
                }
                Err(error)
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) enum ProofStage {
    Entry,
    Return,
}
thread_local! { static PROOF_FAULT: Cell<Option<ProofStage>> = const { Cell::new(None) }; }
pub(crate) struct ModelProofScope {
    previous: Option<ProofStage>,
    _thread: std::marker::PhantomData<Rc<()>>,
}
impl ModelProofScope {
    pub(crate) fn enter(stage: Option<ProofStage>) -> Self {
        Self {
            previous: PROOF_FAULT.with(|fault| fault.replace(stage)),
            _thread: std::marker::PhantomData,
        }
    }
}
impl Drop for ModelProofScope {
    fn drop(&mut self) {
        PROOF_FAULT.with(|fault| fault.set(self.previous));
    }
}
pub(crate) fn retain(
    document: &NativeDocumentHandle,
    receipt: &HostCommitReceipt,
    stage: ProofStage,
) -> Result<RetainedCommitProof, HostAccessError> {
    let injected = PROOF_FAULT.with(|fault| {
        if fault.get() == Some(stage) {
            fault.set(None);
            true
        } else {
            false
        }
    });
    if injected {
        if let Some(probe) = current() {
            probe.record(
                Kind::LiveModelProofError,
                &(
                    stage,
                    stamp(receipt.after()),
                    "model Err(Busy), not a Meari fault",
                ),
            );
        }
        Err(HostAccessError::Busy)
    } else {
        document.retain_commit(receipt)
    }
}

impl Probe {
    pub(crate) fn for_live(case: Case, mode: Mode) -> Rc<Self> {
        let probe = Self::new();
        *probe.live.borrow_mut() = Some(LiveTrace {
            save_snapshot: [None; 2],
            save_snapshots: 0,
            save_observation_pending: false,
            save_observation_woken: false,
            source_action_sent: false,
            source_redraw_sent: false,
            model_cancel_attempted: false,
            model_cancel_recorded: false,
            model_cancel_error: None,
            case,
            mode,
            phase: Phase::Initial,
            entry: None,
            returning: None,
            active: false,
            dirty: false,
            pending: false,
            arm_fault_taken: false,
            wake_error_seen: false,
            arm_recovered: false,
            notifier_installed: false,
            retry_wake_sent: false,
            counts: Arc::new(NotifyCounts::default()),
            chooser_cycle: 0,
            chooser_old_keys: Vec::new(),
            chooser_active_recorded: false,
            chooser_viewing: None,
            chooser_fresh_redraw_pending: false,
            chooser_viewing_ticket: None,
            chooser_actual_item_ref: None,
            persisted: None,
            input: None,
            journey: None,
        });
        probe
    }
    pub(crate) fn is_live(&self) -> bool {
        self.live.try_borrow().is_ok_and(|live| live.is_some())
    }
    pub(crate) fn configure_default_input(
        &self,
        expected: crate::domain::proposals::StoredProposal,
    ) -> Result<(), String> {
        expected.validate().map_err(|error| format!("{error:?}"))?;
        let mut state = self
            .live
            .try_borrow_mut()
            .map_err(|_| "input trace borrowed")?;
        let live = state.as_mut().ok_or("input live trace missing")?;
        if live.mode != Mode::DefaultChooser
            || live.phase != Phase::Initial
            || live.persisted.is_some()
            || live.input.is_some()
            || live.journey.is_some()
            || expected.acceptance.is_some()
            || expected.document.proposal_id
                != if live.case == Case::Document {
                    "journey-b"
                } else {
                    "journey-a"
                }
        {
            return Err(
                "input fixture requires fresh default chooser and original editable seed".into(),
            );
        }
        current_input_fields(&expected, live.case)?;
        live.input = Some(CurrentInput {
            expected,
            progress: InputProgress::Initial,
            first_text_started: false,
            diagnostic_seen: 0,
        });
        Ok(())
    }
    pub(crate) fn configure_persisted_reopen(
        &self,
        stored: crate::domain::proposals::StoredProposal,
    ) -> Result<(), String> {
        let mut state = self
            .live
            .try_borrow_mut()
            .map_err(|_| "persisted probe borrowed")?;
        let live = state.as_mut().ok_or("persisted probe missing")?;
        if live.mode != Mode::DefaultChooser
            || live.phase != Phase::Initial
            || live.persisted.is_some()
        {
            return Err("persisted probe is not a fresh default chooser".into());
        }
        stored.validate().map_err(str::to_owned)?;
        live.persisted = Some(stored);
        Ok(())
    }
    pub(crate) fn uses_explicit_live_fixture_source(&self) -> bool {
        self.live.try_borrow().is_ok_and(|live| {
            live.as_ref()
                .is_some_and(|live| live.mode != Mode::DefaultChooser)
        })
    }
    pub(crate) fn take_live_model_cancel(&self, stage: ProofStage) -> bool {
        let mut live = self.live.borrow_mut();
        let Some(live) = live.as_mut() else {
            return false;
        };
        if live.phase != Phase::Abort
            || live.model_cancel_attempted
            || !matches!(
                (stage, live.mode),
                (ProofStage::Entry, Mode::EntryProofModel)
                    | (ProofStage::Return, Mode::ReturnProofModel)
            )
        {
            return false;
        }
        live.model_cancel_attempted = true;
        true
    }
    pub(crate) fn live_model_cancelled(
        &self,
        stage: ProofStage,
        result: Result<(), crate::assembly::meari::viewer_host::ViewerHostError>,
    ) {
        let ok = result.is_ok();
        let duplicate = {
            let mut live = self.live.borrow_mut();
            let Some(live) = live.as_mut() else {
                return;
            };
            if live.model_cancel_recorded {
                true
            } else {
                live.model_cancel_recorded = true;
                live.model_cancel_error = result.err();
                false
            }
        };
        if duplicate {
            self.live_fault("duplicate model route cancellation".into());
            return;
        }
        self.record(Kind::LiveModelRouteCancelled, &(stage, ok));
        self.checkpoint(&format!("model route cancellation stage={stage:?} ok={ok}"));
    }
    pub(crate) fn take_live_cancel_error(
        &self,
    ) -> Option<crate::assembly::meari::viewer_host::ViewerHostError> {
        self.live
            .borrow_mut()
            .as_mut()
            .and_then(|live| live.model_cancel_error.take())
    }
    /// Refresh only the input observer cache from this same active owner's post-dispatch guards.
    pub(crate) fn input_after_dispatch_guards(&self, snapshot: &SaveSnapshot) {
        let result = (|| -> Result<(), String> {
            let mut state = self
                .live
                .try_borrow_mut()
                .map_err(|_| "input guard trace borrowed")?;
            let Some(live) = state
                .as_mut()
                .filter(|live| live.input.is_some() || live.journey.is_some())
            else {
                return Ok(());
            };
            if snapshot.stage == "after"
                && snapshot.active
                && let (Some(dirty), Some(pending)) = (snapshot.dirty, snapshot.pending)
            {
                live.dirty = dirty;
                live.pending = pending;
                if let Some(journey) = live.journey.as_mut() {
                    journey.observe_guards(snapshot);
                }
            }
            Ok(())
        })();
        if let Err(error) = result {
            self.live_fault(error);
        }
    }
    /// Clean state admits one later observation wake, never a route transition.
    pub(crate) fn live_save_after_dispatch(&self, snapshot: &SaveSnapshot) {
        let result = (|| -> Result<(), String> {
            let mut live = self
                .live
                .try_borrow_mut()
                .map_err(|_| "save observation trace borrowed")?;
            let Some(live) = live.as_mut() else {
                return Ok(());
            };
            if live.phase == Phase::Saving
                && !live.save_observation_woken
                && snapshot.stage == "after"
                && snapshot.active
                && snapshot.dirty == Some(false)
                && snapshot.pending == Some(false)
                && snapshot.recovery == Some(false)
                && snapshot.transport == Some(false)
                && snapshot.token == Some(false)
                && snapshot.confirmed == Some(false)
                && snapshot.projection == Some(false)
                && snapshot.remaining == 0
            {
                live.save_observation_pending = true;
            }
            Ok(())
        })();
        if let Err(error) = result {
            self.live_fault(error);
        }
    }
    pub(crate) fn live_saving_checkpoint(&self, snapshot: SaveSnapshot) {
        let message = {
            let Ok(mut live) = self.live.try_borrow_mut() else {
                return;
            };
            let Some(live) = live.as_mut() else {
                return;
            };
            let stage = match snapshot.stage {
                "before" => 0,
                "after" => 1,
                _ => return,
            };
            if !matches!(live.phase, Phase::Saving | Phase::ReturnReady)
                || live.save_snapshots == 8
                || live.save_snapshot[stage] == Some((live.phase, snapshot))
            {
                return;
            }
            live.save_snapshots += 1;
            live.save_snapshot[stage] = Some((live.phase, snapshot));
            format!("save phase={:?} {snapshot:?}", live.phase)
        };
        self.checkpoint(&message);
    }
    pub(crate) fn live_notifier(
        &self,
        original: Arc<dyn CompletionNotifier>,
    ) -> Arc<dyn CompletionNotifier> {
        let mut live = self.live.borrow_mut();
        let live = live.as_mut().expect("opt-in live fixture only");
        if live.notifier_installed {
            return original;
        }
        live.notifier_installed = true;
        Arc::new(OriginalNotifier {
            original,
            counts: live.counts.clone(),
            fail_once: AtomicBool::new(live.mode == Mode::WakeFailure),
        })
    }
    pub(crate) fn live_notifications(&self) -> Option<serde_json::Value> {
        let live = self.live.try_borrow().ok()?;
        let live = live.as_ref()?;
        let errors = live.counts.errors.lock().ok()?;
        Some(
            serde_json::json!({"arm_calls":live.counts.calls.load(Ordering::Acquire),"original_delegate_attempts":live.counts.delegated.load(Ordering::Acquire),"original_delegate_ok":live.counts.successful.load(Ordering::Acquire),"errors":&*errors,"excludes_original_inspect_continuation":true,"separate_observation_retry_wake":live.retry_wake_sent}),
        )
    }
    pub(crate) fn take_live_arm_busy(&self) -> bool {
        let Ok(mut live) = self.live.try_borrow_mut() else {
            self.fail(Fault::Reentrant);
            return false;
        };
        let Some(live) = live.as_mut() else {
            return false;
        };
        if live.mode != Mode::ArmBusy || live.arm_fault_taken {
            return false;
        }
        live.arm_fault_taken = true;
        true
    }
    pub(crate) fn live_arm_busy(&self, actual_busy: bool) {
        self.record(Kind::LiveArmBusy, &actual_busy);
    }
    pub(crate) fn live_commit(
        &self,
        document: &NativeDocumentHandle,
        receipt: &HostCommitReceipt,
        proof: Option<&RetainedCommitProof>,
        proof_error: Option<&HostAccessError>,
        stage: ProofStage,
    ) {
        let result = (|| -> Result<(), String> {
            let mut live = self
                .live
                .try_borrow_mut()
                .map_err(|_| "live trace borrowed")?;
            let Some(live) = live.as_mut() else {
                return Ok(());
            };
            let target = match stage {
                ProofStage::Entry => &mut live.entry,
                ProofStage::Return => &mut live.returning,
            };
            if target.is_some() {
                return Err("duplicate original commit".into());
            }
            if document
                .stamp()
                .map_err(|error| format!("{error:?}"))?
                .ne(receipt.after())
            {
                return Err("original immediate commit is not current".into());
            }
            if let Some(proof) = proof {
                if live.mode == Mode::DefaultChooser
                    && live.chooser_cycle == 1
                    && stage == ProofStage::Entry
                {
                    let viewing = live
                        .chooser_viewing
                        .as_ref()
                        .ok_or("fresh entry missing first Viewing frame")?;
                    if receipt.after().host_id != viewing.host_id
                        || receipt.after().document_revision <= viewing.document_revision
                    {
                        return Err(
                            "second entry did not advance the original document after Viewing"
                                .into(),
                        );
                    }
                }
                if proof_error.is_some() {
                    return Err("proof and proof error overlap".into());
                }
                if proof.receipt() != receipt {
                    return Err("original proof/receipt differs".into());
                }
                *target = Some((document.clone(), proof.clone()));
            } else {
                let expected = matches!(
                    (stage, live.mode),
                    (ProofStage::Entry, Mode::EntryProofModel)
                        | (ProofStage::Return, Mode::ReturnProofModel)
                );
                if !expected || !matches!(proof_error, Some(HostAccessError::Busy)) {
                    return Err("original immediate proof missing".into());
                }
                live.phase = Phase::Abort;
            }
            self.record(
                match stage {
                    ProofStage::Entry => Kind::LiveEntryCommitted,
                    ProofStage::Return => Kind::LiveReturnCommitted,
                },
                &(
                    live.case,
                    stamp(receipt.before()),
                    stamp(receipt.after()),
                    proof.is_some(),
                ),
            );
            Ok(())
        })();
        if let Err(error) = result {
            self.live_fault(error);
        }
    }
    pub(crate) fn chooser_active_identity(
        &self,
        selected: &crate::application::launch::native_proposal::NativeProposalSelection,
        actual: &crate::domain::proposals::StoredProposal,
        actual_document_ref: Option<&crate::domain::proposals::DocumentTextRef>,
    ) {
        let result = (|| -> Result<(), String> {
            let mut state = self
                .live
                .try_borrow_mut()
                .map_err(|_| "chooser identity trace borrowed")?;
            let Some(live) = state
                .as_mut()
                .filter(|live| live.mode == Mode::DefaultChooser)
            else {
                return Ok(());
            };
            if live.chooser_active_recorded {
                return Ok(());
            }
            let doc = &actual.document;
            use crate::application::launch::native_proposal::NativeProposalTarget;
            let expected_index = if live.chooser_cycle == 0 { 1 } else { 0 };
            let target_matches = if let Some(expected) = live.persisted.as_ref() {
                if serde_json::to_value(actual).map_err(|error| format!("{error:?}"))?
                    != serde_json::to_value(expected).map_err(|error| format!("{error:?}"))?
                {
                    return Err(
                        "actual active review differs from original persisted fixture".into(),
                    );
                }
                persisted_target_matches(&selected.target, expected)
            } else if let Some(journey) = live.journey.as_ref() {
                let expected = journey.entry_snapshot(live.chooser_cycle)?;
                if serde_json::to_value(actual).map_err(|error| format!("{error:?}"))?
                    != serde_json::to_value(expected).map_err(|error| format!("{error:?}"))?
                {
                    return Err(
                        "journey active snapshot differs from original SDK inspection".into(),
                    );
                }
                current_input_target_matches(&selected.target, expected, live.case)
            } else if let Some(input) = live.input.as_ref() {
                if serde_json::to_value(actual).map_err(|error| format!("{error:?}"))?
                    != serde_json::to_value(&input.expected)
                        .map_err(|error| format!("{error:?}"))?
                {
                    return Err(
                        "input active snapshot differs from original seed inspection".into(),
                    );
                }
                current_input_target_matches(&selected.target, &input.expected, live.case)
            } else {
                match (&selected.target, live.case, live.chooser_cycle) {
                    (NativeProposalTarget::Card { index }, Case::Card, _) => {
                        *index == expected_index
                    }
                    (NativeProposalTarget::Highlight { index }, Case::Highlight, _) => {
                        *index == expected_index
                    }
                    (NativeProposalTarget::CueCorrection { index, cue_id }, Case::Cue, 0) => {
                        *index == 1 && cue_id == "b"
                    }
                    (
                        NativeProposalTarget::DocumentCorrection { index, source_ref },
                        Case::Cue,
                        1,
                    ) => {
                        *index == 0
                            && matches!(&doc.source, crate::domain::proposals::SourceSnapshot::Document(source) if source.text_reference("/0").as_ref() == Some(source_ref))
                    }
                    _ => false,
                }
            };
            let expected_id = live
                .persisted
                .as_ref()
                .map(|stored| stored.document.proposal_id.as_str())
                .or_else(|| {
                    live.input
                        .as_ref()
                        .map(|input| input.expected.document.proposal_id.as_str())
                })
                .or_else(|| live.journey.as_ref().map(|journey| journey.proposal_id()))
                .unwrap_or(if live.chooser_cycle == 0 {
                    "journey-a"
                } else {
                    "journey-b"
                });
            if !target_matches || selected.proposal_id != expected_id {
                return Err("actual entry selected an unexpected document or typed item".into());
            }
            if actual.validate().is_err()
                || (live.persisted.is_none()
                    && live.journey.is_none()
                    && actual.acceptance.is_some())
                || selected.proposal_id != doc.proposal_id
                || selected.document_revision != doc.draft_revision
                || selected.source_id != doc.source.source_id()
                || selected.source_revision != doc.source.source_revision()
                || selected.source_sha256 != doc.source.source_sha256()
            {
                return Err("actual active review differs from original selected identity".into());
            }
            let actual_item_ref =
                if let NativeProposalTarget::DocumentCorrection { index, source_ref } =
                    &selected.target
                {
                    let index = usize::try_from(*index)
                        .map_err(|_| "actual document item index overflows")?;
                    let Some(crate::domain::proposals::ProposalPayload::DocumentCorrection {
                        edits,
                    }) = doc.draft.as_ref()
                    else {
                        return Err("actual active document correction draft missing".into());
                    };
                    let edit = edits
                        .get(index)
                        .ok_or("actual active document correction item missing")?;
                    if &edit.source_ref != source_ref {
                        return Err("actual active draft item ref differs from selected ref".into());
                    }
                    Some(edit.source_ref.clone())
                } else {
                    None
                };
            live.chooser_actual_item_ref = actual_item_ref.clone();
            live.chooser_active_recorded = true;
            self.record(Kind::ChooserActiveIdentity, &serde_json::json!({
                "cycle":live.chooser_cycle,"input_selection":selected,
                "actual_review":{"proposal_id":doc.proposal_id,"document_revision":doc.draft_revision,
                "source_id":doc.source.source_id(),"source_revision":doc.source.source_revision(),
                "source_sha256":doc.source.source_sha256(),"repair_document_selection":actual_document_ref,
                "selected_draft_item_ref":actual_item_ref}
            }));
            Ok(())
        })();
        if let Err(error) = result {
            self.live_fault(error);
        }
    }
    pub(crate) fn live_host_state(
        &self,
        active: bool,
        dirty: bool,
        pending: bool,
        observed: bool,
        gate: bool,
        wake_error: Option<&ApiError>,
    ) {
        let result = (|| -> Result<(), String> {
            let mut live = self
                .live
                .try_borrow_mut()
                .map_err(|_| "live trace borrowed")?;
            let Some(live) = live.as_mut() else {
                return Ok(());
            };
            if active && (!observed || !gate) {
                return Err("editor active before original entry observation".into());
            }
            if active && !live.active {
                self.record(Kind::LiveActivated, &(live.case, observed, gate));
            }
            live.active = active;
            live.dirty = dirty;
            live.pending = pending;
            if live.phase == Phase::Saving && active && !dirty && !pending {
                self.record(Kind::LiveSaveSettled, &(active, dirty, pending));
                live.phase = Phase::ReturnReady;
            }
            if let Some(error) = wake_error {
                if live.mode != Mode::WakeFailure
                    || error.code != crate::application::api::ErrorCode::Internal
                    || error.message != "fixture original route notifier fail-once"
                {
                    return Err("unexpected original typed route wake error".into());
                }
                if !live.wake_error_seen {
                    live.wake_error_seen = true;
                    self.record(
                        Kind::LiveWakeError,
                        &(format!("{:?}", error.code), &error.message),
                    );
                }
            }
            Ok(())
        })();
        if let Err(error) = result {
            self.live_fault(error);
        }
    }
    pub(crate) fn live_error_state(
        &self,
        first: Option<&crate::assembly::meari::viewer_host::ViewerHostError>,
        latest: Option<&crate::assembly::meari::viewer_host::ViewerHostError>,
    ) {
        let result = (|| -> Result<(), String> {
            let mut live = self
                .live
                .try_borrow_mut()
                .map_err(|_| "live trace borrowed")?;
            let Some(live) = live.as_mut() else {
                return Ok(());
            };
            if live.mode == Mode::ArmBusy && live.active && !live.arm_recovered {
                if !matches!(
                    first,
                    Some(crate::assembly::meari::viewer_host::ViewerHostError::UnsupportedCallback)
                ) || latest.is_some()
                {
                    return Err("arm recovery lost first error or retained a new failure".into());
                }
                live.arm_recovered = true;
                self.record(Kind::LiveArmRecovered, &true);
            }
            Ok(())
        })();
        if let Err(error) = result {
            self.live_fault(error);
        }
    }
    pub(crate) fn live_return_ready(&self) -> bool {
        self.live.try_borrow().is_ok_and(|live| {
            live.as_ref().is_some_and(|live| {
                !matches!(live.mode, Mode::SourceReturn | Mode::DefaultChooser)
                    && live.phase == Phase::ReturnReady
                    && live.active
                    && !live.dirty
                    && !live.pending
            })
        })
    }
    pub(crate) fn live_return_requested(&self) {
        if let Ok(mut live) = self.live.try_borrow_mut()
            && let Some(live) = live.as_mut()
        {
            live.phase = Phase::Returning;
            self.record(
                Kind::LiveHostReturnRequested,
                &"original typed host request; not a source navigation callback",
            );
        }
    }
    pub(crate) fn live_source_return_requested(&self) {
        let valid = {
            let mut live = self.live.borrow_mut();
            if let Some(live) = live.as_mut() {
                if matches!(live.mode, Mode::SourceReturn | Mode::DefaultChooser)
                    && live.phase == Phase::ReturnReady
                    && live.source_action_sent
                {
                    live.phase = Phase::Returning;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };
        if valid {
            self.record(Kind::LiveSourceReturnRequested, &true);
        } else {
            self.live_fault("Source return did not follow its actual source action".into());
        }
    }
    pub(crate) fn source_return_redraw(&self) -> bool {
        let fresh = {
            let mut state = self.live.borrow_mut();
            state.as_mut().is_some_and(|live| {
                if live.mode == Mode::DefaultChooser
                    && live.phase == Phase::Initial
                    && live.chooser_cycle == 1
                    && live.chooser_fresh_redraw_pending
                {
                    live.chooser_fresh_redraw_pending = false;
                    true
                } else {
                    false
                }
            })
        };
        if fresh {
            self.record(
                Kind::LiveSourceNavigationRedraw,
                &"chooser-fresh-redraw-consumed",
            );
            return true;
        }
        let redraw = {
            let mut live = self.live.borrow_mut();
            if let Some(live) = live.as_mut() {
                if matches!(live.mode, Mode::SourceReturn | Mode::DefaultChooser)
                    && live.phase == Phase::ReturnReady
                    && !live.source_action_sent
                    && !live.source_redraw_sent
                {
                    live.source_redraw_sent = true;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };
        if redraw {
            self.record(Kind::LiveSourceNavigationRedraw, &true);
        }
        redraw
    }
    pub(crate) fn live_return_released(&self) {
        let close = if let Ok(mut live) = self.live.try_borrow_mut()
            && let Some(live) = live.as_mut()
        {
            if live.phase == Phase::ViewingObserved {
                if live.mode == Mode::DefaultChooser && live.input.is_none() {
                    live.phase = Phase::Initial;
                    live.chooser_cycle = 1;
                    live.entry = None;
                    live.returning = None;
                    live.source_action_sent = false;
                    live.source_redraw_sent = false;
                    live.active = false;
                    live.chooser_active_recorded = false;
                    live.chooser_actual_item_ref = None;
                    live.chooser_fresh_redraw_pending = true;
                    self.record(
                        Kind::LiveSourceNavigationRedraw,
                        &"chooser-fresh-redraw-requested",
                    );
                    return;
                }
                live.phase = Phase::Complete;
                self.record(Kind::LiveOwnersReleased, &true);
                true
            } else {
                false
            }
        } else {
            false
        };
        if close {
            self.request_close();
        }
    }
    pub(crate) fn live_fault(&self, error: String) {
        self.record(Kind::LiveFault, &error);
        self.fail(Fault::Observation);
    }
    fn default_chooser_notice(
        &self,
        live: &mut LiveTrace,
        document: &NativeDocumentHandle,
        notice: &NativeWindowNotice,
        observed: Option<&str>,
    ) -> Result<(bool, bool), String> {
        if let Some(journey) = live.journey.as_ref() {
            journey.check_responses()?;
        }
        if live.persisted.is_some() {
            return self.persisted_notice(live, document, notice, observed);
        }
        let frame = notice.frame.as_ref().ok_or("actual frame missing")?;
        let submitted = frame
            .retained
            .as_ref()
            .ok_or("actual retained frame missing")?;
        if !current_projection(document, &frame.document)? {
            return Ok((false, false));
        }
        if live.input.is_some()
            && let Some(result) = self.current_input_notice(live, document, notice, observed)?
        {
            return Ok(result);
        }
        if live.journey.is_some()
            && let Some(result) = self.commit_journey_notice(live, document, notice, observed)?
        {
            return Ok(result);
        }
        match live.phase {
            Phase::Initial if observed == Some("view.start") && self.initial_reflected.get() => {
                if live.chooser_cycle == 1 {
                    if live.chooser_fresh_redraw_pending {
                        return Ok((false, false));
                    }
                    let ticket = serde_json::to_string(&frame.submission.ticket)
                        .map_err(|error| format!("{error:?}"))?;
                    if live.chooser_viewing_ticket.as_ref() == Some(&ticket) {
                        return Ok((false, false));
                    }
                    self.record(
                        Kind::LiveSourceNavigationRedraw,
                        &(
                            "chooser-fresh-actual-frame",
                            stamp(&frame.document),
                            &frame.submission.ticket,
                        ),
                    );
                }
                activate(document, "onChooserList", None)?;
                self.record(
                    Kind::ChooserSourceAction,
                    &(live.chooser_cycle, "list", stamp(&frame.document)),
                );
                live.phase = Phase::ChooserDocuments;
                Ok((true, false))
            }
            Phase::ChooserDocuments if observed == Some("view.start") => {
                let rows = chooser_rows(document, "onChooserDocument")?;
                if rows.len() != 2 || rows.iter().any(|row| row.disabled) {
                    return Ok((false, false));
                }
                let keys = rows.iter().map(|row| row.value.clone()).collect::<Vec<_>>();
                if live.chooser_cycle == 1
                    && keys.iter().any(|key| live.chooser_old_keys.contains(key))
                {
                    return Err("fresh query reused a retired chooser document key".into());
                }
                self.record(
                    Kind::ChooserGeneration,
                    &(live.chooser_cycle, &keys, stamp(&frame.document)),
                );
                live.chooser_old_keys.extend(keys);
                let id = if let Some(journey) = live.journey.as_ref() {
                    journey.proposal_id()
                } else if let Some(input) = live.input.as_ref() {
                    input.expected.document.proposal_id.as_str()
                } else if live.chooser_cycle == 0 {
                    "journey-a"
                } else {
                    "journey-b"
                };
                if !activate_chooser_row(document, "onChooserDocument", id)? {
                    return Err("seeded document absent from current source rows".into());
                }
                self.record(
                    Kind::ChooserSourceAction,
                    &(live.chooser_cycle, "document", id, stamp(&frame.document)),
                );
                live.phase = Phase::ChooserItems;
                Ok((true, false))
            }
            Phase::ChooserItems if observed == Some("view.start") => {
                let rows = chooser_rows(document, "onChooserItem")?;
                if rows.len() != 2 || rows.iter().any(|row| row.disabled) {
                    return Ok((false, false));
                }
                if live.chooser_cycle == 1
                    && rows
                        .iter()
                        .any(|row| live.chooser_old_keys.contains(&row.value))
                {
                    return Err("fresh document reused a retired chooser item key".into());
                }
                live.chooser_old_keys
                    .extend(rows.iter().map(|row| row.value.clone()));
                let label = match (live.case, live.chooser_cycle) {
                    (Case::Card, _) if live.journey.is_some() => "카드 2",
                    (Case::Highlight, _) if live.journey.is_some() => "하이라이트 2",
                    (Case::Cue, _) if live.journey.is_some() => "자막 2",
                    (Case::Document, _) if live.journey.is_some() => "문서 구간 2",
                    (Case::Document, _) if live.input.is_some() => "문서 구간 2",
                    (Case::Card, 0) => "카드 2",
                    (Case::Card, _) => "카드 1",
                    (Case::Highlight, 0) => "하이라이트 2",
                    (Case::Highlight, _) => "하이라이트 1",
                    (_, 0) => "자막 2",
                    (_, _) => "문서 구간 1",
                };
                if !activate_chooser_row(document, "onChooserItem", label)? {
                    return Err("expected typed item absent from current source rows".into());
                }
                self.record(
                    Kind::ChooserSourceAction,
                    &(live.chooser_cycle, "item", label, stamp(&frame.document)),
                );
                live.phase = Phase::ChooserOpen;
                Ok((true, false))
            }
            Phase::ChooserOpen if observed == Some("view.start") => {
                // Original renderer rejects disabled actions; wait for the current enabled source.
                if !source_enabled(document, "onOpenLiveProposal")? {
                    return Ok((false, false));
                }
                activate(document, "onOpenLiveProposal", None)?;
                self.record(
                    Kind::ChooserSourceAction,
                    &(live.chooser_cycle, "open", stamp(&frame.document)),
                );
                live.phase = Phase::Opening;
                Ok((true, false))
            }
            Phase::Opening if observed == Some(live.case.route()) => {
                let (original, proof) =
                    live.entry.as_ref().ok_or("original entry proof missing")?;
                original
                    .validate_retained_frame(proof, submitted)
                    .map_err(|error| format!("{error:?}"))?;
                self.record(
                    Kind::LiveEntryFrame,
                    &(
                        live.case,
                        stamp(proof.receipt().after()),
                        stamp(submitted.stamp()),
                        submitted.ticket(),
                        true,
                    ),
                );
                live.phase = Phase::AwaitingActivation;
                Ok((true, false))
            }
            Phase::AwaitingActivation if live.active && observed == Some(live.case.route()) => {
                if live.dirty || live.pending {
                    return Err("unmodified chooser entry became dirty or pending".into());
                }
                if !live.chooser_active_recorded {
                    return Err("actual active review identity not observed".into());
                }
                let expected_text = if live.chooser_cycle == 0 {
                    live.case.original()
                } else {
                    match live.case {
                        Case::Card | Case::Highlight => "first",
                        _ => "첫 수정",
                    }
                };
                if !displayed(document, live.case.binding(), expected_text)? {
                    return Err("actual active source editor shows a different item".into());
                }
                if let Some(reference) = live.chooser_actual_item_ref.as_ref() {
                    let expected =
                        serde_json::to_string(reference).map_err(|error| format!("{error:?}"))?;
                    let path = exact_source_reference_item(document, &expected)?;
                    self.record(
                        Kind::LiveInputDisplayed,
                        &(
                            "chooser-selected-draft-ref",
                            stamp(&frame.document),
                            path,
                            reference,
                            expected_text,
                        ),
                    );
                }
                if live.chooser_cycle == 1 {
                    self.record(
                        Kind::ChooserJourneyComplete,
                        &(live.case, stamp(&frame.document), true),
                    );
                    live.phase = Phase::Complete;
                    return Ok((false, true));
                }
                live.phase = Phase::ReturnReady;
                Ok((true, false))
            }
            Phase::ReturnReady
                if observed == Some(live.case.route()) && !live.source_action_sent =>
            {
                activate(document, "onNavigate", Some("시청으로 돌아가기"))?;
                live.source_action_sent = true;
                self.record(
                    Kind::LiveSourceNavigationAction,
                    &(live.case, "VIEW", stamp(&frame.document)),
                );
                Ok((true, false))
            }
            Phase::Returning if observed == Some("view.start") => {
                let (original, proof) = live
                    .returning
                    .as_ref()
                    .ok_or("original Viewing proof missing")?;
                let entry = live.entry.as_ref().ok_or("first entry proof missing")?;
                if proof.receipt().after().document_revision
                    <= entry.1.receipt().after().document_revision
                {
                    return Err("Viewing reused first entry receipt".into());
                }
                original
                    .validate_retained_frame(proof, submitted)
                    .map_err(|error| format!("{error:?}"))?;
                self.record(
                    Kind::LiveViewingFrame,
                    &(
                        live.case,
                        stamp(proof.receipt().after()),
                        stamp(submitted.stamp()),
                        submitted.ticket(),
                        true,
                    ),
                );
                live.phase = Phase::ViewingObserved;
                live.chooser_viewing = Some(frame.document.clone());
                live.chooser_viewing_ticket = Some(
                    serde_json::to_string(&frame.submission.ticket)
                        .map_err(|error| format!("{error:?}"))?,
                );
                Ok((true, false))
            }
            _ => Ok((false, false)),
        }
    }
    fn current_input_notice(
        &self,
        live: &mut LiveTrace,
        document: &NativeDocumentHandle,
        notice: &NativeWindowNotice,
        observed: Option<&str>,
    ) -> Result<Option<(bool, bool)>, String> {
        if observed != Some(live.case.route()) || !live.active {
            return Ok(None);
        }
        let frame = notice.frame.as_ref().ok_or("input actual frame missing")?;
        let input = live.input.as_ref().ok_or("input configuration missing")?;
        let original = current_input_fields(&input.expected, live.case)?;
        let progress = input.progress;
        match (live.phase, progress) {
            (Phase::AwaitingActivation, InputProgress::Initial) => {
                if !live.chooser_active_recorded || live.dirty || live.pending {
                    return Err("input source entry is not original clean observed owner".into());
                }
                let observation =
                    current_input_projection(document, &input.expected, live.case, &original)?;
                self.record(
                    Kind::LiveInputDisplayed,
                    &("input-initial", stamp(&frame.document), observation),
                );
                if live.case == Case::Card {
                    live.input.as_mut().unwrap().first_text_started = true;
                    self.input_text_diagnostic("first-text-before", None, None);
                }
                let text_result = set_text(document, original[0].0, input_technical_text(0));
                if live.case == Case::Card {
                    let error_hash = text_result.as_ref().err().map(|error| {
                        meari::domain::build::ContentHash::of(error.as_bytes())
                            .as_str()
                            .to_owned()
                    });
                    self.input_text_diagnostic(
                        if text_result.is_ok() {
                            "first-text-ok"
                        } else {
                            "first-text-error"
                        },
                        error_hash.as_deref(),
                        None,
                    );
                }
                text_result?;
                live.input.as_mut().unwrap().progress = InputProgress::Changed(0);
                live.phase = Phase::Editing;
                Ok(Some((true, false)))
            }
            (Phase::Editing, InputProgress::Changed(index)) => {
                let mut expected = original.clone();
                expected[index].1 = input_technical_text(index).into();
                if !current_input_values_match(document, &expected)? || !live.dirty || live.pending
                {
                    return Ok(Some((false, false)));
                }
                let observation =
                    current_input_projection(document, &input.expected, live.case, &expected)?;
                self.record(
                    Kind::LiveInputDisplayed,
                    &("input-changed", index, stamp(&frame.document), observation),
                );
                set_text(document, original[index].0, &original[index].1)?;
                live.input.as_mut().unwrap().progress = InputProgress::Restored(index);
                Ok(Some((true, false)))
            }
            (Phase::Editing, InputProgress::Restored(index)) => {
                if !current_input_values_match(document, &original)? {
                    return Ok(Some((false, false)));
                }
                let observation =
                    current_input_projection(document, &input.expected, live.case, &original)?;
                self.record(
                    Kind::LiveInputRestored,
                    &("input-restored", index, stamp(&frame.document), observation),
                );
                if index + 1 < original.len() {
                    set_text(
                        document,
                        original[index + 1].0,
                        input_technical_text(index + 1),
                    )?;
                    live.input.as_mut().unwrap().progress = InputProgress::Changed(index + 1);
                } else {
                    if !live.dirty || live.pending || !source_enabled(document, "onSave")? {
                        return Err(
                            "restored current source is not ready for original save prerequisite"
                                .into(),
                        );
                    }
                    save_current_source(document)?;
                    self.record(
                        Kind::LiveSaveRequested,
                        &"input original source Save once; return prerequisite; no Accept",
                    );
                    live.input.as_mut().unwrap().progress = InputProgress::Saving;
                    live.phase = Phase::Saving;
                }
                Ok(Some((true, false)))
            }
            (Phase::Saving, InputProgress::Saving)
                if live.save_observation_pending && !live.save_observation_woken =>
            {
                current_input_projection(document, &input.expected, live.case, &original)?;
                live.save_observation_pending = false;
                live.save_observation_woken = true;
                self.record(
                    Kind::LiveSaveObservationWake,
                    &(stamp(&frame.document), true),
                );
                Ok(Some((true, false)))
            }
            (Phase::ReturnReady, InputProgress::Saving) => {
                if live.dirty || live.pending {
                    return Err("original Save did not drain before source return".into());
                }
                let observation =
                    current_input_projection(document, &input.expected, live.case, &original)?;
                self.record(
                    Kind::LiveInputRestored,
                    &("input-save-displayed", stamp(&frame.document), observation),
                );
                live.input.as_mut().unwrap().progress = InputProgress::Saved;
                Ok(None)
            }
            _ => Ok(None),
        }
    }
    fn persisted_notice(
        &self,
        live: &mut LiveTrace,
        document: &NativeDocumentHandle,
        notice: &NativeWindowNotice,
        observed: Option<&str>,
    ) -> Result<(bool, bool), String> {
        let frame = notice
            .frame
            .as_ref()
            .ok_or("persisted actual frame missing")?;
        let submitted = frame
            .retained
            .as_ref()
            .ok_or("persisted actual retained frame missing")?;
        if !current_projection(document, &frame.document)? {
            return Ok((false, false));
        }
        let expected = live
            .persisted
            .as_ref()
            .ok_or("persisted expectation missing")?;
        match live.phase {
            Phase::Initial if observed == Some("view.start") && self.initial_reflected.get() => {
                if live.chooser_cycle == 1 {
                    if live.chooser_fresh_redraw_pending {
                        return Ok((false, false));
                    }
                    let ticket = serde_json::to_string(&frame.submission.ticket)
                        .map_err(|error| format!("{error:?}"))?;
                    if live.chooser_viewing_ticket.as_ref() == Some(&ticket) {
                        return Ok((false, false));
                    }
                    self.record(
                        Kind::LiveSourceNavigationRedraw,
                        &(
                            "persisted-fresh-actual-frame",
                            stamp(&frame.document),
                            &frame.submission.ticket,
                        ),
                    );
                }
                activate(document, "onChooserList", None)?;
                self.record(
                    Kind::ChooserSourceAction,
                    &(live.chooser_cycle, "list", stamp(&frame.document)),
                );
                live.phase = Phase::ChooserDocuments;
                Ok((true, false))
            }
            Phase::ChooserDocuments if observed == Some("view.start") => {
                let rows = chooser_rows(document, "onChooserDocument")?;
                if rows.is_empty() || rows.iter().any(|row| row.disabled) {
                    return Ok((false, false));
                }
                if rows.len() != 1 {
                    return Err("persisted store exposed unexpected proposal rows".into());
                }
                let keys = rows.iter().map(|row| row.value.clone()).collect::<Vec<_>>();
                if live.chooser_cycle == 1
                    && keys.iter().any(|key| live.chooser_old_keys.contains(key))
                {
                    return Err("persisted fresh list reused old document key".into());
                }
                source_text_element(
                    document,
                    "viewing/proposal-chooser.tsx",
                    "span",
                    &format!("수정본 {} · 항목 1개", expected.document.draft_revision.0),
                )?;
                self.record(
                    Kind::ChooserGeneration,
                    &(live.chooser_cycle, &keys, stamp(&frame.document)),
                );
                live.chooser_old_keys.extend(keys);
                if !activate_chooser_row(
                    document,
                    "onChooserDocument",
                    &expected.document.proposal_id,
                )? {
                    return Err("persisted proposal is absent from current source".into());
                }
                self.record(
                    Kind::ChooserSourceAction,
                    &(
                        live.chooser_cycle,
                        "document",
                        &expected.document.proposal_id,
                        stamp(&frame.document),
                    ),
                );
                live.phase = Phase::ChooserItems;
                Ok((true, false))
            }
            Phase::ChooserItems if observed == Some("view.start") => {
                let rows = chooser_rows(document, "onChooserItem")?;
                if rows.is_empty() || rows.iter().any(|row| row.disabled) {
                    return Ok((false, false));
                }
                if rows.len() != 1 {
                    return Err(
                        "persisted fixture must expose its exact original item count".into(),
                    );
                }
                if live.chooser_cycle == 1
                    && rows
                        .iter()
                        .any(|row| live.chooser_old_keys.contains(&row.value))
                {
                    return Err("persisted fresh selection reused old item key".into());
                }
                live.chooser_old_keys
                    .extend(rows.iter().map(|row| row.value.clone()));
                let label = match live.case {
                    Case::Card => "카드 1",
                    Case::Highlight => "하이라이트 1",
                    Case::Document => "문서 구간 1",
                    Case::Cue => return Err("persisted cue case was not assigned".into()),
                };
                if !activate_chooser_row(document, "onChooserItem", label)? {
                    return Err("persisted original item absent from current source".into());
                }
                self.record(
                    Kind::ChooserSourceAction,
                    &(live.chooser_cycle, "item", label, stamp(&frame.document)),
                );
                live.phase = Phase::ChooserOpen;
                Ok((true, false))
            }
            Phase::ChooserOpen if observed == Some("view.start") => {
                if !source_enabled(document, "onOpenLiveProposal")? {
                    return Ok((false, false));
                }
                activate(document, "onOpenLiveProposal", None)?;
                self.record(
                    Kind::ChooserSourceAction,
                    &(live.chooser_cycle, "open", stamp(&frame.document)),
                );
                live.phase = Phase::Opening;
                Ok((true, false))
            }
            Phase::Opening if observed == Some(live.case.route()) => {
                let (original, proof) = live
                    .entry
                    .as_ref()
                    .ok_or("persisted original entry proof missing")?;
                original
                    .validate_retained_frame(proof, submitted)
                    .map_err(|error| format!("{error:?}"))?;
                self.record(
                    Kind::LiveEntryFrame,
                    &(
                        live.case,
                        stamp(proof.receipt().after()),
                        stamp(submitted.stamp()),
                        submitted.ticket(),
                        true,
                    ),
                );
                live.phase = Phase::AwaitingActivation;
                Ok((true, false))
            }
            Phase::AwaitingActivation if live.active && observed == Some(live.case.route()) => {
                if live.dirty || live.pending || !live.chooser_active_recorded {
                    return Err("persisted active owner is not clean and observed".into());
                }
                let observation = observe_persisted_display(document, expected)?;
                self.record(
                    Kind::LiveInputDisplayed,
                    &(
                        "persisted-feedback",
                        live.chooser_cycle,
                        stamp(&frame.document),
                        observation,
                    ),
                );
                if live.chooser_cycle == 1 {
                    self.record(
                        Kind::ChooserJourneyComplete,
                        &(live.case, stamp(&frame.document), true),
                    );
                    live.phase = Phase::Complete;
                    Ok((false, true))
                } else {
                    live.phase = Phase::ReturnReady;
                    Ok((true, false))
                }
            }
            Phase::ReturnReady
                if observed == Some(live.case.route()) && !live.source_action_sent =>
            {
                activate(document, "onNavigate", Some("시청으로 돌아가기"))?;
                live.source_action_sent = true;
                self.record(
                    Kind::LiveSourceNavigationAction,
                    &(live.case, "VIEW", stamp(&frame.document)),
                );
                Ok((true, false))
            }
            Phase::Returning if observed == Some("view.start") => {
                let (original, proof) = live
                    .returning
                    .as_ref()
                    .ok_or("persisted original Viewing proof missing")?;
                let entry = live
                    .entry
                    .as_ref()
                    .ok_or("persisted first entry proof missing")?;
                if proof.receipt().after().document_revision
                    <= entry.1.receipt().after().document_revision
                {
                    return Err("persisted Viewing reused entry receipt".into());
                }
                original
                    .validate_retained_frame(proof, submitted)
                    .map_err(|error| format!("{error:?}"))?;
                self.record(
                    Kind::LiveViewingFrame,
                    &(
                        live.case,
                        stamp(proof.receipt().after()),
                        stamp(submitted.stamp()),
                        submitted.ticket(),
                        true,
                    ),
                );
                live.phase = Phase::ViewingObserved;
                live.chooser_viewing = Some(frame.document.clone());
                live.chooser_viewing_ticket = Some(
                    serde_json::to_string(&frame.submission.ticket)
                        .map_err(|error| format!("{error:?}"))?,
                );
                Ok((true, false))
            }
            _ => Ok((false, false)),
        }
    }
    fn input_text_diagnostic(
        &self,
        stage: &str,
        error_hash: Option<&str>,
        notify: Option<(bool, bool)>,
    ) {
        let quota = self
            .checkpoint
            .try_borrow()
            .ok()
            .map(|quota| (quota.lines, quota.notices, quota.notices == 8, quota.failed));
        self.checkpoint(&format!("input-diagnostic stage={stage} error_sha256={error_hash:?} probe_return_notify_close={notify:?} quota_lines_notices_notice_cap_failed={quota:?}; general-notice-absence-is-not-nondelivery"));
    }
    pub(crate) fn input_diagnostic_once(&self, slot: u8, stage: &str, details: &str) {
        if !(3..=9).contains(&slot) {
            return;
        }
        let emit = self.live.try_borrow_mut().ok().is_some_and(|mut state| {
            let Some(live) = state.as_mut().filter(|live| live.case == Case::Card) else {
                return false;
            };
            let Some(input) = live.input.as_mut() else {
                return false;
            };
            let bit = 1_u16 << slot;
            if !input.first_text_started || input.diagnostic_seen & bit != 0 {
                return false;
            }
            input.diagnostic_seen |= bit;
            true
        });
        if emit {
            let quota = self
                .checkpoint
                .try_borrow()
                .ok()
                .map(|quota| (quota.lines, quota.notices, quota.notices == 8, quota.failed));
            self.checkpoint(&format!("input-diagnostic slot={slot} stage={stage} {details} quota_lines_notices_notice_cap_failed={quota:?}"));
        }
    }
    fn input_observer_return_diagnostic(&self, notify: bool, close: bool) {
        self.input_diagnostic_once(
            3,
            "probe-return-intent-not-wake-receipt",
            &format!("notify={notify} close={close}"),
        );
    }
    pub(crate) fn live_notice(
        &self,
        document: &NativeDocumentHandle,
        notice: &NativeWindowNotice,
        observed: Option<&str>,
    ) -> bool {
        self.input_diagnostic_once(
            9,
            "first-post-text-notice",
            &format!(
                "frame_revision={:?} retained={}",
                notice
                    .frame
                    .as_ref()
                    .map(|frame| frame.document.document_revision),
                notice
                    .frame
                    .as_ref()
                    .is_some_and(|frame| frame.retained.is_some())
            ),
        );
        if !self.is_live() {
            return false;
        }
        if self.checkpoint_notice_allowed() {
            let phase = self
                .live
                .try_borrow()
                .ok()
                .and_then(|live| live.as_ref().map(|live| live.phase));
            let route = match observed {
                None => "none",
                Some("view.start") => "viewing",
                Some("studio.cards") => "cards",
                Some("studio.shorts.recommendations") => "highlight",
                Some("studio.subtitles.correction") => "subtitles",
                Some(_) => "other",
            };
            self.checkpoint(&format!(
                "notice phase={phase:?} route={route} frame={} retained={}",
                notice.frame.is_some(),
                notice
                    .frame
                    .as_ref()
                    .is_some_and(|frame| frame.retained.is_some())
            ));
        }
        if self.fault().is_some() {
            self.request_close();
            return false;
        }
        let result = (|| -> Result<(bool, bool), String> {
            let mut live = self
                .live
                .try_borrow_mut()
                .map_err(|_| "live trace borrowed")?;
            let live = live.as_mut().ok_or("live fixture missing")?;
            if live.phase == Phase::Abort {
                return Ok((false, true));
            }
            let Some(frame) = notice.frame.as_ref() else {
                return Ok((false, false));
            };
            let Some(submitted) = frame.retained.as_ref() else {
                return Ok((false, false));
            };
            if submitted.stamp() != &frame.document
                || submitted.ticket() != &frame.submission.ticket
            {
                return Err("actual proof/notice DTO mismatch".into());
            }
            if live.mode == Mode::DefaultChooser {
                return self.default_chooser_notice(live, document, notice, observed);
            }
            match live.phase {
                Phase::Initial
                    if observed == Some("view.start") && self.initial_reflected.get() =>
                {
                    activate(document, "onOpenLiveProposal", None)?;
                    self.record(
                        Kind::LiveInitialFrame,
                        &(live.case, stamp(&frame.document), &frame.submission.ticket),
                    );
                    live.phase = Phase::Opening;
                    Ok((true, false))
                }
                Phase::Opening if observed == Some(live.case.route()) => {
                    let (original, proof) =
                        live.entry.as_ref().ok_or("entry commit proof missing")?;
                    original
                        .validate_retained_frame(proof, submitted)
                        .map_err(|error| format!("{error:?}"))?;
                    self.record(
                        Kind::LiveEntryFrame,
                        &(
                            live.case,
                            stamp(proof.receipt().after()),
                            stamp(submitted.stamp()),
                            submitted.ticket(),
                            true,
                        ),
                    );
                    live.phase = Phase::AwaitingActivation;
                    Ok((true, false))
                }
                Phase::Opening
                    if live.mode == Mode::ArmBusy
                        && live.arm_fault_taken
                        && !live.retry_wake_sent
                        && live.entry.is_some() =>
                {
                    // One original observation wake can retry the retained current intent.
                    // This is not a successful route CompletionNotifier call or new SDK intent.
                    live.retry_wake_sent = true;
                    self.record(Kind::LiveArmRetryWake, &true);
                    Ok((true, false))
                }
                Phase::AwaitingActivation if live.active && observed == Some(live.case.route()) => {
                    let current = document.stamp().map_err(|error| format!("{error:?}"))?;
                    if current.document_revision != frame.document.document_revision {
                        return Ok((false, false));
                    }
                    set_text(document, live.case.binding(), "fixture explicit edit")?;
                    live.phase = Phase::Editing;
                    Ok((true, false))
                }
                Phase::Editing if observed == Some(live.case.route()) => {
                    if !current_projection(document, &frame.document)? {
                        return Ok((false, false));
                    }
                    if !displayed(document, live.case.binding(), "fixture explicit edit")? {
                        return Ok((false, false));
                    }
                    self.record(
                        Kind::LiveInputDisplayed,
                        &(
                            live.case,
                            stamp(&frame.document),
                            "fixture explicit edit",
                            true,
                        ),
                    );
                    set_text(document, live.case.binding(), live.case.original())?;
                    live.phase = Phase::Restoring;
                    Ok((true, false))
                }
                Phase::Restoring if observed == Some(live.case.route()) => {
                    if !current_projection(document, &frame.document)? {
                        return Ok((false, false));
                    }
                    if !displayed(document, live.case.binding(), live.case.original())? {
                        return Ok((false, false));
                    }
                    self.record(
                        Kind::LiveInputRestored,
                        &(live.case, stamp(&frame.document), true),
                    );
                    save_current_source(document)?;
                    self.record(
                        Kind::LiveSaveRequested,
                        &"original source onSave; return prerequisite only",
                    );
                    live.phase = Phase::Saving;
                    Ok((true, false))
                }
                Phase::Saving
                    if live.save_observation_pending
                        && !live.save_observation_woken
                        && observed == Some(live.case.route()) =>
                {
                    if !current_projection(document, &frame.document)?
                        || !displayed(document, live.case.binding(), live.case.original())?
                    {
                        return Ok((false, false));
                    }
                    live.save_observation_pending = false;
                    live.save_observation_woken = true;
                    self.record(
                        Kind::LiveSaveObservationWake,
                        &(stamp(&frame.document), true),
                    );
                    Ok((true, false))
                }
                Phase::ReturnReady
                    if live.mode == Mode::SourceReturn
                        && !live.source_action_sent
                        && observed == Some(live.case.route()) =>
                {
                    if !current_projection(document, &frame.document)? {
                        return Ok((false, false));
                    }
                    activate(document, "onNavigate", Some("시청으로 돌아가기"))?;
                    live.source_action_sent = true;
                    self.record(
                        Kind::LiveSourceNavigationAction,
                        &(live.case, "VIEW", stamp(&frame.document)),
                    );
                    Ok((true, false))
                }
                Phase::Returning if observed == Some("view.start") => {
                    let (original, proof) = live
                        .returning
                        .as_ref()
                        .ok_or("new Viewing commit proof missing")?;
                    let entry = live.entry.as_ref().ok_or("entry proof missing")?;
                    if proof.receipt().after().document_revision
                        <= entry.1.receipt().after().document_revision
                    {
                        return Err("Viewing reused entry receipt".into());
                    }
                    original
                        .validate_retained_frame(proof, submitted)
                        .map_err(|error| format!("{error:?}"))?;
                    self.record(
                        Kind::LiveViewingFrame,
                        &(
                            live.case,
                            stamp(proof.receipt().after()),
                            stamp(submitted.stamp()),
                            submitted.ticket(),
                            true,
                        ),
                    );
                    self.record(
                        Kind::LiveWakeCounts,
                        &(
                            live.counts.calls.load(Ordering::Acquire),
                            live.counts.delegated.load(Ordering::Acquire),
                            live.counts.successful.load(Ordering::Acquire),
                            live.counts
                                .errors
                                .lock()
                                .map_err(|_| "notifier errors poisoned")?
                                .len(),
                        ),
                    );
                    live.phase = Phase::ViewingObserved;
                    Ok((true, false))
                }
                _ => Ok((false, false)),
            }
        })();
        match result {
            Ok((notify, close)) => {
                self.input_observer_return_diagnostic(notify, close);
                if close {
                    self.request_close();
                }
                notify
            }
            Err(error) => {
                self.input_observer_return_diagnostic(false, true);
                self.live_fault(error);
                self.request_close();
                false
            }
        }
    }
}
fn input_technical_text(index: usize) -> &'static str {
    if index == 0 {
        "입력 관측 앞·제목·수정 日本語"
    } else {
        "입력 관측 뒷면 독립 값"
    }
}
fn current_input_fields(
    stored: &crate::domain::proposals::StoredProposal,
    case: Case,
) -> Result<Vec<(&'static str, String)>, String> {
    use crate::domain::proposals::ProposalPayload;
    match (case, stored.document.draft.as_ref()) {
        (Case::Card, Some(ProposalPayload::Cards { cards })) if cards.len() == 2 => Ok(vec![
            ("onFrontChange", cards[1].front.clone()),
            ("onBackChange", cards[1].back.clone()),
        ]),
        (Case::Highlight, Some(ProposalPayload::Highlights { highlights }))
            if highlights.len() == 2 =>
        {
            Ok(vec![("onTitleChange", highlights[1].title.clone())])
        }
        (Case::Cue, Some(ProposalPayload::Correction { edits }))
            if edits.len() == 2 && edits[1].cue_id == "b" =>
        {
            Ok(vec![("onDraftChange", edits[1].text.clone())])
        }
        (Case::Document, Some(ProposalPayload::DocumentCorrection { edits }))
            if edits.len() == 2 =>
        {
            Ok(vec![("onDraftChange", edits[1].text.clone())])
        }
        _ => Err("current input requires original seeded second typed item".into()),
    }
}
fn current_input_target_matches(
    target: &crate::application::launch::native_proposal::NativeProposalTarget,
    stored: &crate::domain::proposals::StoredProposal,
    case: Case,
) -> bool {
    use crate::application::launch::native_proposal::NativeProposalTarget;
    use crate::domain::proposals::{ProposalPayload, SourceSnapshot};
    match (case, target, stored.document.draft.as_ref()) {
        (
            Case::Card,
            NativeProposalTarget::Card { index: 1 },
            Some(ProposalPayload::Cards { cards }),
        ) => cards.len() == 2,
        (
            Case::Highlight,
            NativeProposalTarget::Highlight { index: 1 },
            Some(ProposalPayload::Highlights { highlights }),
        ) => highlights.len() == 2,
        (
            Case::Cue,
            NativeProposalTarget::CueCorrection { index: 1, cue_id },
            Some(ProposalPayload::Correction { edits }),
        ) => edits.len() == 2 && cue_id == &edits[1].cue_id && cue_id == "b",
        (
            Case::Document,
            NativeProposalTarget::DocumentCorrection {
                index: 1,
                source_ref,
            },
            Some(ProposalPayload::DocumentCorrection { edits }),
        ) => {
            edits.len() == 2
                && source_ref == &edits[1].source_ref
                && matches!(&stored.document.source, SourceSnapshot::Document(source) if source.text_reference("/1").as_ref() == Some(source_ref))
        }
        _ => false,
    }
}
fn current_input_values_match(
    document: &NativeDocumentHandle,
    fields: &[(&str, String)],
) -> Result<bool, String> {
    for (binding, value) in fields {
        if !displayed(document, binding, value)? {
            return Ok(false);
        }
    }
    Ok(true)
}
fn current_input_binding(
    document: &NativeDocumentHandle,
    binding: &str,
    module: &str,
    signature: meari::domain::host_callbacks::CallbackSignature,
) -> Result<String, String> {
    document
        .try_with(|document| {
            let projection = document.module_projection()?;
            let callbacks = projection
                .callback_bindings()
                .into_iter()
                .filter(|(_, callback)| callback.reference.binding == binding)
                .collect::<Vec<_>>();
            let [(path, callback)] = callbacks.as_slice() else {
                return Err("current source binding is not unique".to_owned());
            };
            if callback.reference.signature != signature {
                return Err("current source binding signature differs".into());
            }
            let element = document
                .elements()
                .iter()
                .find(|element| element.owns_path(path))
                .ok_or("current callback element missing")?;
            if !std::iter::once(&element.path)
                .chain(element.aliases.iter())
                .any(|path| {
                    projection
                        .origins
                        .get(path)
                        .is_some_and(|origin| origin.module.as_str() == module)
                })
            {
                return Err("current callback element module origin differs".into());
            }
            Ok(path.clone())
        })
        .map_err(|error| format!("{error:?}"))?
}
fn current_input_projection(
    document: &NativeDocumentHandle,
    stored: &crate::domain::proposals::StoredProposal,
    case: Case,
    fields: &[(&str, String)],
) -> Result<serde_json::Value, String> {
    use crate::domain::proposals::{ProposalPayload, SourceSnapshot};
    use meari::domain::{build::ContentHash, host_callbacks::CallbackSignature};
    let hash = |text: &str| ContentHash::of(text.as_bytes()).as_str().to_owned();
    let module = match case {
        Case::Card => "components/card-copy-fields.tsx",
        Case::Highlight => "studio/clips/highlight-proposal-panel.tsx",
        _ => "studio/subtitles/correction-panel.tsx",
    };
    let refs = match stored.document.draft.as_ref() {
        Some(ProposalPayload::Cards { cards }) => cards[1].source_refs.clone(),
        Some(ProposalPayload::Highlights { highlights }) => highlights[1].source_refs.clone(),
        Some(ProposalPayload::Correction { edits }) => vec![edits[1].cue_id.clone()],
        Some(ProposalPayload::DocumentCorrection { edits }) => vec![
            serde_json::to_string(&edits[1].source_ref).map_err(|error| format!("{error:?}"))?,
        ],
        _ => return Err("current input selected references missing".into()),
    };
    let shown = source_text_nodes(document, "components/proposal-source.tsx", "li")?;
    if shown.iter().map(|(_, text)| text).ne(refs.iter()) {
        return Err("current input full refs differ in value/order/count".into());
    }
    let source_text = match &stored.document.source {
        SourceSnapshot::Cue(source) => refs
            .iter()
            .map(|id| {
                source
                    .cues
                    .iter()
                    .find(|cue| &cue.cue_id == id)
                    .map(|cue| cue.text.as_str())
                    .ok_or("current input source cue missing")
            })
            .collect::<Result<Vec<_>, _>>()?
            .join("\n"),
        SourceSnapshot::Document(source) => source
            .texts
            .iter()
            .find(|text| text.inner_pointer == "/1")
            .ok_or("current input original text reference missing")?
            .text
            .clone(),
    };
    let source_path = source_text_element(
        document,
        "components/proposal-source.tsx",
        "p",
        &source_text,
    )?;
    let proposal_path = source_text_element(
        document,
        "components/proposal-source.tsx",
        "dd",
        &stored.document.proposal_id,
    )?;
    let mut displayed_fields = Vec::new();
    for (binding, value) in fields {
        let path = current_input_binding(document, binding, module, CallbackSignature::Text)?;
        if !displayed(document, binding, value)? {
            return Err("current Text controlled value/editor buffer differs".into());
        }
        displayed_fields.push(serde_json::json!({"binding":binding,"path_sha256":hash(&path),"value_sha256":hash(value)}));
    }
    let mut actions = Vec::new();
    for binding in ["onSave", "onAccept"] {
        let path = current_input_binding(
            document,
            binding,
            "components/proposal-actions.tsx",
            CallbackSignature::Action,
        )?;
        actions.push(serde_json::json!({"binding":binding,"path_sha256":hash(&path),"enabled":source_enabled(document,binding)?}));
    }
    Ok(
        serde_json::json!({"proposal_id":stored.document.proposal_id,"selected_index":1,"source_path_sha256":hash(&source_path),
        "proposal_path_sha256":hash(&proposal_path),"references_sha256":hash(&serde_json::to_string(&refs).map_err(|error|format!("{error:?}"))?),
        "fields":displayed_fields,"actions":actions,"accept_dispatched":false}),
    )
}
fn persisted_target_matches(
    target: &crate::application::launch::native_proposal::NativeProposalTarget,
    expected: &crate::domain::proposals::StoredProposal,
) -> bool {
    use crate::application::launch::native_proposal::NativeProposalTarget as Target;
    use crate::domain::proposals::ProposalPayload as Payload;
    match (target, expected.document.draft.as_ref()) {
        (Target::Card { index: 0 }, Some(Payload::Cards { cards })) => cards.len() == 1,
        (Target::Highlight { index: 0 }, Some(Payload::Highlights { highlights })) => {
            highlights.len() == 1
        }
        (
            Target::DocumentCorrection {
                index: 0,
                source_ref,
            },
            Some(Payload::DocumentCorrection { edits }),
        ) => edits.len() == 1 && edits[0].source_ref == *source_ref,
        _ => false,
    }
}
fn element_text(element: &meari_native::application::markup::Element) -> String {
    element.text_override.clone().unwrap_or_else(|| {
        element
            .text_runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>()
    })
}
fn source_text_nodes(
    document: &NativeDocumentHandle,
    module: &str,
    tag: &str,
) -> Result<Vec<(String, String)>, String> {
    document
        .try_with(|document| {
            let projection = document.module_projection()?;
            Ok(document
                .elements()
                .iter()
                .filter(|element| {
                    element.dom_tag == tag
                        && std::iter::once(&element.path)
                            .chain(element.aliases.iter())
                            .any(|path| {
                                projection
                                    .origins
                                    .get(path)
                                    .is_some_and(|origin| origin.module.as_str() == module)
                            })
                })
                .map(|element| (element.path.clone(), element_text(element)))
                .collect::<Vec<_>>())
        })
        .map_err(|error| format!("{error:?}"))?
}
fn source_text_element(
    document: &NativeDocumentHandle,
    module: &str,
    tag: &str,
    expected: &str,
) -> Result<String, String> {
    let nodes = source_text_nodes(document, module, tag)?;
    let matches = nodes
        .iter()
        .filter(|(_, text)| text == expected)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [node] => Ok(node.0.clone()),
        _ => Err(format!(
            "current source {module}/{tag} exact text is missing or ambiguous"
        )),
    }
}
fn observe_persisted_display(
    document: &NativeDocumentHandle,
    expected: &crate::domain::proposals::StoredProposal,
) -> Result<serde_json::Value, String> {
    use crate::domain::proposals::{ProposalPayload, SourceSnapshot};
    use crate::presentation::web_ui_adapter::proposals::product_window::ProductWindowLabels;
    let labels = ProductWindowLabels::korean().panel;
    let mut revision = format!("{} {}", labels.revision, expected.document.draft_revision.0);
    if let Some(receipt) = &expected.acceptance {
        revision.push_str(&format!(
            "{}{} {}",
            labels.feedback.separator, labels.feedback.output_revision, receipt.output_revision.0
        ));
    }
    let status = if expected.acceptance.is_some() {
        &labels.feedback.accepted
    } else {
        &labels.feedback.stored
    };
    let proposal_path = source_text_element(
        document,
        "components/proposal-source.tsx",
        "dd",
        &expected.document.proposal_id,
    )?;
    let revision_path =
        source_text_element(document, "components/proposal-source.tsx", "dd", &revision)?;
    let status_path =
        source_text_element(document, "components/proposal-actions.tsx", "p", status)?;
    let quality_path = source_text_element(
        document,
        "components/proposal-source.tsx",
        "p",
        &labels.quality_unknown,
    )?;
    let (references, fields) = match expected.document.draft.as_ref() {
        Some(ProposalPayload::Cards { cards }) if cards.len() == 1 => (
            cards[0].source_refs.clone(),
            vec![
                ("onFrontChange", cards[0].front.as_str()),
                ("onBackChange", cards[0].back.as_str()),
            ],
        ),
        Some(ProposalPayload::Highlights { highlights }) if highlights.len() == 1 => (
            highlights[0].source_refs.clone(),
            vec![("onTitleChange", highlights[0].title.as_str())],
        ),
        Some(ProposalPayload::DocumentCorrection { edits }) if edits.len() == 1 => (
            vec![
                serde_json::to_string(&edits[0].source_ref)
                    .map_err(|error| format!("{error:?}"))?,
            ],
            vec![("onDraftChange", edits[0].text.as_str())],
        ),
        _ => return Err("persisted expected draft is outside assigned case".into()),
    };
    let source_text = match (&expected.document.source, expected.document.draft.as_ref()) {
        (SourceSnapshot::Cue(source), _) => references
            .iter()
            .map(|id| {
                source
                    .cues
                    .iter()
                    .find(|cue| &cue.cue_id == id)
                    .map(|cue| cue.text.as_str())
                    .ok_or("persisted cue reference missing")
            })
            .collect::<Result<Vec<_>, _>>()?
            .join("\n"),
        (SourceSnapshot::Document(source), Some(ProposalPayload::DocumentCorrection { edits })) => {
            source
                .texts
                .iter()
                .find(|text| {
                    source.text_reference(&text.inner_pointer).as_ref()
                        == Some(&edits[0].source_ref)
                })
                .ok_or("persisted full document ref missing")?
                .text
                .clone()
        }
        _ => return Err("persisted source kind differs".into()),
    };
    let source_path = source_text_element(
        document,
        "components/proposal-source.tsx",
        "p",
        &source_text,
    )?;
    let shown_refs = source_text_nodes(document, "components/proposal-source.tsx", "li")?;
    if shown_refs
        .iter()
        .map(|(_, text)| text)
        .ne(references.iter())
    {
        return Err("persisted actual selected sourceRefs differ in value/order/count".into());
    }
    let path_hash = |path: &str| {
        meari::domain::build::ContentHash::of(path.as_bytes())
            .as_str()
            .to_owned()
    };
    let mut field_observations = Vec::new();
    for (binding, text) in fields {
        if !displayed(document, binding, text)? {
            return Err(format!(
                "persisted actual editor {binding} differs from saved result"
            ));
        }
        field_observations.push(serde_json::json!({"binding":binding,"actual_path_sha256":path_hash(&input_path(document,binding)?),"visible_value_sha256":path_hash(text)}));
    }
    Ok(
        serde_json::json!({"proposal_id":expected.document.proposal_id,"document_revision":expected.document.draft_revision,
        "output_revision":expected.acceptance.as_ref().map(|receipt|receipt.output_revision),
        "proposal_node_sha256":path_hash(&proposal_path),"revision_node_sha256":path_hash(&revision_path),"status_node_sha256":path_hash(&status_path),
        "source_node_sha256":path_hash(&source_path),"quality_node_sha256":path_hash(&quality_path),
        "reference_nodes":shown_refs.iter().map(|(path,_)|path_hash(path)).collect::<Vec<_>>(),"editor_fields":field_observations}),
    )
}
fn exact_source_reference_item(
    document: &NativeDocumentHandle,
    expected: &str,
) -> Result<String, String> {
    document
        .try_with(|document| {
            let projection = document.module_projection()?;
            let candidates = document
                .elements()
                .iter()
                .filter(|element| {
                    element.dom_tag == "li"
                        && std::iter::once(&element.path)
                            .chain(element.aliases.iter())
                            .any(|path| {
                                projection.origins.get(path).is_some_and(|origin| {
                                    origin.module.as_str() == "components/proposal-source.tsx"
                                })
                            })
                })
                .collect::<Vec<_>>();
            let [element] = candidates.as_slice() else {
                return Err(
                    "current ProposalSource must show exactly one selected sourceRefs li"
                        .to_string(),
                );
            };
            let text = element.text_override.clone().unwrap_or_else(|| {
                element
                    .text_runs
                    .iter()
                    .map(|run| run.text.as_str())
                    .collect::<String>()
            });
            if text != expected {
                return Err(
                    "actual selected ProposalSource li differs from draft item ref".to_string(),
                );
            }
            Ok(element.path.clone())
        })
        .map_err(|error| format!("{error:?}"))?
}
fn chooser_rows(
    document: &NativeDocumentHandle,
    binding: &str,
) -> Result<Vec<meari_native::domain::widgets::Item>, String> {
    document
        .try_with(|document| {
            let projection = document.module_projection()?;
            let paths = projection
                .callback_bindings()
                .into_iter()
                .filter(|(_, callback)| callback.reference.binding == binding)
                .map(|(path, _)| path)
                .collect::<Vec<_>>();
            let groups = document
                .session
                .widgets
                .values()
                .filter(|widget| widget.family == meari_native::domain::widgets::Family::RadioGroup)
                .filter(|widget| {
                    paths.iter().any(|path| {
                        path == &widget.root || path.starts_with(&(widget.root.clone() + "/"))
                    })
                })
                .collect::<Vec<_>>();
            match groups.as_slice() {
                [group] => Ok(group.items.clone()),
                [] => Ok(Vec::new()),
                _ => Err("ambiguous current source chooser group".to_string()),
            }
        })
        .map_err(|error| format!("{error:?}"))?
}
fn activate_chooser_row(
    document: &NativeDocumentHandle,
    binding: &str,
    label: &str,
) -> Result<bool, String> {
    let rows = chooser_rows(document, binding)?;
    let path = document
        .try_with(|document| {
            let matches = rows
                .iter()
                .filter(|row| {
                    !row.disabled
                        && document.elements().iter().any(|element| {
                            element.owns_path(&row.path)
                                && element.attributes.get("aria-label").is_some_and(|text| {
                                    text == label || text.ends_with(&format!(" · {label}"))
                                })
                        })
                })
                .collect::<Vec<_>>();
            match matches.as_slice() {
                [row] => Ok(Some(row.path.clone())),
                [] => Ok(None),
                _ => Err("ambiguous current source chooser row".to_string()),
            }
        })
        .map_err(|error| format!("{error:?}"))??;
    match path {
        Some(path) => document
            .dispatch_action(meari_native::domain::widgets::Action::Activate { path })
            .map_err(|error| format!("{error:?}")),
        None => Ok(false),
    }
}
fn source_enabled(document: &NativeDocumentHandle, binding: &str) -> Result<bool, String> {
    let path = input_path(document, binding)?;
    document
        .try_with(|document| {
            document
                .elements()
                .iter()
                .find(|element| element.owns_path(&path))
                .map(|element| !element.attributes.contains_key("disabled"))
                .ok_or_else(|| "current source action element missing".to_string())
        })
        .map_err(|error| format!("{error:?}"))?
}
fn save_current_source(document: &NativeDocumentHandle) -> Result<(), String> {
    let valid = document
        .try_with(|document| {
            document.module_projection().map(|projection| {
                let matches = projection
                    .callback_bindings()
                    .into_iter()
                    .filter(|(_, callback)| callback.reference.binding == "onSave")
                    .collect::<Vec<_>>();
                matches.len() == 1
                    && matches[0].1.reference.signature
                        == meari::domain::host_callbacks::CallbackSignature::Action
            })
        })
        .map_err(|error| format!("{error:?}"))??;
    if !valid {
        return Err("current source must expose one Action onSave path".into());
    }
    activate(document, "onSave", None)
}
fn input_path(document: &NativeDocumentHandle, binding: &str) -> Result<String, String> {
    document
        .try_with(|document| {
            document.module_projection().map(|projection| {
                projection
                    .callback_bindings()
                    .into_iter()
                    .find(|(_, callback)| callback.reference.binding == binding)
                    .map(|(path, _)| path)
            })
        })
        .map_err(|error| format!("{error:?}"))??
        .ok_or_else(|| "current source input callback missing".into())
}
fn set_text(document: &NativeDocumentHandle, binding: &str, value: &str) -> Result<(), String> {
    let path = input_path(document, binding)?;
    document
        .try_reproject(
            &document.stamp().map_err(|error| format!("{error:?}"))?,
            |document| document.set_input_value(&path, value),
        )
        .map_err(|error| format!("{error:?}"))?;
    Ok(())
}
fn displayed(document: &NativeDocumentHandle, binding: &str, value: &str) -> Result<bool, String> {
    let path = input_path(document, binding)?;
    document
        .try_with(|document| {
            let element = document
                .elements()
                .iter()
                .find(|element| element.owns_path(&path))
                .ok_or("current input element missing")?;
            let projected = element
                .attributes
                .get("value")
                .ok_or("controlled projected value missing")?;
            let id = document
                .path_id(&element.path)
                .ok_or("current input path id missing")?;
            let node = document
                .renderer
                .document
                .get_element_by_id(id)
                .and_then(|id| document.renderer.document.get_node(id))
                .ok_or("current renderer input node missing")?;
            let editor = node
                .element_data()
                .and_then(|element| element.text_input_data())
                .ok_or("current editable buffer missing")?;
            Ok::<_, String>(projected == value && editor.editor.raw_text() == value)
        })
        .map_err(|error| format!("{error:?}"))?
}
fn current_projection(document: &NativeDocumentHandle, frame: &HostStamp) -> Result<bool, String> {
    let current = document.stamp().map_err(|error| format!("{error:?}"))?;
    Ok(current.host_id == frame.host_id
        && current.owner == frame.owner
        && current.definition_revision == frame.definition_revision
        && current.document_revision == frame.document_revision)
}
