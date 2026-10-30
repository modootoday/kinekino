//! Presentation selection and request lifetime only; SDK owns validation, CAS and persistence.
use super::pending_host::{PendingOutput, PendingRequest};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_player::{
    application::proposals::output_review::{
        InspectOutputReviewTarget, ListOutputDecisions, OutputDecisionHistory,
        OutputReviewInspection, RecordOutputDecision, StoredOutputDecision,
    },
    domain::{api_values::DecimalU64, proposals::output_review::*},
};
use std::rc::Rc;

#[derive(Default)]
pub struct DecisionFields {
    pub verdict: Option<OutputVerdict>,
    pub note: String,
    pub reviewer_label: String,
    pub reviewer_id: Option<String>,
    pub claimed_review_unix_ms: Option<DecimalU64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputReviewPhase {
    Idle,
    Inspecting,
    Ready,
    Recording,
    LoadingHistory,
    Recorded,
    RefreshRequired,
    ResultUnknown,
    Unavailable,
    NeedsInput,
    Failed,
}
#[derive(Debug, PartialEq, Eq)]
pub enum OutputReviewInputError {
    Busy,
    NotReady,
    NotSelected,
    RequiredInput,
    StaleCompletion,
    UnexpectedOutput,
}
#[derive(Clone)]
pub struct OutputReviewToken(Rc<()>);
#[derive(Clone, Copy)]
enum Operation {
    Inspect,
    Record,
    History,
}
struct Active {
    token: OutputReviewToken,
    operation: Operation,
    invalidated: bool,
}
pub struct OutputReviewState {
    proposal_id: String,
    inspection: Option<OutputReviewInspection>,
    selected: bool,
    fields: DecisionFields,
    phase: OutputReviewPhase,
    active: Option<Active>,
    history: Option<OutputDecisionHistory>,
    confirmed: Option<StoredOutputDecision>,
    attempted: Option<RecordOutputDecision>,
    result_unknown: bool,
    error: Option<ApiError>,
}
impl OutputReviewState {
    pub fn new(proposal_id: String) -> Self {
        Self {
            proposal_id,
            inspection: None,
            selected: false,
            fields: DecisionFields::default(),
            phase: OutputReviewPhase::Idle,
            active: None,
            history: None,
            confirmed: None,
            attempted: None,
            result_unknown: false,
            error: None,
        }
    }
    pub fn phase(&self) -> OutputReviewPhase {
        self.phase
    }
    pub fn inspection(&self) -> Option<&OutputReviewInspection> {
        self.inspection.as_ref()
    }
    pub fn selected(&self) -> bool {
        self.selected
    }
    pub fn fields(&self) -> &DecisionFields {
        &self.fields
    }
    pub fn history(&self) -> Option<&OutputDecisionHistory> {
        self.history.as_ref()
    }
    pub fn confirmed(&self) -> Option<&StoredOutputDecision> {
        self.confirmed.as_ref()
    }
    pub fn attempted(&self) -> Option<&RecordOutputDecision> {
        self.attempted.as_ref()
    }
    pub fn error(&self) -> Option<&ApiError> {
        self.error.as_ref()
    }
    pub fn busy(&self) -> bool {
        self.active.is_some()
    }
    pub fn result_unknown(&self) -> bool {
        self.result_unknown
    }
    pub fn can_retry(&self) -> bool {
        !self.busy()
            && self.result_unknown
            && self.selected
            && self
                .inspection
                .as_ref()
                .zip(self.attempted.as_ref())
                .is_some_and(|(inspection, attempted)| {
                    inspection.target == attempted.decision.target
                        && inspection.criteria_identity == attempted.decision.criteria
                })
    }
    pub fn replace_fields(&mut self, fields: DecisionFields) -> Result<(), OutputReviewInputError> {
        self.available()?;
        if self.inspection.is_none() || self.result_unknown {
            return Err(OutputReviewInputError::NotReady);
        }
        self.fields = fields;
        Ok(())
    }
    /// Called by an explicit target-selection action after the inspection is presented.
    pub fn select(&mut self) -> Result<(), OutputReviewInputError> {
        self.available()?;
        if self.inspection.is_none() {
            return Err(OutputReviewInputError::NotReady);
        }
        self.selected = true;
        Ok(())
    }
    /// Assembly calls this when the draft/source/acceptance or document scope changes.
    /// An in-flight ticket must still drain through the existing host and provider ACK.
    pub fn invalidate(&mut self) {
        self.inspection = None;
        self.selected = false;
        self.fields = DecisionFields::default();
        if let Some(active) = &mut self.active {
            active.invalidated = true;
        }
        self.phase = OutputReviewPhase::RefreshRequired;
    }
    fn available(&self) -> Result<(), OutputReviewInputError> {
        if self.busy() {
            Err(OutputReviewInputError::Busy)
        } else {
            Ok(())
        }
    }
    fn begin(
        &mut self,
        operation: Operation,
        phase: OutputReviewPhase,
        request: PendingRequest,
    ) -> Result<(OutputReviewToken, PendingRequest), OutputReviewInputError> {
        self.available()?;
        let token = OutputReviewToken(Rc::new(()));
        self.active = Some(Active {
            token: token.clone(),
            operation,
            invalidated: false,
        });
        self.phase = phase;
        self.error = None;
        Ok((token, request))
    }
    pub fn inspect(
        &mut self,
    ) -> Result<(OutputReviewToken, PendingRequest), OutputReviewInputError> {
        self.available()?;
        self.inspection = None;
        self.selected = false;
        self.fields = DecisionFields::default();
        self.begin(
            Operation::Inspect,
            OutputReviewPhase::Inspecting,
            PendingRequest::InspectOutputReview(InspectOutputReviewTarget {
                proposal_id: self.proposal_id.clone(),
            }),
        )
    }
    pub fn load_history(
        &mut self,
    ) -> Result<(OutputReviewToken, PendingRequest), OutputReviewInputError> {
        self.begin(
            Operation::History,
            OutputReviewPhase::LoadingHistory,
            PendingRequest::OutputDecisions(ListOutputDecisions {
                proposal_id: self.proposal_id.clone(),
            }),
        )
    }
    pub fn record(
        &mut self,
        decision_id: String,
    ) -> Result<(OutputReviewToken, PendingRequest), OutputReviewInputError> {
        self.available()?;
        if self.result_unknown {
            return Err(OutputReviewInputError::NotReady);
        }
        if !self.selected {
            return Err(OutputReviewInputError::NotSelected);
        }
        let inspection = self
            .inspection
            .as_ref()
            .ok_or(OutputReviewInputError::NotReady)?;
        let verdict = self
            .fields
            .verdict
            .ok_or(OutputReviewInputError::RequiredInput)?;
        if decision_id.trim().is_empty()
            || self.fields.note.trim().is_empty()
            || self.fields.reviewer_label.trim().is_empty()
        {
            return Err(OutputReviewInputError::RequiredInput);
        }
        let input = RecordOutputDecision {
            proposal_id: self.proposal_id.clone(),
            decision: OutputDecisionInput {
                decision_id,
                target: inspection.target.clone(),
                criteria: inspection.criteria_identity.clone(),
                verdict,
                note: self.fields.note.clone(),
                declared_reviewer: DeclaredReviewer {
                    label: self.fields.reviewer_label.clone(),
                    reviewer_id: self.fields.reviewer_id.clone(),
                },
                claimed_review_unix_ms: self.fields.claimed_review_unix_ms,
            },
        };
        self.start_record(input)
    }
    fn start_record(
        &mut self,
        input: RecordOutputDecision,
    ) -> Result<(OutputReviewToken, PendingRequest), OutputReviewInputError> {
        let revision = input.decision.target.output_revision;
        self.attempted = Some(input.clone());
        self.begin(
            Operation::Record,
            OutputReviewPhase::Recording,
            PendingRequest::RecordOutputDecision {
                revision,
                input: Box::new(input),
            },
        )
    }
    /// Explicit retry only, with the original decision ID/body after inspect and reselection.
    pub fn retry_record(
        &mut self,
    ) -> Result<(OutputReviewToken, PendingRequest), OutputReviewInputError> {
        self.available()?;
        if !self.result_unknown || !self.selected {
            return Err(OutputReviewInputError::NotSelected);
        }
        let inspection = self
            .inspection
            .as_ref()
            .ok_or(OutputReviewInputError::NotReady)?;
        let input = self
            .attempted
            .clone()
            .ok_or(OutputReviewInputError::NotReady)?;
        if input.decision.target != inspection.target
            || input.decision.criteria != inspection.criteria_identity
        {
            return Err(OutputReviewInputError::NotReady);
        }
        self.start_record(input)
    }
    /// Only after the host no longer owns the terminal ticket; never abandon its cleanup.
    pub fn outcome_unknown(
        &mut self,
        token: &OutputReviewToken,
        error: ApiError,
    ) -> Result<(), OutputReviewInputError> {
        self.require_token(token)?;
        if matches!(
            self.active.as_ref().map(|a| a.operation),
            Some(Operation::Record)
        ) {
            self.result_unknown = true;
        }
        self.active = None;
        self.error = Some(error);
        self.invalidate();
        self.phase = if self.result_unknown {
            OutputReviewPhase::ResultUnknown
        } else {
            OutputReviewPhase::RefreshRequired
        };
        Ok(())
    }
    fn require_token(&self, token: &OutputReviewToken) -> Result<(), OutputReviewInputError> {
        if self
            .active
            .as_ref()
            .is_some_and(|a| Rc::ptr_eq(&a.token.0, &token.0))
        {
            Ok(())
        } else {
            Err(OutputReviewInputError::StaleCompletion)
        }
    }
    pub fn complete(
        &mut self,
        token: &OutputReviewToken,
        outcome: Result<PendingOutput, ApiError>,
    ) -> Result<(), OutputReviewInputError> {
        self.require_token(token)?;
        let active = self.active.as_ref().expect("token checked");
        if !matches!(
            (&active.operation, &outcome),
            (_, Err(_))
                | (
                    Operation::Inspect,
                    Ok(PendingOutput::OutputReviewInspected(_))
                )
                | (
                    Operation::Record,
                    Ok(PendingOutput::OutputDecisionRecorded(_))
                )
                | (
                    Operation::History,
                    Ok(PendingOutput::OutputDecisionsListed(_))
                )
        ) {
            return Err(OutputReviewInputError::UnexpectedOutput);
        }
        // A typed response still belongs to this view's exact request, not another tab.
        let owned_response = match &outcome {
            Ok(PendingOutput::OutputReviewInspected(inspection)) => {
                inspection.target.proposal_id == self.proposal_id
                    && inspection.state.document.proposal_id == self.proposal_id
            }
            Ok(PendingOutput::OutputDecisionRecorded(record)) => self
                .attempted
                .as_ref()
                .is_some_and(|attempted| record.decision.input == attempted.decision),
            Ok(PendingOutput::OutputDecisionsListed(history)) => history
                .records
                .iter()
                .all(|record| record.decision.input.target.proposal_id == self.proposal_id),
            Err(_) => true,
            _ => false,
        };
        if !owned_response {
            return Err(OutputReviewInputError::UnexpectedOutput);
        }
        let active = self.active.take().expect("token checked");
        match outcome {
            Ok(PendingOutput::OutputReviewInspected(inspection)) => {
                if active.invalidated || inspection.target.proposal_id != self.proposal_id {
                    self.invalidate();
                } else {
                    self.inspection = Some(*inspection);
                    self.phase = OutputReviewPhase::Ready;
                }
            }
            Ok(PendingOutput::OutputDecisionRecorded(record)) => {
                self.confirmed = Some(*record);
                self.result_unknown = false;
                self.selected = false;
                self.phase = if active.invalidated {
                    OutputReviewPhase::RefreshRequired
                } else {
                    OutputReviewPhase::Recorded
                };
            }
            Ok(PendingOutput::OutputDecisionsListed(history)) => {
                if self.result_unknown
                    && let Some(attempted) = &self.attempted
                    && let Some(record) = history
                        .records
                        .iter()
                        .find(|record| record.decision.input == attempted.decision)
                {
                    self.confirmed = Some(record.clone());
                    self.result_unknown = false;
                }
                self.history = Some(history);
                self.phase = if self.result_unknown {
                    OutputReviewPhase::ResultUnknown
                } else if active.invalidated {
                    OutputReviewPhase::RefreshRequired
                } else if self.inspection.is_some() {
                    OutputReviewPhase::Ready
                } else {
                    OutputReviewPhase::Idle
                };
            }
            Err(error) => {
                if matches!(active.operation, Operation::Record)
                    && (error
                        .details
                        .get("publication_may_exist")
                        .and_then(serde_json::Value::as_bool)
                        == Some(true)
                        || !matches!(
                            error.code,
                            ErrorCode::Unsupported
                                | ErrorCode::NeedsInput
                                | ErrorCode::RevisionConflict
                                | ErrorCode::InvalidArgument
                        ))
                {
                    self.result_unknown = true;
                }
                self.invalidate();
                self.phase = if self.result_unknown {
                    OutputReviewPhase::ResultUnknown
                } else {
                    match error.code {
                        ErrorCode::Unsupported => OutputReviewPhase::Unavailable,
                        ErrorCode::NeedsInput => OutputReviewPhase::NeedsInput,
                        ErrorCode::RevisionConflict => OutputReviewPhase::RefreshRequired,
                        _ => OutputReviewPhase::Failed,
                    }
                };
                self.error = Some(error);
            }
            _ => unreachable!("output kind checked"),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinekino_player::application::proposals::output_review::CurrentEvidenceStatus;

    fn empty_history() -> PendingOutput {
        PendingOutput::OutputDecisionsListed(OutputDecisionHistory {
            records: Vec::new(),
            current_evidence: CurrentEvidenceStatus::NotChecked,
        })
    }
    #[test]
    fn response_from_another_view_cannot_complete_the_active_request() {
        let mut first = OutputReviewState::new("first".into());
        let mut second = OutputReviewState::new("second".into());
        let (foreign, _) = first.load_history().unwrap();
        let (own, _) = second.load_history().unwrap();
        assert_eq!(
            second.complete(&foreign, Ok(empty_history())),
            Err(OutputReviewInputError::StaleCompletion)
        );
        assert!(second.busy());
        second.complete(&own, Ok(empty_history())).unwrap();
        assert!(!second.busy());
    }
    #[test]
    fn wrong_response_kind_does_not_release_the_pending_owner() {
        let mut state = OutputReviewState::new("proposal".into());
        let (token, _) = state.inspect().unwrap();
        assert_eq!(
            state.complete(&token, Ok(empty_history())),
            Err(OutputReviewInputError::UnexpectedOutput)
        );
        assert!(state.busy());
        state
            .complete(
                &token,
                Err(ApiError::new(ErrorCode::NeedsInput, "No accepted output")),
            )
            .unwrap();
        assert_eq!(state.phase(), OutputReviewPhase::NeedsInput);
        assert!(!state.selected());
    }
    #[test]
    fn history_after_invalidation_never_selects_a_quality_target() {
        let mut state = OutputReviewState::new("proposal".into());
        let (token, _) = state.load_history().unwrap();
        state.invalidate();
        assert_eq!(state.inspect().err(), Some(OutputReviewInputError::Busy));
        state.complete(&token, Ok(empty_history())).unwrap();
        assert_eq!(state.phase(), OutputReviewPhase::RefreshRequired);
        assert!(!state.selected());
        assert!(state.inspection().is_none());
        assert!(matches!(
            state.history().unwrap().current_evidence,
            CurrentEvidenceStatus::NotChecked
        ));
        assert_eq!(
            state.record("new-decision".into()).err(),
            Some(OutputReviewInputError::NotSelected)
        );
    }
    #[test]
    fn failed_inspection_does_not_default_to_a_verdict_or_reuse_a_token() {
        let mut state = OutputReviewState::new("proposal".into());
        let (old, _) = state.inspect().unwrap();
        state
            .complete(
                &old,
                Err(ApiError::new(ErrorCode::RevisionConflict, "Changed")),
            )
            .unwrap();
        let (_current, _) = state.inspect().unwrap();
        assert_eq!(
            state.complete(&old, Ok(empty_history())),
            Err(OutputReviewInputError::StaleCompletion)
        );
        assert!(state.fields().verdict.is_none());
        assert!(!state.can_retry());
    }
}
