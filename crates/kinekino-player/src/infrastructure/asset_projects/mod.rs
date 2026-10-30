//! Fixed-root, no-overwrite project publications; incomplete stages remain separate.
mod admission;
mod records;
pub use admission::NativeAssetProjectLease;
#[cfg(test)]
mod tests;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        asset_projects::{self, AssetProjectStore, StoredAssetProject},
    },
    domain::{
        api_values::DecimalU64,
        asset_generation::{AssetDocument, MAX_ASSET_DOCUMENT_BYTES},
        identifiers::{valid_id, valid_sha256},
    },
    infrastructure::{immutable_storage_paths as paths, native_immutable_publication},
};
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};
const MAX_PUBLICATIONS: usize = 4096;
const MAX_PENDING: usize = 64;
const MAX_HISTORY_BYTES: u64 = 256 * 1024 * 1024;
pub struct AssetProjectFiles {
    root: PathBuf,
    _guards: Vec<File>,
}
fn corrupt() -> ApiError {
    let mut error = ApiError::new(
        ErrorCode::NeedsInput,
        "Incomplete or corrupt asset publication; preserve records for recovery",
    );
    error.remediation = "inspect_asset_project_publications".into();
    error
}
fn io(error: std::io::Error) -> ApiError {
    kinekino_core::file_publication::io_error(error)
}
impl AssetProjectFiles {
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
                "Invalid asset project identity",
            ));
        }
        Ok(self.root.join(format!("asset-{id}")))
    }
}
impl AssetProjectStore for AssetProjectFiles {
    fn load(&self, id: &str, revision: Option<DecimalU64>) -> Result<StoredAssetProject, ApiError> {
        if revision.is_some_and(|revision| revision.0 == 0) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Positive asset publication revision required",
            ));
        }
        let directory = self.directory(id)?;
        let _writer = paths::lock(&self.root.join("asset-projects-writer.lock"))?;
        self.scan(id, &directory, revision)
    }
    fn publish(
        &self,
        expected: Option<DecimalU64>,
        document: &AssetDocument,
    ) -> Result<StoredAssetProject, ApiError> {
        document
            .validate()
            .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
        let directory = self.directory(&document.document_id)?;
        let _writer = paths::lock(&self.root.join("asset-projects-writer.lock"))?;
        let previous = match expected {
            Some(expected) => {
                let current = self.scan_retained(&document.document_id, &directory, None)?;
                if current.project.latest_publication_revision != expected {
                    return Err(ApiError::new(
                        ErrorCode::RevisionConflict,
                        "Asset publication head changed",
                    ));
                }
                Some(current)
            }
            None => None,
        };
        if previous.as_ref().is_some_and(|previous| {
            previous.project.latest_publication_revision.0 >= MAX_PUBLICATIONS as u64
                || previous.project.pending_publication_count.0 >= MAX_PENDING as u64
        }) {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Asset project history/pending bound reached; preserve and inspect records",
            ));
        }
        let publication = asset_projects::prepare_publication(
            previous.as_ref().map(|history| &history.project),
            document,
        )?;
        let bytes = serde_json::to_vec(&publication).map_err(|_| corrupt())?;
        let sha = asset_projects::publication_sha256(&publication)?;
        if bytes.len() > MAX_ASSET_DOCUMENT_BYTES + 4096 {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Asset publication byte limit exceeded",
            ));
        }
        let history_bytes = previous
            .as_ref()
            .map_or(0, |previous| previous.project.history_bytes.0)
            .checked_add(bytes.len() as u64)
            .filter(|total| *total <= MAX_HISTORY_BYTES)
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::ResourceExhausted,
                    "Asset history byte budget would be exceeded",
                )
            })?;
        if expected.is_none() {
            fs::create_dir(&directory).map_err(io)?;
        }
        let _directory = paths::directory(&directory)?;
        let name = format!("r-{}-{sha}.json", publication.publication_revision.0);
        native_immutable_publication::publish(&directory, &name, &bytes).map_err(|mut error| {
            if !error.details.is_object() {
                error.details = serde_json::json!({"original_details":error.details});
            }
            error.details["publication_may_exist"] = serde_json::json!(true);
            error.details["publication_sha256"] = serde_json::json!(sha);
            error.details["publication_revision"] =
                serde_json::json!(publication.publication_revision);
            error
        })?;
        let verified = (|| {
            let mut file = paths::file(&directory.join(name))?;
            let mut observed = Vec::new();
            Read::by_ref(&mut file)
                .take((MAX_ASSET_DOCUMENT_BYTES + 4096 + 1) as u64)
                .read_to_end(&mut observed)
                .map_err(io)?;
            if observed != bytes {
                return Err(corrupt());
            }
            Ok(())
        })();
        verified.map_err(|mut error: ApiError| {
            if !error.details.is_object() {
                error.details = serde_json::json!({"original_details":error.details});
            }
            error.details["publication_committed"] = serde_json::json!(true);
            error.details["publication_sha256"] = serde_json::json!(sha);
            error.details["publication_revision"] =
                serde_json::json!(publication.publication_revision);
            error
        })?;
        Ok(StoredAssetProject {
            publication_sha256: sha,
            latest_publication_revision: publication.publication_revision,
            pending_publication_count: previous.as_ref().map_or(DecimalU64(0), |previous| {
                previous.project.pending_publication_count
            }),
            history_bytes: DecimalU64(history_bytes),
            publication,
        })
    }
}
