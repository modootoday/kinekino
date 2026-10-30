use super::*;
use crate::{
    application::clip_render::subtitles::frames::active_subtitle_runs,
    infrastructure::clip_sources::{ClipContainer, FfmpegClipSources, FixedClipSource},
};
use std::time::Duration;

#[test]
#[ignore = "Pinned existing media/probe/fonts, fresh evidence, CPU active cues; no GUI/inference"]
fn original_source_pts_bind_overlapping_cues_to_one_measured_cpu_box() {
    let root =
        PathBuf::from(std::env::var_os("CLIP_SUBTITLE_RUN_EVIDENCE_DIR").expect("Fresh evidence"));
    fs::create_dir(&root).unwrap();
    let style_path = PathBuf::from(std::env::var_os("CLIP_SUBTITLE_STYLE").expect("Pinned style"));
    let style_bytes = fs::read(&style_path).unwrap();
    assert_eq!(
        hash(&style_bytes),
        std::env::var("CLIP_SUBTITLE_STYLE_SHA256").unwrap()
    );
    let mut style: Value = serde_json::from_slice(&style_bytes).unwrap();
    style["style_id"] = json!("raster-style");
    style["font_px"] = json!(24);
    style["line_height_px"] = json!(32);
    let port = FfmpegClipSources::new(
        BTreeMap::from([(
            "media".into(),
            FixedClipSource {
                path: PathBuf::from(
                    std::env::var_os("CLIP_RENDER_SOURCE").expect("Original media"),
                ),
                container: ClipContainer::Matroska,
            },
        )]),
        &PathBuf::from(std::env::var_os("CLIP_RENDER_PROBE").expect("Pinned probe")),
        &std::env::var("CLIP_RENDER_PROBE_SHA256").unwrap(),
        crate::domain::frame_capture::MAX_MEDIA_BYTES,
        Duration::from_secs(120),
    )
    .unwrap();
    let source = port.inspect_source("media", &Control).unwrap();
    assert_eq!(
        source.facts().source.sha256,
        "f12c070e295b38cfc94ebd61ac3357c3bac82d1015985f1e8da93a4c6496c46d"
    );
    let document = json!({"schema_version":"clip-subtitle-document/1","media":source.facts().source,"time_basis":"media_relative_us","origin":"explicit_manual_snapshot","cues":{"source_id":"captions","source_revision":"1","source_sha256":"c".repeat(64),"cues":[{"cue_id":"first","text":"첫 줄 <>& あ","start_us":"15000000","end_us":"15500000"},{"cue_id":"second","text":"겹친 둘째 줄","start_us":"15250000","end_us":"16000000"}]}});
    let doc_path = root.join("document.json");
    let saved_style = root.join("style.json");
    let doc_sha = write(&doc_path, &document);
    let style_sha = write(&saved_style, &style);
    let mut segment: ClipSegment = serde_json::from_value(json!({"segment_id":"one","media_id":"media","range":{"start_us":"15000000","end_us":"16000000"},"video_stream_index":source.facts().reference_stream_index,"crop_keys":[{"source_us":"15000000","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"disabled"},"subtitles":{"document":{"artifact":{"artifact_id":"captions","revision":"1","sha256":doc_sha},"source_sha256":source.facts().source.sha256},"cue_ids":["second","first"],"style":{"artifact_id":"raster-style","revision":"1","sha256":style_sha}},"subject_ref":null})).unwrap();
    let files = SubtitleAssetFiles::open(
        BTreeMap::from([("captions".into(), doc_path)]),
        BTreeMap::from([("raster-style".into(), saved_style)]),
    )
    .unwrap();
    let owner = prepare_subtitles(&source.facts().source, &segment, &files, &Control)
        .unwrap()
        .unwrap();
    let decoded = source
        .observe_segment(&segment, Duration::from_secs(30), &Control)
        .unwrap();
    fs::write(
        root.join("source.json"),
        serde_json::to_vec_pretty(source.facts()).unwrap(),
    )
    .unwrap();
    fs::write(root.join("frames.raw.json"), decoded.raw_frames()).unwrap();
    fs::write(
        root.join("boundaries.json"),
        serde_json::to_vec_pretty(decoded.observation()).unwrap(),
    )
    .unwrap();
    let spans = decoded.subtitle_frames(&segment, &owner, &Control).unwrap();
    assert_eq!(decoded.observation().boundaries.included_frames.0, 24);
    assert_eq!(
        spans
            .iter()
            .map(|s| (s.first_frame.0, s.end_frame.0))
            .collect::<Vec<_>>(),
        vec![(0, 12), (6, 24)]
    );
    let runs = active_subtitle_runs(&spans, 24, &Control).unwrap();
    assert_eq!(
        runs.iter()
            .map(|r| (r.first_frame.0, r.end_frame.0, r.cue_indices().to_vec()))
            .collect::<Vec<_>>(),
        vec![(0, 6, vec![0]), (6, 12, vec![0, 1]), (12, 24, vec![1])]
    );
    fs::write(
        root.join("spans.json"),
        serde_json::to_vec_pretty(&spans).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("runs.json"),
        serde_json::to_vec_pretty(&runs).unwrap(),
    )
    .unwrap();
    for (index, run) in runs.iter().enumerate() {
        let image = render_cues(&owner, run.cue_indices(), 640, 360, &Control).unwrap();
        assert_eq!(
            image.facts().layout.content.height,
            if index == 1 { 64.0 } else { 32.0 }
        );
        if index == 1 {
            assert_eq!(image.facts().cue_ids, vec!["first", "second"]);
            assert!(image.html.contains("첫 줄 &lt;&gt;&amp; あ\n겹친 둘째 줄"));
            assert_eq!(
                image.facts().cue_text_sha256,
                vec![
                    hash("첫 줄 <>& あ".as_bytes()),
                    hash("겹친 둘째 줄".as_bytes())
                ]
            );
        }
        fs::write(root.join(format!("run-{index}.png")), image.png()).unwrap();
        fs::write(
            root.join(format!("run-{index}.json")),
            serde_json::to_vec_pretty(image.facts()).unwrap(),
        )
        .unwrap();
    }
    assert_eq!(
        render_cues(&owner, &[1, 0], 640, 360, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::InvalidArgument
    );
    let mut stale = segment.clone();
    stale.range.end_us.0 -= 1;
    assert_eq!(
        decoded
            .subtitle_frames(&stale, &owner, &Control)
            .unwrap_err()
            .code,
        ErrorCode::SourceExpired
    );
    drop(owner);
    style["max_lines"] = json!(1);
    let narrow_path = root.join("style-one-line.json");
    segment.subtitles.as_mut().unwrap().style.sha256 = write(&narrow_path, &style);
    let narrow_files = SubtitleAssetFiles::open(
        BTreeMap::from([("captions".into(), root.join("document.json"))]),
        BTreeMap::from([("raster-style".into(), narrow_path)]),
    )
    .unwrap();
    let narrow = prepare_subtitles(&source.facts().source, &segment, &narrow_files, &Control)
        .unwrap()
        .unwrap();
    assert_eq!(
        render_cues(&narrow, &[0, 1], 640, 360, &Control)
            .err()
            .unwrap()
            .code,
        ErrorCode::NeedsInput
    );
    assert_eq!(hash(&fs::read(style_path).unwrap()), hash(&style_bytes));
    fs::write(root.join("verification.json"),serde_json::to_vec_pretty(&json!({"original_source_sha256":source.facts().source.sha256,"decoded_original_frames":24,"original_owner_mapping_verified":true,"active_sets_and_two_line_cpu_render_verified":true,"aggregate_overflow_rejected":true,"caption_text_is_manual_fixture":true,"video_overlay_or_jobs_or_cli_verified":false,"quality_approved":false,"backend_glyph_choice_observed":false})).unwrap()).unwrap();
}
