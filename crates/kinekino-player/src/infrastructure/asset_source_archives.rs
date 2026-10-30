//! Administrative fixed-root archive readback. No Jobs completion or review authority.
mod imports;
mod publication;
mod service;
pub use imports::RetainedAssetSourceImport;
pub use publication::{AssetSourceArchivePin, PublishedAssetSourceArchive};
pub use service::NativeAssetSourceArchiveService;
#[cfg(test)]
mod tests;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        asset_bundle::AssetSourceManifest,
        asset_projects::AssetPublication,
        review_source::MutationControl,
    },
    domain::identifiers::{valid_id, valid_sha256},
    infrastructure::{
        asset_inputs::{FixedAssetImage, FixedAssetInputs, FixedAssetMask},
        content_digest, immutable_storage_paths as paths,
        storage_directory_identity::StorageDirectoryIdentity,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};
const MAX_METADATA: u64 = crate::application::asset_artifacts::MAX_ASSET_PUBLICATION_BYTES;
pub struct AssetSourceArchiveFiles {
    root: PathBuf,
    guards: Vec<File>,
}
pub struct RetainedAssetSourceArchive {
    identities: BTreeMap<String, StorageDirectoryIdentity>,
    directory: PathBuf,
    manifest: AssetSourceManifest,
    project: AssetPublication,
    _guards: Vec<File>,
}
fn changed() -> ApiError {
    ApiError::new(
        ErrorCode::SourceExpired,
        "Source archive differs from exact metadata/files; preserve evidence",
    )
}
fn read(file: &mut File, limit: u64) -> Result<Vec<u8>, ApiError> {
    file.seek(SeekFrom::Start(0))
        .map_err(kinekino_core::file_publication::io_error)?;
    let mut bytes = Vec::new();
    Read::by_ref(file)
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(kinekino_core::file_publication::io_error)?;
    if bytes.len() as u64 > limit {
        return Err(changed());
    }
    Ok(bytes)
}
impl AssetSourceArchiveFiles {
    pub fn open(root: &Path) -> Result<Self, ApiError> {
        let (_, guards) = paths::root(root)?;
        Ok(Self {
            root: root.to_path_buf(),
            guards,
        })
    }
    pub fn inspect(
        &self,
        id: &str,
        sha: &str,
        control: &dyn MutationControl,
    ) -> Result<RetainedAssetSourceArchive, ApiError> {
        if !valid_id(id) || !valid_sha256(sha) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Source archive identity/SHA required",
            ));
        }
        control.checkpoint()?;
        let mut guards = self
            .guards
            .iter()
            .map(File::try_clone)
            .collect::<Result<Vec<_>, _>>()
            .map_err(kinekino_core::file_publication::io_error)?;
        let directory = self.root.join(format!("sources-{id}"));
        let guard = paths::directory(&directory)?;
        let identity = StorageDirectoryIdentity::observe(&guard)?;
        guards.push(guard);
        let mut marker = paths::file(&directory.join("sources.json"))?;
        let mut identities = BTreeMap::from([(
            "sources.json".into(),
            StorageDirectoryIdentity::observe(&marker)?,
        )]);
        let (_, observed) =
            content_digest::sha256_with_checkpoint(&mut marker, MAX_METADATA, || {
                control.checkpoint()
            })
            .map_err(|e| e.into_api_error())?;
        if observed != sha {
            return Err(changed());
        }
        let bytes = read(&mut marker, MAX_METADATA)?;
        let manifest: AssetSourceManifest =
            serde_json::from_slice(&bytes).map_err(|_| changed())?;
        guards.push(marker);
        let mut file = paths::file(&directory.join("project.json"))?;
        identities.insert(
            "project.json".into(),
            StorageDirectoryIdentity::observe(&file)?,
        );
        let project_bytes = read(&mut file, MAX_METADATA)?;
        guards.push(file);
        let project = manifest.validate_project(&project_bytes, bytes.len() as u64, control)?;
        let expected: BTreeSet<_> = manifest
            .originals
            .iter()
            .map(|f| f.name.as_str())
            .chain(["project.json", "sources.json"])
            .collect();
        let names = || -> Result<(), ApiError> {
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
        names()?;
        for descriptor in &manifest.originals {
            let mut file = paths::file(&directory.join(&descriptor.name))?;
            identities.insert(
                descriptor.name.clone(),
                StorageDirectoryIdentity::observe(&file)?,
            );
            let result =
                content_digest::sha256_with_checkpoint(&mut file, descriptor.bytes.0, || {
                    control.checkpoint()
                })
                .map_err(|e| e.into_api_error())?;
            if result != (descriptor.bytes.0, descriptor.sha256.clone()) {
                return Err(changed());
            }
            guards.push(file);
        }
        names()?;
        let witness = paths::directory(&directory)?;
        if StorageDirectoryIdentity::observe(&witness)? != identity {
            return Err(changed());
        }
        guards.push(witness);
        control.checkpoint()?;
        Ok(RetainedAssetSourceArchive {
            identities,
            directory,
            manifest,
            project,
            _guards: guards,
        })
    }
}
impl RetainedAssetSourceArchive {
    pub fn manifest(&self) -> &AssetSourceManifest {
        &self.manifest
    }
    pub fn project(&self) -> &AssetPublication {
        &self.project
    }
    /// A host catalog for subsequent ordinary source admission; not a review lease.
    pub fn frame_inputs(&self, id: &str) -> Result<FixedAssetInputs, ApiError> {
        let binding = self
            .manifest
            .bindings
            .iter()
            .find(|b| b.frame_id == id)
            .ok_or_else(changed)?;
        FixedAssetInputs::new(
            BTreeMap::from([(
                binding.image.blob.artifact_id.clone(),
                FixedAssetImage {
                    pin: binding.image.clone(),
                    path: self
                        .directory
                        .join(format!("source-{}.png", binding.image.blob.sha256)),
                },
            )]),
            BTreeMap::from([(
                binding.mask.blob.artifact_id.clone(),
                FixedAssetMask {
                    pin: binding.mask.clone(),
                    path: self
                        .directory
                        .join(format!("source-{}.png", binding.mask.blob.sha256)),
                },
            )]),
        )
    }
}
