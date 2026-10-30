use super::super::vad_job_asset::{FixedVadFile, GuardedAsset};
use crate::application::audio_vad::product_release::{
    ConfiguredVadRelease, ProductArtifact, ProductArtifactRole,
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::domain::validate_runtime_notice_sizes;
use kinekino_vad_executor::application::{
    ChildNativeDeployment, ChildNativeModule, ChildPinnedFile, MAX_RELEASE_MANIFEST_BYTES,
    ParentSelectedRelease, SnapshotLease, decode_release_manifest,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
pub(super) struct OpenedProductRelease {
    pub root: PathBuf,
    pub release: Arc<ConfiguredVadRelease>,
    pub selected: Arc<ParentSelectedRelease>,
    _leases: Vec<Arc<dyn SnapshotLease>>,
}
fn validate_notices(artifacts: &[ProductArtifact]) -> Result<(), ApiError> {
    validate_runtime_notice_sizes(artifacts.iter().filter_map(|artifact| {
        matches!(&artifact.role, ProductArtifactRole::Notice).then_some(artifact.pin.bytes())
    }))
}
impl OpenedProductRelease {
    pub(super) fn open(root: &Path) -> Result<Self, ApiError> {
        let release = Arc::new(ConfiguredVadRelease::configured()?);
        let native = release.native_deployment()?;
        if native.minimum_windows_build == 0
            || native.system_module_names.is_empty()
            || !native
                .artifacts
                .iter()
                .any(|a| matches!(&a.role, ProductArtifactRole::Dependency { .. }))
            || !native
                .artifacts
                .iter()
                .any(|a| matches!(&a.role, ProductArtifactRole::Notice))
        {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Product native dependencies/notices and supported Windows baseline are incomplete",
            ));
        }
        validate_notices(&native.artifacts)?;
        if !root
            .try_exists()
            .map_err(kinekino_core::file_publication::io_error)?
            || !root
                .join("release.manifest.json")
                .try_exists()
                .map_err(kinekino_core::file_publication::io_error)?
        {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Configured product release bundle/manifest is not installed",
            ));
        }
        let mut names = vec!["release.manifest.json".to_owned()];
        let mut ids = Vec::new();
        let mut dependencies = Vec::new();
        let mut notices = Vec::new();
        let mut worker = None;
        let mut leases = Vec::new();
        let manifest = GuardedAsset::new(
            FixedVadFile {
                path: root.join("release.manifest.json"),
                pin: release.manifest().clone(),
            },
            MAX_RELEASE_MANIFEST_BYTES,
        )?;
        let (raw, lease) = manifest.snapshot()?;
        leases.push(lease);
        let decoded = decode_release_manifest(&raw, release.manifest())?;
        if decoded.profile() != release.profile()
            || decoded.artifacts().len() != native.artifacts.len()
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Product manifest differs from independently configured release",
            ));
        }
        for file in &native.artifacts {
            if file.name.is_empty()
                || file.name == "."
                || file.name == ".."
                || file.name.ends_with('.')
                || !file
                    .name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
                || names.iter().any(|n| n.eq_ignore_ascii_case(file.name))
                || ids.contains(&file.pin.id().to_owned())
                || !decoded.artifacts().contains(&file.pin)
            {
                return Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Product release filenames/artifact set are ambiguous",
                ));
            }
            names.push(file.name.into());
            ids.push(file.pin.id().to_owned());
            if !root
                .join(file.name)
                .try_exists()
                .map_err(kinekino_core::file_publication::io_error)?
            {
                return Err(ApiError::new(
                    ErrorCode::NeedsInput,
                    "A configured product artifact/notice is not installed",
                ));
            }
            let asset = GuardedAsset::new(
                FixedVadFile {
                    path: root.join(file.name),
                    pin: file.pin.clone(),
                },
                kinekino_vad::domain::MAX_RUNTIME_BYTES,
            )?;
            let (_, lease) = asset.snapshot()?;
            leases.push(lease);
            let pinned = ChildPinnedFile {
                path: root
                    .join(file.name)
                    .to_str()
                    .ok_or_else(|| {
                        ApiError::new(ErrorCode::InvalidArgument, "Product path is not Unicode")
                    })?
                    .into(),
                pin: file.pin.clone(),
            };
            match &file.role {
                ProductArtifactRole::Worker => {
                    if worker.replace(pinned.path).is_some() {
                        return Err(ApiError::new(
                            ErrorCode::RevisionConflict,
                            "Multiple product worker artifacts",
                        ));
                    }
                }
                ProductArtifactRole::Dependency { startup } => {
                    dependencies.push(ChildNativeModule {
                        file: pinned,
                        required_at_startup: *startup,
                    })
                }
                ProductArtifactRole::Notice => notices.push(pinned),
                ProductArtifactRole::Model | ProductArtifactRole::License => {}
            }
        }
        let worker_path = worker.ok_or_else(|| {
            ApiError::new(ErrorCode::NeedsInput, "Dedicated product worker is missing")
        })?;
        let selected = ParentSelectedRelease::authorize(
            release.profile().clone(),
            release.manifest().clone(),
            release.as_ref(),
        )?
        .with_native_deployment(ChildNativeDeployment {
            worker_path,
            modules: dependencies,
            notices,
            system_module_names: native.system_module_names.clone(),
            minimum_windows_build: native.minimum_windows_build,
        })?;
        Ok(Self {
            root: root.into(),
            release,
            selected: Arc::new(selected),
            _leases: leases,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinekino_media_contracts::api_values::DecimalU64;
    use kinekino_vad::domain::{
        ArtifactPin, MAX_RUNTIME_NOTICE_BYTES, MAX_RUNTIME_NOTICES_TOTAL_BYTES,
    };
    fn artifacts(sizes: &[u64]) -> Vec<ProductArtifact> {
        sizes
            .iter()
            .enumerate()
            .map(|(i, size)| ProductArtifact {
                name: "synthetic-notice.txt",
                pin: ArtifactPin::new(format!("notice-{i}"), "f".repeat(64), DecimalU64(*size))
                    .unwrap(),
                role: ProductArtifactRole::Notice,
            })
            .collect()
    }
    #[test]
    fn parent_notice_policy_does_not_apply_notice_limit_to_model_weights() {
        let mut values = artifacts(&[338_088, 1073]);
        values.push(ProductArtifact {
            name: "model.onnx",
            pin: ArtifactPin::new("weights".into(), "a".repeat(64), DecimalU64(2_243_022)).unwrap(),
            role: ProductArtifactRole::Model,
        });
        validate_notices(&values).unwrap();
    }
    #[test]
    fn parent_notice_policy_preserves_file_total_and_count_bounds() {
        let half = MAX_RUNTIME_NOTICES_TOTAL_BYTES / 2;
        for sizes in [
            vec![MAX_RUNTIME_NOTICE_BYTES],
            vec![half, half],
            vec![65_536; 16],
        ] {
            validate_notices(&artifacts(&sizes)).unwrap();
        }
        for sizes in [
            vec![MAX_RUNTIME_NOTICE_BYTES + 1],
            vec![half, half + 1],
            vec![1; 17],
        ] {
            assert_eq!(
                validate_notices(&artifacts(&sizes)).err().unwrap().code,
                ErrorCode::InvalidArgument
            );
        }
    }
}
