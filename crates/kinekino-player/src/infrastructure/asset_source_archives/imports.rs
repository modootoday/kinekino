//! Create-only import transaction; incomplete project/plan is never silently adopted.
use super::*;
use crate::{
    application::{
        asset_bundle::{AssetProjectAdmission, AssetProjectLease, SavedAssetProjectPin},
        asset_generation::{AssetImageLease, AssetInputAdmission, AssetMaskLease},
        asset_projects::{self, AssetProjectStore},
        asset_source_archives::*,
    },
    infrastructure::{asset_projects::NativeAssetProjectLease, native_immutable_publication},
};
use sha2::{Digest, Sha256};
pub struct RetainedAssetSourceImport {
    receipt: AssetSourceImportReceipt,
    archive: RetainedAssetSourceArchive,
    _project: NativeAssetProjectLease,
    _guards: Vec<File>,
    identities: BTreeMap<String, StorageDirectoryIdentity>,
}
impl RetainedAssetSourceImport {
    pub fn receipt(&self) -> &AssetSourceImportReceipt {
        &self.receipt
    }
    pub fn archive(&self) -> &RetainedAssetSourceArchive {
        &self.archive
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn encode(value: &impl serde::Serialize) -> Result<Vec<u8>, ApiError> {
    serde_json::to_vec(value).map_err(|_| changed())
}
fn names(
    directory: &Path,
    expected: &[&str],
    control: &dyn MutationControl,
) -> Result<(), ApiError> {
    let mut seen = BTreeSet::new();
    for entry in fs::read_dir(directory).map_err(kinekino_core::file_publication::io_error)? {
        control.checkpoint()?;
        let name = entry
            .map_err(kinekino_core::file_publication::io_error)?
            .file_name()
            .into_string()
            .map_err(|_| changed())?;
        if !expected.contains(&name.as_str()) || !seen.insert(name) {
            return Err(changed());
        }
    }
    if seen.len() != expected.len() {
        return Err(changed());
    }
    Ok(())
}
fn write(
    directory: &Path,
    name: &str,
    bytes: &[u8],
    control: &dyn MutationControl,
) -> Result<File, ApiError> {
    let mut handle = native_immutable_publication::publish_retained(directory, name, bytes)?;
    handle
        .seek(SeekFrom::Start(0))
        .map_err(kinekino_core::file_publication::io_error)?;
    let observed = content_digest::sha256_with_checkpoint(&mut handle, bytes.len() as u64, || {
        control.checkpoint()
    })
    .map_err(|e| e.into_api_error())?;
    if observed != (bytes.len() as u64, hash(bytes)) {
        return Err(changed());
    }
    Ok(handle)
}
impl NativeAssetSourceArchiveService {
    pub fn import_project(
        &self,
        input: &ImportAssetSources,
        control: &dyn MutationControl,
    ) -> Result<ImportedAssetSources, ApiError> {
        InspectAssetSources {
            archive_id: input.archive_id.clone(),
            manifest_sha256: input.manifest_sha256.clone(),
        }
        .validate()?;
        let archive = self
            .archives
            .inspect(&input.archive_id, &input.manifest_sha256, control)?;
        let observed = service::observation(&input.archive_id, &input.manifest_sha256, &archive)?;
        let plan = prepare_import(&observed, input)?;
        let mut retained_bytes = 0u64;
        let mut inputs = Vec::new();
        for binding in &archive.manifest.bindings {
            control.checkpoint()?;
            let catalog = archive.frame_inputs(&binding.frame_id)?;
            let image = catalog.open_image(&binding.image, control)?;
            let mask = catalog.open_mask(&binding.mask, control)?;
            let bytes = image.rgba().len() as u64
                + mask.coverage().len() as u64
                + image.original_png().map_or(0, |b| b.len() as u64)
                + mask.original_png().map_or(0, |b| b.len() as u64);
            retained_bytes = retained_bytes
                .checked_add(bytes)
                .filter(|n| *n <= crate::application::asset_bundle::MAX_ASSET_RETAINED_BYTES)
                .ok_or_else(|| {
                    ApiError::new(
                        ErrorCode::ResourceExhausted,
                        "Import source admission memory budget exceeded",
                    )
                })?;
            inputs.push((image, mask));
        }
        let _writer = paths::lock(&self.archives.root.join("source-import-writer.lock"))?;
        match self.projects.load(&input.new_document_id, None) {
            Ok(_) => {
                return Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Import requires a new project identity",
                ));
            }
            Err(e) if e.code == ErrorCode::NotFound => {}
            Err(e) => return Err(e),
        }
        let expected = asset_projects::prepare_publication(None, &plan.document)?;
        let expected_pin = SavedAssetProjectPin {
            document_id: input.new_document_id.clone(),
            publication_revision: expected.publication_revision,
            publication_sha256: asset_projects::publication_sha256(&expected)?,
        };
        let plan_bytes = encode(&plan)?;
        let receipt = AssetSourceImportReceipt {
            schema_version: "asset-source-import/1".into(),
            source: plan.source.clone(),
            original_project: plan.original_project.clone(),
            imported_project: expected_pin,
            plan_sha256: hash(&plan_bytes),
            quality_approved: false,
            review_admitted: false,
        };
        let commit_bytes = encode(&receipt)?;
        let commit_sha256 = hash(&commit_bytes);
        let directory = self
            .archives
            .root
            .join(format!("import-{}", input.new_document_id));
        let mut directory_created = false;
        let mut project_may_exist = false;
        let mut project_created = false;
        let mut marker_committed = false;
        let result = (|| {
            control.checkpoint()?;
            fs::create_dir(&directory).map_err(kinekino_core::file_publication::io_error)?;
            directory_created = true;
            let guard = paths::directory(&directory)?;
            let identity = StorageDirectoryIdentity::observe(&guard)?;
            let plan_file = write(&directory, "plan.json", &plan_bytes, control)?;
            let plan_identity = StorageDirectoryIdentity::observe(&plan_file)?;
            names(&directory, &["plan.json"], control)?;
            control.checkpoint()?;
            project_may_exist = true;
            let stored = asset_projects::create(self.projects.as_ref(), &plan.document)?;
            project_created = true;
            if stored.publication != expected
                || stored.publication_sha256 != receipt.imported_project.publication_sha256
            {
                return Err(changed());
            }
            control.checkpoint()?;
            // Set committed immediately after the original no-replace rename,
            // before any cancellable readback of that completed marker.
            let mut commit_file = native_immutable_publication::publish_retained(
                &directory,
                "commit.json",
                &commit_bytes,
            )?;
            marker_committed = true;
            let commit_identity = StorageDirectoryIdentity::observe(&commit_file)?;
            commit_file
                .seek(SeekFrom::Start(0))
                .map_err(kinekino_core::file_publication::io_error)?;
            let check = content_digest::sha256_with_checkpoint(
                &mut commit_file,
                commit_bytes.len() as u64,
                || control.checkpoint(),
            )
            .map_err(|e| e.into_api_error())?;
            if check != (commit_bytes.len() as u64, commit_sha256.clone())
                || StorageDirectoryIdentity::observe(&guard)? != identity
            {
                return Err(changed());
            }
            names(&directory, &["plan.json", "commit.json"], control)?;
            drop(plan_file);
            drop(commit_file);
            let reopened = self.inspect_import(&input.new_document_id, &commit_sha256, control)?;
            if reopened.identities.get("plan.json") != Some(&plan_identity)
                || reopened.identities.get("commit.json") != Some(&commit_identity)
            {
                return Err(changed());
            }
            Ok(ImportedAssetSources {
                receipt: receipt.clone(),
                commit_sha256: commit_sha256.clone(),
                project: stored,
            })
        })();
        result.map_err(|mut error:ApiError|{error.details=serde_json::json!({"primary_details":error.details,"import_recovery":{"document_id":input.new_document_id,"commit_sha256":commit_sha256,"directory_created":directory_created,"project_may_exist":project_may_exist,"project_created":project_created,"marker_committed":marker_committed,"automatic_retry":false,"partial_files_removed":false}});error})
    }
    pub fn inspect_import(
        &self,
        id: &str,
        expected_sha: &str,
        control: &dyn MutationControl,
    ) -> Result<RetainedAssetSourceImport, ApiError> {
        if !valid_id(id) || !valid_sha256(expected_sha) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Exact import identity/commit SHA required",
            ));
        }
        control.checkpoint()?;
        let mut guards = self
            .archives
            .guards
            .iter()
            .map(File::try_clone)
            .collect::<Result<Vec<_>, _>>()
            .map_err(kinekino_core::file_publication::io_error)?;
        let directory = self.archives.root.join(format!("import-{id}"));
        let guard = paths::directory(&directory)?;
        let identity = StorageDirectoryIdentity::observe(&guard)?;
        guards.push(guard);
        let mut commit = paths::file(&directory.join("commit.json")).map_err(|mut e| {
            if e.code == ErrorCode::NotFound {
                e.code = ErrorCode::NeedsInput;
                e.message =
                    "Import plan/project may exist without commit; inspect preserved state".into();
            }
            e
        })?;
        let commit_bytes = read(&mut commit, MAX_METADATA)?;
        if hash(&commit_bytes) != expected_sha {
            return Err(changed());
        }
        let receipt: AssetSourceImportReceipt =
            serde_json::from_slice(&commit_bytes).map_err(|_| changed())?;
        let mut identities = BTreeMap::from([(
            "commit.json".into(),
            StorageDirectoryIdentity::observe(&commit)?,
        )]);
        guards.push(commit);
        if receipt.schema_version != "asset-source-import/1"
            || receipt.quality_approved
            || receipt.review_admitted
            || receipt.imported_project.document_id != id
            || receipt.imported_project.publication_revision.0 != 1
        {
            return Err(changed());
        }
        receipt.source.validate()?;
        receipt.imported_project.validate()?;
        receipt.original_project.validate()?;
        let archive = self.archives.inspect(
            &receipt.source.archive_id,
            &receipt.source.manifest_sha256,
            control,
        )?;
        let mut plan_file = paths::file(&directory.join("plan.json"))?;
        let plan_bytes = read(&mut plan_file, MAX_METADATA)?;
        if hash(&plan_bytes) != receipt.plan_sha256 {
            return Err(changed());
        }
        identities.insert(
            "plan.json".into(),
            StorageDirectoryIdentity::observe(&plan_file)?,
        );
        guards.push(plan_file);
        let plan: AssetSourceImportPlan =
            serde_json::from_slice(&plan_bytes).map_err(|_| changed())?;
        let expected = prepare_import(
            &service::observation(
                &receipt.source.archive_id,
                &receipt.source.manifest_sha256,
                &archive,
            )?,
            &ImportAssetSources {
                archive_id: receipt.source.archive_id.clone(),
                manifest_sha256: receipt.source.manifest_sha256.clone(),
                new_document_id: id.into(),
                acknowledge_unreviewed_masks: true,
            },
        )?;
        if encode(&plan)? != encode(&expected)?
            || encode(&plan.original_project)? != encode(&receipt.original_project)?
        {
            return Err(changed());
        }
        let project = self
            .projects
            .open_project(&receipt.imported_project, control)?;
        let expected_publication = asset_projects::prepare_publication(None, &plan.document)?;
        if project.publication() != &expected_publication {
            return Err(changed());
        }
        names(&directory, &["plan.json", "commit.json"], control)?;
        let witness = paths::directory(&directory)?;
        if StorageDirectoryIdentity::observe(&witness)? != identity {
            return Err(changed());
        }
        guards.push(witness);
        control.checkpoint()?;
        Ok(RetainedAssetSourceImport {
            receipt,
            archive,
            _project: project,
            _guards: guards,
            identities,
        })
    }
}
