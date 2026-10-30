use super::*;
use crate::domain::proposals::output_review::{ObservedRecordTime, RecordClock};

#[test]
fn catalog_binds_three_task_scopes_to_full_four_clause_documents() {
    let mut hashes = std::collections::BTreeSet::new();
    for (task, id) in [
        (
            ProposalTask::Correction,
            "accepted-correction-source-review",
        ),
        (ProposalTask::Cards, "accepted-cards-source-review"),
        (
            ProposalTask::Highlights,
            "accepted-highlights-source-review",
        ),
    ] {
        let document = criteria_for(task).unwrap();
        assert_eq!(document.task, task);
        assert_eq!(document.criteria_id, id);
        assert_eq!(document.version, "1");
        assert_eq!(document.clauses.len(), 4);
        let identity = document.identity().unwrap();
        assert!(hashes.insert(identity.content_sha256.clone()));
        let presented = require_current_criteria(&identity).unwrap();
        assert_eq!(presented, document);
        let reopened: crate::domain::proposals::output_review::CriteriaDocument =
            serde_json::from_slice(&serde_json::to_vec(&document).unwrap()).unwrap();
        assert_eq!(document, reopened);
    }
}

#[test]
fn changed_text_purpose_version_or_task_requires_catalog_representing() {
    let original = criteria_for(ProposalTask::Correction).unwrap();
    for case in 0..5 {
        let mut substituted = original.clone();
        match case {
            0 => substituted.clauses[0].expected.push('!'),
            1 => substituted.purpose.push('!'),
            2 => substituted.version = "2".into(),
            3 => substituted.task = ProposalTask::Cards,
            _ => substituted.criteria_id = "caller-criteria".into(),
        }
        // Even a self-consistent hash of caller prose does not select trusted criteria.
        let identity = substituted.identity().unwrap();
        assert_eq!(
            require_current_criteria(&identity).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
    }
    let current = require_current_criteria(&original.identity().unwrap()).unwrap();
    assert_eq!(current, original);
}

struct ObservedClock;
impl OutputReviewClock for ObservedClock {
    fn observe_record_time(&self) -> Result<ObservedRecordTime, ApiError> {
        Ok(ObservedRecordTime {
            unix_ms: DecimalU64(23),
            clock: RecordClock::LocalSystemUnixMs,
        })
    }
}
#[test]
fn clock_port_returns_only_local_record_observation_without_claimed_reviewer_time() {
    let clock: &dyn OutputReviewClock = &ObservedClock;
    let observed = clock.observe_record_time().unwrap();
    assert_eq!(observed.unix_ms.0, 23);
    assert_eq!(observed.clock, RecordClock::LocalSystemUnixMs);
    let value = serde_json::to_value(observed).unwrap();
    assert!(value.get("claimed_review_unix_ms").is_none());
    assert!(value.get("reviewer_id").is_none());
}

struct FailingClock;
impl OutputReviewClock for FailingClock {
    fn observe_record_time(&self) -> Result<ObservedRecordTime, ApiError> {
        let mut error = ApiError::new(ErrorCode::EngineFailed, "Synthetic record clock failure");
        error.details = serde_json::json!({"phase":"observe_record_time"});
        error.remediation = "inspect_record_clock".into();
        error.request_id = "fixture-clock".into();
        Err(error)
    }
}
#[test]
fn clock_failure_stays_an_error_without_a_fabricated_timestamp() {
    let clock: &dyn OutputReviewClock = &FailingClock;
    let error = clock.observe_record_time().unwrap_err();
    assert_eq!(error.code, ErrorCode::EngineFailed);
    assert_eq!(error.details["phase"], "observe_record_time");
    assert_eq!(error.remediation, "inspect_record_clock");
    assert_eq!(error.request_id, "fixture-clock");
}
