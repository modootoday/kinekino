//! Original decoded-frame membership, before output fps conversion. No decoder authority.
use super::*;
use crate::application::clip_render::evaluate_frame_boundaries;
use crate::domain::media_timeline::Rounding;

#[derive(Debug, Serialize)]
pub struct ClipSubtitleFrameSpan {
    pub cue_id: String,
    pub start_threshold_pts: String,
    pub end_threshold_pts: String,
    /// Zero-based half-open offsets into the included source frames. Equal means no frame.
    pub first_frame: DecimalU64,
    pub end_frame: DecimalU64,
}

#[derive(Debug, Serialize)]
pub struct ClipSubtitleRun {
    pub first_frame: DecimalU64,
    pub end_frame: DecimalU64,
    cue_indices: Vec<usize>,
}
impl ClipSubtitleRun {
    pub fn cue_indices(&self) -> &[usize] {
        &self.cue_indices
    }
}

/// Facts only: the native renderer must retain the original source/decoded owner.
pub fn active_subtitle_runs(
    spans: &[ClipSubtitleFrameSpan],
    included_frames: u64,
    control: &dyn MutationControl,
) -> Result<Vec<ClipSubtitleRun>, ApiError> {
    control.checkpoint()?;
    if spans.len() > crate::domain::clip_document::MAX_SELECTED_CUES
        || included_frames == 0
        || included_frames > crate::application::clip_render::MAX_CLIP_BOUNDARY_FRAMES as u64
    {
        return Err(invalid("Bounded subtitle frame spans required"));
    }
    let mut ids = BTreeSet::new();
    let mut events = BTreeSet::from([0, included_frames]);
    for span in spans {
        control.checkpoint()?;
        if !valid_id(&span.cue_id)
            || !ids.insert(&span.cue_id)
            || span.first_frame.0 > span.end_frame.0
            || span.end_frame.0 > included_frames
        {
            return Err(invalid(
                "Distinct ordered half-open subtitle spans required",
            ));
        }
        if span.first_frame != span.end_frame {
            events.insert(span.first_frame.0);
            events.insert(span.end_frame.0);
        }
    }
    let events: Vec<_> = events.into_iter().collect();
    let mut runs = Vec::new();
    for pair in events.windows(2) {
        control.checkpoint()?;
        let active: Vec<_> = spans
            .iter()
            .enumerate()
            .filter_map(|(index, span)| {
                (span.first_frame.0 <= pair[0] && pair[0] < span.end_frame.0).then_some(index)
            })
            .collect();
        if !active.is_empty() {
            runs.push(ClipSubtitleRun {
                first_frame: DecimalU64(pair[0]),
                end_frame: DecimalU64(pair[1]),
                cue_indices: active,
            });
        }
    }
    Ok(runs)
}

pub fn map_subtitle_frames<D: ClipSubtitleDocumentLease, S: ClipSubtitleStyleLease>(
    source: &ClipSourcePin,
    segment: &ClipSegment,
    decoded_pts: &[i64],
    owner: &PreparedClipSubtitles<D, S>,
    control: &dyn MutationControl,
) -> Result<Vec<ClipSubtitleFrameSpan>, ApiError> {
    control.checkpoint()?;
    owner.require_binding(source, segment)?;
    let boundary = evaluate_frame_boundaries(source, segment, decoded_pts)?;
    let first_pts = boundary
        .first_included_pts
        .parse::<i64>()
        .map_err(|_| invalid("Invalid evaluated first frame PTS"))?;
    let first = decoded_pts.partition_point(|pts| *pts < first_pts);
    let included = &decoded_pts[first..first + boundary.included_frames.0 as usize];
    let timeline = source.clock.transform().map_err(invalid)?;
    let mut spans = Vec::with_capacity(owner.cues().len());
    for cue in owner.cues() {
        control.checkpoint()?;
        let start = cue.source_start_us.0.max(segment.range.start_us.0);
        let end = cue.source_end_us.0.min(segment.range.end_us.0);
        let threshold = |time| {
            timeline
                .us_to_pts(time, Rounding::Up)
                .map(|value| value.value)
                .map_err(|_| ApiError::new(ErrorCode::NeedsInput, "Subtitle timestamp overflow"))
        };
        let a = threshold(start)?;
        let b = threshold(end)?;
        spans.push(ClipSubtitleFrameSpan {
            cue_id: cue.cue_id.clone(),
            start_threshold_pts: a.to_string(),
            end_threshold_pts: b.to_string(),
            first_frame: DecimalU64(included.partition_point(|pts| *pts < a) as u64),
            end_frame: DecimalU64(included.partition_point(|pts| *pts < b) as u64),
        });
    }
    control.checkpoint()?;
    Ok(spans)
}

#[cfg(test)]
mod tests;
