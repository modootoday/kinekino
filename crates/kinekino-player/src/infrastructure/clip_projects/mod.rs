//! Immutable clip publication history under the original native root and writer lock.
mod records;
#[cfg(test)]
mod tests;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        clip_projects::{self, ClipProjectStore, StoredClipProject},
    },
    domain::{api_values::DecimalU64, clip_document::ClipDocument, identifiers::valid_id},
    infrastructure::immutable_storage_paths as paths,
};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};
const MAX_RECORD_BYTES: usize = crate::domain::clip_document::MAX_DOCUMENT_BYTES + 4096;
const MAX_ENTRIES: usize = 4096;
pub struct ClipProjectFiles {
    root: PathBuf,
    _guards: Vec<File>,
}
impl ClipProjectFiles {
    pub fn open(root: &Path) -> Result<Self, ApiError> {
        let (root, guards) = paths::root(root)?;
        Ok(Self {
            root,
            _guards: guards,
        })
    }
    fn directory(&self, id: &str) -> Result<PathBuf, ApiError> {
        if !valid_id(id) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid clip project identity",
            ));
        }
        Ok(self.root.join(format!("clip-{id}")))
    }
}
impl ClipProjectStore for ClipProjectFiles {
    fn load(&self, id: &str, requested: Option<DecimalU64>) -> Result<StoredClipProject, ApiError> {
        if requested.is_some_and(|revision| revision.0 == 0) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid clip publication revision",
            ));
        }
        let directory = self.directory(id)?;
        let _writer = paths::lock(&self.root.join("clip-writer.lock"))?;
        let _guard = paths::directory(&directory)?;
        records::scan(&directory, id, requested)
    }
    fn publish(
        &self,
        expected: Option<DecimalU64>,
        document: &ClipDocument,
    ) -> Result<StoredClipProject, ApiError> {
        document
            .validate()
            .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
        let directory = self.directory(&document.document_id)?;
        let _writer = paths::lock(&self.root.join("clip-writer.lock"))?;
        let existing_guard = if expected.is_some() {
            Some(paths::directory(&directory)?)
        } else {
            None
        };
        let previous = match expected {
            Some(expected) => {
                let current = records::scan(&directory, &document.document_id, None)?;
                if current.latest_publication_revision != expected {
                    return Err(ApiError::new(
                        ErrorCode::RevisionConflict,
                        "Clip publication head changed",
                    ));
                }
                Some(current)
            }
            None => None,
        };
        let publication = clip_projects::prepare_publication(previous.as_ref(), document)?;
        let bytes = serde_json::to_vec(&publication).map_err(|_| corrupt())?;
        if bytes.len() > MAX_RECORD_BYTES {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Clip publication byte limit exceeded",
            ));
        }
        let sha = clip_projects::publication_sha256(&publication)?;
        if previous.is_none() {
            fs::create_dir(&directory).map_err(kinekino_core::file_publication::io_error)?;
        }
        let _guard = match existing_guard {
            Some(guard) => guard,
            None => paths::directory(&directory)?,
        };
        // Pending attempts count too: a new final name must remain readable after a crash.
        let count = fs::read_dir(&directory)
            .map_err(kinekino_core::file_publication::io_error)?
            .take(MAX_ENTRIES + 1)
            .collect::<Result<Vec<_>, _>>()
            .map_err(kinekino_core::file_publication::io_error)?
            .len();
        if count >= MAX_ENTRIES {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Clip publication entry limit exceeded; preserve original evidence",
            ));
        }
        if previous.is_none() && count != 0 {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "New clip project directory is not empty; preserve original evidence",
            ));
        }
        super::native_immutable_publication::publish(
            &directory,
            &format!("r-{:020}-{sha}.json", publication.publication_revision.0),
            &bytes,
        )?;
        Ok(StoredClipProject {
            publication_sha256: sha,
            latest_publication_revision: publication.publication_revision,
            publication,
        })
    }
}
fn corrupt() -> ApiError {
    let mut error = ApiError::new(
        ErrorCode::NeedsInput,
        "Incomplete or corrupt clip publication; preserve original evidence",
    );
    error.remediation = "inspect_original_clip_project_history".into();
    error
}
