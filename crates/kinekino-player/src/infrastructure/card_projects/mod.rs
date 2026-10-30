//! Card project records reuse fixed-root guards and exclusive writer locks.
use crate::infrastructure::os_policy::{OpenOptionsPolicy, ShareMode};
mod records;
#[cfg(test)]
mod tests;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        card_projects::{self, CardProjectStore, StoredCardProject},
    },
    domain::{
        api_values::DecimalU64,
        card_document::{CardDocument, MAX_DOCUMENT_BYTES},
        identifiers::valid_id,
    },
    infrastructure::immutable_storage_paths as paths,
};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub struct CardProjectFiles {
    root: PathBuf,
    _guards: Vec<File>,
}

impl CardProjectFiles {
    pub fn open(root: &Path) -> Result<Self, ApiError> {
        let (root, guards) = paths::root(root)?;
        Ok(Self {
            root,
            _guards: guards,
        })
    }
    fn identity(&self, id: &str) -> Result<PathBuf, ApiError> {
        if !valid_id(id) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid card project identity",
            ));
        }
        Ok(self.root.join(format!("card-{id}")))
    }
}

impl CardProjectStore for CardProjectFiles {
    fn load(&self, id: &str, revision: Option<DecimalU64>) -> Result<StoredCardProject, ApiError> {
        if revision.is_some_and(|n| n.0 == 0) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid publication revision",
            ));
        }
        let dir = self.identity(id)?;
        let _writer = paths::lock(&self.root.join("card-writer.lock"))?;
        self.scan(id, &dir, revision)
    }
    fn publish(
        &self,
        expected: Option<DecimalU64>,
        document: &CardDocument,
    ) -> Result<StoredCardProject, ApiError> {
        document
            .validate()
            .map_err(|_| ApiError::new(ErrorCode::InvalidArgument, "Invalid card document"))?;
        let dir = self.identity(&document.document_id)?;
        let _writer = paths::lock(&self.root.join("card-writer.lock"))?;
        let previous = if let Some(expected) = expected {
            let current = self.scan(&document.document_id, &dir, None)?;
            if expected != current.latest_publication_revision {
                return Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Card publication head changed",
                ));
            }
            Some(current)
        } else {
            None
        };
        let publication = card_projects::prepare_publication(previous.as_ref(), document)?;
        let bytes = serde_json::to_vec(&publication).map_err(|_| corrupt())?;
        let sha = card_projects::publication_sha256(&publication)?;
        if expected.is_none() {
            fs::create_dir(&dir).map_err(io_error)?;
        }
        let _guard = paths::directory(&dir)?;
        let revision = dir.join(format!("r-{}", publication.publication_revision.0));
        fs::create_dir(&revision).map_err(io_error)?;
        let _revision_guard = paths::directory(&revision)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .share(ShareMode::ReadOnly)
            .open(revision.join(format!("{sha}.json")))
            .map_err(io_error)?;
        file.write_all(&bytes).map_err(io_error)?;
        file.sync_all().map_err(io_error)?;
        Ok(StoredCardProject {
            publication_sha256: sha,
            latest_publication_revision: publication.publication_revision,
            publication,
        })
    }
}

fn corrupt() -> ApiError {
    let mut error = ApiError::new(
        ErrorCode::NeedsInput,
        "Incomplete or corrupt card project publication; preserve for recovery",
    );
    error.remediation = "recover_card_project_publication".into();
    error
}
fn io_error(error: std::io::Error) -> ApiError {
    ApiError::new(
        match error.kind() {
            std::io::ErrorKind::NotFound => ErrorCode::NotFound,
            std::io::ErrorKind::AlreadyExists => ErrorCode::RevisionConflict,
            std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
            _ => ErrorCode::EngineFailed,
        },
        format!("Card project storage I/O: {error}"),
    )
}
