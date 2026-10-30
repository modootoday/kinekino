//! Native journal bound to the original Jobs directory and a retained batch root.
mod publication;
mod records;
#[cfg(test)]
mod tests;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        job_batches::{
            BatchDocument, BatchLease, BatchStore, validate_initial, validate_successor,
        },
    },
    domain::identifiers::valid_id,
    infrastructure::{
        immutable_storage_paths as paths, storage_directory_identity::StorageDirectoryIdentity,
    },
};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

pub struct BatchFiles {
    root: PathBuf,
    scope: String,
    _guards: Vec<File>,
}
impl BatchFiles {
    /// The host supplies its original Jobs root, never a request-selected path.
    /// Opening does not submit, retry, clean evidence, or recover partial records.
    pub fn open(jobs_root: &Path) -> Result<Self, ApiError> {
        let (jobs_root, mut guards) = paths::root(jobs_root)?;
        let jobs_identity = StorageDirectoryIdentity::observe(guards.last().ok_or_else(corrupt)?)?;
        let _writer = paths::lock(&jobs_root.join("batch-writer.lock"))?;
        let root = jobs_root.join("batches");
        match fs::create_dir(&root) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(io(error)),
        }
        let batch_guard = paths::directory(&root)?;
        let batch_identity = StorageDirectoryIdentity::observe(&batch_guard)?;
        let binding =
            serde_json::to_vec(&("player.job-batches.native/1", jobs_identity, batch_identity))
                .map_err(|_| corrupt())?;
        let scope = hash(&binding);
        guards.push(batch_guard);
        Ok(Self {
            root,
            scope,
            _guards: guards,
        })
    }
    fn directory(&self, id: &str) -> Result<PathBuf, ApiError> {
        if !valid_id(id) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid batch identity",
            ));
        }
        Ok(self.root.join(format!("batch-{id}")))
    }
    fn writer(&self) -> Result<File, ApiError> {
        paths::lock(
            &self
                .root
                .parent()
                .ok_or_else(corrupt)?
                .join("batch-writer.lock"),
        )
    }
}
struct Lease<'a> {
    store: &'a BatchFiles,
    id: String,
    directory: PathBuf,
    _writer: File,
    _guard: File,
}
impl BatchStore for BatchFiles {
    fn scope_sha256(&self) -> &str {
        &self.scope
    }
    fn create(&self, document: &BatchDocument) -> Result<(), ApiError> {
        validate_initial(document, &self.scope)?;
        let directory = self.directory(&document.plan.batch_id)?;
        let _writer = self.writer()?;
        fs::create_dir(&directory).map_err(io)?;
        let _guard = paths::directory(&directory)?;
        let record = records::Record::initial(document)?;
        publication::publish(&directory, &record)
    }
    fn acquire(&self, id: &str) -> Result<Box<dyn BatchLease + '_>, ApiError> {
        let directory = self.directory(id)?;
        let writer = self.writer()?;
        let guard = paths::directory(&directory)?;
        Ok(Box::new(Lease {
            store: self,
            id: id.into(),
            directory,
            _writer: writer,
            _guard: guard,
        }))
    }
}
impl BatchLease for Lease<'_> {
    fn load(&self) -> Result<BatchDocument, ApiError> {
        Ok(records::scan(&self.directory, &self.store.scope, &self.id)?.0)
    }
    fn publish(&mut self, expected: u64, document: &BatchDocument) -> Result<(), ApiError> {
        let (previous, parent) = records::scan(&self.directory, &self.store.scope, &self.id)?;
        if previous.revision.0 != expected {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Batch journal revision changed",
            ));
        }
        validate_successor(&previous, document)?;
        let record = records::Record::successor(&previous, document, parent)?;
        publication::publish(&self.directory, &record)
    }
}
fn corrupt() -> ApiError {
    let mut error = ApiError::new(
        ErrorCode::NeedsInput,
        "Incomplete or corrupt batch journal; preserve evidence",
    );
    error.remediation = "inspect_original_batch_journal".into();
    error
}
fn io(error: std::io::Error) -> ApiError {
    kinekino_core::file_publication::io_error(error)
}
fn hash(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}
