use super::*;
use crate::{
    application::{
        asset_bundle::{AssetProjectLease, PreparedAssetSources},
        asset_generation::AssetInputAdmission,
    },
    infrastructure::native_immutable_publication,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetSourceArchivePin {
    pub archive_id: String,
    pub manifest_sha256: String,
    pub manifest_bytes: crate::domain::api_values::DecimalU64,
}
pub struct PublishedAssetSourceArchive {
    pub pin: AssetSourceArchivePin,
    pub archive: RetainedAssetSourceArchive,
}
impl AssetSourceArchiveFiles {
    /// Independent source storage only; this does not exercise a Jobs publication CAS.
    pub fn publish<L: AssetProjectLease, P: AssetInputAdmission>(
        &self,
        id: &str,
        sources: &PreparedAssetSources<L, P>,
        control: &dyn MutationControl,
    ) -> Result<PublishedAssetSourceArchive, ApiError> {
        if !valid_id(id) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Source archive identity required",
            ));
        }
        control.checkpoint()?;
        let files = sources.files();
        let marker = files.last().ok_or_else(changed)?;
        if marker.descriptor.name != "sources.json" {
            return Err(changed());
        }
        let pin = AssetSourceArchivePin {
            archive_id: id.into(),
            manifest_sha256: marker.descriptor.sha256.clone(),
            manifest_bytes: marker.descriptor.bytes,
        };
        let directory = self.root.join(format!("sources-{id}"));
        let _writer = paths::lock(&self.root.join("source-archive-writer.lock"))?;
        let mut created = false;
        let mut committed = false;
        let result = (|| {
            fs::create_dir(&directory).map_err(kinekino_core::file_publication::io_error)?;
            created = true;
            let guard = paths::directory(&directory)?;
            let identity = StorageDirectoryIdentity::observe(&guard)?;
            let mut handles = Vec::new();
            let mut identities = BTreeMap::new();
            let mut expected = BTreeSet::new();
            for file in files {
                control.checkpoint()?;
                let mut handle = native_immutable_publication::publish_retained(
                    &directory,
                    &file.descriptor.name,
                    file.contents,
                )?;
                if file.descriptor.name == "sources.json" {
                    committed = true;
                }
                let before = StorageDirectoryIdentity::observe(&handle)?;
                identities.insert(
                    file.descriptor.name.clone(),
                    StorageDirectoryIdentity::observe(&handle)?,
                );
                handle
                    .seek(SeekFrom::Start(0))
                    .map_err(kinekino_core::file_publication::io_error)?;
                let observed = content_digest::sha256_with_checkpoint(
                    &mut handle,
                    file.descriptor.bytes.0,
                    || control.checkpoint(),
                )
                .map_err(|e| e.into_api_error())?;
                if observed != (file.descriptor.bytes.0, file.descriptor.sha256.clone())
                    || StorageDirectoryIdentity::observe(&handle)? != before
                {
                    return Err(changed());
                }
                expected.insert(file.descriptor.name.as_str());
                handles.push(handle);
                let actual = fs::read_dir(&directory)
                    .map_err(kinekino_core::file_publication::io_error)?
                    .map(|entry| {
                        entry
                            .map_err(kinekino_core::file_publication::io_error)?
                            .file_name()
                            .into_string()
                            .map_err(|_| changed())
                    })
                    .collect::<Result<BTreeSet<_>, _>>()?;
                if actual.len() != expected.len()
                    || actual.iter().any(|name| !expected.contains(name.as_str()))
                    || StorageDirectoryIdentity::observe(&guard)? != identity
                {
                    return Err(changed());
                }
            }
            // Original writer handles have DELETE access; close them before strict read-only reopen.
            drop(handles);
            let archive = self.inspect(id, &pin.manifest_sha256, control)?;
            if archive.identities != identities {
                return Err(changed());
            }
            Ok(PublishedAssetSourceArchive {
                pin: pin.clone(),
                archive,
            })
        })();
        result.map_err(|mut error:ApiError|{error.details=serde_json::json!({"primary_details":error.details,"source_archive_recovery":{"archive_id":id,"manifest_sha256":pin.manifest_sha256,"directory_created":created,"marker_committed":committed,"partial_files_removed":false,"automatic_retry":false}});error})
    }
}
