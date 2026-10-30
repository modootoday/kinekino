use super::*;

#[test]
fn live_quantized_profile_requires_matching_hints_and_preserves_observed_frames() {
    use crate::application::review_playback::ReviewContentClass;
    let frames = [98958,99000,99042,99083,99125,99167].into_iter().map(|pts| serde_json::json!({"pts":pts,"best_effort_timestamp":pts,"duration":41,"key_frame":0})).collect::<Vec<_>>();
    let mut value = serde_json::json!({"frames":frames,"streams":[{"index":0,"time_base":"1/1000","avg_frame_rate":"24/1","r_frame_rate":"24/1"}]});
    let raw = serde_json::to_string(&value).unwrap();
    let (parsed, _, _, cadence) = parse_live(&raw, &ReviewContentClass::MatroskaHeader).unwrap();
    assert_eq!(
        parsed
            .iter()
            .map(|frame| frame.duration)
            .collect::<Vec<_>>(),
        vec![41; 6]
    );
    assert_eq!(cadence.unwrap().rate_numerator, 24);
    assert!(parse(&raw).is_err());
    assert!(parse_live(&raw, &ReviewContentClass::IsoBmffHeader).is_err());
    value["streams"][0]["r_frame_rate"] = "25/1".into();
    assert!(parse_live(&value.to_string(), &ReviewContentClass::MatroskaHeader).is_err());
    value["streams"][0]
        .as_object_mut()
        .unwrap()
        .remove("r_frame_rate");
    assert!(parse_live(&value.to_string(), &ReviewContentClass::MatroskaHeader).is_err());
}
#[test]
fn signed_pts_are_preserved_and_ambiguous_or_missing_fields_are_rejected() {
    let raw = r#"{"frames":[{"pts":-2,"best_effort_timestamp":-2,"duration":2,"key_frame":1},{"pts":0,"best_effort_timestamp":0,"duration":3,"key_frame":0}],"streams":[{"index":0,"time_base":"1/24"}]}"#;
    assert_eq!(
        parse(raw).unwrap().0,
        vec![
            VideoFrame {
                pts: -2,
                duration: 2
            },
            VideoFrame {
                pts: 0,
                duration: 3
            }
        ]
    );
    for bad in [
        raw.replace("\"pts\":0", "\"pts\":1"),
        raw.replace("\"duration\":3", "\"duration\":0"),
        raw.replace("\"key_frame\":0", "\"key_frame\":2"),
        raw.replace("\"pts\":0", "\"pts\":0,\"pts\":0"),
        raw.replace("\"pts\":0", "\"pts\":0.0"),
        raw.replace("\"best_effort_timestamp\":0,", ""),
        raw.replace("1/24", "0/24"),
        raw.replace(
            "\"streams\":[{\"index\":0,\"time_base\":\"1/24\"}]",
            "\"streams\":[]",
        ),
        raw.replace("\"index\":0", "\"index\":0,\"index\":1"),
    ] {
        assert!(parse(&bad).is_err(), "accepted {bad}");
    }
    assert!(parse(r#"{"frames":[]}"#).is_err());
}
#[test]
#[ignore = "explicit central allocation: reads preserved ffprobe evidence; no decoder execution"]
fn retained_real_video_window_matches_preserved_ticks() {
    let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .join("runtime/verification/sprite-frame-pts-datalab-r1/frames.stdout.json");
    let sha = "ba26bd11b6bee0fef8fcb5a461f4b9601beee05abcf26e275bb935d2df954a34";
    let evidence = RetainedVideoFrames::open(&path, sha).unwrap();
    assert_eq!(evidence.frames().len(), 240);
    assert_eq!(evidence.time_base().ratio(), (1, 12288));
    assert_eq!(evidence.stream_index(), 0);
    assert_eq!(evidence.frames()[0].pts, 196608);
    assert_eq!(evidence.frames()[239].pts, 318976);
    for tick in [208896, 313344] {
        assert!(evidence.frames().iter().any(|frame| frame.pts == tick));
    }
    assert_eq!(evidence.sha256(), sha);
    assert!(std::fs::OpenOptions::new().write(true).open(&path).is_err());
    assert!(RetainedVideoFrames::open(&path, &"a".repeat(64)).is_err());
}
