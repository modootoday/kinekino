use crate::application::catalog::{
    CaptureMetadata, CatalogArtifacts, CatalogCaptureService, CatalogService, RawArtifact,
};
use crate::infrastructure::immutable_storage_paths as paths;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};
mod episode_results;
mod plugin;
pub use plugin::{PluginParser, document_plugin};
mod raw;
mod results;
#[cfg(test)]
mod tests;

pub fn capture_service(
    root: &Path,
) -> Result<crate::application::catalog::CatalogCaptureService, ApiError> {
    Ok(capture_service_with_artifacts(Arc::new(
        CatalogFiles::open(root)?,
    )))
}
pub fn capture_service_with_artifacts(
    artifacts: Arc<dyn CatalogArtifacts>,
) -> CatalogCaptureService {
    CatalogCaptureService::new(Arc::new(PluginParser), artifacts)
}
pub fn open(root: &Path) -> Result<CatalogService, ApiError> {
    let artifacts = Arc::new(CatalogFiles::open(root)?);
    let http = document_plugin().map_err(crate::application::catalog::provider_error)?;
    // Revision is read once the plugin is open.
    Ok(
        CatalogService::new(http, Arc::new(PluginParser), artifacts.clone())
            .with_results(artifacts.clone())
            .with_raw(artifacts),
    )
}
pub fn archive(root: &Path) -> Result<crate::application::catalog::CatalogArchive, ApiError> {
    // Verified readback re-normalizes the stored request, which needs the source's location
    // rules; opening an installed plugin installs them. Without one, reads fail typed.
    let _ = document_plugin();
    let files = Arc::new(CatalogFiles::open(root)?);
    Ok(
        crate::application::catalog::CatalogArchive::new(files.clone(), files.clone())
            .with_episode_results(files),
    )
}
pub struct CatalogFiles {
    root: PathBuf,
    _guards: Vec<File>,
}
impl CatalogFiles {
    pub fn open(root: &Path) -> Result<Self, ApiError> {
        let (root, mut guards) = paths::root(root)?;
        for directory in ["objects", "captures", "results"] {
            let child = root.join(directory);
            match fs::create_dir(&child) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(kinekino_core::file_publication::io_error(e)),
            }
            guards.push(paths::directory(&child)?);
        }
        Ok(Self {
            root,
            _guards: guards,
        })
    }
    fn immutable(&self, relative: &str, content: &[u8]) -> Result<(), ApiError> {
        let path = self.root.join(relative);
        match kinekino_core::file_publication::publish(
            path.to_str().ok_or_else(|| {
                ApiError::new(ErrorCode::InvalidArgument, "Catalog root must be Unicode")
            })?,
            content,
        ) {
            Ok(()) => {}
            Err(e) if e.code == ErrorCode::RevisionConflict => {}
            Err(e) => return Err(e),
        }
        let file = paths::file(&path)?;
        let mut stored = Vec::new();
        file.take(content.len() as u64 + 1)
            .read_to_end(&mut stored)
            .map_err(kinekino_core::file_publication::io_error)?;
        if stored != content {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Catalog artifact identity mismatch",
            ));
        }
        Ok(())
    }
}
impl CatalogArtifacts for CatalogFiles {
    fn save(&self, raw: &[u8], metadata: &CaptureMetadata) -> Result<RawArtifact, ApiError> {
        if raw.len() > 2 * 1024 * 1024
            || metadata.body_sha256 != format!("{:x}", Sha256::digest(raw))
            || metadata.body_bytes.0 != raw.len() as u64
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid catalog body identity",
            ));
        }
        let manifest = serde_json::to_vec(metadata).map_err(|_| {
            ApiError::new(ErrorCode::Internal, "Catalog metadata serialization failed")
        })?;
        if manifest.len() > 64 * 1024 {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Catalog metadata exceeds limit",
            ));
        }
        let artifact_id = format!("{:x}", Sha256::digest(&manifest));
        let body_path = format!("objects/{}.body", metadata.body_sha256);
        let metadata_path = format!("captures/{artifact_id}.json");
        self.immutable(&body_path, raw)?;
        self.immutable(&metadata_path, &manifest)?;
        Ok(RawArtifact {
            artifact_id,
            body_sha256: metadata.body_sha256.clone(),
            body_bytes: metadata.body_bytes,
            body_path,
            metadata_path,
        })
    }
}
