//! Real native file IO with synthetic clip values; no source media/encoder claim.
use super::*;
use crate::domain::clip_document::{ClipEdit, ClipEditor};
use serde_json::json;
fn document() -> ClipDocument {
    serde_json::from_value(json!({"schema_version":"1","document_id":"clip-project","revision":"1","sources":[{"media_id":"source","sha256":"a".repeat(64),"duration_us":"60000000","width":1,"height":1,"clock":{"origin_pts":"0","time_base_numerator":1,"time_base_denominator":1000,"observation_sha256":"b".repeat(64)},"tracks":[{"stream_index":0,"kind":"video","codec_id":"rawvideo"}]}],"segments":[{"segment_id":"first","media_id":"source","range":{"start_us":"0","end_us":"30000000"},"video_stream_index":0,"crop_keys":[{"source_us":"0","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"disabled"},"subtitles":null,"subject_ref":null}],"preset":{"catalog":{"artifact_id":"candidate","revision":"1","sha256":"c".repeat(64)},"width":1,"height":1,"frame_rate":{"numerator":1,"denominator":1},"container":"mp4","video_codec":"h264","audio_codec":null,"color":"sdr_bt709"},"cut_mode":"exact_reencode"})).unwrap()
}
#[test]
#[ignore = "Central native clip store allocation with retained publication/root identity evidence"]
fn native_clip_store_reopens_immutable_history_and_rejects_cas_overwrite_and_partial_files() {
    let parent = PathBuf::from(
        std::env::var_os("CLIP_PROJECT_EVIDENCE_DIR").expect("Explicit evidence root"),
    );
    assert!(parent.is_absolute());
    let evidence = parent.join(format!("clip-store-{}", std::process::id()));
    fs::create_dir(&evidence).unwrap();
    let root = evidence.join("projects");
    fs::create_dir(&root).unwrap();
    let store = ClipProjectFiles::open(&root).unwrap();
    let identity =
        crate::infrastructure::storage_directory_identity::StorageDirectoryIdentity::observe(
            store._guards.last().unwrap(),
        )
        .unwrap();
    fs::write(
        evidence.join("root-identity.json"),
        serde_json::to_vec_pretty(&identity).unwrap(),
    )
    .unwrap();
    assert!(fs::rename(&root, evidence.join("root-move-denied")).is_err());
    let initial = document();
    let created = clip_projects::create(&store, &initial).unwrap();
    assert_eq!(created.publication.publication_revision.0, 1);
    assert_eq!(
        clip_projects::create(&store, &initial).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let directory = store.directory("clip-project").unwrap();
    let first_path = directory.join(format!(
        "r-00000000000000000001-{}.json",
        created.publication_sha256
    ));
    let first_bytes = fs::read(&first_path).unwrap();
    let mut editor = ClipEditor::open(initial).unwrap();
    editor
        .apply(
            DecimalU64(1),
            ClipEdit::Duplicate {
                segment_id: "first".into(),
                new_segment_id: "second".into(),
                position: 1,
            },
        )
        .unwrap();
    editor
        .apply(
            DecimalU64(2),
            ClipEdit::Reorder {
                segment_ids: vec!["second".into(), "first".into()],
            },
        )
        .unwrap();
    let saved = clip_projects::save(&store, DecimalU64(1), editor.document()).unwrap();
    assert_eq!(saved.latest_publication_revision.0, 2);
    assert_eq!(saved.publication.document.revision.0, 3);
    assert_eq!(
        saved.publication.parent_publication_sha256.as_deref(),
        Some(created.publication_sha256.as_str())
    );
    assert_eq!(
        clip_projects::save(&store, DecimalU64(1), editor.document())
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let historical = store.load("clip-project", Some(DecimalU64(1))).unwrap();
    assert_eq!(historical.latest_publication_revision.0, 2);
    assert_eq!(historical.publication.document.revision.0, 1);
    assert_eq!(
        clip_projects::prepare_publication(Some(&historical), editor.document())
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    assert_eq!(
        store
            .load("clip-project", Some(DecimalU64(3)))
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    {
        let _writer = paths::lock(&store.root.join("clip-writer.lock")).unwrap();
        assert_eq!(
            store.load("clip-project", None).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        assert!(
            crate::infrastructure::native_immutable_publication::publish(
                &directory,
                first_path.file_name().unwrap().to_str().unwrap(),
                b"attempted overwrite retained as pending"
            )
            .is_err()
        );
        assert_eq!(fs::read(&first_path).unwrap(), first_bytes);
    }
    fs::write(
        directory.join(".pending-interrupted.json"),
        b"partial never published",
    )
    .unwrap();
    let expected = serde_json::to_vec(&saved).unwrap();
    assert_eq!(
        serde_json::to_vec(&store.load("clip-project", None).unwrap()).unwrap(),
        expected
    );
    fs::write(
        evidence.join("saved.json"),
        serde_json::to_vec_pretty(&saved).unwrap(),
    )
    .unwrap();
    drop(store);
    let store = ClipProjectFiles::open(&root).unwrap();
    assert_eq!(
        serde_json::to_vec(&store.load("clip-project", None).unwrap()).unwrap(),
        expected
    );
    let foreign = store.directory("foreign").unwrap();
    fs::create_dir(&foreign).unwrap();
    fs::copy(&first_path, foreign.join(first_path.file_name().unwrap())).unwrap();
    assert_eq!(
        store.load("foreign", None).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    fs::write(
        directory.join(format!("r-00000000000000000003-{}.json", "f".repeat(64))),
        b"partial final record",
    )
    .unwrap();
    assert_eq!(
        store.load("clip-project", None).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    assert_eq!(fs::read(first_path).unwrap(), first_bytes);
    fs::write(evidence.join("verification.json"), serde_json::to_vec_pretty(&json!({"created":true,"saved_publication_revision":"2","edit_revision":"3","historical_read":true,"root_move_denied":true,"writer_exclusive":true,"no_overwrite":true,"pending_preserved":true,"partial_final_rejected":true,"foreign_project_identity_rejected":true,"source_media_validated":false,"encoder_executed":false})).unwrap()).unwrap();
}
