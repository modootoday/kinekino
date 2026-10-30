//! Pinned byte readback owns ordinary root/directory/file handles; no job state mutation.
mod pixels;
use super::*;
use crate::{
    application::asset_artifacts::{AssetArtifactPublication, MAX_ASSET_PUBLICATION_BYTES},
    infrastructure::{
        content_digest, immutable_storage_paths as paths,
        storage_directory_identity::StorageDirectoryIdentity,
    },
};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::Path,
};
pub struct AssetArtifactFiles {
    root: PathBuf,
    guards: Vec<File>,
}
pub struct RetainedAssetArtifact {
    manifest: crate::application::asset_bundle::AssetBundleManifest,
    publication: AssetArtifactPublication,
    sha256: String,
    bytes: DecimalU64,
    _guards: Vec<File>,
}
impl RetainedAssetArtifact {
    pub fn publication(&self) -> &AssetArtifactPublication {
        &self.publication
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn bytes(&self) -> DecimalU64 {
        self.bytes
    }
}
fn changed() -> ApiError {
    error(
        ErrorCode::SourceExpired,
        "Asset artifact differs from pinned bytes/ordinary directory; preserve evidence",
    )
}
impl AssetArtifactFiles {
    pub fn open(root: &Path) -> Result<Self, ApiError> {
        let (root, guards) = paths::root(root)?;
        Ok(Self { root, guards })
    }
    pub fn inspect(
        &self,
        id: &str,
        expected_sha256: &str,
        control: &dyn MutationControl,
    ) -> Result<RetainedAssetArtifact, ApiError> {
        if !crate::domain::identifiers::valid_id(id)
            || !crate::domain::identifiers::valid_sha256(expected_sha256)
        {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Exact asset artifact identity/SHA required",
            ));
        }
        control.checkpoint()?;
        let mut guards = self
            .guards
            .iter()
            .map(File::try_clone)
            .collect::<Result<Vec<_>, _>>()
            .map_err(kinekino_core::file_publication::io_error)?;
        let directory = self.root.join(format!("asset-{id}"));
        let guard = paths::directory(&directory)?;
        let original = StorageDirectoryIdentity::observe(&guard)?;
        guards.push(guard);
        let mut marker = paths::file(&directory.join("publication.json"))?;
        let (length, sha) = content_digest::sha256_with_checkpoint(
            &mut marker,
            MAX_ASSET_PUBLICATION_BYTES,
            || control.checkpoint(),
        )
        .map_err(|failure| failure.into_api_error())?;
        if sha != expected_sha256 {
            return Err(changed());
        }
        marker
            .seek(SeekFrom::Start(0))
            .map_err(kinekino_core::file_publication::io_error)?;
        let mut bytes = Vec::new();
        Read::by_ref(&mut marker)
            .take(MAX_ASSET_PUBLICATION_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(kinekino_core::file_publication::io_error)?;
        let publication: AssetArtifactPublication =
            serde_json::from_slice(&bytes).map_err(|_| {
                error(
                    ErrorCode::NeedsInput,
                    "Strict asset publication required; preserve records",
                )
            })?;
        publication.validate(id, length)?;
        let expected: BTreeSet<_> = publication
            .files
            .iter()
            .map(|file| file.name.as_str())
            .chain(Some("publication.json"))
            .collect();
        let check_names = || -> Result<(), ApiError> {
            let mut count = 0;
            for entry in
                fs::read_dir(&directory).map_err(kinekino_core::file_publication::io_error)?
            {
                control.checkpoint()?;
                let entry = entry.map_err(kinekino_core::file_publication::io_error)?;
                count += 1;
                if count > expected.len()
                    || entry
                        .file_name()
                        .to_str()
                        .is_none_or(|name| !expected.contains(name))
                {
                    return Err(changed());
                }
            }
            if count != expected.len() {
                return Err(changed());
            }
            Ok(())
        };
        check_names()?;
        guards.push(marker);
        let file_base = guards.len();
        let mut project_bytes = Vec::new();
        let mut manifest_bytes = Vec::new();
        for file in &publication.files {
            control.checkpoint()?;
            let mut handle = paths::file(&directory.join(&file.name))?;
            let observed =
                content_digest::sha256_with_checkpoint(&mut handle, file.bytes.0, || {
                    control.checkpoint()
                })
                .map_err(|failure| failure.into_api_error())?;
            if observed != (file.bytes.0, file.sha256.clone()) {
                return Err(changed());
            }
            if file.name == "project.json" || file.name == "manifest.json" {
                if file.bytes.0 > MAX_ASSET_PUBLICATION_BYTES {
                    return Err(changed());
                }
                handle
                    .seek(SeekFrom::Start(0))
                    .map_err(kinekino_core::file_publication::io_error)?;
                let buffer = if file.name == "project.json" {
                    &mut project_bytes
                } else {
                    &mut manifest_bytes
                };
                Read::by_ref(&mut handle)
                    .take(MAX_ASSET_PUBLICATION_BYTES + 1)
                    .read_to_end(buffer)
                    .map_err(kinekino_core::file_publication::io_error)?;
            }
            guards.push(handle);
        }
        let manifest = publication.validate_contents(&project_bytes, &manifest_bytes, control)?;
        for (index, layer) in manifest.layers.iter().enumerate() {
            let start = file_base + 1 + index * 3;
            pixels::verify(
                layer,
                &guards[start],
                &guards[start + 1],
                &guards[start + 2],
                control,
            )?;
        }
        check_names()?;
        let witness = paths::directory(&directory)?;
        if StorageDirectoryIdentity::observe(&witness)? != original {
            return Err(changed());
        }
        guards.push(witness);
        control.checkpoint()?;
        Ok(RetainedAssetArtifact {
            manifest,
            publication,
            sha256: sha,
            bytes: DecimalU64(length),
            _guards: guards,
        })
    }
}
impl crate::application::asset_artifacts::AssetArtifactInspector for AssetArtifactFiles {
    fn inspect_artifact(
        &self,
        input: &crate::application::asset_artifacts::InspectAssetArtifact,
        control: &dyn MutationControl,
    ) -> Result<crate::application::asset_artifacts::AssetArtifactInspection, ApiError> {
        let retained = self.inspect(&input.artifact_id, &input.publication_sha256, control)?;
        Ok(
            crate::application::asset_artifacts::AssetArtifactInspection {
                publication: retained.publication.clone(),
                publication_sha256: retained.sha256.clone(),
                publication_bytes: retained.bytes,
                manifest: retained.manifest.clone(),
            },
        )
    }
}
