use super::super::super::raw_review_state::RawReviewAction;
use super::*;
use crate::test_support::review_package;
use kinekino_player::{
    application::model_review::{ReviewPackageView, StoredDecision},
    domain::model_review::ReviewVerdict,
};

fn selected() -> (PendingReviewController, Rc<RefCell<Probe>>, ReviewScope) {
    let (mut controller, probe, _) = fixture_profile(true);
    let scope = controller.session.scope().clone();
    controller
        .enable_raw_reviews(Box::new(|| Ok("synthetic-record-1".into())))
        .unwrap();
    controller
        .raw_review_intent(&scope, RawReviewAction::Inspect("a".repeat(64)))
        .unwrap();
    reply(
        &probe,
        "reviews.inspect",
        Ok(ReviewPackageView {
            package_sha256: "a".repeat(64),
            package: review_package::package(),
            evidence_verified: true,
            review_ready: true,
            automatic_application_enabled: false,
        }),
    );
    assert!(controller.poll().unwrap());
    controller
        .raw_review_intent(&scope, RawReviewAction::Select("sample-001".into()))
        .unwrap();
    controller
        .raw_review_intent(&scope, RawReviewAction::Verdict(ReviewVerdict::Deferred))
        .unwrap();
    controller
        .raw_review_intent(
            &scope,
            RawReviewAction::Note("Synthetic fixture only".into()),
        )
        .unwrap();
    (controller, probe, scope)
}

#[test]
fn raw_record_ack_retry_and_parent_close_keep_typed_terminal_without_resubmission() {
    let (mut controller, probe, scope) = selected();
    let acks_before = probe.borrow().acks;
    controller
        .raw_review_intent(&scope, RawReviewAction::Record)
        .unwrap();
    reply::<StoredDecision>(
        &probe,
        "reviews.record_decision",
        Err(ApiError::new(
            ErrorCode::EngineFailed,
            "original record outcome unavailable",
        )),
    );
    probe.borrow_mut().fail_ack = true;
    assert!(controller.poll().is_err());
    assert!(controller.active.is_some());
    assert_eq!(controller.host.pending_tickets().len(), 1);
    assert!(controller.unprojected_raw_terminal.is_none());
    assert_eq!(probe.borrow().acks, acks_before);
    probe.borrow_mut().fail_ack = false;
    controller.quiesce_for_parent_close().unwrap();
    assert!(controller.active.is_none());
    assert!(controller.host.pending_tickets().is_empty());
    assert_eq!(probe.borrow().acks, acks_before + 1);
    assert_eq!(
        probe
            .borrow()
            .calls
            .iter()
            .filter(|command| command.as_str() == "reviews.record_decision")
            .count(),
        1
    );
    let carrier = controller.unprojected_raw_terminal.as_ref().unwrap();
    assert_eq!(
        carrier.transport.as_ref().unwrap().0.capability(),
        "reviews.record_decision"
    );
    assert_eq!(
        carrier.rejected.outcome.as_ref().err().unwrap().message,
        "original record outcome unavailable"
    );
    let original_carrier = &**carrier as *const RawReviewRetained;
    assert!(controller.available().is_err());
    assert!(!controller.poll().unwrap());
    controller.close_sdk().unwrap();
    assert_eq!(
        &**controller.unprojected_raw_terminal.as_ref().unwrap() as *const RawReviewRetained,
        original_carrier
    );
    assert!(controller.host.pending_tickets().is_empty());
    assert_eq!(probe.borrow().shutdowns, 9);
}

#[test]
fn raw_record_error_allows_explicit_history_without_save_accept_or_replay() {
    let (mut controller, probe, scope) = selected();
    controller
        .raw_review_intent(&scope, RawReviewAction::Record)
        .unwrap();
    reply::<StoredDecision>(
        &probe,
        "reviews.record_decision",
        Err(ApiError::new(
            ErrorCode::EngineFailed,
            "record result unavailable",
        )),
    );
    assert!(controller.poll().unwrap());
    assert!(controller.raw_review().unwrap().record_result_unknown);
    controller
        .raw_review_intent(&scope, RawReviewAction::Record)
        .unwrap();
    assert!(controller.active.is_none());
    controller
        .raw_review_intent(&scope, RawReviewAction::History)
        .unwrap();
    reply(
        &probe,
        "reviews.decisions",
        Ok(Vec::<StoredDecision>::new()),
    );
    assert!(controller.poll().unwrap());
    assert!(!controller.raw_review().unwrap().record_result_unknown);
    assert_eq!(
        probe
            .borrow()
            .calls
            .iter()
            .filter(|command| command.as_str() == "reviews.record_decision")
            .count(),
        1
    );
    assert!(
        !probe
            .borrow()
            .calls
            .iter()
            .any(|command| command == "proposals.edit" || command == "proposals.accept")
    );
    assert!(controller.host.pending_tickets().is_empty());
    controller.close_sdk().unwrap();
}

fn retained_live() -> (PendingReviewController, Rc<RefCell<Probe>>, ReviewScope) {
    let (mut controller, probe, scope) = selected();
    controller
        .raw_review_intent(&scope, RawReviewAction::Record)
        .unwrap();
    controller.invalidate_raw_review();
    reply::<StoredDecision>(
        &probe,
        "reviews.record_decision",
        Err(ApiError::new(
            ErrorCode::EngineFailed,
            "original uncertain record",
        )),
    );
    assert!(controller.poll().unwrap());
    assert!(controller.unprojected_raw_terminal.is_some());
    (controller, probe, scope)
}

fn recovery_inspected(
    controller: &mut PendingReviewController,
    probe: &Rc<RefCell<Probe>>,
    scope: &ReviewScope,
) {
    controller
        .raw_review_intent(scope, RawReviewAction::InspectRetained)
        .unwrap();
    reply(
        probe,
        "reviews.inspect",
        Ok(ReviewPackageView {
            package_sha256: "a".repeat(64),
            package: review_package::package(),
            evidence_verified: true,
            review_ready: true,
            automatic_application_enabled: false,
        }),
    );
    assert!(controller.poll().unwrap());
}

#[test]
fn retained_recovery_ack_retry_preserves_original_and_explicit_release_clears_selection() {
    let (mut controller, probe, scope) = retained_live();
    let mut foreign = scope.clone();
    foreign.proposal_id = "foreign-owner".into();
    let calls_before = probe.borrow().calls.len();
    assert!(
        controller
            .raw_review_intent(&foreign, RawReviewAction::InspectRetained)
            .is_err()
    );
    assert!(controller.raw_recovery.is_none());
    assert_eq!(probe.borrow().calls.len(), calls_before);
    let original =
        &**controller.unprojected_raw_terminal.as_ref().unwrap() as *const RawReviewRetained;
    controller.raw_review.as_mut().unwrap().package_input = "foreign edited selection".into();
    controller
        .raw_review_intent(&scope, RawReviewAction::InspectRetained)
        .unwrap();
    reply(
        &probe,
        "reviews.inspect",
        Ok(ReviewPackageView {
            package_sha256: "a".repeat(64),
            package: review_package::package(),
            evidence_verified: true,
            review_ready: true,
            automatic_application_enabled: false,
        }),
    );
    probe.borrow_mut().fail_ack = true;
    assert!(controller.poll().is_err());
    assert!(
        controller
            .raw_recovery
            .as_ref()
            .unwrap()
            .inspected
            .is_none()
    );
    assert!(
        controller
            .raw_review_intent(&scope, RawReviewAction::ReleaseRetained)
            .is_err()
    );
    assert_eq!(
        &**controller.unprojected_raw_terminal.as_ref().unwrap() as *const RawReviewRetained,
        original
    );
    probe.borrow_mut().fail_ack = false;
    assert!(controller.poll().unwrap());
    assert!(!controller.raw_recovery.as_ref().unwrap().can_release());
    controller
        .raw_review_intent(&scope, RawReviewAction::HistoryRetained)
        .unwrap();
    reply(
        &probe,
        "reviews.decisions",
        Ok(Vec::<StoredDecision>::new()),
    );
    assert!(controller.poll().unwrap());
    assert!(controller.raw_recovery.as_ref().unwrap().can_release());
    assert!(
        controller
            .raw_review_intent(&scope, RawReviewAction::Record)
            .is_err()
    );
    controller
        .raw_review_intent(&scope, RawReviewAction::ReleaseRetained)
        .unwrap();
    assert!(controller.unprojected_raw_terminal.is_none());
    let recovery = controller.raw_recovery.as_ref().unwrap();
    assert!(recovery.released);
    assert!(recovery.history.as_ref().unwrap().is_empty());
    assert!(recovery.summary().contains("다시 선택"));
    let state = controller.raw_review.as_ref().unwrap();
    assert!(state.package.is_none() && state.selected_item.is_none());
    assert!(state.package_input.is_empty() && state.item_input.is_empty());
    assert!(state.verdict.is_none() && state.attempted_record.is_none());
    controller
        .raw_review_intent(&scope, RawReviewAction::Record)
        .unwrap();
    assert!(controller.active.is_none());
    assert!(
        controller
            .raw_review_intent(&scope, RawReviewAction::ReleaseRetained)
            .is_err()
    );
    assert_eq!(
        probe
            .borrow()
            .calls
            .iter()
            .filter(|command| command.as_str() == "reviews.record_decision")
            .count(),
        1
    );
    assert!(
        !probe
            .borrow()
            .calls
            .iter()
            .any(|command| command == "proposals.edit" || command == "proposals.accept")
    );
    controller.close_sdk().unwrap();
}

#[test]
fn retained_recovery_read_failure_and_parent_close_never_release_or_replay() {
    let (mut controller, probe, scope) = retained_live();
    recovery_inspected(&mut controller, &probe, &scope);
    controller
        .raw_review_intent(&scope, RawReviewAction::HistoryRetained)
        .unwrap();
    reply::<Vec<StoredDecision>>(
        &probe,
        "reviews.decisions",
        Err(ApiError::new(
            ErrorCode::EngineFailed,
            "history unavailable",
        )),
    );
    assert!(controller.poll().unwrap());
    assert!(!controller.raw_recovery.as_ref().unwrap().can_release());
    assert!(
        controller
            .raw_review_intent(&scope, RawReviewAction::ReleaseRetained)
            .is_err()
    );
    let original =
        &**controller.unprojected_raw_terminal.as_ref().unwrap() as *const RawReviewRetained;
    assert_eq!(
        controller
            .unprojected_raw_terminal
            .as_ref()
            .unwrap()
            .rejected
            .outcome
            .as_ref()
            .err()
            .unwrap()
            .message,
        "original uncertain record"
    );
    controller
        .raw_review_intent(&scope, RawReviewAction::HistoryRetained)
        .unwrap();
    reply(
        &probe,
        "reviews.decisions",
        Ok(Vec::<StoredDecision>::new()),
    );
    probe.borrow_mut().fail_ack = true;
    assert!(controller.close_sdk().is_err());
    assert!(controller.active.is_some());
    assert!(
        controller
            .raw_review_intent(&scope, RawReviewAction::ReleaseRetained)
            .is_err()
    );
    probe.borrow_mut().fail_ack = false;
    controller.close_sdk().unwrap();
    assert_eq!(
        &**controller.unprojected_raw_terminal.as_ref().unwrap() as *const RawReviewRetained,
        original
    );
    assert!(controller.host.pending_tickets().is_empty());
    assert_eq!(
        probe
            .borrow()
            .calls
            .iter()
            .filter(|command| command.as_str() == "reviews.record_decision")
            .count(),
        1
    );
}

#[test]
fn retained_recovery_foreign_package_is_kept_typed_and_cannot_release() {
    let (mut controller, probe, scope) = retained_live();
    controller
        .raw_review_intent(&scope, RawReviewAction::InspectRetained)
        .unwrap();
    reply(
        &probe,
        "reviews.inspect",
        Ok(ReviewPackageView {
            package_sha256: "b".repeat(64),
            package: review_package::package(),
            evidence_verified: true,
            review_ready: true,
            automatic_application_enabled: false,
        }),
    );
    assert!(controller.poll().unwrap());
    let recovery = controller.raw_recovery.as_ref().unwrap();
    assert!(
        matches!(&recovery.unmatched, Some(Ok(PendingOutput::RawReviewInspected(view))) if view.package_sha256 == "b".repeat(64))
    );
    assert!(!recovery.can_release());
    assert!(
        controller
            .raw_review_intent(&scope, RawReviewAction::ReleaseRetained)
            .is_err()
    );
    assert!(
        controller
            .raw_review_intent(&scope, RawReviewAction::InspectRetained)
            .is_err()
    );
    controller.close_sdk().unwrap();
    assert!(controller.unprojected_raw_terminal.is_some());
}

#[test]
fn retained_package_only_inspect_requires_ack_and_releases_without_fabricated_history() {
    let (mut controller, probe, scope) = selected();
    controller
        .raw_review_intent(&scope, RawReviewAction::Inspect("a".repeat(64)))
        .unwrap();
    controller.invalidate_raw_review();
    reply(
        &probe,
        "reviews.inspect",
        Ok(ReviewPackageView {
            package_sha256: "a".repeat(64),
            package: review_package::package(),
            evidence_verified: true,
            review_ready: true,
            automatic_application_enabled: false,
        }),
    );
    assert!(controller.poll().unwrap());
    assert!(
        controller
            .unprojected_raw_terminal
            .as_ref()
            .unwrap()
            .rejected
            .token
            .package_only_read()
    );
    controller
        .raw_review_intent(&scope, RawReviewAction::InspectRetained)
        .unwrap();
    reply(
        &probe,
        "reviews.inspect",
        Ok(ReviewPackageView {
            package_sha256: "a".repeat(64),
            package: review_package::package(),
            evidence_verified: true,
            review_ready: true,
            automatic_application_enabled: false,
        }),
    );
    probe.borrow_mut().fail_ack = true;
    assert!(controller.poll().is_err());
    assert!(
        controller
            .raw_review_intent(&scope, RawReviewAction::ReleaseRetained)
            .is_err()
    );
    probe.borrow_mut().fail_ack = false;
    assert!(controller.poll().unwrap());
    assert!(
        controller
            .raw_review_intent(&scope, RawReviewAction::HistoryRetained)
            .is_err()
    );
    controller
        .raw_review_intent(&scope, RawReviewAction::ReleaseRetained)
        .unwrap();
    let recovery = controller.raw_recovery.as_ref().unwrap();
    assert!(recovery.released && recovery.history.is_none());
    assert!(recovery.summary().contains("이력은 조회하지 않았고"));
    assert!(
        !probe
            .borrow()
            .calls
            .iter()
            .any(|command| command == "reviews.decisions" || command == "reviews.record_decision")
    );
    controller.close_sdk().unwrap();
}
