use super::*;
use serde_json::{Value, json};
fn values() -> (ClipSourcePin, ClipSegment) {
    let source=serde_json::from_value(json!({"media_id":"source","sha256":"a".repeat(64),"duration_us":"60000000","width":16,"height":16,"clock":{"origin_pts":"-9000","time_base_numerator":1,"time_base_denominator":90000,"observation_sha256":"b".repeat(64)},"tracks":[{"stream_index":0,"kind":"video","codec_id":"h264"}]})).unwrap();
    let segment=serde_json::from_value(json!({"segment_id":"one","media_id":"source","range":{"start_us":"0","end_us":"1000000"},"video_stream_index":0,"crop_keys":[{"source_us":"0","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"disabled"},"subtitles":null,"subject_ref":null})).unwrap();
    (source, segment)
}
fn window() -> Value {
    json!({"streams":[{"index":0,"time_base":"1/90000"}],"frames":[{"stream_index":0,"pts":-9000,"best_effort_timestamp":-9000},{"stream_index":0,"pts":36000,"best_effort_timestamp":36000},{"stream_index":0,"pts":81000,"best_effort_timestamp":81000}]})
}
#[test]
fn parser_checks_exact_stream_clock_and_original_pts_and_preserves_signed_interval() {
    let (source, segment) = values();
    assert_eq!(
        interval(&source.clock, &segment).unwrap(),
        "-0.100000%2.900000"
    );
    assert_eq!(
        parse(&serde_json::to_vec(&window()).unwrap(), &source, &segment)
            .unwrap()
            .0
            .included_frames
            .0,
        2
    );
    for mismatch in 0..4 {
        let mut raw = window();
        match mismatch {
            0 => raw["streams"][0]["time_base"] = json!("1/1000"),
            1 => raw["frames"][1]["stream_index"] = json!(2),
            2 => raw["frames"][1]["best_effort_timestamp"] = json!(36001),
            _ => {
                raw["frames"][1].as_object_mut().unwrap().remove("pts");
            }
        }
        assert!(parse(&serde_json::to_vec(&raw).unwrap(), &source, &segment).is_err());
    }
}
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
#[test]
#[ignore = "Explicit original media, pinned local FFprobe and fresh evidence; decoded PTS only, no encoding"]
fn actual_source_window_observes_included_and_excluded_pts_with_retained_original_source() {
    let root =
        PathBuf::from(std::env::var_os("CLIP_BOUNDARY_EVIDENCE_DIR").expect("Fresh evidence"));
    assert!(root.is_absolute());
    std::fs::create_dir(&root).unwrap();
    let path = PathBuf::from(std::env::var_os("CLIP_BOUNDARY_SOURCE").expect("Original source"));
    let probe = PathBuf::from(std::env::var_os("CLIP_BOUNDARY_PROBE").expect("Pinned probe"));
    let sha = std::env::var("CLIP_BOUNDARY_PROBE_SHA256")
        .unwrap()
        .to_lowercase();
    let catalog = BTreeMap::from([(
        "source".into(),
        FixedClipSource {
            path,
            container: ClipContainer::Matroska,
        },
    )]);
    let port = FfmpegClipSources::new(
        catalog,
        &probe,
        &sha,
        crate::domain::frame_capture::MAX_MEDIA_BYTES,
        Duration::from_secs(120),
    )
    .unwrap();
    let source = port.inspect_source("source", &Control).unwrap();
    let segment:ClipSegment=serde_json::from_value(json!({"segment_id":"one","media_id":"source","range":{"start_us":"15000000","end_us":"16000000"},"video_stream_index":source.facts().reference_stream_index,"crop_keys":[{"source_us":"15000000","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"disabled"},"subtitles":null,"subject_ref":null})).unwrap();
    let observed = source
        .observe_segment(&segment, Duration::from_secs(30), &Control)
        .unwrap();
    let facts = observed.observation();
    let bounds = &facts.boundaries;
    assert!(bounds.included_frames.0 > 0);
    assert!(
        bounds.first_included_pts.parse::<i64>().unwrap()
            >= bounds.start_threshold_pts.parse::<i64>().unwrap()
    );
    assert!(
        bounds.last_included_pts.parse::<i64>().unwrap()
            < bounds.end_threshold_pts.parse::<i64>().unwrap()
    );
    assert!(
        bounds.first_excluded_pts.parse::<i64>().unwrap()
            >= bounds.end_threshold_pts.parse::<i64>().unwrap()
    );
    assert_eq!(observed.source().observation(), source.observation());
    std::fs::write(
        root.join("source-facts.json"),
        serde_json::to_vec_pretty(source.facts()).unwrap(),
    )
    .unwrap();
    std::fs::write(root.join("frame-window.json"), observed.raw_frames()).unwrap();
    std::fs::write(
        root.join("boundaries.json"),
        serde_json::to_vec_pretty(facts).unwrap(),
    )
    .unwrap();
    let repeated = source
        .observe_segment(&segment, Duration::from_secs(30), &Control)
        .unwrap();
    assert_eq!(
        repeated.observation().raw_frame_window_sha256,
        facts.raw_frame_window_sha256
    );
    let mut foreign = segment;
    foreign.media_id = "foreign".into();
    match source.observe_segment(&foreign, Duration::from_secs(30), &Control) {
        Err(error) => assert_eq!(error.code, ErrorCode::InvalidArgument),
        Ok(_) => panic!("foreign segment accepted"),
    }
    std::fs::write(root.join("verification.json"),serde_json::to_vec_pretty(&json!({"source_sha256":facts.source_sha256,"raw_frame_window_sha256":facts.raw_frame_window_sha256,"stable_across_reopen":true,"retained_original_source":true,"foreign_segment_rejected":true,"negative_pts_actual_sample":false,"encoded":false,"quality_approved":false})).unwrap()).unwrap();
    let retained = observed.into_retained();
    let same_pin_other_owner = port.inspect_source("source", &Control).unwrap();
    assert_eq!(same_pin_other_owner.observation(), source.observation());
    assert!(retained.belongs_to(&source));
    assert!(!retained.belongs_to(&same_pin_other_owner));
    std::fs::write(root.join("custody-verification.json"),serde_json::to_vec_pretty(&json!({"original_source_lease_matches_retained_decoded_owner":true,"same_pin_reopened_lease_rejected_as_original_owner":true,"serialized_source_pin_alone_is_not_custody":true})).unwrap()).unwrap();
}
