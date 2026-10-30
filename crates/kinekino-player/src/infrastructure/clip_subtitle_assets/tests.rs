use super::*;
use crate::{
    application::clip_render::subtitles::prepare_subtitles,
    domain::clip_document::{ClipSegment, ClipSourcePin},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
struct Cancelled;
impl MutationControl for Cancelled {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Err(error(ErrorCode::Cancelled, "Fixture cancelled"))
    }
}
fn write(path: &Path, value: &Value) -> String {
    let bytes = serde_json::to_vec_pretty(value).unwrap();
    fs::write(path, &bytes).unwrap();
    format!("{:x}", Sha256::digest(bytes))
}
fn files(document: &Path, style: &Path) -> SubtitleAssetFiles {
    SubtitleAssetFiles::open(
        BTreeMap::from([("captions".into(), document.into())]),
        BTreeMap::from([("subtitle-style".into(), style.into())]),
    )
    .unwrap()
}
#[test]
#[ignore = "Fresh evidence; native original file guards and actual common Meari fonts, no renderer/GUI/models"]
fn original_document_style_font_pins_and_half_open_selected_cues_are_retained() {
    let root = PathBuf::from(
        std::env::var_os("CLIP_SUBTITLE_ASSET_EVIDENCE_DIR").expect("Fresh evidence"),
    );
    fs::create_dir(&root).unwrap();
    let source: ClipSourcePin = serde_json::from_value(json!({"media_id":"media","sha256":"a".repeat(64),"duration_us":"60000000","width":1280,"height":720,"clock":{"origin_pts":"-9000","time_base_numerator":1,"time_base_denominator":90000,"observation_sha256":"b".repeat(64)},"tracks":[{"stream_index":0,"kind":"video","codec_id":"h264"}]})).unwrap();
    let document = json!({"schema_version":"clip-subtitle-document/1","media":source,"time_basis":"media_relative_us","origin":"explicit_manual_snapshot","cues":{"source_id":"captions","source_revision":"1","source_sha256":"c".repeat(64),"cues":[{"cue_id":"first","text":"원문 <>& 보존","start_us":"9000000","end_us":"10500000"},{"cue_id":"second","text":"겹친 원문","start_us":"10250000","end_us":"11250000"},{"cue_id":"after","text":"끝 경계 밖","start_us":"11000000","end_us":"12000000"}]}});
    let style = json!({"schema_version":"clip-subtitle-style/1","style_id":"subtitle-style","revision":"1","fonts":[{"asset_path":"fonts/noto/NotoSansKR[wght].ttf","sha256":"194018e6b2b293a7964f037b25c0249ce1418bc9ab3c971060a03aa57861e252"},{"asset_path":"fonts/noto/NotoSansJP[wght].ttf","sha256":"c2f3b4d463500a2ddcd3849cded1fceeb9fd6d1c32e6cbecd568453ba50fc68f"}],"font_px":40,"line_height_px":56,"box_normalized":{"left":100000,"top":750000,"width":800000,"height":200000},"foreground_rgba":[255,255,255,255],"background_rgba":[0,0,0,160],"alignment":"center","max_lines":3});
    let document_path = root.join("document.json");
    let style_path = root.join("style.json");
    let doc_sha = write(&document_path, &document);
    let style_sha = write(&style_path, &style);
    let mut segment: ClipSegment = serde_json::from_value(json!({"segment_id":"one","media_id":"media","range":{"start_us":"10000000","end_us":"11000000"},"video_stream_index":0,"crop_keys":[{"source_us":"10000000","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"disabled"},"subtitles":{"document":{"artifact":{"artifact_id":"captions","revision":"1","sha256":doc_sha},"source_sha256":source.sha256},"cue_ids":["after","second","first"],"style":{"artifact_id":"subtitle-style","revision":"1","sha256":style_sha}},"subject_ref":null})).unwrap();
    let prepared = prepare_subtitles(
        &source,
        &segment,
        &files(&document_path, &style_path),
        &Control,
    )
    .unwrap()
    .unwrap();
    assert_eq!(prepared.cues().len(), 2);
    assert_eq!(prepared.cues()[0].cue_id, "first");
    assert_eq!(prepared.cues()[0].text, "원문 <>& 보존");
    assert_eq!(
        (
            prepared.cues()[0].output_start_us.0,
            prepared.cues()[0].output_end_us.0
        ),
        (0, 500000)
    );
    assert_eq!(
        (
            prepared.cues()[1].output_start_us.0,
            prepared.cues()[1].output_end_us.0
        ),
        (250000, 1000000)
    );
    assert_eq!(prepared.omitted_outside_clip(), &["after"]);
    assert_eq!(prepared.style().fonts().len(), 2);
    assert!(
        prepared
            .style()
            .fonts()
            .iter()
            .all(|font| font.bytes.0 > 0 && font.license.contains("OFL"))
    );
    assert!(
        fs::OpenOptions::new()
            .write(true)
            .open(&document_path)
            .is_err()
    );
    assert!(
        fs::OpenOptions::new()
            .write(true)
            .open(&style_path)
            .is_err()
    );
    assert!(fs::rename(&document_path, root.join("renamed-document.json")).is_err());
    fs::write(root.join("admitted.json"), serde_json::to_vec_pretty(&json!({"cues":prepared.cues(),"omitted_outside_clip":prepared.omitted_outside_clip(),"fonts":prepared.style().fonts(),"media_is_fixture":true,"text_source_provenance_verified":false,"glyph_shaping_verified":false,"rendered":false,"quality_approved":false})).unwrap()).unwrap();
    drop(prepared);
    let error_of = |segment: &ClipSegment, document: &Path, style: &Path| {
        prepare_subtitles(&source, segment, &files(document, style), &Control)
            .err()
            .expect("Expected explicit rejection")
            .code
    };
    let original = segment.clone();
    segment.subtitles.as_mut().unwrap().document.artifact.sha256 = "0".repeat(64);
    assert_eq!(
        error_of(&segment, &document_path, &style_path),
        ErrorCode::SourceExpired
    );
    segment = original.clone();
    segment.subtitles.as_mut().unwrap().cue_ids = vec!["absent".into()];
    assert_eq!(
        error_of(&segment, &document_path, &style_path),
        ErrorCode::SourceExpired
    );
    segment = original.clone();
    let mut stale = document.clone();
    stale["media"]["clock"]["observation_sha256"] = json!("d".repeat(64));
    let stale_path = root.join("document-stale-clock.json");
    segment.subtitles.as_mut().unwrap().document.artifact.sha256 = write(&stale_path, &stale);
    assert_eq!(
        error_of(&segment, &stale_path, &style_path),
        ErrorCode::SourceExpired
    );
    segment = original.clone();
    let mut missing = document.clone();
    missing["cues"]["cues"][0]["start_us"] = Value::Null;
    missing["cues"]["cues"][0]["end_us"] = Value::Null;
    let missing_path = root.join("document-missing-time.json");
    segment.subtitles.as_mut().unwrap().document.artifact.sha256 = write(&missing_path, &missing);
    assert_eq!(
        error_of(&segment, &missing_path, &style_path),
        ErrorCode::NeedsInput
    );
    segment = original.clone();
    let mut wrong_font = style.clone();
    wrong_font["fonts"][0]["sha256"] = json!("0".repeat(64));
    let wrong_font_path = root.join("style-wrong-font.json");
    segment.subtitles.as_mut().unwrap().style.sha256 = write(&wrong_font_path, &wrong_font);
    assert_eq!(
        error_of(&segment, &document_path, &wrong_font_path),
        ErrorCode::SourceExpired
    );
    assert_eq!(
        prepare_subtitles(
            &source,
            &original,
            &files(&document_path, &style_path),
            &Cancelled
        )
        .err()
        .unwrap()
        .code,
        ErrorCode::Cancelled
    );
    fs::write(root.join("verification.json"), serde_json::to_vec_pretty(&json!({"native_document_style_guards_verified":true,"common_font_bytes_and_license_verified":true,"half_open_clip_and_overlap_preserved":true,"stale_clock_wrong_pin_missing_cue_missing_time_wrong_font_rejected":true,"cancel_before_io_rejected":true,"media_is_fixture":true,"renderer_or_jobs_or_cli_verified":false,"glyph_shaping_verified":false,"quality_approved":false})).unwrap()).unwrap();
}
