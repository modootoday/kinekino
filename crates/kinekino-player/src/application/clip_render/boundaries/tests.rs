use super::*;
use serde_json::json;
#[test]
fn crop_switches_use_original_pts_with_vfr_gaps_equal_offsets_and_excluded_end() {
    let (source, mut segment) = values();
    let key = segment.crop_keys[0];
    for time in [1000, 2000, 500000, 999999] {
        let mut next = key;
        next.source_us = DecimalU64(time);
        segment.crop_keys.push(next);
    }
    let boundary =
        evaluate_frame_boundaries(&source, &segment, &[-9000, -8000, 0, 36000, 80999, 81000])
            .unwrap();
    let offsets: Vec<_> = boundary
        .crop_switches
        .iter()
        .map(|key| key.frame_offset.0)
        .collect();
    assert_eq!(offsets, vec![0, 1, 1, 3, 5]);
    assert_eq!(boundary.crop_switches[1].threshold_pts, "-8910");
    assert_eq!(boundary.crop_switches[4].threshold_pts, "81000");
}
pub(crate) fn values() -> (ClipSourcePin, ClipSegment) {
    let source=serde_json::from_value(json!({"media_id":"source","sha256":"a".repeat(64),"duration_us":"60000000","width":16,"height":16,"clock":{"origin_pts":"-9000","time_base_numerator":1,"time_base_denominator":90000,"observation_sha256":"b".repeat(64)},"tracks":[{"stream_index":0,"kind":"video","codec_id":"h264"}]})).unwrap();
    let segment=serde_json::from_value(json!({"segment_id":"one","media_id":"source","range":{"start_us":"0","end_us":"1000000"},"video_stream_index":0,"crop_keys":[{"source_us":"0","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"disabled"},"subtitles":null,"subject_ref":null})).unwrap();
    (source, segment)
}
#[test]
fn negative_origin_and_half_open_end_use_original_integer_frame_pts() {
    let (source, segment) = values();
    let facts = evaluate_frame_boundaries(&source, &segment, &[-9000, 36000, 81000]).unwrap();
    assert_eq!(facts.start_threshold_pts, "-9000");
    assert_eq!(facts.end_threshold_pts, "81000");
    assert_eq!(facts.first_included_pts, "-9000");
    assert_eq!(facts.last_included_pts, "36000");
    assert_eq!(facts.first_excluded_pts, "81000");
    assert_eq!(facts.included_frames.0, 2);
}
#[test]
fn fractional_thresholds_round_up_without_float_and_keep_vfr_membership() {
    let (mut source, mut segment) = values();
    source.clock.origin_pts = "-5".into();
    source.clock.time_base_denominator = 30000;
    segment.range.start_us.0 = 1;
    segment.range.end_us.0 = 33334;
    let facts = evaluate_frame_boundaries(&source, &segment, &[-5, -4, 20, 995, 996]).unwrap();
    assert_eq!(facts.start_threshold_pts, "-4");
    assert_eq!(facts.end_threshold_pts, "996");
    assert_eq!(facts.last_included_pts, "995");
    assert_eq!(facts.included_frames.0, 3);
}
#[test]
fn missing_coverage_duplicates_foreign_stream_and_frame_budget_are_explicit_errors() {
    let (source, mut segment) = values();
    for pts in [
        vec![-9000, 0],
        vec![-9000, -9000, 81000],
        vec![-8000, 0, 81000],
        vec![81000, 90000],
        vec![],
    ] {
        assert!(evaluate_frame_boundaries(&source, &segment, &pts).is_err());
    }
    assert_eq!(
        evaluate_frame_boundaries(&source, &segment, &vec![0; MAX_CLIP_BOUNDARY_FRAMES + 1])
            .unwrap_err()
            .code,
        ErrorCode::ResourceExhausted
    );
    segment.video_stream_index = 1;
    assert_eq!(
        evaluate_frame_boundaries(&source, &segment, &[-9000, 81000])
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
}
