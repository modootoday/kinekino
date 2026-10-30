//! Pinned local administrative host document; request JSON cannot configure it.
use super::*;
use crate::infrastructure::immutable_storage_paths as paths;
use std::{io::Read, path::Path};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HostDocument {
    schema_version: String,
    build: AssetBuildHostConfig,
}
pub struct OpenedAssetBuildHost {
    pub sources: Arc<dyn crate::application::asset_source_archives::AssetSourceArchivePort>,
    pub artifacts: Arc<dyn crate::application::asset_artifacts::AssetArtifactInspector>,
    pub projects: Arc<dyn crate::application::asset_projects::AssetProjectStore>,
    pub executor: Arc<dyn JobExecutor>,
    pub config_sha256: String,
}
impl OpenedAssetBuildHost {
    pub fn open(path: &Path, expected_sha256: &str) -> Result<Self, ApiError> {
        if !crate::domain::identifiers::valid_sha256(expected_sha256) {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Pinned asset host configuration SHA256 required",
            ));
        }
        let name = path
            .file_name()
            .ok_or_else(|| error(ErrorCode::InvalidArgument, "Asset host filename required"))?;
        let (parent, mut guards) = paths::root(path.parent().ok_or_else(|| {
            error(
                ErrorCode::InvalidArgument,
                "Absolute asset host configuration required",
            )
        })?)?;
        let mut file = paths::file(&parent.join(name))?;
        if file
            .metadata()
            .map_err(kinekino_core::file_publication::io_error)?
            .len()
            > 8 * 1024 * 1024
        {
            return Err(error(
                ErrorCode::ResourceExhausted,
                "Asset host configuration byte budget exceeded",
            ));
        }
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(8 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(kinekino_core::file_publication::io_error)?;
        if bytes.len() > 8 * 1024 * 1024 || hash(&bytes) != expected_sha256 {
            return Err(error(
                ErrorCode::SourceExpired,
                "Asset host configuration differs from its pin",
            ));
        }
        let document: HostDocument = serde_json::from_slice(&bytes).map_err(|_| {
            error(
                ErrorCode::InvalidArgument,
                "Strict asset-build-host/1 configuration required",
            )
        })?;
        if document.schema_version != "asset-build-host/1"
            || document.build.images.is_empty()
            || document.build.masks.is_empty()
            || document.build.images.len() > 4096
            || document.build.masks.len() > 4096
        {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Asset host schema/catalog count invalid",
            ));
        }
        let mut executor = AssetBuildJobExecutor::open(document.build)?;
        executor.host_sha256 = hash(
            &serde_json::to_vec(&[executor.host_sha256.as_str(), expected_sha256])
                .map_err(|_| error(ErrorCode::Internal, "Asset host pin encoding failed"))?,
        );
        guards.push(file);
        executor._host_guards = Arc::new(guards);
        Ok(Self {
            sources: Arc::new(
                crate::infrastructure::asset_source_archives::NativeAssetSourceArchiveService {
                    _host_guards: executor._host_guards.clone(),
                    projects: executor.projects.clone(),
                    inputs: executor.inputs.clone(),
                    archives:
                        crate::infrastructure::asset_source_archives::AssetSourceArchiveFiles::open(
                            &executor.config.artifact_root,
                        )?,
                },
            ),
            artifacts: Arc::new(super::AssetArtifactFiles::open(
                &executor.config.artifact_root,
            )?),
            projects: executor.projects.clone(),
            executor: Arc::new(executor),
            config_sha256: expected_sha256.into(),
        })
    }
}
