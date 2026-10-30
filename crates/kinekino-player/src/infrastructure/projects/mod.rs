use crate::infrastructure::os_policy::{OpenOptionsPolicy, ShareMode};
mod records;
#[cfg(test)]
mod tests;
use crate::{
    application::projects::ProjectStore,
    domain::{api_values::DecimalU64, identifiers::valid_id, projects::*},
    infrastructure::immutable_storage_paths as paths,
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub struct ProjectFiles {
    root: PathBuf,
    _guards: Vec<File>,
}
fn io_error(error: std::io::Error) -> ApiError {
    ApiError::new(
        match error.kind() {
            std::io::ErrorKind::NotFound => ErrorCode::NotFound,
            std::io::ErrorKind::AlreadyExists => ErrorCode::RevisionConflict,
            std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
            _ => ErrorCode::EngineFailed,
        },
        format!("Project storage I/O: {error}"),
    )
}
fn corrupt() -> ApiError {
    let mut error = ApiError::new(
        ErrorCode::NeedsInput,
        "Project publication is incomplete or corrupt; preserve it for recovery",
    );
    error.remediation = "recover_project_publication".into();
    error
}
impl ProjectFiles {
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
                "Invalid project identity",
            ));
        }
        Ok(self.root.join(format!("p-{id}")))
    }
}
impl ProjectStore for ProjectFiles {
    fn load(&self, id: &str, revision: Option<u64>) -> Result<StoredProjectRevision, ApiError> {
        let dir = self.identity(id)?;
        let _writer = paths::lock(&self.root.join("writer.lock"))?;
        self.scan(id, &dir, revision)
    }
    fn publish(
        &self,
        expected: Option<u64>,
        project: &ProjectRevision,
    ) -> Result<StoredProjectRevision, ApiError> {
        project
            .validate()
            .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
        let bytes = serde_json::to_vec(project).map_err(|_| corrupt())?;
        let sha = canonical_sha256(project).map_err(|_| corrupt())?;
        let dir = self.identity(&project.project_id)?;
        let _writer = paths::lock(&self.root.join("writer.lock"))?;
        let next = match expected {
            None => {
                if project.revision.0 != 1 {
                    return Err(corrupt());
                }
                fs::create_dir(&dir).map_err(io_error)?;
                1
            }
            Some(expected) => {
                let current = self.scan(&project.project_id, &dir, None)?;
                if current.latest_revision.0 != expected {
                    return Err(ApiError::new(
                        ErrorCode::RevisionConflict,
                        "Project revision changed",
                    ));
                }
                expected.checked_add(1).ok_or_else(corrupt)?
            }
        };
        if project.revision.0 != next {
            return Err(corrupt());
        }
        let _guard = paths::directory(&dir)?;
        let revision = dir.join(format!("r-{next}"));
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
        Ok(StoredProjectRevision {
            revision_sha256: sha,
            latest_revision: project.revision,
            project: project.clone(),
        })
    }
}
