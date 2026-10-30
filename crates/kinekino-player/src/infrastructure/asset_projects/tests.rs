use super::*;
use crate::application::api::AssetClient;
use crate::{application::asset_projects, domain::asset_generation::*};
use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
#[test]
fn typed_sdk_uses_original_store_and_reports_storage_only_capabilities() {
    use crate::application::api::{PlayerSdk, Request, RequestContext};
    let root = root();
    let sdk =
        PlayerSdk::default().with_asset_project_store(AssetProjectFiles::open(&root).unwrap());
    let first = sdk
        .create_asset_project(RequestContext::default(), draft("typed"))
        .unwrap();
    let mut edited = first.publication.document.clone();
    edited.revision = DecimalU64(2);
    edited.layers[0].transform.pivot.x = 0;
    let context = RequestContext {
        expected_revision: Some("1".into()),
        ..RequestContext::default()
    };
    let second = sdk
        .save_asset_project(context.clone(), edited.clone())
        .unwrap();
    assert_eq!(
        sdk.save_asset_project(context, edited).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let old = sdk
        .inspect_asset_project(
            RequestContext::default(),
            asset_projects::InspectAssetProject {
                document_id: "typed".into(),
                publication_revision: Some(DecimalU64(1)),
            },
        )
        .unwrap();
    assert_eq!(old.publication, first.publication);
    assert_eq!(
        old.latest_publication_revision,
        second.latest_publication_revision
    );
    let caps = sdk.execute(Request::new("capabilities")).data.unwrap();
    assert_eq!(caps["asset_projects"]["backend_available"], true);
    assert_eq!(caps["asset_projects"]["source_admitted"], false);
    assert_eq!(caps["asset_projects"]["quality_approved"], false);
    let mut request = Request::new("schema");
    request.payload = serde_json::json!({"command":"asset_projects.create"});
    let schema = sdk.execute(request).data.unwrap();
    assert_eq!(
        schema["request"]["properties"]["payload"]["properties"]["document"]["properties"]["schema_version"]
            ["const"],
        "asset-project/1"
    );
    let unavailable = PlayerSdk::default()
        .inspect_asset_project(
            RequestContext::default(),
            asset_projects::InspectAssetProject {
                document_id: "typed".into(),
                publication_revision: None,
            },
        )
        .unwrap_err();
    assert_eq!(unavailable.code, ErrorCode::Unsupported);
}
#[test]
fn retained_history_denies_write_until_publication_scope_ends() {
    let root = root();
    let store = AssetProjectFiles::open(&root).unwrap();
    let project = store.publish(None, &draft("retained")).unwrap();
    let directory = store.directory("retained").unwrap();
    let path = directory.join(format!("r-1-{}.json", project.publication_sha256));
    let retained = store.scan_retained("retained", &directory, None).unwrap();
    assert_eq!(
        OpenOptions::new().write(true).open(&path).is_err(),
        crate::infrastructure::os_policy::has_mandatory_share_modes()
    );
    drop(retained);
    assert!(OpenOptions::new().write(true).open(&path).is_ok());
}
fn root() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "asset-project-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir(&path).unwrap();
    path
}
fn transform() -> AssetTransform {
    AssetTransform {
        trim: false,
        mask_threshold: 0,
        feather_radius: 0,
        scale: PixelScale {
            numerator: 1,
            denominator: 1,
        },
        pivot: NormalizedPivot {
            x: 500000,
            y: 1000000,
        },
        alpha_mode: AlphaMode::Straight,
        color: AssetColor::Srgb,
    }
}
fn draft(id: &str) -> AssetDocument {
    AssetDocument {
        schema_version: "asset-project/1".into(),
        document_id: id.into(),
        revision: DecimalU64(1),
        sources: BTreeMap::new(),
        order_basis: AssetOrderBasis::SingleFrame,
        frames: Vec::new(),
        layers: vec![AssetLayer {
            layer_id: "foreground".into(),
            kind: LayerKind::Foreground,
            visible: true,
            locked: false,
            transform: transform(),
        }],
    }
}
fn input() -> AssetFrameInput {
    let image = AssetImagePin {
        blob: AssetBlobPin {
            artifact_id: "source".into(),
            revision: DecimalU64(1),
            sha256: "a".repeat(64),
        },
        width: 2,
        height: 1,
        origin: AssetFrameOrigin::StaticPng,
        declared_color: AssetColor::Srgb,
    };
    let mask = AssetMaskPin {
        blob: AssetBlobPin {
            artifact_id: "mask".into(),
            revision: DecimalU64(1),
            sha256: "b".repeat(64),
        },
        source: image.clone(),
        encoding: AssetMaskEncoding::Gray8Coverage,
        origin: AssetMaskOrigin::ImportedUnverified,
    };
    AssetFrameInput {
        image,
        review: AssetMaskReview::UnreviewedAcknowledged {
            mask: mask.blob.clone(),
        },
        mask,
    }
}
fn selected(mut document: AssetDocument, input: AssetFrameInput) -> AssetDocument {
    document.sources.insert(
        "frame-001".into(),
        AssetFrameSlot {
            position: AssetFramePosition::StaticPng,
            selected_image: Some(input.image.clone()),
        },
    );
    document.frames = vec![AssetFrameEntry::Available {
        frame_id: "frame-001".into(),
        input: Box::new(input),
    }];
    document
}
#[test]
fn actual_project_create_save_history_and_reopen_keep_original_publication() {
    let root = root();
    let store = AssetProjectFiles::open(&root).unwrap();
    let mut document = draft("project");
    let first = asset_projects::create(&store, &document).unwrap();
    document = selected(document, input());
    document.revision.0 = 2;
    let second = asset_projects::save(&store, DecimalU64(1), &document).unwrap();
    assert_eq!(
        second.publication.parent_publication_sha256,
        Some(first.publication_sha256.clone())
    );
    assert_eq!(second.latest_publication_revision.0, 2);
    drop(store);
    let reopened = AssetProjectFiles::open(&root).unwrap();
    assert_eq!(reopened.load("project", None).unwrap(), second);
    let original = reopened.load("project", Some(DecimalU64(1))).unwrap();
    assert_eq!(original.publication, first.publication);
    assert_eq!(original.publication_sha256, first.publication_sha256);
    assert_eq!(original.latest_publication_revision.0, 2);
    assert_eq!(
        second
            .publication
            .document
            .frame_request("frame-001", "foreground")
            .unwrap(),
        input().request(LayerKind::Foreground, transform()).unwrap()
    );
}
#[test]
fn stale_cas_nonadvancing_edits_and_duplicate_create_never_add_a_publication() {
    let root = root();
    let store = AssetProjectFiles::open(&root).unwrap();
    let mut document = draft("project");
    asset_projects::create(&store, &document).unwrap();
    assert_eq!(
        asset_projects::create(&store, &document)
            .err()
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        asset_projects::save(&store, DecimalU64(1), &document)
            .err()
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
    document.revision.0 = 2;
    asset_projects::save(&store, DecimalU64(1), &document).unwrap();
    document.revision.0 = 3;
    assert_eq!(
        asset_projects::save(&store, DecimalU64(1), &document)
            .err()
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        store
            .load("project", None)
            .unwrap()
            .latest_publication_revision
            .0,
        2
    );
}
#[test]
fn removed_frame_identity_cannot_be_rebound_or_erase_its_source_slot() {
    let root = root();
    let store = AssetProjectFiles::open(&root).unwrap();
    let mut document = selected(draft("project"), input());
    asset_projects::create(&store, &document).unwrap();
    document.frames.clear();
    document.revision.0 = 2;
    asset_projects::save(&store, DecimalU64(1), &document).unwrap();
    let mut rebound = input();
    rebound.image.blob.sha256 = "c".repeat(64);
    rebound.mask.source = rebound.image.clone();
    let mut bad = selected(document.clone(), rebound);
    bad.revision.0 = 3;
    assert_eq!(
        asset_projects::save(&store, DecimalU64(2), &bad)
            .err()
            .unwrap()
            .code,
        ErrorCode::SourceExpired
    );
    document.revision.0 = 3;
    document.sources.clear();
    assert_eq!(
        asset_projects::save(&store, DecimalU64(2), &document)
            .err()
            .unwrap()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        store
            .load("project", None)
            .unwrap()
            .latest_publication_revision
            .0,
        2
    );
}
#[test]
fn missing_frames_and_source_clock_order_preserve_integer_ticks_and_explicit_gaps() {
    let mut document = draft("sequence");
    document.order_basis = AssetOrderBasis::SourceVideoClock;
    for (id, pts) in [("one", "9007199254740993"), ("two", "9007199254740994")] {
        document.sources.insert(
            id.into(),
            AssetFrameSlot {
                position: AssetFramePosition::Video {
                    expected: AssetVideoPosition {
                        source_sha256: "a".repeat(64),
                        stream_index: 2,
                        pts: pts.into(),
                        clock: crate::domain::clip_document::SourceClock {
                            origin_pts: "-1000".into(),
                            time_base_numerator: 1,
                            time_base_denominator: 90000,
                            observation_sha256: "b".repeat(64),
                        },
                        rotation_degrees: 90,
                    },
                },
                selected_image: None,
            },
        );
        document.frames.push(AssetFrameEntry::Missing {
            frame_id: id.into(),
            reason: "Expected sample unavailable".into(),
        });
    }
    document.validate().unwrap();
    assert_eq!(
        document.frame_request("one", "foreground").unwrap_err(),
        AssetFrameSelectionError::MissingFrame
    );
    let encoded = serde_json::to_vec(&document).unwrap();
    let decoded: AssetDocument = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded, document);
    document.frames.reverse();
    assert!(document.validate().is_err());
    document.order_basis = AssetOrderBasis::DeclaredFrameOrder;
    document.validate().unwrap();
}
#[test]
fn incomplete_stage_is_preserved_and_does_not_hide_committed_head_or_manual_retry() {
    let root = root();
    let store = AssetProjectFiles::open(&root).unwrap();
    let mut document = draft("project");
    let first = asset_projects::create(&store, &document).unwrap();
    let stage = root.join("asset-project/.pending-1-0.json");
    fs::write(&stage, b"preserved incomplete bytes").unwrap();
    let loaded = store.load("project", None).unwrap();
    assert_eq!(loaded.publication_sha256, first.publication_sha256);
    assert_eq!(loaded.pending_publication_count.0, 1);
    document.revision.0 = 2;
    let second = asset_projects::save(&store, DecimalU64(1), &document).unwrap();
    assert_eq!(second.pending_publication_count.0, 1);
    assert_eq!(fs::read(&stage).unwrap(), b"preserved incomplete bytes");
    assert_eq!(store.load("project", None).unwrap(), second);
}
#[test]
fn incomplete_create_and_conflicting_committed_revision_require_inspection_without_overwrite() {
    let root = root();
    let store = AssetProjectFiles::open(&root).unwrap();
    fs::create_dir(root.join("asset-empty")).unwrap();
    fs::write(root.join("asset-empty/.pending-1-0.json"), b"partial").unwrap();
    assert_eq!(
        store.load("empty", None).err().unwrap().code,
        ErrorCode::NeedsInput
    );
    assert_eq!(
        asset_projects::create(&store, &draft("empty"))
            .err()
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
    let first = asset_projects::create(&store, &draft("project")).unwrap();
    let bytes = serde_json::to_vec(&first.publication).unwrap();
    fs::write(
        root.join(format!("asset-project/r-1-{}.json", "c".repeat(64))),
        &bytes,
    )
    .unwrap();
    assert_eq!(
        store.load("project", None).err().unwrap().code,
        ErrorCode::NeedsInput
    );
    assert_eq!(
        fs::read(root.join("asset-empty/.pending-1-0.json")).unwrap(),
        b"partial"
    );
}
#[test]
fn canonical_bytes_and_actual_writer_exclusion_are_checked() {
    let root = root();
    let store = AssetProjectFiles::open(&root).unwrap();
    let first = asset_projects::create(&store, &draft("project")).unwrap();
    let lock = paths::lock(&root.join("asset-projects-writer.lock")).unwrap();
    assert_eq!(
        asset_projects::create(&store, &draft("blocked"))
            .err()
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
    assert!(!root.join("asset-blocked").exists());
    drop(lock);
    let path = root.join(format!(
        "asset-project/r-1-{}.json",
        first.publication_sha256
    ));
    let mut bytes = fs::read(&path).unwrap();
    fs::write(root.join("original-publication.bytes"), &bytes).unwrap();
    bytes.push(b' ');
    fs::write(&path, &bytes).unwrap();
    assert_eq!(
        store.load("project", None).err().unwrap().code,
        ErrorCode::NeedsInput
    );
}
#[test]
fn hidden_layer_is_not_silently_rendered_and_locked_layer_can_still_export() {
    let mut document = selected(draft("project"), input());
    document.layers[0].locked = true;
    assert!(document.frame_request("frame-001", "foreground").is_ok());
    document.layers[0].visible = false;
    assert_eq!(
        document
            .frame_request("frame-001", "foreground")
            .unwrap_err(),
        AssetFrameSelectionError::HiddenLayer
    );
}
mod bundle;
mod existing;
