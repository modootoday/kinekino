use super::*;
use crate::application::clip_render::{ClipPixelCrop, ManualClipAudio};
use crate::domain::clip_document::FitMode;
use serde_json::{Value, json};
fn atom(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut data = ((payload.len() + 8) as u32).to_be_bytes().to_vec();
    data.extend(kind);
    data.extend(payload);
    data
}
#[test]
fn original_movie_header_rejects_fragmented_missing_duplicate_and_truncated_boxes() {
    let mut payload = vec![0; 20];
    payload[12..16].copy_from_slice(&1000u32.to_be_bytes());
    payload[16..20].copy_from_slice(&1000u32.to_be_bytes());
    let movie = atom(b"moov", &atom(b"mvhd", &payload));
    let mut data = atom(b"ftyp", b"isom");
    data.extend(&movie);
    assert_eq!(movie::header(&data).unwrap(), (1000, 1000));
    let mut fragmented = data.clone();
    fragmented.extend(atom(b"moof", b""));
    assert!(movie::header(&fragmented).is_err());
    let mut duplicate = data.clone();
    duplicate.extend(&movie);
    assert!(movie::header(&duplicate).is_err());
    assert!(movie::header(&data[..data.len() - 1]).is_err());
    assert!(movie::header(&atom(b"ftyp", b"isom")).is_err());
    let mut v1 = vec![0; 32];
    v1[0] = 1;
    v1[20..24].copy_from_slice(&48000u32.to_be_bytes());
    v1[24..32].copy_from_slice(&48000u64.to_be_bytes());
    let mut wide = atom(b"ftyp", b"isom");
    wide.extend(atom(b"moov", &atom(b"mvhd", &v1)));
    assert_eq!(movie::header(&wide).unwrap(), (48000, 48000));
}
fn plan() -> ManualClipPlan {
    ManualClipPlan {
        document_sha256: "a".repeat(64),
        source_media_id: "source".into(),
        source_sha256: "b".repeat(64),
        source_observation_sha256: "c".repeat(64),
        segment_id: "one".into(),
        video_stream_index: 0,
        start_pts: 15000,
        end_pts: 16000,
        crop_sections: vec![crate::application::clip_render::ManualClipCropSection {
            source_us: crate::domain::api_values::DecimalU64(15000000),
            threshold_pts: "15000".into(),
            frame_offset: crate::domain::api_values::DecimalU64(0),
            crop: ClipPixelCrop {
                left: 0,
                top: 0,
                width: 1280,
                height: 544,
            },
            fit: FitMode::Fit,
        }],
        width: 1080,
        height: 1920,
        fps_numerator: 30,
        fps_denominator: 1,
        duration_us: 1_000_000,
        input_seek_start_us: 13_000_000,
        audio: Some(ManualClipAudio {
            stream_index: 1,
            volume_percent: 75,
            muted: false,
            absolute_start_us: 15_000_000,
            absolute_end_us: 16_000_000,
        }),
    }
}
fn metadata() -> Value {
    json!({"streams":[{"codec_type":"video","codec_name":"h264","width":1080,"height":1920,"pix_fmt":"yuv420p","sample_aspect_ratio":"1:1","color_range":"tv","color_space":"bt709","color_transfer":"bt709","color_primaries":"bt709","nb_read_frames":"30","r_frame_rate":"30/1","start_pts":0,"time_base":"1/15360","duration_ts":15360},{"codec_type":"audio","codec_name":"aac","start_pts":0,"time_base":"1/48000","duration_ts":48000,"sample_rate":"48000","channels":2}],"format":{"start_time":"0.000000","duration":"1.000000"}})
}
#[test]
fn output_metadata_checks_frame_audio_movie_clock_and_declared_tags_without_color_approval() {
    let plan = plan();
    let actual = metadata::validate(&plan, &metadata(), 1000, 1000).unwrap();
    assert_eq!(actual.video_frames.0, 30);
    assert_eq!(actual.audio_duration_us.unwrap().0, 1_000_000);
    for fault in 0..7 {
        let mut raw = metadata();
        match fault {
            0 => raw["streams"][0]["nb_read_frames"] = json!("29"),
            1 => raw["streams"][1]["duration_ts"] = json!(49024),
            2 => raw["format"]["duration"] = json!("1.066667"),
            3 => raw["streams"][0]["start_pts"] = json!(1024),
            4 => raw["streams"][0]["color_transfer"] = json!("smpte2084"),
            5 => raw["streams"]
                .as_array_mut()
                .unwrap()
                .push(json!({"codec_type":"subtitle"})),
            _ => raw["streams"][0]["r_frame_rate"] = json!("24/1"),
        };
        assert!(
            metadata::validate(&plan, &raw, 1000, 1000).is_err(),
            "fault {fault}"
        );
    }
}
