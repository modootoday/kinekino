use super::*;
mod runs;
use crate::{
    application::clip_render::subtitles::prepare_subtitles, domain::clip_document::ClipSegment,
    infrastructure::clip_subtitle_assets::SubtitleAssetFiles,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
fn write(path: &Path, value: &Value) -> String {
    let bytes = serde_json::to_vec_pretty(value).unwrap();
    fs::write(path, &bytes).unwrap();
    hash(&bytes)
}
fn prepare(root: &Path, document: &Value, style: &Value, suffix: &str) -> Admitted {
    let doc_path = root.join(format!("document-{suffix}.json"));
    let style_path = root.join(format!("style-{suffix}.json"));
    let doc_sha = write(&doc_path, document);
    let style_sha = write(&style_path, style);
    let source = serde_json::from_value(document["media"].clone()).unwrap();
    let segment: ClipSegment = serde_json::from_value(json!({"segment_id":"one","media_id":"media","range":{"start_us":"10000000","end_us":"11000000"},"video_stream_index":0,"crop_keys":[{"source_us":"10000000","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"disabled"},"subtitles":{"document":{"artifact":{"artifact_id":"captions","revision":"1","sha256":doc_sha},"source_sha256":document["media"]["sha256"]},"cue_ids":["first"],"style":{"artifact_id":"raster-style","revision":"1","sha256":style_sha}},"subject_ref":null})).unwrap();
    let files = SubtitleAssetFiles::open(
        BTreeMap::from([("captions".into(), doc_path)]),
        BTreeMap::from([("raster-style".into(), style_path)]),
    )
    .unwrap();
    prepare_subtitles(&source, &segment, &files, &Control)
        .unwrap()
        .unwrap()
}
#[test]
#[ignore = "Pinned existing fixture records, fresh CPU evidence, common fonts; no window/GUI/models"]
fn common_font_cpu_image_checks_transparency_layout_cmap_and_png_roundtrip() {
    let root = PathBuf::from(
        std::env::var_os("CLIP_SUBTITLE_RASTER_EVIDENCE_DIR").expect("Fresh evidence"),
    );
    fs::create_dir(&root).unwrap();
    let doc_path = PathBuf::from(
        std::env::var_os("CLIP_SUBTITLE_DOCUMENT").expect("Existing admitted fixture"),
    );
    let style_path =
        PathBuf::from(std::env::var_os("CLIP_SUBTITLE_STYLE").expect("Existing admitted style"));
    let doc_bytes = fs::read(&doc_path).unwrap();
    let style_bytes = fs::read(&style_path).unwrap();
    assert_eq!(
        hash(&doc_bytes),
        std::env::var("CLIP_SUBTITLE_DOCUMENT_SHA256").unwrap()
    );
    assert_eq!(
        hash(&style_bytes),
        std::env::var("CLIP_SUBTITLE_STYLE_SHA256").unwrap()
    );
    let mut document: Value = serde_json::from_slice(&doc_bytes).unwrap();
    document["cues"]["cues"][0]["text"] = json!("원문 <>& あ");
    let mut style: Value = serde_json::from_slice(&style_bytes).unwrap();
    style["style_id"] = json!("raster-style");
    style["font_px"] = json!(24);
    style["line_height_px"] = json!(32);
    style["background_rgba"] = json!([200, 20, 10, 128]);
    let admitted = prepare(&root, &document, &style, "positive");
    let image = render_cue(&admitted, 0, 640, 360, &Control).unwrap();
    fs::write(root.join("subtitle.png"), image.png()).unwrap();
    fs::write(root.join("subtitle.rgba"), image.rgba()).unwrap();
    fs::write(root.join("subtitle.html"), &image.html).unwrap();
    fs::write(
        root.join("facts.json"),
        serde_json::to_vec_pretty(image.facts()).unwrap(),
    )
    .unwrap();
    assert_eq!(&image.rgba()[..4], &[0, 0, 0, 0]);
    let b = image.facts().layout.expected;
    let pixel = ((b.y + b.height - 3) * 640 + b.x + 3) as usize * 4;
    let rgba = &image.rgba()[pixel..pixel + 4];
    assert!(
        (i32::from(rgba[0]) - 200).abs() <= 2
            && (i32::from(rgba[1]) - 20).abs() <= 2
            && (i32::from(rgba[2]) - 10).abs() <= 2
            && rgba[3] == 128,
        "{rgba:?}"
    );
    assert!(
        image
            .rgba()
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| p[0] > 220 && p[1] > 220 && p[2] > 220 && p[3] > 220)
    );
    assert!(image.html.contains("&lt;&gt;&amp;"));
    let mut reader = png::Decoder::new(Cursor::new(image.png()))
        .read_info()
        .unwrap();
    assert_eq!((reader.info().width, reader.info().height), (640, 360));
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let frame = reader.next_frame(&mut pixels).unwrap();
    assert_eq!(&pixels[..frame.buffer_size()], image.rgba());
    assert!(image.facts().nominal_cmap_checked);
    assert!(!image.facts().backend_glyph_choice_observed);
    drop(image);
    drop(admitted);
    let mut absent = document.clone();
    absent["cues"]["cues"][0]["text"] = json!("\u{10ffff}");
    let absent = prepare(&root, &absent, &style, "missing-cmap");
    assert_eq!(
        render_cue(&absent, 0, 640, 360, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::NeedsInput
    );
    let mut overflow = style.clone();
    overflow["box_normalized"]["width"] = json!(100000);
    overflow["box_normalized"]["height"] = json!(250000);
    overflow["max_lines"] = json!(1);
    let overflow = prepare(&root, &document, &overflow, "overflow");
    assert_eq!(
        render_cue(&overflow, 0, 640, 360, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::NeedsInput
    );
    assert_eq!(hash(&fs::read(&doc_path).unwrap()), hash(&doc_bytes));
    assert_eq!(hash(&fs::read(&style_path).unwrap()), hash(&style_bytes));
    fs::write(root.join("verification.json"), serde_json::to_vec_pretty(&json!({"cpu_text_pixels_observed":true,"straight_alpha_roundtrip_verified":true,"nominal_cmap_checked":true,"missing_cmap_and_measured_overflow_rejected":true,"original_records_preserved":true,"backend_glyph_choice_observed":false,"ink_overflow_measured":false,"video_or_jobs_or_cli_verified":false,"media_is_fixture":true,"quality_approved":false})).unwrap()).unwrap();
}
