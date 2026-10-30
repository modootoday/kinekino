use super::*;
use crate::{
    application::{
        clip_projects,
        clip_render::{
            prepare_encoder, prepare_media,
            subtitles::{ClipSubtitleStyleLease, prepare_subtitles},
        },
    },
    domain::{api_values::DecimalU64, clip_document::ClipDocument},
    infrastructure::{
        clip_encoder::FixedClipEncoder,
        clip_projects::ClipProjectFiles,
        clip_sources::{ClipContainer, FfmpegClipSources, FixedClipSource},
        clip_subtitle_assets::SubtitleAssetFiles,
    },
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::{fs, path::PathBuf};
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn write(path: &Path, value: &Value) -> String {
    let bytes = serde_json::to_vec_pretty(value).unwrap();
    fs::write(path, &bytes).unwrap();
    hash(&bytes)
}
#[test]
#[ignore = "Pinned FFmpeg/FFprobe/common fonts, fresh evidence; synthetic black source, real guarded image overlay; no GUI/models"]
fn guarded_common_font_overlay_switches_exactly_on_source_frame_6_12_18_and_end() {
    let root = PathBuf::from(
        std::env::var_os("CLIP_SUBTITLE_VIDEO_EVIDENCE_DIR").expect("Fresh evidence"),
    );
    fs::create_dir(&root).unwrap();
    let tool = PathBuf::from(std::env::var_os("CLIP_RENDER_ENCODER").unwrap());
    let pin = std::env::var("CLIP_RENDER_ENCODER_SHA256").unwrap();
    let (_verified_tool_path, _tool_guards) =
        crate::infrastructure::frame_capture::tools::open(&tool, &pin).unwrap();
    let mut fixture = Command::new(&tool);
    fixture.args([
        "-hide_banner",
        "-nostdin",
        "-v",
        "error",
        "-threads",
        "1",
        "-f",
        "lavfi",
        "-i",
        "color=c=black:s=64x64:r=24",
        "-frames:v",
        "25",
        "-c:v",
        "ffv1",
        "-threads:v",
        "1",
        "-f",
        "matroska",
        "-fd",
        "1",
        "fd:",
    ]);
    let fixture = process::run_with_recovery(
        &mut fixture,
        Instant::now() + Duration::from_secs(30),
        1024 * 1024,
        Some(&Control),
        None,
    )
    .unwrap();
    let source_path = root.join("source.mkv");
    fs::write(&source_path, &fixture.stdout).unwrap();
    let probe = PathBuf::from(std::env::var_os("CLIP_RENDER_PROBE").unwrap());
    let probe_sha = std::env::var("CLIP_RENDER_PROBE_SHA256").unwrap();
    let sources = FfmpegClipSources::new(
        BTreeMap::from([(
            "source".into(),
            FixedClipSource {
                path: source_path,
                container: ClipContainer::Matroska,
            },
        )]),
        &probe,
        &probe_sha,
        crate::domain::frame_capture::MAX_MEDIA_BYTES,
        Duration::from_secs(30),
    )
    .unwrap();
    let initial = sources.inspect_source("source", &Control).unwrap();
    let style_path = PathBuf::from(std::env::var_os("CLIP_SUBTITLE_STYLE").expect("Pinned style"));
    let style_bytes = fs::read(&style_path).unwrap();
    assert_eq!(
        hash(&style_bytes),
        std::env::var("CLIP_SUBTITLE_STYLE_SHA256").unwrap()
    );
    let mut style: Value = serde_json::from_slice(&style_bytes).unwrap();
    style["style_id"] = json!("overlay-style");
    style["font_px"] = json!(12);
    style["line_height_px"] = json!(16);
    style["box_normalized"] = json!({"left":0,"top":500000,"width":1000000,"height":500000});
    style["background_rgba"] = json!([200, 20, 10, 128]);
    let text = json!({"schema_version":"clip-subtitle-document/1","media":initial.facts().source,"time_basis":"media_relative_us","origin":"explicit_manual_snapshot","cues":{"source_id":"captions","source_revision":"1","source_sha256":"c".repeat(64),"cues":[{"cue_id":"first","text":"가","start_us":"250000","end_us":"500000"},{"cue_id":"second","text":"나","start_us":"750000","end_us":"1000000"}]}});
    let text_path = root.join("captions.json");
    let saved_style = root.join("style.json");
    let text_sha = write(&text_path, &text);
    let style_sha = write(&saved_style, &style);
    let mut doc:ClipDocument=serde_json::from_value(json!({"schema_version":"1","document_id":"overlay-short","revision":"1","sources":[initial.facts().source],"segments":[{"segment_id":"one","media_id":"source","range":{"start_us":"0","end_us":"1000000"},"video_stream_index":initial.facts().reference_stream_index,"crop_keys":[{"source_us":"0","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"disabled"},"subtitles":{"document":{"artifact":{"artifact_id":"captions","revision":"1","sha256":text_sha},"source_sha256":initial.facts().source.sha256},"cue_ids":["second","first"],"style":{"artifact_id":"overlay-style","revision":"1","sha256":style_sha}},"subject_ref":null}],"preset":{"catalog":{"artifact_id":"fixture","revision":"1","sha256":"c".repeat(64)},"width":64,"height":64,"frame_rate":{"numerator":24,"denominator":1},"container":"mp4","video_codec":"h264","audio_codec":null,"color":"sdr_bt709"},"cut_mode":"exact_reencode"})).unwrap();
    use crate::application::clip_render::presets::*;
    let mut definition = BuiltinClipPresets::entries().unwrap().remove(1).definition;
    definition.preset_id = "overlay-local".into();
    definition.origin = ClipPresetOrigin::HostManual;
    definition.label = "Local overlay fixture".into();
    definition.profile.width = 64;
    definition.profile.height = 64;
    definition.profile.frame_rate.numerator = 24;
    definition.max_duration_us.0 = 1_000_000;
    let bytes = serde_json::to_vec_pretty(&definition).unwrap();
    let preset_path = root.join("preset.json");
    fs::write(&preset_path, &bytes).unwrap();
    doc.preset = definition.output_preset(crate::domain::clip_document::ArtifactPin {
        artifact_id: definition.preset_id.clone(),
        revision: definition.revision,
        sha256: hash(&bytes),
    });
    let presets = crate::infrastructure::clip_presets::FixedClipPresets::open(BTreeMap::from([(
        definition.preset_id.clone(),
        preset_path,
    )]))
    .unwrap();
    doc.validate().unwrap();
    let projects = root.join("projects");
    fs::create_dir(&projects).unwrap();
    let store = ClipProjectFiles::open(&projects).unwrap();
    let saved = clip_projects::create(&store, &doc).unwrap();
    let media = prepare_media(
        &store,
        SavedClipProjectPin {
            document_id: doc.document_id.clone(),
            publication_revision: DecimalU64(1),
            publication_sha256: saved.publication_sha256,
        },
        &sources,
        &Control,
    )
    .unwrap();
    let encoder = prepare_encoder(
        &doc,
        &presets,
        &FixedClipEncoder {
            path: tool.clone(),
            sha256: pin.clone(),
            inventory_timeout: Duration::from_secs(30),
        },
        &Control,
    )
    .unwrap();
    assert!(encoder.requirements().filters.contains("overlay"));
    assert!(!encoder.requirements().filters.contains("subtitles"));
    let boundary = media.sources()[0]
        .observe_segment(&doc.segments[0], Duration::from_secs(30), &Control)
        .unwrap();
    let files = SubtitleAssetFiles::open(
        BTreeMap::from([("captions".into(), text_path)]),
        BTreeMap::from([("overlay-style".into(), saved_style)]),
    )
    .unwrap();
    let admitted = prepare_subtitles(&initial.facts().source, &doc.segments[0], &files, &Control)
        .unwrap()
        .unwrap();
    assert_eq!(admitted.style().style().font_px, 12);
    let temporary = PathBuf::from(std::env::var_os("TMP").expect("Fixed cache tmp"));
    let overlay =
        prepare_subtitle_overlay(&media, &encoder, &boundary, &admitted, &temporary, &Control)
            .unwrap();
    assert_eq!(overlay.images.len(), 2);
    let working_path = overlay.working.as_ref().unwrap().path().to_path_buf();
    {
        assert!(
            fs::OpenOptions::new()
                .write(true)
                .open(&working_path)
                .is_err()
        );
        assert!(fs::rename(&working_path, working_path.with_extension("moved")).is_err());
    }
    let color = DeclaredBt709Limited {
        source_sha256: initial.facts().source.sha256.clone(),
    };
    assert_eq!(
        encode_candidate(
            &media,
            &encoder,
            &boundary,
            &color,
            Duration::from_secs(30),
            1024 * 1024,
            &Control
        )
        .err()
        .unwrap()
        .code,
        ErrorCode::Unsupported
    );
    let args = super::super::command::arguments_with_overlay(
        media.sources()[0].path(),
        media.sources()[0].demuxer(),
        &overlay.plan,
        Some(&overlay),
    )
    .unwrap();
    fs::write(
        root.join("arguments.json"),
        serde_json::to_vec_pretty(&args.iter().map(|a| a.to_string_lossy()).collect::<Vec<_>>())
            .unwrap(),
    )
    .unwrap();
    let candidate = encode_candidate_with_subtitles(
        &overlay,
        &color,
        Duration::from_secs(30),
        1024 * 1024,
        &Control,
    )
    .unwrap();
    let output = root.join("candidate.mp4");
    fs::write(&output, candidate.bytes()).unwrap();
    fs::write(
        root.join("render.json"),
        serde_json::to_vec_pretty(candidate.facts()).unwrap(),
    )
    .unwrap();
    let verifier = ClipOutputVerifier::open(
        &probe,
        &probe_sha,
        &tool,
        &pin,
        &temporary,
        Duration::from_secs(30),
    )
    .unwrap();
    let verified = verifier.verify(&candidate, &Control).unwrap();
    fs::write(
        root.join("output-verification.json"),
        serde_json::to_vec_pretty(verified.verification()).unwrap(),
    )
    .unwrap();
    let mut decode = Command::new(&tool);
    decode
        .args([
            "-hide_banner",
            "-nostdin",
            "-v",
            "error",
            "-xerror",
            "-threads",
            "1",
            "-protocol_whitelist",
            "file",
            "-f",
            "mov",
            "-enable_drefs",
            "0",
            "-i",
        ])
        .arg(&output)
        .args([
            "-an",
            "-sn",
            "-dn",
            "-vf",
            "format=rgb24",
            "-threads:v",
            "1",
            "-f",
            "rawvideo",
            "-",
        ]);
    let decoded = process::run_with_recovery(
        &mut decode,
        Instant::now() + Duration::from_secs(30),
        1024 * 1024,
        Some(&Control),
        None,
    )
    .unwrap();
    fs::write(root.join("decoded.rgb"), &decoded.stdout).unwrap();
    assert_eq!(decoded.stdout.len(), 64 * 64 * 3 * 24);
    let mut samples = Vec::new();
    for (frame, pixels) in decoded.stdout.as_chunks::<12288>().0.iter().enumerate() {
        let sample = &pixels[(60 * 64 + 4) * 3..(60 * 64 + 4) * 3 + 3];
        let active = (6..12).contains(&frame) || (18..24).contains(&frame);
        samples.push(json!({"frame":frame,"expected_active":active,"rgb":sample}));
        if active {
            assert!(
                sample[0] > 70 && sample[0] < 130 && sample[1] < 30 && sample[2] < 30,
                "frame{frame}: {sample:?}"
            );
        } else {
            assert!(sample.iter().all(|v| *v < 6), "frame{frame}: {sample:?}");
        }
    }
    drop(overlay);
    assert!(!working_path.exists());
    fs::write(
        root.join("samples.json"),
        serde_json::to_vec_pretty(&samples).unwrap(),
    )
    .unwrap();
    fs::write(root.join("verification.json"),serde_json::to_vec_pretty(&json!({"pinned_engine_source_frame_boundaries_verified":true,"frames":24,"active_ranges":[[6,12],[18,24]],"straight_alpha_sample_checked":true,"original_png_guards_retained":true,"output_metadata_and_full_decode_verified":true,"source_and_caption_are_fixtures":true,"jobs_or_cli_verified":false,"quality_approved":false,"color_verified":false})).unwrap()).unwrap();
}
