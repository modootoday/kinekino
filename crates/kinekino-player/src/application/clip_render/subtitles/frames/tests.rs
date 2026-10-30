use super::*;
use serde_json::json;

#[test]
fn active_sets_preserve_overlap_order_exact_boundaries_and_empty_gaps() {
    let span = |id: &str, a, b| ClipSubtitleFrameSpan {
        cue_id: id.into(),
        start_threshold_pts: "0".into(),
        end_threshold_pts: "0".into(),
        first_frame: DecimalU64(a),
        end_frame: DecimalU64(b),
    };
    let spans = [
        span("a", 0, 3),
        span("b", 2, 5),
        span("empty", 1, 1),
        span("c", 7, 8),
    ];
    let runs = active_subtitle_runs(&spans, 8, &Control).unwrap();
    let values: Vec<_> = runs
        .iter()
        .map(|run| {
            (
                run.first_frame.0,
                run.end_frame.0,
                run.cue_indices().to_vec(),
            )
        })
        .collect();
    assert_eq!(
        values,
        vec![
            (0, 2, vec![0]),
            (2, 3, vec![0, 1]),
            (3, 5, vec![1]),
            (7, 8, vec![3])
        ]
    );
    assert!(active_subtitle_runs(&[span("a", 4, 3)], 8, &Control).is_err());
    assert!(active_subtitle_runs(&[span("a", 0, 9)], 8, &Control).is_err());
    assert!(active_subtitle_runs(&[span("a", 0, 1), span("a", 2, 3)], 8, &Control).is_err());
}

struct Document(ClipSubtitleDocument);
impl ClipSubtitleDocumentLease for Document {
    fn document(&self) -> &ClipSubtitleDocument {
        &self.0
    }
}
struct Style(ClipSubtitleStyle);
impl ClipSubtitleStyleLease for Style {
    fn style(&self) -> &ClipSubtitleStyle {
        &self.0
    }
    fn fonts(&self) -> &[ClipSubtitleFontPin] {
        &[]
    }
}
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
// Synthetic application owners only: no font bytes, admission or native decoder claim.
fn fixture() -> (
    ClipSourcePin,
    ClipSegment,
    PreparedClipSubtitles<Document, Style>,
) {
    let source: ClipSourcePin = serde_json::from_value(json!({"media_id":"media","sha256":"a".repeat(64),"duration_us":"60000000","width":16,"height":16,"clock":{"origin_pts":"-9000","time_base_numerator":1,"time_base_denominator":90000,"observation_sha256":"b".repeat(64)},"tracks":[{"stream_index":0,"kind":"video","codec_id":"h264"}]})).unwrap();
    let selection = json!({"document":{"artifact":{"artifact_id":"captions","revision":"1","sha256":"c".repeat(64)},"source_sha256":source.sha256},"cue_ids":["first","overlap","gap"],"style":{"artifact_id":"style","revision":"1","sha256":"d".repeat(64)}});
    let segment: ClipSegment = serde_json::from_value(json!({"segment_id":"one","media_id":"media","range":{"start_us":"0","end_us":"1000000"},"video_stream_index":0,"crop_keys":[{"source_us":"0","crop":{"left":0,"top":0,"width":1000000,"height":1000000},"fit":"fit"}],"audio":{"kind":"disabled"},"subtitles":selection,"subject_ref":null})).unwrap();
    let document = Document(serde_json::from_value(json!({"schema_version":"clip-subtitle-document/1","media":source,"time_basis":"media_relative_us","origin":"explicit_manual_snapshot","cues":{"source_id":"captions","source_revision":"1","source_sha256":"c".repeat(64),"cues":[]}})).unwrap());
    let style = Style(serde_json::from_value(json!({"schema_version":"clip-subtitle-style/1","style_id":"style","revision":"1","fonts":[{"asset_path":"fixture.ttf","sha256":"d".repeat(64)}],"font_px":24,"line_height_px":32,"box_normalized":{"left":0,"top":0,"width":1000000,"height":1000000},"foreground_rgba":[255,255,255,255],"background_rgba":[0,0,0,0],"alignment":"center","max_lines":3})).unwrap());
    let cues = [
        ("first", 0, 500000),
        ("overlap", 250000, 1000000),
        ("gap", 1, 2),
    ]
    .into_iter()
    .map(|(id, start, end)| SelectedClipSubtitleCue {
        cue_id: id.into(),
        text: id.into(),
        source_start_us: DecimalU64(start),
        source_end_us: DecimalU64(end),
        output_start_us: DecimalU64(start),
        output_end_us: DecimalU64(end),
    })
    .collect();
    let owner = PreparedClipSubtitles {
        segment: segment.clone(),
        selection: segment.subtitles.clone().unwrap(),
        document,
        style,
        cues,
        omitted_outside_clip: vec![],
    };
    (source, segment, owner)
}
#[test]
fn negative_origin_vfr_overlap_and_subframe_cue_keep_half_open_source_offsets() {
    let (source, segment, owner) = fixture();
    let spans = map_subtitle_frames(
        &source,
        &segment,
        &[-9000, -4500, 13500, 36000, 72000, 81000],
        &owner,
        &Control,
    )
    .unwrap();
    let offsets: Vec<_> = spans
        .iter()
        .map(|s| (s.cue_id.as_str(), s.first_frame.0, s.end_frame.0))
        .collect();
    assert_eq!(
        offsets,
        vec![("first", 0, 3), ("overlap", 2, 5), ("gap", 1, 1)]
    );
    assert_eq!(spans[0].end_threshold_pts, "36000");
    assert_eq!(spans[1].end_threshold_pts, "81000");
    assert_eq!(spans[2].start_threshold_pts, "-8999");
    assert_eq!(spans[2].end_threshold_pts, "-8999");
}
#[test]
fn stale_media_segment_and_uncovered_or_unordered_window_are_rejected() {
    let (source, segment, owner) = fixture();
    let pts = [-9000, 36000, 81000];
    let mut stale = source.clone();
    stale.clock.observation_sha256 = "e".repeat(64);
    assert_eq!(
        map_subtitle_frames(&stale, &segment, &pts, &owner, &Control)
            .unwrap_err()
            .code,
        ErrorCode::SourceExpired
    );
    let mut changed = segment.clone();
    changed.range.end_us.0 -= 1;
    assert_eq!(
        map_subtitle_frames(&source, &changed, &pts, &owner, &Control)
            .unwrap_err()
            .code,
        ErrorCode::SourceExpired
    );
    for invalid_pts in [
        vec![-9000, 0],
        vec![-9000, -9000, 81000],
        vec![0, 36000, 81000],
    ] {
        assert!(map_subtitle_frames(&source, &segment, &invalid_pts, &owner, &Control).is_err());
    }
    struct Cancel;
    impl MutationControl for Cancel {
        fn checkpoint(&self) -> Result<(), ApiError> {
            Err(ApiError::new(ErrorCode::Cancelled, "fixture"))
        }
    }
    assert_eq!(
        map_subtitle_frames(&source, &segment, &pts, &owner, &Cancel)
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
}
