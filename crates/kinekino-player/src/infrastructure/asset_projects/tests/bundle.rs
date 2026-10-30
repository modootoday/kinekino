use super::*;
use crate::application::{asset_bundle::*, review_source::MutationControl};
struct Control(bool);
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if self.0 {
            Err(ApiError::new(
                ErrorCode::Cancelled,
                "Original attempt cancelled",
            ))
        } else {
            Ok(())
        }
    }
}
fn request(saved: &asset_projects::StoredAssetProject) -> AssetBundleRequest {
    AssetBundleRequest {
        project: SavedAssetProjectPin {
            document_id: saved.publication.document.document_id.clone(),
            publication_revision: saved.publication.publication_revision,
            publication_sha256: saved.publication_sha256.clone(),
        },
        missing_frames: MissingAssetFramePolicy::Reject,
        review_artifact_only: true,
    }
}
#[test]
fn original_project_history_and_ancestors_survive_catalog_drop_and_new_head() {
    let root = root();
    let store = AssetProjectFiles::open(&root).unwrap();
    let mut document = selected(draft("captured"), input());
    let saved = store.publish(None, &document).unwrap();
    let capture = capture(&store, &request(&saved), &Control(false)).unwrap();
    assert_eq!(
        AssetBundleOutputBuilder::<_, crate::infrastructure::asset_inputs::FixedAssetInputs>::new(
            &capture
        )
        .finish(&Control(false))
        .err()
        .unwrap()
        .code,
        ErrorCode::NeedsInput
    );
    document.revision.0 = 2;
    document.layers[0].transform.pivot.x = 0;
    store.publish(Some(DecimalU64(1)), &document).unwrap();
    drop(store);
    let path = root
        .join("asset-captured")
        .join(format!("r-1-{}.json", saved.publication_sha256));
    assert_eq!(
        OpenOptions::new().write(true).open(&path).is_err(),
        crate::infrastructure::os_policy::has_mandatory_share_modes()
    );
    assert_eq!(capture.publication().document.revision.0, 1);
    assert_eq!(capture.plan().units.len(), 1);
    let renamed = root.with_extension("renamed");
    // Windows open directory handles pin the ancestors; POSIX renames them freely.
    let moved = fs::rename(&root, &renamed);
    assert_eq!(
        moved.is_err(),
        crate::infrastructure::os_policy::has_mandatory_share_modes()
    );
    if moved.is_ok() {
        fs::rename(&renamed, &root).unwrap();
    }
    drop(capture);
    assert!(OpenOptions::new().write(true).open(&path).is_ok());
}
#[test]
fn gaps_are_explicit_and_visible_locked_layers_keep_document_order() {
    let root = root();
    let store = AssetProjectFiles::open(&root).unwrap();
    let mut document = selected(draft("gaps"), input());
    document.order_basis = AssetOrderBasis::DeclaredFrameOrder;
    document.sources.insert(
        "missing".into(),
        AssetFrameSlot {
            position: AssetFramePosition::StaticPng,
            selected_image: None,
        },
    );
    document.frames.insert(
        0,
        AssetFrameEntry::Missing {
            frame_id: "missing".into(),
            reason: "Original frame unavailable".into(),
        },
    );
    let mut back = document.layers[0].clone();
    back.layer_id = "background".into();
    back.kind = LayerKind::Background;
    back.locked = true;
    document.layers.push(back.clone());
    back.layer_id = "hidden".into();
    back.visible = false;
    document.layers.push(back);
    let saved = store.publish(None, &document).unwrap();
    let mut req = request(&saved);
    assert_eq!(
        capture(&store, &req, &Control(false)).err().unwrap().code,
        ErrorCode::NeedsInput
    );
    req.missing_frames = MissingAssetFramePolicy::PreserveGap;
    let captured = capture(&store, &req, &Control(false)).unwrap();
    assert_eq!(
        captured.plan().missing_frame_policy,
        MissingAssetFramePolicy::PreserveGap
    );
    assert_eq!(captured.plan().frames.len(), 2);
    assert!(captured.plan().frames[0].missing_reason.is_some());
    assert_eq!(
        captured
            .plan()
            .units
            .iter()
            .map(|u| (u.frame_position.0, u.layer_id.as_str()))
            .collect::<Vec<_>>(),
        vec![(1, "foreground"), (1, "background")]
    );
}
#[test]
fn cancelled_forged_and_empty_capture_never_become_a_render_plan() {
    let root = root();
    let store = AssetProjectFiles::open(&root).unwrap();
    let saved = store
        .publish(None, &selected(draft("reject"), input()))
        .unwrap();
    let mut req = request(&saved);
    assert_eq!(
        capture(&store, &req, &Control(true)).err().unwrap().code,
        ErrorCode::Cancelled
    );
    req.project.publication_sha256 = "c".repeat(64);
    assert_eq!(
        capture(&store, &req, &Control(false)).err().unwrap().code,
        ErrorCode::SourceExpired
    );
    req = request(&saved);
    req.review_artifact_only = false;
    assert_eq!(
        capture(&store, &req, &Control(false)).err().unwrap().code,
        ErrorCode::NeedsInput
    );
    let empty = store.publish(None, &draft("empty")).unwrap();
    assert_eq!(
        capture(&store, &request(&empty), &Control(false))
            .err()
            .unwrap()
            .code,
        ErrorCode::NeedsInput
    );
}

#[test]
#[ignore = "Pinned existing PNG request/inputs and fresh evidence; original project lease retained through real raster, no Jobs/GUI/model approval"]
fn original_project_and_source_leases_survive_real_layer_render() {
    use crate::infrastructure::{
        asset_bundle_encoding::RustAssetLayerEncoder,
        asset_inputs::{FixedAssetImage, FixedAssetInputs, FixedAssetMask},
        asset_raster::RustAssetRaster,
    };
    use sha2::{Digest, Sha256};
    let evidence = PathBuf::from(std::env::var_os("ASSET_PROJECT_EVIDENCE_DIR").unwrap());
    fs::create_dir(&evidence).unwrap();
    let bytes = fs::read(PathBuf::from(
        std::env::var_os("ASSET_PROJECT_REQUEST").unwrap(),
    ))
    .unwrap();
    let hash = |bytes: &[u8]| format!("{:x}", Sha256::digest(bytes));
    assert_eq!(
        hash(&bytes),
        std::env::var("ASSET_PROJECT_REQUEST_SHA256").unwrap()
    );
    let frame: AssetFrameRequest = serde_json::from_slice(&bytes).unwrap();
    let image_path = PathBuf::from(std::env::var_os("ASSET_INPUT_IMAGE").unwrap());
    let mask_path = PathBuf::from(std::env::var_os("ASSET_INPUT_MASK").unwrap());
    let project_root = evidence.join("projects");
    fs::create_dir(&project_root).unwrap();
    let store = AssetProjectFiles::open(&project_root).unwrap();
    let mut document = selected(
        draft("original"),
        AssetFrameInput {
            image: frame.image.clone(),
            mask: frame.mask.clone(),
            review: frame.review.clone(),
        },
    );
    document.layers[0].transform = frame.transform;
    let mut back = document.layers[0].clone();
    back.layer_id = "background".into();
    back.kind = LayerKind::Background;
    back.transform.alpha_mode = AlphaMode::Premultiplied;
    document.layers.push(back);
    let saved = store.publish(None, &document).unwrap();
    let captured = capture(&store, &request(&saved), &Control(false)).unwrap();
    drop(store);
    let inputs = FixedAssetInputs::new(
        BTreeMap::from([(
            frame.image.blob.artifact_id.clone(),
            FixedAssetImage {
                pin: frame.image.clone(),
                path: image_path.clone(),
            },
        )]),
        BTreeMap::from([(
            frame.mask.blob.artifact_id.clone(),
            FixedAssetMask {
                pin: frame.mask.clone(),
                path: mask_path.clone(),
            },
        )]),
    )
    .unwrap();
    assert_eq!(
        captured
            .capture_sources(&inputs, &Control(true))
            .err()
            .unwrap()
            .code,
        ErrorCode::Cancelled
    );
    let sources = captured.capture_sources(&inputs, &Control(false)).unwrap();
    assert_eq!(sources.manifest().originals.len(), 2);
    assert_eq!(sources.manifest().bindings.len(), 1);
    assert!(!sources.manifest().quality_approved);
    assert!(!sources.manifest().review_admitted);
    let storage = AssetProjectFiles::open(&project_root).unwrap();
    let mut hidden = saved.publication.document.clone();
    hidden.revision = DecimalU64(hidden.revision.0 + 1);
    for layer in &mut hidden.layers {
        layer.visible = false;
    }
    let hidden_saved = storage
        .publish(Some(saved.publication.publication_revision), &hidden)
        .unwrap();
    let hidden_request = request(&hidden_saved);
    assert_eq!(
        capture(&storage, &hidden_request, &Control(false))
            .err()
            .unwrap()
            .code,
        ErrorCode::NeedsInput
    );
    let storage_capture = capture_source_project(
        &storage,
        &hidden_request.project,
        MissingAssetFramePolicy::Reject,
        &Control(false),
    )
    .unwrap();
    let hidden_sources = storage_capture
        .capture_sources(&inputs, &Control(false))
        .unwrap();
    assert_eq!(hidden_sources.files().len(), 4);
    assert!(hidden_sources.manifest().bindings.len() == 1);
    drop(hidden_sources);
    drop(storage_capture);
    drop(storage);
    let foreground = captured
        .render_frame(0, &inputs, &RustAssetRaster, &Control(false))
        .unwrap();
    let background = captured
        .render_frame(1, &inputs, &RustAssetRaster, &Control(false))
        .unwrap();
    let mut builder = AssetBundleOutputBuilder::new(&captured);
    assert_eq!(
        builder
            .push(
                background
                    .encode(&RustAssetLayerEncoder, &Control(false))
                    .unwrap(),
                &Control(false)
            )
            .err()
            .unwrap()
            .code,
        ErrorCode::SourceExpired
    );
    builder
        .push(
            foreground
                .encode(&RustAssetLayerEncoder, &Control(false))
                .unwrap(),
            &Control(false),
        )
        .unwrap();
    builder
        .push(
            captured
                .render_frame(1, &inputs, &RustAssetRaster, &Control(false))
                .unwrap()
                .encode(&RustAssetLayerEncoder, &Control(false))
                .unwrap(),
            &Control(false),
        )
        .unwrap();
    let output = builder.finish(&Control(false)).unwrap();
    fn assert_owned<T: Send + 'static>(_: &T) {}
    assert_owned(&output);
    drop(inputs);
    drop(captured);
    let original = project_root
        .join("asset-original")
        .join(format!("r-1-{}.json", saved.publication_sha256));
    assert!(OpenOptions::new().write(true).open(&original).is_err());
    assert!(OpenOptions::new().write(true).open(&image_path).is_err());
    assert!(OpenOptions::new().write(true).open(&mask_path).is_err());
    assert_eq!(output.manifest().layers.len(), 2);
    assert_eq!(
        output.manifest().layers[0].raw_alpha_mode,
        AlphaMode::Straight
    );
    assert_eq!(
        output.manifest().layers[1].raw_alpha_mode,
        AlphaMode::Premultiplied
    );
    assert_eq!(
        output.manifest().layers[1].png_conversion,
        AssetPngConversion::UnassociateSrgb8WithQuantization
    );
    assert!(!output.manifest().quality_approved);
    assert_eq!(output.files().len(), 8);
    use std::io::Write;
    for file in output.files() {
        assert_eq!(hash(file.contents), file.descriptor.sha256);
        assert_eq!(file.contents.len() as u64, file.descriptor.bytes.0);
        let mut writer = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(evidence.join(&file.descriptor.name))
            .unwrap();
        writer.write_all(file.contents).unwrap();
        writer.sync_all().unwrap();
        assert_eq!(
            hash(&fs::read(evidence.join(&file.descriptor.name)).unwrap()),
            file.descriptor.sha256
        );
    }
    let manifest_sha = output.manifest_file().sha256.clone();
    fs::write(evidence.join("verification.json"),serde_json::to_vec_pretty(&serde_json::json!({"plan":output.manifest().plan,"manifest_sha256":manifest_sha,"output_files":8,"raw_premultiplied_and_straight_png_distinct":true,"out_of_order_rejected":true,"original_project_retained":true,"original_sources_retained":true,"catalog_dropped_before_result_check":true,"source_pts":null,"quality_approved":false,"jobs_verified":false,"sdk_cli_verified":false,"gui_verified":false})).unwrap()).unwrap();
    let source_root = evidence.join("original-sources");
    fs::create_dir(&source_root).unwrap();
    for file in sources.files() {
        assert_eq!(hash(file.contents), file.descriptor.sha256);
        fs::write(source_root.join(&file.descriptor.name), file.contents).unwrap();
    }
    assert_eq!(sources.files().len(), 4);
    use crate::infrastructure::asset_source_archives::AssetSourceArchiveFiles;
    let published_root = evidence.join("published-sources");
    fs::create_dir(&published_root).unwrap();
    let publisher = AssetSourceArchiveFiles::open(&published_root).unwrap();
    let published = publisher
        .publish("complete", &sources, &Control(false))
        .unwrap();
    assert_eq!(published.archive.manifest().originals.len(), 2);
    assert_eq!(
        publisher
            .publish("complete", &sources, &Control(false))
            .err()
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
    let published_png = published_root
        .join("sources-complete")
        .join(&published.archive.manifest().originals[0].name);
    assert!(OpenOptions::new().write(true).open(&published_png).is_err());
    struct CancelWhenExists(PathBuf);
    impl MutationControl for CancelWhenExists {
        fn checkpoint(&self) -> Result<(), ApiError> {
            if self.0.exists() {
                Err(ApiError::new(
                    ErrorCode::Cancelled,
                    "Injected cancellation at actual source publication boundary",
                ))
            } else {
                Ok(())
            }
        }
    }
    let early = publisher
        .publish(
            "cancelled",
            &sources,
            &CancelWhenExists(published_root.join("sources-cancelled/project.json")),
        )
        .err()
        .unwrap();
    assert_eq!(early.code, ErrorCode::Cancelled);
    assert_eq!(
        early.details["source_archive_recovery"]["marker_committed"],
        false
    );
    assert!(
        published_root
            .join("sources-cancelled/project.json")
            .is_file()
    );
    assert!(
        !published_root
            .join("sources-cancelled/sources.json")
            .exists()
    );
    let late = publisher
        .publish(
            "late",
            &sources,
            &CancelWhenExists(published_root.join("sources-late/sources.json")),
        )
        .err()
        .unwrap();
    assert_eq!(late.code, ErrorCode::Cancelled);
    assert_eq!(
        late.details["source_archive_recovery"]["marker_committed"],
        true
    );
    let recovered = publisher
        .inspect("late", &published.pin.manifest_sha256, &Control(false))
        .unwrap();
    assert_eq!(recovered.manifest().bindings.len(), 1);
    fs::write(evidence.join("source-publication.json"),serde_json::to_vec_pretty(&serde_json::json!({"complete":published.pin,"before_marker_error":early,"after_marker_error":late,"jobs_publication":false})).unwrap()).unwrap();
    drop(recovered);
    drop(published);
    drop(publisher);
    drop(output);
    assert!(OpenOptions::new().write(true).open(&image_path).is_err());
    assert!(OpenOptions::new().write(true).open(&mask_path).is_err());
    drop(sources);
    assert!(OpenOptions::new().write(true).open(&original).is_ok());
    assert!(OpenOptions::new().write(true).open(&image_path).is_ok());
    assert!(OpenOptions::new().write(true).open(&mask_path).is_ok());
    assert_eq!(
        hash(&fs::read(image_path).unwrap()),
        frame.image.blob.sha256
    );
    assert_eq!(hash(&fs::read(mask_path).unwrap()), frame.mask.blob.sha256);
}
