use super::*;
use crate::application::api::ReviewClient;
use crate::{
    application::api::{PlayerSdk, Request, RequestContext},
    domain::proposals::output_review::*,
};

fn accepted() -> StoredProposal {
    let mut state = snapshot();
    let output = ProposalPayload::Correction {
        edits: vec![Correction {
            cue_id: "cue-1".into(),
            text: "Synthetic output.".into(),
        }],
    };
    state.document.draft = Some(output.clone());
    state.document.state = DraftState::Editable;
    state.document.import_error = None;
    state.document.draft_revision = DecimalU64(2);
    state.document.generated.raw_text = serde_json::to_string(&output).unwrap();
    state.acceptance = Some(AcceptanceReceipt {
        proposal_id: "proposal-1".into(),
        draft_revision: DecimalU64(1),
        output_revision: DecimalU64(2),
        source_id: "source-1".into(),
        source_revision: DecimalU64(1),
        source_sha256: "a".repeat(64),
        output,
        automatic_application_enabled: false,
    });
    state
}
struct Store {
    state: StoredProposal,
    appends: Arc<AtomicUsize>,
}
struct Session<'a>(&'a Store);
impl ProposalStore for Store {
    fn load(&self, _: &str) -> Result<StoredProposal, ApiError> {
        panic!("wire must use canonical session")
    }
    fn publish(&self, _: Option<u64>, _: &StoredProposal) -> Result<(), ApiError> {
        panic!("wire must use session append")
    }
    fn output_review_available(&self) -> bool {
        true
    }
    fn open_output_review(
        &self,
        _: &str,
    ) -> Result<Option<Box<dyn OutputReviewSession + '_>>, ApiError> {
        Ok(Some(Box::new(Session(self))))
    }
    fn output_decisions(&self, _: &str) -> Result<OutputDecisionHistory, ApiError> {
        Ok(OutputDecisionHistory {
            records: Vec::new(),
            current_evidence: CurrentEvidenceStatus::NotChecked,
        })
    }
}
impl OutputReviewSession for Session<'_> {
    fn current(&self) -> &StoredProposal {
        &self.0.state
    }
    fn verify_current_evidence(&self) -> Result<(), ApiError> {
        Ok(())
    }
    fn append(&mut self, _: &PreparedOutputDecision) -> Result<StoredOutputDecision, ApiError> {
        self.0.appends.fetch_add(1, Ordering::SeqCst);
        Err(ApiError::new(
            ErrorCode::EngineFailed,
            "Synthetic append boundary",
        ))
    }
}
fn decision(state: &StoredProposal) -> RecordOutputDecision {
    let criteria = criteria_for(state.document.generated.task).unwrap();
    RecordOutputDecision {
        proposal_id: "proposal-1".into(),
        decision: OutputDecisionInput {
            decision_id: "decision-1".into(),
            target: OutputReviewTarget::from_stored(state).unwrap(),
            criteria: criteria.identity().unwrap(),
            verdict: OutputVerdict::Deferred,
            note: "Synthetic only".into(),
            declared_reviewer: DeclaredReviewer {
                label: "Declared fixture".into(),
                reviewer_id: None,
            },
            claimed_review_unix_ms: None,
        },
    }
}
#[test]
fn optional_some_without_advertised_capability_is_not_publicly_supported() {
    let sessions = Arc::new(AtomicUsize::new(0));
    let store = BorrowingStore {
        snapshot: snapshot(),
        sessions: sessions.clone(),
    };
    assert!(
        matches!(inspect_target(&store,InspectOutputReviewTarget {proposal_id:"proposal-1".into()}),Err(error) if error.code==ErrorCode::Unsupported)
    );
    assert_eq!(sessions.load(Ordering::SeqCst), 0);
    let legacy = LegacyStore::default();
    assert!(
        output_decisions(
            &legacy,
            ListOutputDecisions {
                proposal_id: "proposal-1".into()
            }
        )
        .is_err()
    );
    no_calls(&legacy);
}
#[test]
fn typed_client_inspect_and_history_share_canonical_session_and_wire_scope() {
    let state = accepted();
    let appends = Arc::new(AtomicUsize::new(0));
    let client = PlayerSdk::default().with_proposal_store(Store {
        state: state.clone(),
        appends: appends.clone(),
    });
    let view = client
        .inspect_output_review_target(
            RequestContext::default(),
            InspectOutputReviewTarget {
                proposal_id: "proposal-1".into(),
            },
        )
        .unwrap();
    view.target.validate_against(&view.state).unwrap();
    assert_eq!(
        view.criteria_identity,
        view.criteria_document.identity().unwrap()
    );
    assert_eq!(
        view.evidence_observation,
        EvidenceObservation::NotHistoricallyBound
    );
    assert_eq!(
        serde_json::to_value(view.state).unwrap(),
        serde_json::to_value(state).unwrap()
    );
    let history = client
        .output_decisions(
            RequestContext::default(),
            ListOutputDecisions {
                proposal_id: "proposal-1".into(),
            },
        )
        .unwrap();
    assert_eq!(history.current_evidence, CurrentEvidenceStatus::NotChecked);
    assert_eq!(appends.load(Ordering::SeqCst), 0);
}
#[test]
fn context_and_route_failures_precede_append_and_original_append_error_survives() {
    let state = accepted();
    let appends = Arc::new(AtomicUsize::new(0));
    let client = PlayerSdk::default().with_proposal_store(Store {
        state: state.clone(),
        appends: appends.clone(),
    });
    let payload = decision(&state);
    let mut context = RequestContext::default();
    assert_eq!(
        client
            .record_output_decision(context.clone(), payload.clone())
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    context.expected_revision = Some("1".into());
    assert_eq!(
        client
            .record_output_decision(context.clone(), payload.clone())
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    context.expected_revision = Some("2".into());
    context.idempotency_key = Some("transport".into());
    assert_eq!(
        client
            .record_output_decision(context.clone(), payload.clone())
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    context.idempotency_key = None;
    let mut routed = payload.clone();
    routed.proposal_id = "other".into();
    assert_eq!(
        client
            .record_output_decision(context.clone(), routed)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(appends.load(Ordering::SeqCst), 0);
    let error = client.record_output_decision(context, payload).unwrap_err();
    assert_eq!(error.code, ErrorCode::EngineFailed);
    assert_eq!(error.message, "Synthetic append boundary");
    assert_eq!(appends.load(Ordering::SeqCst), 1);
}
#[test]
fn missing_acceptance_and_unknown_request_fields_do_not_create_quality_decisions() {
    let original = accepted();
    let mut edited = original.clone();
    edited.acceptance = None;
    edited.document.draft_revision = DecimalU64(3);
    let appends = Arc::new(AtomicUsize::new(0));
    let client = PlayerSdk::default().with_proposal_store(Store {
        state: edited,
        appends: appends.clone(),
    });
    assert_eq!(
        client
            .inspect_output_review_target(
                RequestContext::default(),
                InspectOutputReviewTarget {
                    proposal_id: "proposal-1".into()
                }
            )
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    let mut request = Request::new("proposals.record_output_decision");
    request.context.expected_revision = Some("2".into());
    request.payload = serde_json::to_value(decision(&original)).unwrap();
    assert_eq!(
        client.execute(request.clone()).error.unwrap().code,
        ErrorCode::RevisionConflict
    );
    request.payload["observed_record_time"] = serde_json::json!({"unix_ms":"23"});
    assert_eq!(
        client.execute(request).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(appends.load(Ordering::SeqCst), 0);
}
