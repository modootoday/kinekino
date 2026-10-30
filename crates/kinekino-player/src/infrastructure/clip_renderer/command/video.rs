use super::*;
use crate::application::clip_render::{ManualClipCropSection, ManualClipPlan};
fn crop_filter(section: &ManualClipCropSection, plan: &ManualClipPlan) -> String {
    let crop = &section.crop;
    let sizing = match section.fit {
        FitMode::Fit => format!(
            "scale={}:{}:force_original_aspect_ratio=decrease:force_divisible_by=2:in_color_matrix=bt709:out_color_matrix=bt709:in_range=tv:out_range=tv,pad={}:{}:(ow-iw)/2:(oh-ih)/2:color=black",
            plan.width, plan.height, plan.width, plan.height
        ),
        FitMode::Fill => format!(
            "scale={}:{}:force_original_aspect_ratio=increase:force_divisible_by=2:in_color_matrix=bt709:out_color_matrix=bt709:in_range=tv:out_range=tv,crop={}:{}",
            plan.width, plan.height, plan.width, plan.height
        ),
    };
    format!(
        "crop={}:{}:{}:{}:exact=1,{},setsar=1",
        crop.width, crop.height, crop.left, crop.top, sizing
    )
}
pub(super) fn graph(plan: &ManualClipPlan) -> Result<String, ApiError> {
    graph_with_overlay(plan, None)
}
pub(super) fn graph_with_overlay(
    plan: &ManualClipPlan,
    overlay: Option<&super::super::subtitles::PreparedClipOverlay<'_, '_>>,
) -> Result<String, ApiError> {
    let sections = &plan.crop_sections;
    if sections.is_empty()
        || sections.len() > crate::domain::clip_document::MAX_CROP_KEYS
        || sections[0].frame_offset.0 != 0
        || sections
            .windows(2)
            .any(|pair| pair[0].frame_offset.0 > pair[1].frame_offset.0)
        || sections.iter().any(|section| {
            section.frame_offset.0
                > crate::application::clip_render::MAX_CLIP_BOUNDARY_FRAMES as u64
        })
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Ordered decoded section crop plan required",
        ));
    }
    let prefix = format!(
        "[0:{}]trim=start_pts={}:end_pts={},setpts=PTS-STARTPTS",
        plan.video_stream_index, plan.start_pts, plan.end_pts
    );
    let color = "setparams=range=limited:color_primaries=bt709:color_trc=bt709:colorspace=bt709";
    if sections.len() == 1 {
        if let Some(overlay) = overlay {
            return Ok(overlay.finish_graph(
                format!(
                    "{prefix},{},{color},format=rgba[subtitle-base]",
                    crop_filter(&sections[0], plan)
                ),
                plan,
            ));
        }
        return Ok(format!(
            "{prefix},{},{color},format=yuv420p,fps={}/{}[v]",
            crop_filter(&sections[0], plan),
            plan.fps_numerator,
            plan.fps_denominator
        ));
    }
    let mut graph = format!("{prefix},split={}", sections.len());
    for i in 0..sections.len() {
        graph.push_str(&format!("[source{i}]"));
    }
    for (i, section) in sections.iter().enumerate() {
        graph.push_str(&format!(
            ";[source{i}]{},format=yuv420p[crop{i}]",
            crop_filter(section, plan)
        ));
    }
    let mut previous = "crop0".to_owned();
    for (i, section) in sections.iter().enumerate().skip(1) {
        // FFmpeg's input frame_count_out is one-based here. The actual pixel
        // transition test must confirm this binding with the pinned engine.
        let label = format!("selected{i}");
        graph.push_str(&format!(";[{previous}][crop{i}]blend=all_expr='if(lte(N,{}),A,B)':shortest=1:repeatlast=0[{label}]", section.frame_offset.0));
        previous = label;
    }
    if let Some(overlay) = overlay {
        graph.push_str(&format!(";[{previous}]{color},format=rgba[subtitle-base]"));
        return Ok(overlay.finish_graph(graph, plan));
    }
    graph.push_str(&format!(
        ";[{previous}]{color},fps={}/{}[v]",
        plan.fps_numerator, plan.fps_denominator
    ));
    Ok(graph)
}
