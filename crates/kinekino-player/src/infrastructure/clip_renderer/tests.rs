use super::*;
#[path = "section_tests.rs"]
mod sections;
use crate::{
    application::{
        clip_projects::{self, ClipProjectStore},
        clip_render::*,
    },
    domain::{api_values::DecimalU64, clip_document::*},
    infrastructure::{
        clip_encoder::FixedClipEncoder,
        clip_projects::ClipProjectFiles,
        clip_sources::{ClipContainer, FfmpegClipSources, FixedClipSource},
    },
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
fn document(source: &ClipSourcePin) -> ClipDocument {
    let mut document: ClipDocument = serde_json::from_value(json!({"schema_version":"1","document_id":"render-one","revision":"1","sources":[source],"segments":[{"segment_id":"one","media_id":source.media_id,"range":{"start_us":"15000000","end_us":"16000000"},"video_stream_index":0,"crop_keys":[{"source_us":"15000000","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"selected","stream_index":1,"volume_percent":75,"muted":false},"subtitles":null,"subject_ref":null}],"preset":{"catalog":{"artifact_id":"fixture-manual-profile","revision":"1","sha256":"c".repeat(64)},"width":1080,"height":1920,"frame_rate":{"numerator":30,"denominator":1},"container":"mp4","video_codec":"h264","audio_codec":"aac","color":"sdr_bt709"},"cut_mode":"exact_reencode"})).unwrap();
    document.preset = crate::application::clip_render::presets::BuiltinClipPresets::entries()
        .unwrap()
        .remove(0)
        .preset;
    document
}
#[test]
fn command_uses_owned_seekable_stdout_explicit_tracks_integer_trim_and_no_shell_payload() {
    let plan = ManualClipPlan {
        document_sha256: "a".repeat(64),
        source_media_id: "source".into(),
        source_sha256: "b".repeat(64),
        source_observation_sha256: "c".repeat(64),
        segment_id: "one".into(),
        video_stream_index: 3,
        start_pts: -9000,
        end_pts: 81000,
        crop_sections: vec![ManualClipCropSection {
            source_us: DecimalU64(0),
            threshold_pts: "-9000".into(),
            frame_offset: DecimalU64(0),
            crop: ClipPixelCrop {
                left: 1,
                top: 2,
                width: 640,
                height: 480,
            },
            fit: FitMode::Fill,
        }],
        width: 1080,
        height: 1920,
        fps_numerator: 30000,
        fps_denominator: 1001,
        duration_us: 1_000_000,
        input_seek_start_us: -100_000,
        audio: Some(ManualClipAudio {
            stream_index: 5,
            volume_percent: 75,
            muted: false,
            absolute_start_us: -100_000,
            absolute_end_us: 900_000,
        }),
    };
    let args = command::arguments(
        std::path::Path::new("C:/private/input.mkv"),
        "matroska",
        &plan,
    )
    .unwrap();
    let args: Vec<_> = args.iter().map(|arg| arg.to_str().unwrap()).collect();
    let graph = args[args
        .iter()
        .position(|arg| *arg == "-filter_complex")
        .unwrap()
        + 1];
    assert!(graph.contains("[0:3]trim=start_pts=-9000:end_pts=81000"));
    assert!(graph.contains("[0:5]atrim=start=-0.100000:end=0.900000"));
    assert!(graph.contains("volume=0.75"));
    assert!(graph.contains("crop=640:480:1:2:exact=1"));
    assert_eq!(args.last(), Some(&"fd:"));
    assert!(!args.contains(&"-y"));
    assert!(!args.contains(&"-c:s"));
}
#[test]
#[ignore = "Explicit original source, pinned FFmpeg/FFprobe and fresh evidence; candidate encode only"]
fn actual_manual_candidate_combines_saved_media_encoder_boundaries_and_selected_audio() {
    let root = PathBuf::from(std::env::var_os("CLIP_RENDER_EVIDENCE_DIR").expect("Fresh evidence"));
    assert!(root.is_absolute());
    fs::create_dir(&root).unwrap();
    let source_path =
        PathBuf::from(std::env::var_os("CLIP_RENDER_SOURCE").expect("Original source"));
    let probe = PathBuf::from(std::env::var_os("CLIP_RENDER_PROBE").expect("Pinned probe"));
    let probe_sha = std::env::var("CLIP_RENDER_PROBE_SHA256")
        .unwrap()
        .to_lowercase();
    let encoder_path =
        PathBuf::from(std::env::var_os("CLIP_RENDER_ENCODER").expect("Pinned encoder"));
    let encoder_sha = std::env::var("CLIP_RENDER_ENCODER_SHA256")
        .unwrap()
        .to_lowercase();
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
        Duration::from_secs(120),
    )
    .unwrap();
    let initial = sources.inspect_source("source", &Control).unwrap();
    let doc = document(&initial.facts().source);
    doc.validate().unwrap();
    let store_path = root.join("projects");
    fs::create_dir(&store_path).unwrap();
    let store = ClipProjectFiles::open(&store_path).unwrap();
    let saved = clip_projects::create(&store, &doc).unwrap();
    let media = prepare_media(
        &store,
        SavedClipProjectPin {
            document_id: doc.document_id.clone(),
            publication_revision: DecimalU64(1),
            publication_sha256: saved.publication_sha256.clone(),
        },
        &sources,
        &Control,
    )
    .unwrap();
    let encoder = prepare_encoder(
        &doc,
        &crate::application::clip_render::presets::BuiltinClipPresets,
        &FixedClipEncoder {
            path: encoder_path.clone(),
            sha256: encoder_sha,
            inventory_timeout: Duration::from_secs(30),
        },
        &Control,
    )
    .unwrap();
    let boundary = media.sources()[0]
        .observe_segment(&doc.segments[0], Duration::from_secs(30), &Control)
        .unwrap();
    let color = DeclaredBt709Limited {
        source_sha256: initial.facts().source.sha256.clone(),
    };
    let mut candidate = encode_candidate(
        &media,
        &encoder,
        &boundary,
        &color,
        Duration::from_secs(120),
        64 * 1024 * 1024,
        &Control,
    )
    .unwrap();
    let output = root.join("candidate.mp4");
    fs::write(&output, candidate.bytes()).unwrap();
    fs::write(
        root.join("candidate-facts.json"),
        serde_json::to_vec_pretty(candidate.facts()).unwrap(),
    )
    .unwrap();
    fs::write(root.join("encoder.stderr"), candidate.stderr()).unwrap();
    fs::write(
        root.join("boundary.json"),
        serde_json::to_vec_pretty(boundary.observation()).unwrap(),
    )
    .unwrap();
    let mut inspect = Command::new(&probe);
    inspect
        .args([
            "-v",
            "error",
            "-count_frames",
            "-show_entries",
            "stream=codec_type,codec_name,width,height,nb_read_frames,r_frame_rate,start_pts,start_time,time_base,duration_ts,duration:format=start_time,duration",
            "-of",
            "json",
        ])
        .arg(&output);
    let observed = process::run_with_recovery(
        &mut inspect,
        Instant::now() + Duration::from_secs(30),
        262144,
        Some(&Control),
        None,
    )
    .unwrap();
    fs::write(root.join("output-probe.json"), &observed.stdout).unwrap();
    let metadata: Value = serde_json::from_slice(&observed.stdout).unwrap();
    let streams = metadata["streams"].as_array().unwrap();
    let video = streams
        .iter()
        .find(|stream| stream["codec_type"] == "video")
        .unwrap();
    let audio = streams
        .iter()
        .find(|stream| stream["codec_type"] == "audio")
        .unwrap();
    assert_eq!(video["codec_name"], "h264");
    assert_eq!(video["width"], 1080);
    assert_eq!(video["height"], 1920);
    assert_eq!(video["nb_read_frames"], "30");
    assert_eq!(video["r_frame_rate"], "30/1");
    assert_eq!(audio["codec_name"], "aac");
    assert_eq!(video["start_time"], "0.000000");
    assert_eq!(metadata["format"]["duration"], "1.000000");
    let mut decode = Command::new(encoder.lease().path());
    decode
        .args(["-v", "error", "-threads", "2", "-i"])
        .arg(&output)
        .args(["-map", "0", "-f", "null", "-"]);
    let decoded = process::run_with_recovery(
        &mut decode,
        Instant::now() + Duration::from_secs(30),
        65536,
        Some(&Control),
        None,
    )
    .unwrap();
    fs::write(root.join("decode.stderr"), &decoded.stderr).unwrap();
    let after = sources.inspect_source("source", &Control).unwrap();
    assert_eq!(after.facts().source.sha256, initial.facts().source.sha256);
    assert_eq!(
        store
            .load(&doc.document_id, Some(DecimalU64(1)))
            .unwrap()
            .publication_sha256,
        saved.publication_sha256
    );
    let mut revised = doc.clone();
    revised.revision = DecimalU64(2);
    revised.segments[0].audio = AudioSelection::Selected {
        stream_index: 1,
        volume_percent: 50,
        muted: false,
    };
    let next = clip_projects::save(&store, DecimalU64(1), &revised).unwrap();
    let changed = prepare_media(
        &store,
        SavedClipProjectPin {
            document_id: doc.document_id.clone(),
            publication_revision: DecimalU64(2),
            publication_sha256: next.publication_sha256,
        },
        &sources,
        &Control,
    )
    .unwrap();
    match encode_candidate(
        &changed,
        &encoder,
        &boundary,
        &color,
        Duration::from_secs(120),
        64 * 1024 * 1024,
        &Control,
    ) {
        Err(error) => assert_eq!(error.code, ErrorCode::SourceExpired),
        Ok(_) => panic!("encoder document mismatch accepted"),
    }
    let wrong = DeclaredBt709Limited {
        source_sha256: "0".repeat(64),
    };
    match encode_candidate(
        &media,
        &encoder,
        &boundary,
        &wrong,
        Duration::from_secs(120),
        64 * 1024 * 1024,
        &Control,
    ) {
        Err(error) => assert_eq!(error.code, ErrorCode::SourceExpired),
        Ok(_) => panic!("foreign color declaration accepted"),
    }
    let verifier = ClipOutputVerifier::open(
        &probe,
        &probe_sha,
        &encoder_path,
        &encoder.lease().inventory().tool_sha256,
        &std::env::temp_dir(),
        Duration::from_secs(30),
    )
    .unwrap();
    let original = candidate.bytes[0];
    candidate.bytes[0] ^= 1;
    match verifier.verify(&candidate, &Control) {
        Err(error) => assert_eq!(error.code, ErrorCode::SourceExpired),
        Ok(_) => panic!("tampered candidate accepted"),
    }
    candidate.bytes[0] = original;
    let verified_owner = verifier.verify_owned(candidate, &Control).unwrap();
    let candidate = verified_owner.candidate();
    let verified = verified_owner.as_verified();
    let working_path = verified.spool_path().to_path_buf();
    assert!(
        std::fs::OpenOptions::new()
            .write(true)
            .open(verified.spool_path())
            .is_err()
    );
    fs::write(
        root.join("typed-output-verification.json"),
        serde_json::to_vec_pretty(verified.verification()).unwrap(),
    )
    .unwrap();
    fs::write(root.join("typed-output-probe.json"), verified.raw_probe()).unwrap();
    let artifacts = root.join("artifacts");
    fs::create_dir(&artifacts).unwrap();
    let publication = ClipArtifactStore::open(&artifacts).unwrap();
    let controller = PublicationControl::new(false, false);
    let published = publication
        .publish("first", &verified, &controller)
        .unwrap();
    let receipt = published.receipt().clone();
    assert!(published.receipt().review_artifact_only);
    assert!(!published.receipt().quality_approved);
    assert!(published.execution().is_none());
    assert_eq!(published.verification()["decode_integrity_verified"], true);
    assert_eq!(
        published.render_facts()["output_sha256"],
        candidate.facts.output_sha256
    );
    assert!(
        std::fs::OpenOptions::new()
            .write(true)
            .open(artifacts.join("clip-first/clip.mp4"))
            .is_err()
    );
    match publication.publish("first", &verified, &controller) {
        Err(error) => assert_eq!(error.code, ErrorCode::RevisionConflict),
        Ok(_) => panic!("artifact overwritten"),
    }
    assert_eq!(
        controller
            .reserve_calls
            .load(std::sync::atomic::Ordering::SeqCst),
        1
    );
    match publication.inspect("first", &"0".repeat(64), &Control) {
        Err(error) => assert_eq!(error.code, ErrorCode::SourceExpired),
        Ok(_) => panic!("wrong manifest pin accepted"),
    }
    drop(published);
    let reopened = publication
        .inspect("first", &receipt.manifest_sha256, &Control)
        .unwrap();
    assert_eq!(reopened.receipt().manifest_sha256, receipt.manifest_sha256);
    drop(reopened);
    let damaged = artifacts.join("clip-first/clip.mp4");
    let mut bad = candidate.bytes.clone();
    bad[0] ^= 1;
    fs::write(&damaged, &bad).unwrap();
    match publication.inspect("first", &receipt.manifest_sha256, &Control) {
        Err(error) => assert_eq!(error.code, ErrorCode::SourceExpired),
        Ok(_) => panic!("changed artifact bytes accepted"),
    }
    fs::write(&damaged, candidate.bytes()).unwrap();
    let restored = publication
        .inspect("first", &receipt.manifest_sha256, &Control)
        .unwrap();
    drop(restored);
    let cancelled = PublicationControl::new(true, false);
    match publication.publish("cancelled", &verified, &cancelled) {
        Err(error) => assert_eq!(error.code, ErrorCode::Cancelled),
        Ok(_) => panic!("cancelled publication committed"),
    }
    assert!(!artifacts.join("clip-cancelled/manifest.json").exists());
    assert_eq!(
        fs::read(artifacts.join("clip-cancelled/clip.mp4")).unwrap(),
        candidate.bytes()
    );
    let late = PublicationControl::new(false, true);
    let committed = publication.publish("late", &verified, &late).unwrap();
    assert!(crate::application::jobs::ExecutionControl::cancellation_requested(&late));
    assert_eq!(committed.receipt().artifact_id, "late");
    let mut expiry = PublicationControl::new(false, false);
    expiry.expire_on_begin = true;
    let pin = match publication.publish("timed", &verified, &expiry) {
        Err(error) => {
            assert_eq!(error.code, ErrorCode::Timeout);
            assert_eq!(error.details["publication_committed"], true);
            error.details["manifest_sha256"]
                .as_str()
                .unwrap()
                .to_owned()
        }
        Ok(_) => panic!("post-commit deadline ignored"),
    };
    let reconciled = publication.inspect("timed", &pin, &Control).unwrap();
    assert_eq!(reconciled.receipt().artifact_id, "timed");
    drop(verified);
    assert!(
        working_path.exists(),
        "Borrowed view does not release the original proof owner"
    );
    fs::write(
        root.join("publication-receipt.json"),
        serde_json::to_vec_pretty(&receipt).unwrap(),
    )
    .unwrap();
    fs::write(root.join("verification.json"),serde_json::to_vec_pretty(&json!({"source_sha256":initial.facts().source.sha256,"candidate_sha256":candidate.facts().output_sha256,"video_frames":30,"width":1080,"height":1920,"selected_audio_reencoded_aac":true,"typed_output_verifier_passed":true,"review_artifact_manifest_sha256":receipt.manifest_sha256,"immutable_publication_reopen_and_tamper_checks":true,"cancel_before_commit_and_late_cancel_preserve_result":true,"publication_control_is_fixture":true,"color_verified":false,"subtitles_rendered":false,"jobs_or_cli_verified":false,"quality_approved":false})).unwrap()).unwrap();
    drop(verified_owner);
    assert!(!working_path.exists());
}

struct PublicationControl {
    expire_on_begin: bool,
    expired: std::sync::atomic::AtomicBool,
    deadline: Instant,
    reserve_calls: std::sync::atomic::AtomicUsize,
    cancel_at_reserve: bool,
    late: bool,
    cancelled: std::sync::atomic::AtomicBool,
}
impl PublicationControl {
    fn new(cancel_at_reserve: bool, late: bool) -> Self {
        Self {
            expire_on_begin: false,
            expired: std::sync::atomic::AtomicBool::new(false),
            deadline: Instant::now() + Duration::from_secs(30),
            reserve_calls: std::sync::atomic::AtomicUsize::new(0),
            cancel_at_reserve,
            late,
            cancelled: std::sync::atomic::AtomicBool::new(false),
        }
    }
}
impl crate::application::jobs::ExecutionControl for PublicationControl {
    fn cancellation_requested(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::SeqCst)
    }
    fn deadline(&self) -> Instant {
        if self.expired.load(std::sync::atomic::Ordering::SeqCst) {
            self.deadline - Duration::from_secs(31)
        } else {
            self.deadline
        }
    }
    fn reserve_publication(&self) -> Result<(), ApiError> {
        self.reserve_calls
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if self.cancel_at_reserve {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "fixture cancel at reserve",
            ));
        }
        if self.late {
            self.cancelled
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        Ok(())
    }
    fn begin_verification(&self) -> Result<(), ApiError> {
        if self.expire_on_begin {
            self.expired
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        Ok(())
    }
}
