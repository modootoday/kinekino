use super::*;
use crate::test_support::review_package;

fn state() -> RawReviewState {
    let mut sequence = 0;
    RawReviewState::new(Box::new(move || {
        sequence += 1;
        Ok(format!("synthetic-raw-decision-{sequence}"))
    }))
}
fn accepted(
    state: &mut RawReviewState,
    token: RawReviewToken,
    outcome: Result<PendingOutput, ApiError>,
) -> bool {
    state.complete(token, outcome).is_ok()
}
fn inspect(state: &mut RawReviewState) {
    let (token, _) = state
        .prepare(RawReviewAction::Inspect("a".repeat(64)))
        .unwrap()
        .unwrap();
    let mut package = review_package::package();
    let mut other = package.items[0].clone();
    other.item_id = "sample-002".into();
    package.items.push(other);
    assert!(accepted(
        state,
        token,
        Ok(PendingOutput::RawReviewInspected(Box::new(
            ReviewPackageView {
                package_sha256: "a".repeat(64),
                package,
                evidence_verified: true,
                review_ready: true,
                automatic_application_enabled: false,
            }
        )))
    ));
}
#[test]
fn late_asset_and_history_cannot_overwrite_a_different_item_selection() {
    let mut state = state();
    inspect(&mut state);
    state
        .prepare(RawReviewAction::Select("sample-001".into()))
        .unwrap();
    let (read, request) = state.prepare(RawReviewAction::ReadSource).unwrap().unwrap();
    let PendingRequest::ReadRawReviewAsset(request) = request else {
        panic!("typed asset request required");
    };
    let (history, _) = state.prepare(RawReviewAction::History).unwrap().unwrap();
    state
        .prepare(RawReviewAction::Select("sample-002".into()))
        .unwrap();
    assert!(!accepted(
        &mut state,
        read,
        Ok(PendingOutput::RawReviewAsset(ReviewTextAsset {
            package_sha256: "a".repeat(64),
            asset_sha256: request.asset_sha256,
            bytes: kinekino_player::domain::api_values::DecimalU64(
                review_package::SOURCE.len() as u64
            ),
            media_type: "text/plain".into(),
            utf8: "late original item".into(),
        }))
    ));
    assert!(!accepted(
        &mut state,
        history,
        Ok(PendingOutput::RawReviewHistory(Vec::new()))
    ));
    assert_eq!(state.selected_item.as_deref(), Some("sample-002"));
    assert!(state.assets.is_empty());
    assert!(state.history.is_none());
    assert!(state.saved.is_none());
}
#[test]
fn raw_rejection_uses_original_package_item_without_editing_or_accepting_proposal() {
    let mut state = state();
    inspect(&mut state);
    state
        .prepare(RawReviewAction::Select("sample-001".into()))
        .unwrap();
    state
        .prepare(RawReviewAction::Verdict(ReviewVerdict::NeedsChanges))
        .unwrap();
    state
        .prepare(RawReviewAction::Note(
            "Synthetic fixture verdict; not a human approval".into(),
        ))
        .unwrap();
    let (token, request) = state.prepare(RawReviewAction::Record).unwrap().unwrap();
    let PendingRequest::RecordRawReview(input) = request else {
        panic!("raw decision must use reviews contract");
    };
    assert_eq!(input.package_sha256, "a".repeat(64));
    assert_eq!(input.item_id, "sample-001");
    assert_eq!(input.verdict, ReviewVerdict::NeedsChanges);
    assert!(input.issue_ranges.is_empty());
    let decision = input
        .bind(&state.package.as_ref().unwrap().package)
        .unwrap();
    assert!(!decision.automatic_application_enabled);
    assert!(accepted(
        &mut state,
        token,
        Ok(PendingOutput::RawReviewRecorded(StoredDecision {
            decision_sha256: "b".repeat(64),
            decision,
        }))
    ));
    assert_eq!(
        state.saved.as_ref().unwrap().decision.verdict,
        ReviewVerdict::NeedsChanges
    );
    assert!(state.history.is_none());
}
#[test]
fn unconfirmed_record_requires_explicit_history_and_close_invalidates_view_only() {
    let mut state = state();
    inspect(&mut state);
    state
        .prepare(RawReviewAction::Select("sample-001".into()))
        .unwrap();
    state
        .prepare(RawReviewAction::Verdict(ReviewVerdict::Deferred))
        .unwrap();
    state
        .prepare(RawReviewAction::Note("Fixture only".into()))
        .unwrap();
    let (token, _) = state.prepare(RawReviewAction::Record).unwrap().unwrap();
    assert!(accepted(
        &mut state,
        token,
        Err(ApiError::new(
            ErrorCode::EngineFailed,
            "record result unavailable"
        ))
    ));
    assert!(state.record_result_unknown);
    assert!(state.prepare(RawReviewAction::Record).is_err());
    let (history, _) = state.prepare(RawReviewAction::History).unwrap().unwrap();
    assert!(accepted(
        &mut state,
        history,
        Ok(PendingOutput::RawReviewHistory(Vec::new()))
    ));
    assert!(!state.record_result_unknown);
    let (late, _) = state.prepare(RawReviewAction::History).unwrap().unwrap();
    state.invalidate();
    assert!(!accepted(
        &mut state,
        late,
        Ok(PendingOutput::RawReviewHistory(Vec::new()))
    ));
    assert_eq!(state.selected_item.as_deref(), Some("sample-001"));
}
#[test]
fn native12_is_opt_in_and_preserves_legacy_capability_sets() {
    use kinekino_player::assembly::meari::proposal_manifest::*;
    use meari::domain::build::TargetKind;
    assert_eq!(proposal_manifest().capabilities.len(), 4);
    assert_eq!(async_proposal_manifest().capabilities.len(), 5);
    assert_eq!(output_review_proposal_manifest().capabilities.len(), 8);
    let raw = raw_review_proposal_manifest();
    assert_eq!(raw.capabilities.len(), 12);
    for command in RAW_REVIEW_CAPABILITIES {
        let cap = &raw.capabilities[command];
        assert!(cap.targets.contains(&TargetKind::Native));
        assert!(!cap.targets.contains(&TargetKind::Web));
        assert!(cap.error_schema.is_some());
        assert!(cap.event_schema.is_none());
    }
}

#[test]
fn rejected_late_error_retains_exact_token_operation_and_original_error_allocation() {
    let mut state = state();
    inspect(&mut state);
    state
        .prepare(RawReviewAction::Select("sample-001".into()))
        .unwrap();
    state
        .prepare(RawReviewAction::Verdict(ReviewVerdict::Deferred))
        .unwrap();
    state
        .prepare(RawReviewAction::Note("Fixture only".into()))
        .unwrap();
    let (token, _) = state.prepare(RawReviewAction::Record).unwrap().unwrap();
    let owner = token.owner.clone();
    let error = ApiError::new(ErrorCode::EngineFailed, "original raw record failure");
    let original = error.message.as_ptr();
    state
        .prepare(RawReviewAction::Select("sample-002".into()))
        .unwrap();
    let rejected = match state.complete(token, Err(error)) {
        Err(rejected) => rejected,
        Ok(()) => panic!("late raw error must return the original carrier"),
    };
    assert!(Arc::ptr_eq(&owner, &rejected.token.owner));
    assert_eq!(rejected.token.package_sha256, "a".repeat(64));
    assert_eq!(rejected.token.item_id.as_deref(), Some("sample-001"));
    assert!(
        matches!(&rejected.token.operation, Operation::Record(id) if id == "synthetic-raw-decision-1")
    );
    assert_eq!(
        rejected.outcome.as_ref().err().unwrap().message.as_ptr(),
        original
    );
    assert_eq!(state.selected_item.as_deref(), Some("sample-002"));
    assert!(state.saved.is_none());
    assert!(state.error.is_none());
}
