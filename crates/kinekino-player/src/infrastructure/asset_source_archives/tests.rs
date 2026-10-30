use super::*;
use crate::{
    application::{asset_generation, review_source::MutationControl},
    infrastructure::asset_raster::RustAssetRaster,
};
struct Observe;
impl MutationControl for Observe {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
#[test]
#[ignore = "Pinned existing original source archive, expected raster SHA and fresh evidence required"]
fn original_archive_reopens_admits_renders_and_retains_sources() {
    let source = PathBuf::from(std::env::var_os("SOURCE_ARCHIVE_INPUT").unwrap());
    let root = PathBuf::from(std::env::var_os("SOURCE_ARCHIVE_EVIDENCE").unwrap());
    let pin = std::env::var("SOURCE_ARCHIVE_SHA256").unwrap();
    fs::create_dir(&root).unwrap();
    let directory = root.join("sources-existing");
    fs::create_dir(&directory).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());
        fs::copy(entry.path(), directory.join(entry.file_name())).unwrap();
    }
    let files = AssetSourceArchiveFiles::open(&root).unwrap();
    assert!(
        files
            .inspect("existing", &"0".repeat(64), &Observe)
            .is_err()
    );
    let archive = files.inspect("existing", &pin, &Observe).unwrap();
    let manifest = archive.manifest().clone();
    let project = archive.project().clone();
    use crate::application::asset_source_archives::{
        AssetSourcesObservation, ImportAssetSources, prepare_import,
    };
    let observed = AssetSourcesObservation {
        archive_id: "existing".into(),
        manifest_sha256: pin.clone(),
        manifest_bytes: crate::domain::api_values::DecimalU64(
            fs::metadata(directory.join("sources.json")).unwrap().len(),
        ),
        manifest: manifest.clone(),
        project: project.clone(),
    };
    let import = ImportAssetSources {
        archive_id: "existing".into(),
        manifest_sha256: pin.clone(),
        new_document_id: "imported-editable".into(),
        acknowledge_unreviewed_masks: true,
    };
    let plan = prepare_import(&observed, &import).unwrap();
    assert_eq!(plan.document.document_id, "imported-editable");
    assert_eq!(plan.document.revision.0, 1);
    assert_eq!(plan.document.sources, project.document.sources);
    assert_eq!(plan.document.layers, project.document.layers);
    assert!(!plan.quality_approved);
    assert!(!plan.review_admitted);
    let mut denied = import.clone();
    denied.acknowledge_unreviewed_masks = false;
    assert_eq!(
        prepare_import(&observed, &denied).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    denied = import.clone();
    denied.new_document_id = project.document.document_id.clone();
    assert_eq!(
        prepare_import(&observed, &denied).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let mut forged = observed.clone();
    forged.manifest.quality_approved = true;
    assert_eq!(
        prepare_import(&forged, &import).unwrap_err().code,
        ErrorCode::SourceExpired
    );
    use crate::infrastructure::asset_projects::AssetProjectFiles;
    use std::sync::Arc;
    let project_root = root.join("imported-projects");
    fs::create_dir(&project_root).unwrap();
    let service = NativeAssetSourceArchiveService {
        _host_guards: Arc::new(Vec::new()),
        projects: Arc::new(AssetProjectFiles::open(&project_root).unwrap()),
        inputs: Arc::new(
            archive
                .frame_inputs(&manifest.bindings[0].frame_id)
                .unwrap(),
        ),
        archives: AssetSourceArchiveFiles::open(&root).unwrap(),
    };
    let imported = service.import_project(&import, &Observe).unwrap();
    assert_eq!(imported.project.publication.document, plan.document);
    assert_eq!(imported.project.publication.publication_revision.0, 1);
    assert!(
        imported
            .project
            .publication
            .parent_publication_sha256
            .is_none()
    );
    assert_eq!(
        service.import_project(&import, &Observe).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let retained = service
        .inspect_import(&import.new_document_id, &imported.commit_sha256, &Observe)
        .unwrap();
    assert_eq!(retained.receipt().source.manifest_sha256, pin);
    assert!(!retained.receipt().quality_approved);
    drop(retained);
    struct CancelAt(PathBuf);
    impl MutationControl for CancelAt {
        fn checkpoint(&self) -> Result<(), ApiError> {
            if self.0.exists() {
                Err(ApiError::new(
                    ErrorCode::Cancelled,
                    "Injected import boundary cancellation",
                ))
            } else {
                Ok(())
            }
        }
    }
    let mut before = import.clone();
    before.new_document_id = "before".into();
    let error = service
        .import_project(&before, &CancelAt(root.join("import-before/plan.json")))
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert_eq!(error.details["import_recovery"]["project_may_exist"], false);
    assert!(root.join("import-before/plan.json").is_file());
    assert!(!root.join("import-before/commit.json").exists());
    let mut after = import.clone();
    after.new_document_id = "after".into();
    let error_after = service
        .import_project(&after, &CancelAt(project_root.join("asset-after")))
        .unwrap_err();
    assert_eq!(error_after.code, ErrorCode::Cancelled);
    assert_eq!(
        error_after.details["import_recovery"]["project_created"],
        true
    );
    assert_eq!(
        error_after.details["import_recovery"]["marker_committed"],
        false
    );
    assert!(!root.join("import-after/commit.json").exists());
    let mut late = import.clone();
    late.new_document_id = "late".into();
    let error_late = service
        .import_project(&late, &CancelAt(root.join("import-late/commit.json")))
        .unwrap_err();
    assert_eq!(error_late.code, ErrorCode::Cancelled);
    assert_eq!(
        error_late.details["import_recovery"]["marker_committed"],
        true
    );
    let recovered = service
        .inspect_import(
            "late",
            error_late.details["import_recovery"]["commit_sha256"]
                .as_str()
                .unwrap(),
            &Observe,
        )
        .unwrap();
    assert_eq!(recovered.receipt().imported_project.document_id, "late");
    drop(recovered);
    fs::write(root.join("import-results.json"),serde_json::to_vec_pretty(&serde_json::json!({"imported":imported,"before":error,"after":error_after,"late":error_late,"jobs_verified":false,"cli_verified":false})).unwrap()).unwrap();
    drop(service);
    let binding = &manifest.bindings[0];
    let inputs = archive.frame_inputs(&binding.frame_id).unwrap();
    let request = project
        .document
        .frame_request(&binding.frame_id, &project.document.layers[0].layer_id)
        .unwrap();
    let prepared = asset_generation::prepare(&request, &inputs, &Observe).unwrap();
    let path = directory.join(&manifest.originals[0].name);
    drop(files);
    drop(archive);
    drop(inputs);
    assert!(fs::OpenOptions::new().write(true).open(&path).is_err());
    let rendered = prepared.render(&RustAssetRaster, &Observe).unwrap();
    use sha2::{Digest, Sha256};
    assert_eq!(
        format!("{:x}", Sha256::digest(&rendered.pixels().rgba)),
        std::env::var("SOURCE_ARCHIVE_EXPECTED_RGBA_SHA256").unwrap()
    );
    drop(rendered);
    assert!(fs::OpenOptions::new().write(true).open(&path).is_ok());
    let reader = AssetSourceArchiveFiles::open(&root).unwrap();
    let project_bytes = fs::read(directory.join("project.json")).unwrap();
    let marker_len = fs::metadata(directory.join("sources.json")).unwrap().len();
    let mut wrong = manifest.clone();
    wrong.bindings[0].frame_id = "foreign".into();
    assert!(
        wrong
            .validate_project(&project_bytes, marker_len, &Observe)
            .is_err()
    );
    wrong = manifest.clone();
    wrong.originals[0].name = "../escape.png".into();
    assert!(
        wrong
            .validate_project(&project_bytes, marker_len, &Observe)
            .is_err()
    );
    wrong = manifest.clone();
    wrong.review_admitted = true;
    assert!(
        wrong
            .validate_project(&project_bytes, marker_len, &Observe)
            .is_err()
    );
    fs::write(directory.join("extra"), b"unexpected").unwrap();
    assert!(reader.inspect("existing", &pin, &Observe).is_err());
    fs::remove_file(directory.join("extra")).unwrap();
    let mut bytes = fs::read(&path).unwrap();
    bytes[0] ^= 1;
    fs::write(path, bytes).unwrap();
    assert!(reader.inspect("existing", &pin, &Observe).is_err());
}
