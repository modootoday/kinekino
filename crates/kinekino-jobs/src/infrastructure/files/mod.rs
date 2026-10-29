mod events;
mod queries;
mod sources;
#[cfg(all(test, windows))]
mod tests;
use crate::{
    application::{ExecutionOwner, JobAdmission, JobEvent, JobStore},
    domain::{JobRecord, JobState},
};
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

pub const MAX_JOBS: usize = 128;
pub const MAX_ATTEMPTS: u64 = 16;
pub(super) const MAX_EVENTS: usize = 16384;
pub(super) const MAX_EVENT_BYTES: usize = 132 * 1024;
pub(super) const MAX_PAGE_BYTES: usize = 1024 * 1024;

/// Inject the platform's existing rooted-directory/reparse/lock policy; never duplicate it here.
pub trait RootedJobPaths: Send + Sync {
    fn root(&self, path: &Path) -> Result<(PathBuf, Vec<File>), ApiError>;
    fn directory(&self, path: &Path) -> Result<File, ApiError>;
    fn file(&self, path: &Path) -> Result<File, ApiError>;
    fn lock(&self, path: &Path) -> Result<File, ApiError>;
}
pub struct JobFiles {
    root: PathBuf,
    events: PathBuf,
    sources: PathBuf,
    paths: Arc<dyn RootedJobPaths>,
    _guards: Vec<File>,
    gate: Mutex<()>,
}
fn corrupt() -> ApiError {
    let mut error = ApiError::new(
        ErrorCode::NeedsInput,
        "Job journal is incomplete or corrupt; preserve it for explicit recovery",
    );
    error.remediation = "recover_job_journal".into();
    error
}
fn invalid(message: impl Into<String>) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn exhausted(message: &str) -> ApiError {
    ApiError::new(ErrorCode::ResourceExhausted, message)
}
fn io_error(error: std::io::Error) -> ApiError {
    kinekino_core::file_publication::io_error(error)
}
fn acquire_writer(paths: &dyn RootedJobPaths, root: &Path) -> Result<File, ApiError> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
    loop {
        match paths.lock(&root.join("jobs-writer.lock")) {
            Err(error)
                if error.code == ErrorCode::RevisionConflict
                    && std::time::Instant::now() < deadline =>
            {
                std::thread::sleep(std::time::Duration::from_millis(2))
            }
            result => return result,
        }
    }
}
impl JobFiles {
    fn writer_lock(&self) -> Result<File, ApiError> {
        acquire_writer(self.paths.as_ref(), &self.root)
    }
    fn local_gate(&self) -> Result<MutexGuard<'_, ()>, ApiError> {
        self.gate
            .lock()
            .map_err(|_| ApiError::new(ErrorCode::EngineFailed, "Job store gate unavailable"))
    }
    pub fn open(root: &Path, paths: Arc<dyn RootedJobPaths>) -> Result<Self, ApiError> {
        let (root, mut guards) = paths.root(root)?;
        let _writer = acquire_writer(paths.as_ref(), &root)?;
        let events = root.join("job-events");
        let sources = root.join("job-sources");
        for path in [&events, &sources] {
            match fs::create_dir(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(io_error(error)),
            }
            guards.push(paths.directory(path)?);
        }
        Ok(Self {
            root,
            events,
            sources,
            paths,
            _guards: guards,
            gate: Mutex::new(()),
        })
    }
    fn append(&self, expected: Option<u64>, job: &JobRecord) -> Result<JobAdmission, ApiError> {
        let _local = self.local_gate()?;
        job.validate().map_err(invalid)?;
        if job.attempt.0 > MAX_ATTEMPTS {
            return Err(exhausted(
                "Job attempt limit reached; preserve this journal and submit a new job",
            ));
        }
        let _writer = self.writer_lock()?;
        let headers = self.headers()?;
        if let Some(tail) = headers.last() {
            self.read_event(tail)?;
        }
        if expected.is_none()
            && let Some(submission) = &job.submission
            && let Some(existing) = self.find_submission_in(&headers, &submission.key_sha256)?
        {
            if existing
                .submission
                .as_ref()
                .is_none_or(|old| old.request_sha256 != submission.request_sha256)
            {
                let mut error = ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Idempotency key is bound to a different normalized job request",
                );
                error.details = serde_json::json!({"conflict_kind":"idempotency_conflict","job_id":existing.job_id});
                return Err(error);
            }
            let header = headers
                .iter()
                .rev()
                .find(|header| header.job_id == existing.job_id)
                .ok_or_else(corrupt)?;
            return Ok(JobAdmission {
                event: self.read_event(header)?,
                created: false,
            });
        }
        let previous = headers
            .iter()
            .rev()
            .find(|header| header.job_id == job.job_id);
        match (expected, previous) {
            (None, None) => {
                if job.revision.0 != 1 || job.attempt.0 != 1 || job.state != JobState::Queued {
                    return Err(invalid("New job must begin at queued revision1/attempt1"));
                }
                let ids: std::collections::BTreeSet<_> =
                    headers.iter().map(|header| &header.job_id).collect();
                if ids.len() >= MAX_JOBS {
                    return Err(exhausted("Job journal identity limit reached"));
                }
                <Self as crate::application::SourceBlobStore>::read_utf8(self, &job.input.source)?;
            }
            (Some(expected), Some(header)) => {
                let previous = self.read_event(header)?.job;
                if previous.revision.0 != expected {
                    return Err(ApiError::new(
                        ErrorCode::RevisionConflict,
                        "Job revision changed",
                    ));
                }
                job.validate_successor(&previous).map_err(invalid)?;
            }
            _ => {
                return Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Job identity or revision changed",
                ));
            }
        }
        if headers.len() >= MAX_EVENTS {
            return Err(exhausted("Job journal event limit reached"));
        }
        let event = JobEvent {
            sequence: DecimalU64(headers.len() as u64 + 1),
            job: job.clone(),
        };
        let bytes = serde_json::to_vec(&event).map_err(|_| corrupt())?;
        if bytes.len() > MAX_EVENT_BYTES {
            return Err(exhausted("Job event exceeds storage bound"));
        }
        let sha = sources::digest(&bytes);
        let directory = self
            .events
            .join(format!("e-{}-{}", event.sequence.0, job.job_id.as_str()));
        fs::create_dir(&directory).map_err(io_error)?;
        let _guard = self.paths.directory(&directory)?;
        let path = directory.join(format!("{sha}.json"));
        kinekino_core::file_publication::publish(
            path.to_str()
                .ok_or_else(|| invalid("Job storage path is not UTF8"))?,
            &bytes,
        )?;
        Ok(JobAdmission {
            event,
            created: expected.is_none(),
        })
    }
}
struct Owner {
    _guard: File,
}
impl ExecutionOwner for Owner {}
impl JobStore for JobFiles {
    fn find_submission(&self, key_sha256: &str) -> Result<Option<JobRecord>, ApiError> {
        let _local = self.local_gate()?;
        if !crate::domain::valid_sha256(key_sha256) {
            return Err(invalid("Invalid submission key digest"));
        }
        let _writer = self.writer_lock()?;
        let headers = self.headers()?;
        if let Some(tail) = headers.last() {
            self.read_event(tail)?;
        }
        self.find_submission_in(&headers, key_sha256)
    }
    fn admit(&self, job: &JobRecord) -> Result<JobAdmission, ApiError> {
        self.append(None, job)
    }
    fn get(&self, id: &crate::domain::JobId) -> Result<JobRecord, ApiError> {
        self.query_job(id)
    }
    fn publish(&self, expected: Option<u64>, job: &JobRecord) -> Result<JobEvent, ApiError> {
        self.append(expected, job).map(|admission| admission.event)
    }
    fn read_events(
        &self,
        after: u64,
        limit: usize,
    ) -> Result<crate::application::EventPage, ApiError> {
        self.query_events(after, limit)
    }
    fn list(
        &self,
        cursor: Option<&crate::application::JobListCursor>,
        limit: usize,
    ) -> Result<crate::application::JobPage, ApiError> {
        self.query_jobs(cursor, limit)
    }
    fn acquire_execution_owner(&self) -> Result<Box<dyn ExecutionOwner>, ApiError> {
        Ok(Box::new(Owner {
            _guard: self.paths.lock(&self.root.join("jobs-owner.lock"))?,
        }))
    }
}
