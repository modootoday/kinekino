//! Explicit commands under the original journal writer lease; never a worker.
use super::{BatchDocument, BatchLease, BatchProgress, BatchStore, document::invalid};
use crate::application::{
    api::{ApiError, ErrorCode},
    jobs::{GetJob, JobState, JobsApi, RetryJob},
};

#[derive(Debug, Clone, Copy)]
pub enum BatchAction {
    Submit,
    Refresh,
    RetryFailed,
}

/// The host must bind store and Jobs to the same trusted native root identity.
/// Errors from Jobs are journaled per item. A journal error stops the command:
/// it may follow admission, so only a new explicit command may reconcile it.
pub fn run(
    store: &dyn BatchStore,
    jobs: &dyn JobsApi,
    batch_id: &str,
    expected_revision: u64,
    action: BatchAction,
) -> Result<BatchDocument, ApiError> {
    let mut lease = store.acquire(batch_id)?;
    let mut document = lease.load()?;
    document.validate()?;
    if document.plan.batch_id != batch_id
        || document.plan.scope_sha256 != store.scope_sha256()
        || document.revision.0 != expected_revision
    {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Batch identity, binding or expected revision differs",
        ));
    }
    for index in 0..document.entries.len() {
        match action {
            BatchAction::Submit => submit(lease.as_mut(), &mut document, index, jobs)?,
            BatchAction::Refresh => {
                refresh(lease.as_mut(), &mut document, index, jobs)?;
            }
            BatchAction::RetryFailed => retry(lease.as_mut(), &mut document, index, jobs)?,
        }
    }
    Ok(document)
}

fn publish(
    lease: &mut dyn BatchLease,
    document: &mut BatchDocument,
    index: usize,
    progress: BatchProgress,
) -> Result<(), ApiError> {
    if serde_json::to_vec(&document.entries[index].progress)
        .map_err(|_| invalid("Invalid prior batch progress"))?
        == serde_json::to_vec(&progress).map_err(|_| invalid("Invalid batch progress"))?
    {
        return Ok(());
    }
    let revision = document.revision.0;
    let next = revision
        .checked_add(1)
        .ok_or_else(|| ApiError::new(ErrorCode::ResourceExhausted, "Batch revision exhausted"))?;
    let previous = std::mem::replace(&mut document.entries[index].progress, progress);
    document.revision.0 = next;
    let result = document
        .validate()
        .and_then(|()| lease.publish(revision, document));
    if result.is_err() {
        document.entries[index].progress = previous;
        document.revision.0 = revision;
    }
    result
}

fn submit(
    lease: &mut dyn BatchLease,
    document: &mut BatchDocument,
    index: usize,
    jobs: &dyn JobsApi,
) -> Result<(), ApiError> {
    match document.entries[index].progress {
        BatchProgress::Waiting | BatchProgress::SubmitError { .. } => {
            publish(lease, document, index, BatchProgress::SubmitPending)?;
        }
        BatchProgress::SubmitPending => {}
        _ => return Ok(()),
    }
    let entry = &document.entries[index];
    let key = document
        .plan
        .submission_key(&entry.input.item_id)
        .map_err(invalid)?;
    let progress = match jobs.submit(entry.input.request.clone(), &key) {
        Ok(result) => BatchProgress::Observed {
            job: Box::new(result.job),
            last_error: None,
        },
        Err(error) => BatchProgress::SubmitError {
            error: Box::new(error),
        },
    };
    publish(lease, document, index, progress)
}

fn refresh(
    lease: &mut dyn BatchLease,
    document: &mut BatchDocument,
    index: usize,
    jobs: &dyn JobsApi,
) -> Result<bool, ApiError> {
    let Some(before) = document.entries[index].progress.job() else {
        return Ok(false);
    };
    if before.state == JobState::Completed {
        return Ok(false);
    }
    let result = jobs.get(GetJob {
        job_id: before.job_id.clone(),
    });
    let observed = result.is_ok();
    let progress = match result {
        Ok(job) => {
            // An unchanged observation cannot settle an unacknowledged retry.
            if matches!(
                document.entries[index].progress,
                BatchProgress::RetryPending { .. }
            ) && job.revision == before.revision
            {
                if serde_json::to_vec(&job).map_err(|_| invalid("Invalid retry observation"))?
                    != serde_json::to_vec(before).map_err(|_| invalid("Invalid retry pin"))?
                {
                    return Err(invalid("Same retry revision changed its record"));
                }
                return Ok(true);
            }
            BatchProgress::Observed {
                job: Box::new(job),
                last_error: None,
            }
        }
        Err(error) => match &document.entries[index].progress {
            BatchProgress::RetryPending { before, .. } => BatchProgress::RetryPending {
                before: before.clone(),
                last_error: Some(Box::new(error)),
            },
            BatchProgress::Observed { job, .. } => BatchProgress::Observed {
                job: job.clone(),
                last_error: Some(Box::new(error)),
            },
            _ => unreachable!("Only bound entries were queried"),
        },
    };
    publish(lease, document, index, progress)?;
    Ok(observed)
}

fn retry(
    lease: &mut dyn BatchLease,
    document: &mut BatchDocument,
    index: usize,
    jobs: &dyn JobsApi,
) -> Result<(), ApiError> {
    // Reconcile first. A prior retry may have executed without its ACK being saved.
    if !refresh(lease, document, index, jobs)? {
        return Ok(());
    }
    let before = match &document.entries[index].progress {
        BatchProgress::Observed {
            job,
            last_error: None,
        } if job.state == JobState::Failed => {
            let before = job.clone();
            publish(
                lease,
                document,
                index,
                BatchProgress::RetryPending {
                    before: before.clone(),
                    last_error: None,
                },
            )?;
            before
        }
        BatchProgress::RetryPending { before, .. } if before.state == JobState::Failed => {
            before.clone()
        }
        _ => return Ok(()),
    };
    let progress = match jobs.retry(
        RetryJob {
            job_id: before.job_id.clone(),
        },
        before.revision.0,
    ) {
        Ok(job) => BatchProgress::Observed {
            job: Box::new(job),
            last_error: None,
        },
        Err(error) => BatchProgress::RetryPending {
            before,
            last_error: Some(Box::new(error)),
        },
    };
    publish(lease, document, index, progress)
}
