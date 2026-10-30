use super::{BatchDocument, BatchProgress, document::invalid};
use crate::application::{
    api::{ApiError, ErrorCode},
    jobs::JobState,
};

pub fn validate_initial(document: &BatchDocument, scope_sha256: &str) -> Result<(), ApiError> {
    document.validate()?;
    if document
        .entries
        .iter()
        .any(|entry| entry.input.request.expected_source_sha256.is_none())
    {
        return Err(invalid(
            "Initial batch journal requires captured source pins",
        ));
    }
    if document.plan.scope_sha256 != scope_sha256 {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Batch belongs to another Jobs store binding",
        ));
    }
    if document.revision.0 != 1
        || document
            .entries
            .iter()
            .any(|entry| !matches!(entry.progress, BatchProgress::Waiting))
    {
        return Err(invalid(
            "Initial batch journal requires revision one and unsubmitted entries",
        ));
    }
    Ok(())
}

/// Journal policy only. It neither executes jobs nor duplicates Jobs retry admission.
pub fn validate_successor(previous: &BatchDocument, next: &BatchDocument) -> Result<(), ApiError> {
    previous.validate()?;
    next.validate()?;
    if previous.plan != next.plan || previous.revision.0.checked_add(1) != Some(next.revision.0) {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Batch identity/input or revision changed",
        ));
    }
    let mut changes = 0;
    for (old, new) in previous.entries.iter().zip(&next.entries) {
        let old_bytes =
            serde_json::to_vec(&old.progress).map_err(|_| invalid("Invalid old progress"))?;
        let new_bytes =
            serde_json::to_vec(&new.progress).map_err(|_| invalid("Invalid next progress"))?;
        if old_bytes == new_bytes {
            continue;
        }
        changes += 1;
        let allowed = matches!(
            (&old.progress, &new.progress),
            (BatchProgress::Waiting, BatchProgress::SubmitPending)
                | (
                    BatchProgress::SubmitPending,
                    BatchProgress::Observed { .. } | BatchProgress::SubmitError { .. }
                )
                | (
                    BatchProgress::SubmitError { .. },
                    BatchProgress::SubmitPending
                )
                | (
                    BatchProgress::Observed { .. },
                    BatchProgress::Observed { .. } | BatchProgress::RetryPending { .. }
                )
                | (
                    BatchProgress::RetryPending { .. },
                    BatchProgress::RetryPending { .. } | BatchProgress::Observed { .. }
                )
        );
        if !allowed {
            return Err(invalid(
                "Batch progress skipped its durable intent or discarded a job identity",
            ));
        }
        if let (Some(before), Some(after)) = (old.progress.job(), new.progress.job()) {
            if before.job_id != after.job_id
                || before.input_sha256 != after.input_sha256
                || after.revision.0 < before.revision.0
                || after.attempt.0 < before.attempt.0
            {
                return Err(invalid(
                    "Batch observation changed job identity/input or moved backwards",
                ));
            }
            if (before.state == JobState::Completed
                || before.revision == after.revision
                || matches!(
                    (&old.progress, &new.progress),
                    (
                        BatchProgress::RetryPending { .. },
                        BatchProgress::RetryPending { .. }
                    )
                ))
                && serde_json::to_vec(before).map_err(|_| invalid("Invalid prior job"))?
                    != serde_json::to_vec(after).map_err(|_| invalid("Invalid next job"))?
            {
                return Err(invalid(
                    "Batch changed a completed job, same revision or pending retry pin",
                ));
            }
        }
    }
    if changes != 1 {
        return Err(invalid(
            "Each batch journal revision must capture exactly one item change",
        ));
    }
    Ok(())
}
