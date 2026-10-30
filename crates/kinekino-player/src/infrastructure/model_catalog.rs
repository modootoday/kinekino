//! Fixed-root local manifest reader. No network, model loading or writes.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        model_catalog::{LocalFileState, ModelCatalogPort, ModelFileObservation},
    },
    domain::{
        api_values::DecimalU64,
        model_catalog::{ModelCatalog, ModelManifest},
    },
    infrastructure::{content_digest, immutable_storage_paths as rooted},
};
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};
mod verification;
pub struct LocalModelCatalog {
    root: PathBuf,
    _guards: Vec<File>,
    verify_lock: std::sync::Mutex<()>,
}
impl LocalModelCatalog {
    pub fn open(root: &Path) -> Result<Self, ApiError> {
        let (_, guards) = rooted::root(root)?;
        Ok(Self {
            root: root.into(),
            _guards: guards,
            verify_lock: std::sync::Mutex::new(()),
        })
    }
}
impl ModelCatalogPort for LocalModelCatalog {
    fn verification_available(&self) -> bool {
        true
    }
    fn verify_files(
        &self,
        input: &crate::application::model_catalog::VerifyModel,
        control: &dyn crate::application::review_source::MutationControl,
    ) -> Result<crate::application::model_catalog::ModelVerification, ApiError> {
        verification::verify(self, input, control)
    }
    fn snapshot(&self) -> Result<(ModelCatalog, String), ApiError> {
        let mut file = rooted::file(&self.root.join("catalog.json"))?;
        let (_, sha) = content_digest::sha256(&mut file, 256 * 1024)?;
        file.seek(SeekFrom::Start(0)).map_err(io_error)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(io_error)?;
        let catalog: ModelCatalog = serde_json::from_slice(&bytes)
            .map_err(|_| ApiError::new(ErrorCode::InvalidArgument, "Malformed model catalog"))?;
        catalog
            .validate()
            .map_err(|s| ApiError::new(ErrorCode::InvalidArgument, s))?;
        Ok((catalog, sha))
    }
    fn inspect_files(&self, model: &ModelManifest) -> Result<Vec<ModelFileObservation>, ApiError> {
        model
            .validate()
            .map_err(|s| ApiError::new(ErrorCode::InvalidArgument, s))?;
        model
            .files
            .iter()
            .map(|file| {
                let relative = format!(
                    "artifacts/{}/{}/{}",
                    model.model_id, model.revision, file.path
                );
                let mut path = self.root.clone();
                let mut guards = Vec::new();
                let parts = relative.split('/').collect::<Vec<_>>();
                for (index, part) in parts.iter().enumerate() {
                    path.push(part);
                    match fs::symlink_metadata(&path) {
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                            return Ok(ModelFileObservation {
                                path: file.path.clone(),
                                state: LocalFileState::Missing,
                                observed_bytes: None,
                            });
                        }
                        Err(error) => return Err(io_error(error)),
                        Ok(_) => {}
                    }
                    if index + 1 == parts.len() {
                        let guard = rooted::file(&path)?;
                        let bytes = guard.metadata().map_err(io_error)?.len();
                        return Ok(ModelFileObservation {
                            path: file.path.clone(),
                            state: if bytes == file.bytes.0 {
                                LocalFileState::PresentUnverified
                            } else {
                                LocalFileState::SizeMismatch
                            },
                            observed_bytes: Some(DecimalU64(bytes)),
                        });
                    }
                    guards.push(rooted::directory(&path)?);
                }
                Err(ApiError::new(ErrorCode::Internal, "Model path had no file"))
            })
            .collect()
    }
}
fn io_error(error: std::io::Error) -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, error.to_string())
}
