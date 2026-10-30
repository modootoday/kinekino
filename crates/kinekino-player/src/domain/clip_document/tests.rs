use super::*;
mod editor;
use crate::domain::media_timeline::Rounding;
fn pin(id: &str) -> ArtifactPin {
    ArtifactPin {
        artifact_id: id.into(),
        revision: DecimalU64(1),
        sha256: "a".repeat(64),
    }
}
fn document() -> ClipDocument {
    ClipDocument {
        schema_version: "1".into(),
        document_id: "manual-short".into(),
        revision: DecimalU64(1),
        sources: vec![ClipSourcePin {
            media_id: "source-1".into(),
            sha256: "b".repeat(64),
            duration_us: DecimalU64(60_000_000),
            width: 1920,
            height: 1080,
            clock: SourceClock {
                origin_pts: "-9000".into(),
                time_base_numerator: 1,
                time_base_denominator: 90000,
                observation_sha256: "c".repeat(64),
            },
            tracks: vec![
                SourceTrack {
                    stream_index: 0,
                    kind: SourceTrackKind::Video,
                    codec_id: "h264".into(),
                },
                SourceTrack {
                    stream_index: 1,
                    kind: SourceTrackKind::Audio,
                    codec_id: "aac".into(),
                },
            ],
        }],
        segments: vec![ClipSegment {
            segment_id: "segment-1".into(),
            media_id: "source-1".into(),
            range: SourceRange {
                start_us: DecimalU64(0),
                end_us: DecimalU64(30_000_000),
            },
            video_stream_index: 0,
            crop_keys: vec![CropKey {
                source_us: DecimalU64(0),
                crop: NormalizedCrop {
                    left: 0,
                    top: 0,
                    width: CROP_SCALE,
                    height: CROP_SCALE,
                },
                fit: FitMode::Fit,
            }],
            audio: AudioSelection::Selected {
                stream_index: 1,
                volume_percent: 100,
                muted: false,
            },
            subtitles: Some(SubtitleSelection {
                document: SourceBoundArtifact {
                    artifact: pin("cue-document"),
                    source_sha256: "b".repeat(64),
                },
                cue_ids: vec!["cue-1".into()],
                style: pin("common-subtitle-style"),
            }),
            subject_ref: None,
        }],
        preset: ClipOutputPreset {
            catalog: pin("proposed-sdr-preset"),
            width: 1080,
            height: 1920,
            frame_rate: FrameRate {
                numerator: 30000,
                denominator: 1001,
            },
            container: Container::Mp4,
            video_codec: VideoCodec::H264,
            audio_codec: Some(AudioCodec::Aac),
            color: OutputColor::SdrBt709,
        },
        cut_mode: CutMode::ExactReencode,
    }
}
#[test]
fn shared_project_round_trip_keeps_explicit_choices_and_integer_precision() {
    let mut value = document();
    value.validate().unwrap();
    assert_eq!(value.duration_us().unwrap().0, 30_000_000);
    value.sources[0].duration_us = DecimalU64(9_007_199_254_740_993);
    let bytes = serde_json::to_vec(&value).unwrap();
    let restored: ClipDocument = serde_json::from_slice(&bytes).unwrap();
    restored.validate().unwrap();
    assert_eq!(restored, value);
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    let mut unknown = serde_json::to_value(&value).unwrap();
    unknown["verified"] = serde_json::json!(true);
    assert!(serde_json::from_value::<ClipDocument>(unknown).is_err());
}
#[test]
fn half_open_ranges_and_output_duration_overflow_are_rejected() {
    for (start, end) in [(0, 0), (30_000_001, 30_000_000), (0, 60_000_001)] {
        let mut value = document();
        value.segments[0].range = SourceRange {
            start_us: DecimalU64(start),
            end_us: DecimalU64(end),
        };
        assert!(value.validate().is_err());
    }
    let mut value = document();
    value.sources[0].duration_us = DecimalU64(u64::MAX);
    value.segments[0].range.end_us = DecimalU64(u64::MAX);
    let mut second = value.segments[0].clone();
    second.segment_id = "segment-2".into();
    second.range.end_us = DecimalU64(1);
    value.segments.push(second);
    assert_eq!(value.validate(), Err("Clip output duration overflow"));
}
#[test]
fn crop_overflow_endpoints_and_stream_identity_do_not_silently_clamp() {
    for crop in [
        NormalizedCrop {
            left: u32::MAX,
            top: 0,
            width: 1,
            height: 1,
        },
        NormalizedCrop {
            left: 0,
            top: 0,
            width: 0,
            height: 1,
        },
    ] {
        assert!(crop.validate().is_err());
    }
    let mut value = document();
    let mut key = value.segments[0].crop_keys[0];
    key.source_us = value.segments[0].range.end_us;
    value.segments[0].crop_keys.push(key);
    assert!(value.validate().is_err());
    let mut value = document();
    value.segments[0].video_stream_index = 1;
    assert!(value.validate().is_err());
    let mut value = document();
    value.segments[0]
        .subtitles
        .as_mut()
        .unwrap()
        .document
        .source_sha256 = "d".repeat(64);
    assert!(value.validate().is_err());
}
#[test]
fn source_clock_uses_original_integer_transform_without_claiming_custody() {
    let value = document();
    let transform = value.sources[0].clock.transform().unwrap();
    assert_eq!(transform.pts_to_us(-9000, Rounding::Down).unwrap().value, 0);
    assert_eq!(
        transform.pts_to_us(0, Rounding::Down).unwrap().value,
        100_000
    );
    assert!(transform.pts_to_us(-9001, Rounding::Down).is_err());
    assert!(
        FrameRate {
            numerator: 60,
            denominator: 2
        }
        .validate()
        .is_err()
    );
    assert!(
        FrameRate {
            numerator: 0,
            denominator: 1
        }
        .validate()
        .is_err()
    );
    let mut clock = value.sources[0].clock.clone();
    clock.origin_pts = "-09000".into();
    assert!(clock.transform().is_err());
}
