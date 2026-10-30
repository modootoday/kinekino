//! Journal contract tests; no worker, disk publication, or batch CLI acceptance.
mod orchestration;
use super::*;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        jobs::{JobRecord, JobState, SubmitJob, VerifiedJobOutput},
    },
    domain::api_values::DecimalU64,
};
use kinekino_jobs::domain::{PreparedJobInput, SourceBlobRef, SubmissionRef};
use serde_json::json;

fn document() -> BatchDocument {
    BatchDocument::new(
        "a".repeat(64),
        "batch-1".into(),
        vec![BatchJobInput {
            item_id: "first".into(),
            request: SubmitJob {
                operation: "subtitles.convert".into(),
                parameters: json!({"fixture":true}),
                expected_source_sha256: Some("b".repeat(64)),
                execution_timeout_ms: DecimalU64(30000),
            },
        }],
    )
    .unwrap()
}
fn reopen(value: &BatchDocument) -> BatchDocument {
    serde_json::from_slice(&serde_json::to_vec(value).unwrap()).unwrap()
}
fn next(value: &BatchDocument, progress: BatchProgress) -> BatchDocument {
    let mut next = reopen(value);
    next.revision.0 += 1;
    next.entries[0].progress = progress;
    next
}
fn job(document: &BatchDocument) -> JobRecord {
    let request = &document.entries[0].input.request;
    let mut job = JobRecord::queued(
        "job-00000000000000000000000000000001"
            .to_owned()
            .try_into()
            .unwrap(),
        PreparedJobInput {
            operation: request.operation.clone(),
            parameters: request.parameters.clone(),
            source: SourceBlobRef {
                sha256: "b".repeat(64),
                bytes: DecimalU64(1),
            },
            execution_timeout_ms: request.execution_timeout_ms,
        },
    )
    .unwrap();
    let key = document.plan.submission_key("first").unwrap();
    job.submission = Some(SubmissionRef {
        key_sha256: super::document::hash(key.as_bytes()),
        request_sha256: request_sha256(request).unwrap(),
    });
    job.validate().unwrap();
    job
}
fn observed(job: JobRecord) -> BatchProgress {
    BatchProgress::Observed {
        job: Box::new(job),
        last_error: None,
    }
}
fn bound() -> BatchDocument {
    let initial = document();
    let pending = next(&initial, BatchProgress::SubmitPending);
    validate_successor(&initial, &pending).unwrap();
    let bound = next(&pending, observed(job(&initial)));
    validate_successor(&pending, &bound).unwrap();
    bound
}
#[test]
fn scoped_submission_key_survives_reopen_and_changes_with_identity_or_input() {
    let initial = document();
    let key = initial.plan.submission_key("first").unwrap();
    assert_eq!(key.len(), 70);
    assert_eq!(reopen(&initial).plan.submission_key("first").unwrap(), key);
    for field in ["scope", "batch", "request"] {
        let mut plan = initial.plan.clone();
        match field {
            "scope" => plan.scope_sha256 = "b".repeat(64),
            "batch" => plan.batch_id = "batch-2".into(),
            _ => plan.items[0].request_sha256 = "c".repeat(64),
        }
        assert_ne!(plan.submission_key("first").unwrap(), key);
    }
    assert!(initial.plan.submission_key("unknown").is_err());
}

#[test]
fn initial_journal_cannot_import_an_observed_job_or_foreign_store_scope() {
    let initial = document();
    validate_initial(&initial, &"a".repeat(64)).unwrap();
    assert_eq!(
        validate_initial(&initial, &"b".repeat(64))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let mut forged = next(&initial, observed(job(&initial)));
    forged.revision = DecimalU64(1);
    assert!(validate_initial(&forged, &"a".repeat(64)).is_err());
    let mut wrong_revision = reopen(&initial);
    wrong_revision.revision = DecimalU64(2);
    assert!(validate_initial(&wrong_revision, &"a".repeat(64)).is_err());
}
#[test]
fn immutable_inputs_unique_ids_and_revision_cas_are_enforced() {
    let initial = document();
    let mut duplicate = initial.plan.clone();
    duplicate.items.push(duplicate.items[0].clone());
    assert!(duplicate.validate().is_err());
    let mut empty = initial.plan.clone();
    empty.items.clear();
    assert!(empty.validate().is_err());
    let mut changed = next(&initial, BatchProgress::SubmitPending);
    changed.entries[0].input.request.execution_timeout_ms = DecimalU64(1);
    assert!(changed.validate().is_err());
    let mut stale = next(&initial, BatchProgress::SubmitPending);
    stale.revision.0 += 1;
    assert_eq!(
        validate_successor(&initial, &stale).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let noop = next(&initial, BatchProgress::Waiting);
    assert!(validate_successor(&initial, &noop).is_err());
}
#[test]
fn observation_requires_a_prior_intent_and_rejects_foreign_jobs() {
    let initial = document();
    let skipped = next(&initial, observed(job(&initial)));
    assert!(validate_successor(&initial, &skipped).is_err());
    let mut foreign = job(&initial);
    foreign.submission.as_mut().unwrap().key_sha256 = "d".repeat(64);
    assert!(next(&initial, observed(foreign)).validate().is_err());
    let mut wrong_source = job(&initial);
    wrong_source.input.source.sha256 = "c".repeat(64);
    wrong_source.input_sha256 = wrong_source.input.sha256().unwrap();
    assert!(next(&initial, observed(wrong_source)).validate().is_err());
    let bound = bound();
    let mut swapped = bound.entries[0].progress.job().unwrap().clone();
    swapped.job_id = "job-00000000000000000000000000000002"
        .to_owned()
        .try_into()
        .unwrap();
    assert!(validate_successor(&bound, &next(&bound, observed(swapped))).is_err());
}
#[test]
fn completed_job_and_output_receipt_are_preserved_including_after_reopen() {
    let bound = bound();
    let mut completed = bound.entries[0].progress.job().unwrap().clone();
    completed.start().unwrap();
    completed.begin_publication().unwrap();
    completed.begin_verification().unwrap();
    completed
        .complete(VerifiedJobOutput {
            path: "C:/fixture/retained.vtt".into(),
            sha256: "e".repeat(64),
            bytes: DecimalU64(42),
            receipt: json!({"fixture":true,"file_identity":"original"}),
        })
        .unwrap();
    let complete = next(&bound, observed(completed.clone()));
    validate_successor(&bound, &complete).unwrap();
    let restored = reopen(&complete);
    restored.validate().unwrap();
    let retry = next(
        &restored,
        BatchProgress::RetryPending {
            before: Box::new(completed.clone()),
            last_error: None,
        },
    );
    assert!(retry.validate().is_err());
    let mut rewritten = completed;
    rewritten.output.as_mut().unwrap().sha256 = "f".repeat(64);
    assert!(validate_successor(&complete, &next(&complete, observed(rewritten))).is_err());
}
#[test]
fn submit_error_preserves_original_fields_and_replay_uses_the_same_key() {
    let initial = document();
    let pending = next(&initial, BatchProgress::SubmitPending);
    let mut error = ApiError::new(ErrorCode::EngineFailed, "response not observed");
    error.retryable = true;
    error.request_id = "original-request".into();
    error.remediation = "inspect_original_submission".into();
    error.details = json!({"execution":"unknown"});
    let failed = next(
        &pending,
        BatchProgress::SubmitError {
            error: Box::new(error),
        },
    );
    validate_successor(&pending, &failed).unwrap();
    let restored = reopen(&failed);
    let BatchProgress::SubmitError { error } = &restored.entries[0].progress else {
        panic!("Original error missing")
    };
    assert_eq!(error.request_id, "original-request");
    assert!(error.retryable);
    assert_eq!(error.details["execution"], "unknown");
    let replay = next(&restored, BatchProgress::SubmitPending);
    validate_successor(&restored, &replay).unwrap();
    assert_eq!(
        replay.plan.submission_key("first").unwrap(),
        initial.plan.submission_key("first").unwrap()
    );
}
#[test]
fn retry_journal_keeps_original_job_and_delegates_publication_safety_to_jobs() {
    let bound = bound();
    let mut failed = bound.entries[0].progress.job().unwrap().clone();
    failed.start().unwrap();
    failed
        .fail(
            kinekino_jobs::domain::JobFailure {
                code: ErrorCode::EngineFailed,
                message: "fixture engine failure".into(),
                remediation: String::new(),
                details: json!({}),
                retryable: false,
            },
            false,
        )
        .unwrap();
    let failed_document = next(&bound, observed(failed.clone()));
    validate_successor(&bound, &failed_document).unwrap();
    let intent = next(
        &failed_document,
        BatchProgress::RetryPending {
            before: Box::new(failed.clone()),
            last_error: None,
        },
    );
    validate_successor(&failed_document, &intent).unwrap();
    let mut retried = failed;
    retried.retry().unwrap();
    assert_eq!(retried.state, JobState::Queued);
    let admitted = next(&intent, observed(retried));
    validate_successor(&intent, &admitted).unwrap();
    assert_eq!(admitted.entries[0].progress.job().unwrap().attempt.0, 2);
    let mut ambiguous = job(&document());
    ambiguous.start().unwrap();
    ambiguous.begin_publication().unwrap();
    ambiguous
        .fail(
            kinekino_jobs::domain::JobFailure {
                code: ErrorCode::EngineFailed,
                message: "fixture publication uncertainty".into(),
                remediation: String::new(),
                details: json!({}),
                retryable: false,
            },
            false,
        )
        .unwrap();
    assert!(
        ambiguous.retry().is_err(),
        "Batch must never bypass the original Jobs publication gate"
    );
}
