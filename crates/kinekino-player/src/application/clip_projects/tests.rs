use super::*;
use crate::domain::clip_document::*;
fn document() -> ClipDocument {
    serde_json::from_value(serde_json::json!({"schema_version":"1","document_id":"clip-project","revision":"1","sources":[{"media_id":"source","sha256":"a".repeat(64),"duration_us":"60000000","width":1,"height":1,"clock":{"origin_pts":"0","time_base_numerator":1,"time_base_denominator":1000,"observation_sha256":"b".repeat(64)},"tracks":[{"stream_index":0,"kind":"video","codec_id":"rawvideo"}]}],"segments":[{"segment_id":"first","media_id":"source","range":{"start_us":"0","end_us":"30000000"},"video_stream_index":0,"crop_keys":[{"source_us":"0","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"disabled"},"subtitles":null,"subject_ref":null}],"preset":{"catalog":{"artifact_id":"candidate","revision":"1","sha256":"c".repeat(64)},"width":1,"height":1,"frame_rate":{"numerator":1,"denominator":1},"container":"mp4","video_codec":"h264","audio_codec":null,"color":"sdr_bt709"},"cut_mode":"exact_reencode"})).unwrap()
}
fn stored(document: &ClipDocument) -> StoredClipProject {
    let publication = prepare_publication(None, document).unwrap();
    StoredClipProject {
        publication_sha256: publication_sha256(&publication).unwrap(),
        latest_publication_revision: publication.publication_revision,
        publication,
    }
}
#[test]
fn saved_revision_and_edit_revision_are_distinct_and_parent_is_exact() {
    let initial = document();
    let previous = stored(&initial);
    let old_bytes = serde_json::to_vec(&previous).unwrap();
    let mut editor = ClipEditor::open(initial).unwrap();
    editor
        .apply(
            DecimalU64(1),
            ClipEdit::SetCutMode {
                cut_mode: CutMode::FastKeyframe,
            },
        )
        .unwrap();
    editor.undo(DecimalU64(2)).unwrap();
    let next = prepare_publication(Some(&previous), editor.document()).unwrap();
    assert_eq!(next.publication_revision.0, 2);
    assert_eq!(next.document.revision.0, 3);
    assert_eq!(
        next.parent_publication_sha256,
        Some(previous.publication_sha256.clone())
    );
    assert_eq!(serde_json::to_vec(&previous).unwrap(), old_bytes);
    assert_ne!(
        publication_sha256(&next).unwrap(),
        previous.publication_sha256
    );
}
#[test]
fn stale_edits_source_refresh_and_foreign_identity_are_rejected() {
    let initial = document();
    let previous = stored(&initial);
    assert_eq!(
        prepare_publication(Some(&previous), &initial)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    for change_source in [true, false] {
        let mut next = initial.clone();
        next.revision = DecimalU64(2);
        if change_source {
            next.sources[0].sha256 = "d".repeat(64)
        } else {
            next.document_id = "foreign".into()
        }
        assert_eq!(
            prepare_publication(Some(&previous), &next)
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
    }
    let mut invalid_initial = initial;
    invalid_initial.revision = DecimalU64(2);
    assert_eq!(
        prepare_publication(None, &invalid_initial)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
}
#[test]
fn historical_or_corrupt_previous_header_cannot_be_used_as_current_head() {
    let initial = document();
    let mut next = initial.clone();
    next.revision = DecimalU64(2);
    let mut historical = stored(&initial);
    historical.latest_publication_revision = DecimalU64(2);
    assert_eq!(
        prepare_publication(Some(&historical), &next)
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    let mut corrupt = stored(&initial);
    corrupt.publication_sha256 = "e".repeat(64);
    assert_eq!(
        prepare_publication(Some(&corrupt), &next).unwrap_err().code,
        ErrorCode::NeedsInput
    );
}
