use super::*;
use crate::application::{
    model_catalog::{ModelFileCheck, ModelFileCheckState, ModelVerification, VerifyModel},
    review_source::MutationControl,
};
struct Retained {
    file: File,
    _directories: Vec<File>,
}
fn retain(
    root: &Path,
    model: &ModelManifest,
    path: &str,
    control: &dyn MutationControl,
) -> Result<Option<Retained>, ApiError> {
    let relative = format!("artifacts/{}/{}/{path}", model.model_id, model.revision);
    let mut current = root.to_path_buf();
    let mut directories = Vec::new();
    let parts = relative.split('/').collect::<Vec<_>>();
    for (index, part) in parts.iter().enumerate() {
        control.checkpoint()?;
        current.push(part);
        match fs::symlink_metadata(&current) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(io_error(error)),
            Ok(_) => {}
        }
        if index + 1 == parts.len() {
            return Ok(Some(Retained {
                file: rooted::file(&current)?,
                _directories: directories,
            }));
        }
        directories.push(rooted::directory(&current)?);
    }
    Err(ApiError::new(ErrorCode::Internal, "Model path had no file"))
}
pub(super) fn verify(
    store: &LocalModelCatalog,
    input: &VerifyModel,
    control: &dyn MutationControl,
) -> Result<ModelVerification, ApiError> {
    input.validate()?;
    control.checkpoint()?;
    let _exclusive = store.verify_lock.try_lock().map_err(|error| {
        ApiError::new(
            match error {
                std::sync::TryLockError::WouldBlock => ErrorCode::ResourceExhausted,
                std::sync::TryLockError::Poisoned(_) => ErrorCode::EngineFailed,
            },
            "Original model verifier is busy or failed",
        )
    })?;
    // Retain the original catalog file until every model-file hash is complete.
    let mut catalog_file = rooted::file(&store.root.join("catalog.json"))?;
    let (_, digest) = content_digest::sha256_with_checkpoint(&mut catalog_file, 256 * 1024, || {
        control.checkpoint()
    })
    .map_err(content_digest::DigestFailure::into_api_error)?;
    if digest != input.catalog_sha256 {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Catalog changed before model verification",
        ));
    }
    catalog_file.seek(SeekFrom::Start(0)).map_err(io_error)?;
    let mut bytes = Vec::new();
    catalog_file.read_to_end(&mut bytes).map_err(io_error)?;
    let catalog: ModelCatalog = serde_json::from_slice(&bytes)
        .map_err(|_| ApiError::new(ErrorCode::InvalidArgument, "Malformed model catalog"))?;
    catalog
        .validate()
        .map_err(|s| ApiError::new(ErrorCode::InvalidArgument, s))?;
    let model = catalog
        .models
        .iter()
        .find(|m| m.model_id == input.model_id && m.revision == input.revision)
        .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Model revision not declared"))?;
    let total = model
        .files
        .iter()
        .try_fold(0u64, |total, file| total.checked_add(file.bytes.0))
        .ok_or_else(|| ApiError::new(ErrorCode::ResourceExhausted, "Model byte sum overflow"))?;
    if total > input.maximum_bytes.0 {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Declared model exceeds verification byte budget",
        ));
    }
    let mut retained = Vec::new();
    for file in &model.files {
        retained.push(retain(&store.root, model, &file.path, control)?);
    }
    let mut observations = Vec::new();
    for (declared, retained) in model.files.iter().zip(retained.iter_mut()) {
        control.checkpoint()?;
        let mut check = ModelFileCheck {
            path: declared.path.clone(),
            expected_bytes: declared.bytes,
            expected_sha256: declared.sha256.clone(),
            observed_bytes: None,
            observed_sha256: None,
            state: ModelFileCheckState::Missing,
        };
        if let Some(retained) = retained {
            let observed = retained.file.metadata().map_err(io_error)?.len();
            check.observed_bytes = Some(DecimalU64(observed));
            if observed != declared.bytes.0 {
                check.state = ModelFileCheckState::SizeMismatch;
            } else {
                let (count, digest) = content_digest::sha256_with_checkpoint(
                    &mut retained.file,
                    declared.bytes.0,
                    || control.checkpoint(),
                )
                .map_err(content_digest::DigestFailure::into_api_error)?;
                check.observed_bytes = Some(DecimalU64(count));
                check.observed_sha256 = Some(digest.clone());
                check.state = if count != declared.bytes.0 {
                    ModelFileCheckState::SizeMismatch
                } else if digest != declared.sha256 {
                    ModelFileCheckState::HashMismatch
                } else {
                    ModelFileCheckState::Matched
                };
            }
        }
        observations.push(check);
    }
    control.checkpoint()?;
    // All successfully opened original files and directory guards still coexist here.
    Ok(ModelVerification {
        model_id: model.model_id.clone(),
        revision: model.revision.clone(),
        files: observations,
    })
}
