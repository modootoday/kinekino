//! Default chooser commit-journey observations; original SDK and lifetime owners stay in the parent.
use super::*;

/// Observations of the original recording client, never a replacement SDK/provider.
pub(super) struct CommitJourney {
    original: crate::domain::proposals::StoredProposal,
    calls: Arc<Mutex<Vec<crate::application::api::Request>>>,
    replies: Arc<Mutex<Vec<serde_json::Value>>>,
    progress: CommitProgress,
    action_start: usize,
    saved: Option<crate::domain::proposals::StoredProposal>,
    accepted: Option<crate::domain::proposals::StoredProposal>,
    guards: Option<SaveSnapshot>,
    owner: Option<HostStamp>,
    diagnostics: AcceptDiagnostics,
}
#[derive(Default)]
struct AcceptDiagnostics {
    seen: Vec<String>,
    omitted: bool,
}
struct AcceptObservation {
    wait: &'static str,
    phase: Phase,
    dirty: bool,
    pending: bool,
    wake: (bool, bool),
    frame: u64,
    inspected: Option<u64>,
    feedback: Option<bool>,
}
#[derive(Clone, Copy)]
enum CommitProgress {
    Initial,
    Changed(usize),
    Saving,
    Accepting,
    Accepted,
    Reopened,
}
impl CommitJourney {
    // Observation only: no additional SDK call, host turn, lock wait, or admission.
    fn diagnose_accept(&mut self, probe: &Probe, observation: AcceptObservation) {
        let calls = self.calls.try_lock().ok().map(|calls| calls.len());
        let replies = self.replies.try_lock().ok().map(|replies| {
            (
                replies.len(),
                replies.get(self.action_start).is_some(),
                replies.get(self.action_start + 1).is_some(),
                replies
                    .iter()
                    .position(|reply| reply["ok"] != true || !reply["error"].is_null()),
                replies
                    .iter()
                    .find(|reply| !reply["error"].is_null())
                    .map(|reply| {
                        meari::domain::build::ContentHash::of(reply["error"].to_string().as_bytes())
                    }),
            )
        });
        let guard = self.guards.map(|guard| {
            (
                guard.stage,
                guard.revision,
                guard.dirty,
                guard.pending,
                self.settled(),
            )
        });
        let key = format!(
            "accept-diag wait={} phase={:?} calls={calls:?} replies={replies:?} guard={guard:?} dirty={} pending={} wake={:?} inspected={:?} feedback={:?}",
            observation.wait,
            observation.phase,
            observation.dirty,
            observation.pending,
            observation.wake,
            observation.inspected,
            observation.feedback
        );
        if self.diagnostics.seen.contains(&key) {
            return;
        }
        if self.diagnostics.seen.len() == 8 || key.len() > 440 {
            if !self.diagnostics.omitted {
                probe.checkpoint("accept-diag omitted: distinct-state cap8 or message budget440");
                self.diagnostics.omitted = true;
            }
            return;
        }
        probe.checkpoint(&format!("{key} frame={}", observation.frame));
        self.diagnostics.seen.push(key);
    }
    pub(super) fn entry_snapshot(
        &self,
        cycle: u8,
    ) -> Result<&crate::domain::proposals::StoredProposal, String> {
        if cycle == 0 {
            Ok(&self.original)
        } else {
            self.accepted
                .as_ref()
                .ok_or_else(|| "journey reopened before accepted inspection".into())
        }
    }
    pub(super) fn proposal_id(&self) -> &str {
        self.original.document.proposal_id.as_str()
    }
    pub(super) fn observe_guards(&mut self, snapshot: &SaveSnapshot) {
        self.guards = Some(*snapshot);
    }
    pub(super) fn check_responses(&self) -> Result<(), String> {
        let calls = self.calls.lock().map_err(|_| "journey calls poisoned")?;
        let replies = self
            .replies
            .lock()
            .map_err(|_| "journey replies poisoned")?;
        if replies.len() > calls.len() {
            return Err("journey response transcript precedes its original request".into());
        }
        for (index, reply) in replies.iter().enumerate() {
            if reply["request_id"] != calls[index].context.request_id
                || reply["ok"] != true
                || !reply["error"].is_null()
            {
                return Err(format!(
                    "journey original SDK response {index} failed/mismatched; retained in transcript"
                ));
            }
        }
        Ok(())
    }
    fn settled(&self) -> bool {
        self.guards.is_some_and(|snapshot| {
            snapshot.stage == "after"
                && snapshot.active
                && snapshot.dirty == Some(false)
                && snapshot.pending == Some(false)
                && snapshot.recovery == Some(false)
                && snapshot.transport == Some(false)
                && snapshot.token == Some(false)
                && snapshot.confirmed == Some(false)
                && snapshot.projection == Some(false)
                && snapshot.remaining == 0
        })
    }
    fn mutation_inspection(
        &self,
        command: &str,
    ) -> Result<Option<crate::domain::proposals::StoredProposal>, String> {
        self.check_responses()?;
        let calls = self.calls.lock().map_err(|_| "journey calls poisoned")?;
        let replies = self
            .replies
            .lock()
            .map_err(|_| "journey replies poisoned")?;
        let expected = [command, "proposals.inspect"];
        if calls.len() > self.action_start + expected.len() {
            return Err("journey observed an extra request before mutation readback".into());
        }
        for (call, expected) in calls[self.action_start..].iter().zip(expected) {
            if call.command != expected
                || call.payload["proposal_id"] != self.original.document.proposal_id
            {
                return Err(
                    "journey original mutation/readback command or proposal differs".into(),
                );
            }
        }
        if replies.len() < self.action_start + 2 {
            return Ok(None);
        }
        let stored: crate::domain::proposals::StoredProposal =
            serde_json::from_value(replies[self.action_start + 1]["data"].clone())
                .map_err(|error| format!("journey original inspect decode: {error:?}"))?;
        stored
            .validate()
            .map_err(|error| format!("journey original inspect: {error:?}"))?;
        if stored.document.proposal_id != self.original.document.proposal_id
            || serde_json::to_value(&stored.document.source)
                .map_err(|error| format!("{error:?}"))?
                != serde_json::to_value(&self.original.document.source)
                    .map_err(|error| format!("{error:?}"))?
        {
            return Err("journey original inspect changed proposal/source identity".into());
        }
        let mutation = if command == "proposals.edit" {
            serde_json::to_value(&stored.document)
        } else {
            serde_json::to_value(&stored.acceptance)
        }
        .map_err(|error| format!("{error:?}"))?;
        if mutation != replies[self.action_start]["data"] {
            return Err(
                "journey original mutation response differs from its original inspection".into(),
            );
        }
        Ok(Some(stored))
    }
}

impl Probe {
    pub(crate) fn configure_commit_journey(
        &self,
        original: crate::domain::proposals::StoredProposal,
        calls: Arc<Mutex<Vec<crate::application::api::Request>>>,
        replies: Arc<Mutex<Vec<serde_json::Value>>>,
    ) -> Result<(), String> {
        if self
            .live
            .try_borrow()
            .map_err(|_| "journey trace borrowed")?
            .as_ref()
            .is_some_and(|live| live.journey.is_some())
        {
            return Err("commit journey already configured".into());
        }
        // Reuse the existing seed/target admission without changing the I4 state machine.
        self.configure_default_input(original)?;
        let mut state = self
            .live
            .try_borrow_mut()
            .map_err(|_| "journey trace borrowed")?;
        let live = state.as_mut().ok_or("journey live trace missing")?;
        let input = live.input.take().ok_or("journey seed admission missing")?;
        live.journey = Some(CommitJourney {
            original: input.expected,
            calls,
            replies,
            progress: CommitProgress::Initial,
            action_start: 0,
            saved: None,
            accepted: None,
            guards: None,
            owner: None,
            diagnostics: AcceptDiagnostics::default(),
        });
        Ok(())
    }
    pub(super) fn commit_journey_notice(
        &self,
        live: &mut LiveTrace,
        document: &NativeDocumentHandle,
        notice: &NativeWindowNotice,
        observed: Option<&str>,
    ) -> Result<Option<(bool, bool)>, String> {
        if observed != Some(live.case.route()) || !live.active {
            return Ok(None);
        }
        let frame = notice
            .frame
            .as_ref()
            .ok_or("journey actual frame missing")?;
        let journey = live.journey.as_mut().ok_or("journey state missing")?;
        let fields = journey_fields(&journey.original, live.case)?;
        if live.chooser_cycle == 0
            && journey
                .owner
                .as_ref()
                .is_some_and(|owner| owner.owner != frame.document.owner)
        {
            return Err("journey mutation crossed its original document owner".into());
        }
        if matches!(
            journey.progress,
            CommitProgress::Saving | CommitProgress::Accepting
        ) && live.save_observation_pending
            && !live.save_observation_woken
        {
            live.save_observation_pending = false;
            live.save_observation_woken = true;
            if matches!(journey.progress, CommitProgress::Accepting) {
                journey.diagnose_accept(
                    self,
                    AcceptObservation {
                        wait: "observation-wake",
                        phase: live.phase,
                        dirty: live.dirty,
                        pending: live.pending,
                        wake: (live.save_observation_pending, live.save_observation_woken),
                        frame: frame.document.document_revision,
                        inspected: None,
                        feedback: None,
                    },
                );
            }
            self.record(
                Kind::LiveSaveObservationWake,
                &(
                    stamp(&frame.document),
                    "journey original post-dispatch observation",
                ),
            );
            return Ok(Some((true, false)));
        }
        match journey.progress {
            CommitProgress::Initial if live.phase == Phase::AwaitingActivation => {
                if !live.chooser_active_recorded || live.dirty || live.pending {
                    return Err("journey entry is not the original clean inspected owner".into());
                }
                let original = current_input_fields(&journey.original, live.case)?;
                let projection =
                    current_input_projection(document, &journey.original, live.case, &original)?;
                self.record(
                    Kind::LiveInputDisplayed,
                    &("journey-initial", stamp(&frame.document), projection),
                );
                journey.owner = Some(frame.document.clone());
                set_text(document, fields[0].0, &fields[0].1)?;
                journey.progress = CommitProgress::Changed(0);
                live.phase = Phase::Editing;
                Ok(Some((true, false)))
            }
            CommitProgress::Changed(index) => {
                let mut shown = current_input_fields(&journey.original, live.case)?;
                for (shown, expected) in shown.iter_mut().zip(&fields).take(index + 1) {
                    shown.1.clone_from(&expected.1);
                }
                if !live.dirty || live.pending || !current_input_values_match(document, &shown)? {
                    return Ok(Some((false, false)));
                }
                if source_enabled(document, "onAccept")? {
                    return Err(
                        "dirty journey exposed an enabled Accept instead of SaveRequired".into(),
                    );
                }
                let projection =
                    current_input_projection(document, &journey.original, live.case, &shown)?;
                self.record(
                    Kind::LiveInputDisplayed,
                    &("journey-changed", index, stamp(&frame.document), projection),
                );
                if index + 1 < fields.len() {
                    set_text(document, fields[index + 1].0, &fields[index + 1].1)?;
                    journey.progress = CommitProgress::Changed(index + 1);
                } else {
                    if !source_enabled(document, "onSave")? {
                        return Err("journey source Save is disabled".into());
                    }
                    journey.action_start = journey
                        .calls
                        .lock()
                        .map_err(|_| "journey calls poisoned")?
                        .len();
                    save_current_source(document)?;
                    self.record(
                        Kind::LiveSaveRequested,
                        &"journey explicit source Save once; no automatic Accept",
                    );
                    journey.progress = CommitProgress::Saving;
                    live.phase = Phase::Saving;
                }
                Ok(Some((true, false)))
            }
            CommitProgress::Saving => {
                let Some(saved) = journey.mutation_inspection("proposals.edit")? else {
                    return Ok(Some((false, false)));
                };
                if live.phase != Phase::ReturnReady
                    || !journey.settled()
                    || live.dirty
                    || live.pending
                    || !journey_feedback_ready(document, &saved)?
                {
                    return Ok(Some((false, false)));
                }
                if saved.acceptance.is_some() || current_input_fields(&saved, live.case)? != fields
                {
                    return Err(
                        "journey Save inspection differs from the actual edited fields".into(),
                    );
                }
                let projection = current_input_projection(document, &saved, live.case, &fields)?;
                self.record(
                    Kind::LiveInputDisplayed,
                    &(
                        "journey-saved",
                        stamp(&frame.document),
                        projection,
                        saved.document.draft_revision,
                    ),
                );
                if !source_enabled(document, "onAccept")? {
                    return Err("clean saved journey has no enabled explicit Accept".into());
                }
                journey.saved = Some(saved);
                journey.action_start = journey
                    .calls
                    .lock()
                    .map_err(|_| "journey calls poisoned")?
                    .len();
                activate(document, "onAccept", None)?;
                self.record(
                    Kind::LiveInputDisplayed,
                    &("journey-accept-requested", stamp(&frame.document)),
                );
                journey.progress = CommitProgress::Accepting;
                journey.guards = None;
                // A distinct original mutation gets its one existing observation wake.
                // Global event/checkpoint/deadline budgets and snapshot counts are not reset.
                live.save_observation_pending = false;
                live.save_observation_woken = false;
                // The separate Accept needs the same bounded ReturnReady observation as Save.
                live.source_redraw_sent = false;
                live.phase = Phase::Saving;
                journey.diagnose_accept(
                    self,
                    AcceptObservation {
                        wait: "dispatched",
                        phase: live.phase,
                        dirty: live.dirty,
                        pending: live.pending,
                        wake: (live.save_observation_pending, live.save_observation_woken),
                        frame: frame.document.document_revision,
                        inspected: None,
                        feedback: None,
                    },
                );
                Ok(Some((true, false)))
            }
            CommitProgress::Accepting => {
                let Some(accepted) = journey.mutation_inspection("proposals.accept")? else {
                    journey.diagnose_accept(
                        self,
                        AcceptObservation {
                            wait: "inspection",
                            phase: live.phase,
                            dirty: live.dirty,
                            pending: live.pending,
                            wake: (live.save_observation_pending, live.save_observation_woken),
                            frame: frame.document.document_revision,
                            inspected: None,
                            feedback: None,
                        },
                    );
                    return Ok(Some((false, false)));
                };
                if live.phase != Phase::ReturnReady
                    || !journey.settled()
                    || live.dirty
                    || live.pending
                    || !journey_feedback_ready(document, &accepted)?
                {
                    let feedback = (live.phase == Phase::ReturnReady
                        && journey.settled()
                        && !live.dirty
                        && !live.pending)
                        .then_some(false);
                    journey.diagnose_accept(
                        self,
                        AcceptObservation {
                            wait: "guard-or-feedback",
                            phase: live.phase,
                            dirty: live.dirty,
                            pending: live.pending,
                            wake: (live.save_observation_pending, live.save_observation_woken),
                            frame: frame.document.document_revision,
                            inspected: Some(accepted.document.draft_revision.0),
                            feedback,
                        },
                    );
                    return Ok(Some((false, false)));
                }
                journey.diagnose_accept(
                    self,
                    AcceptObservation {
                        wait: "ready",
                        phase: live.phase,
                        dirty: live.dirty,
                        pending: live.pending,
                        wake: (live.save_observation_pending, live.save_observation_woken),
                        frame: frame.document.document_revision,
                        inspected: Some(accepted.document.draft_revision.0),
                        feedback: Some(true),
                    },
                );
                if accepted.acceptance.is_none()
                    || current_input_fields(&accepted, live.case)? != fields
                {
                    return Err(
                        "journey accepted inspection lost the actual saved fields/receipt".into(),
                    );
                }
                let saved = journey
                    .saved
                    .as_ref()
                    .ok_or("journey original saved inspection missing")?;
                if serde_json::to_value(&accepted.document.draft)
                    .map_err(|error| format!("{error:?}"))?
                    != serde_json::to_value(&saved.document.draft)
                        .map_err(|error| format!("{error:?}"))?
                {
                    return Err("journey acceptance changed the original saved draft".into());
                }
                let mut projection =
                    current_input_projection(document, &accepted, live.case, &fields)?;
                projection["accept_dispatched"] = serde_json::Value::Bool(true);
                self.record(
                    Kind::LiveInputDisplayed,
                    &(
                        "journey-accepted",
                        stamp(&frame.document),
                        projection,
                        accepted.document.draft_revision,
                        accepted
                            .acceptance
                            .as_ref()
                            .map(|receipt| receipt.output_revision),
                    ),
                );
                journey.accepted = Some(accepted);
                journey.progress = CommitProgress::Accepted;
                Ok(None) // Only now admit the original source VIEW action.
            }
            CommitProgress::Accepted
                if live.chooser_cycle == 1 && live.phase == Phase::AwaitingActivation =>
            {
                let accepted = journey
                    .accepted
                    .as_ref()
                    .ok_or("journey accepted inspection missing")?;
                if live.dirty
                    || live.pending
                    || !live.chooser_active_recorded
                    || !journey_feedback_ready(document, accepted)?
                {
                    return Ok(Some((false, false)));
                }
                let mut projection =
                    current_input_projection(document, accepted, live.case, &fields)?;
                projection["accept_dispatched"] = serde_json::Value::Bool(true);
                self.record(
                    Kind::LiveInputDisplayed,
                    &(
                        "journey-reopened",
                        stamp(&frame.document),
                        projection,
                        accepted.document.draft_revision,
                        accepted
                            .acceptance
                            .as_ref()
                            .map(|receipt| receipt.output_revision),
                    ),
                );
                self.record(
                    Kind::ChooserJourneyComplete,
                    &(live.case, stamp(&frame.document), true),
                );
                journey.progress = CommitProgress::Reopened;
                live.phase = Phase::Complete;
                Ok(Some((false, true)))
            }
            CommitProgress::Initial | CommitProgress::Accepted | CommitProgress::Reopened => {
                Ok(None)
            }
        }
    }
}

fn journey_fields(
    original: &crate::domain::proposals::StoredProposal,
    case: Case,
) -> Result<Vec<(&'static str, String)>, String> {
    let mut fields = current_input_fields(original, case)?;
    for (index, (_, value)) in fields.iter_mut().enumerate() {
        *value = input_technical_text(index).into();
    }
    Ok(fields)
}
fn journey_feedback_ready(
    document: &NativeDocumentHandle,
    stored: &crate::domain::proposals::StoredProposal,
) -> Result<bool, String> {
    use crate::presentation::web_ui_adapter::proposals::product_window::ProductWindowLabels;
    let labels = ProductWindowLabels::korean().panel;
    let mut revision = format!("{} {}", labels.revision, stored.document.draft_revision.0);
    let status = if let Some(receipt) = &stored.acceptance {
        revision.push_str(&format!(
            "{}{} {}",
            labels.feedback.separator, labels.feedback.output_revision, receipt.output_revision.0
        ));
        &labels.feedback.accepted
    } else {
        &labels.feedback.stored
    };
    for (module, tag, value) in [
        ("components/proposal-source.tsx", "dd", revision.as_str()),
        ("components/proposal-actions.tsx", "p", status.as_str()),
    ] {
        let nodes = source_text_nodes(document, module, tag)?;
        let matches = nodes.iter().filter(|(_, text)| text == value).count();
        if matches > 1 {
            return Err("journey current feedback is ambiguous".into());
        }
        if matches == 0 {
            return Ok(false);
        }
    }
    Ok(true)
}
