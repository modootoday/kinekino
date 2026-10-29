use super::{ExecutionOwner, JobStore};
use crate::domain::JobId;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::sync::Arc;

pub struct ExecutionSession {
    store: Arc<dyn JobStore>,
    _owner: Box<dyn ExecutionOwner>,
}
impl ExecutionSession {
    /// Acquire ownership and durably interrupt old work before starting any worker.
    pub fn acquire(store: Arc<dyn JobStore>) -> Result<(Self, Vec<JobId>), ApiError> {
        let owner = store.acquire_execution_owner()?;
        let mut interrupted = Vec::new();
        let mut cursor = None;
        loop {
            let page = store.list(cursor.as_ref(), 100)?;
            for snapshot in page.jobs {
                let mut current = store.get(&snapshot.job_id)?;
                let previous = current.revision.0;
                if current
                    .interrupt()
                    .map_err(|message| ApiError::new(ErrorCode::NeedsInput, message))?
                {
                    store.publish(Some(previous), &current)?;
                    interrupted.push(current.job_id);
                }
            }
            match page.next_cursor {
                Some(next) => {
                    if cursor.as_ref().is_some_and(|old: &super::JobListCursor| {
                        next.through_sequence != old.through_sequence
                            || next.after_created_sequence.0 <= old.after_created_sequence.0
                    }) {
                        return Err(ApiError::new(
                            ErrorCode::NeedsInput,
                            "Job recovery cursor did not advance",
                        ));
                    }
                    cursor = Some(next);
                }
                None => break,
            }
        }
        Ok((
            Self {
                store,
                _owner: owner,
            },
            interrupted,
        ))
    }
    pub fn store(&self) -> &dyn JobStore {
        self.store.as_ref()
    }
    pub fn store_handle(&self) -> Arc<dyn JobStore> {
        self.store.clone()
    }
}
