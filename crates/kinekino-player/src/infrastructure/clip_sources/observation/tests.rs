use super::*;
use serde_json::{Value, json};
fn raw() -> Value {
    json!({"streams":[{"index":4,"codec_type":"audio","codec_name":"aac"},{"index":2,"codec_type":"video","codec_name":"h264","width":1920,"height":1080,"time_base":"1/90000","start_pts":-9000,"duration_ts":900000,"disposition":{"attached_pic":0}},{"index":0,"codec_type":"video","codec_name":"mjpeg","disposition":{"attached_pic":1}}],"format":{"duration":"10.007000"}})
}
fn observed(raw: &Value) -> Result<ProbeFacts, ApiError> {
    parse(
        &serde_json::to_vec(raw).unwrap(),
        "media",
        &"a".repeat(64),
        &"b".repeat(64),
    )
}
#[test]
fn integer_negative_origin_and_reference_stream_ignore_attached_picture() {
    let raw = raw();
    let facts = observed(&raw).unwrap();
    assert_eq!(facts.reference_stream_index, 2);
    assert_eq!(facts.source.clock.origin_pts, "-9000");
    assert_eq!(facts.source.duration_us.0, 10_000_000);
    assert_eq!(facts.duration_basis, DurationBasis::ReferenceStreamTicks);
    assert_eq!(
        facts
            .source
            .tracks
            .iter()
            .map(|track| track.stream_index)
            .collect::<Vec<_>>(),
        [0, 2, 4]
    );
    let changed = parse(
        &serde_json::to_vec(&raw).unwrap(),
        "media",
        &"a".repeat(64),
        &"c".repeat(64),
    )
    .unwrap();
    assert_ne!(
        facts.source.clock.observation_sha256,
        changed.source.clock.observation_sha256
    );
    assert_eq!(facts.raw_probe_sha256, changed.raw_probe_sha256);
}
#[test]
fn reported_span_is_explicit_and_decimal_conversion_preserves_leading_zeros() {
    assert_eq!(seconds("0.000001").unwrap(), 1);
    assert_eq!(seconds("10.007000").unwrap(), 10_007_000);
    assert_eq!(seconds("1.01").unwrap(), 1_010_000);
    for invalid in ["-1.0", "1e3", "0.0000001", "18446744073709551615.0", "NaN"] {
        assert!(seconds(invalid).is_err());
    }
    let mut raw = raw();
    raw["streams"][1]
        .as_object_mut()
        .unwrap()
        .remove("duration_ts");
    let facts = observed(&raw).unwrap();
    assert_eq!(facts.duration_basis, DurationBasis::ContainerReportedSpan);
    assert_eq!(facts.source.duration_us.0, 10_007_000);
    assert_ne!(
        facts.source.clock.observation_sha256,
        observed(&super::tests::raw())
            .unwrap()
            .source
            .clock
            .observation_sha256
    );
}
#[test]
fn missing_origin_duplicate_indexes_and_unbounded_metadata_are_rejected() {
    for invalid in 0..4 {
        let mut raw = raw();
        match invalid {
            0 => {
                raw["streams"][1]
                    .as_object_mut()
                    .unwrap()
                    .remove("start_pts");
            }
            1 => raw["streams"][0]["index"] = json!(2),
            2 => raw["streams"][1]["width"] = json!(u32::MAX),
            _ => raw["streams"][1]["time_base"] = json!("1/0"),
        }
        assert!(observed(&raw).is_err());
    }
}
