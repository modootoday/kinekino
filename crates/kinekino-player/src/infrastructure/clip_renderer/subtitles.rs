//! Guarded common-font images bound to the original decoded/source/encoder owners.
use super::*;
use crate::infrastructure::clip_working_file::ClipWorkingFile;
use crate::{
    application::clip_render::subtitles::{PreparedClipSubtitles, frames::ClipSubtitleRun},
    infrastructure::clip_subtitle_assets::{NativeSubtitleDocument, NativeSubtitleStyle},
};
#[cfg(feature = "native-proposal-window")]
use crate::{
    application::clip_render::{
        manual_clip_plan_with_subtitles, subtitles::frames::active_subtitle_runs,
    },
    infrastructure::clip_subtitle_raster,
};
use std::{ffi::OsString, path::Path};
type Admitted = PreparedClipSubtitles<NativeSubtitleDocument, NativeSubtitleStyle>;
#[derive(Clone, Serialize)]
pub struct ClipOverlayFacts {
    pub input_transport: String,
    pub working_input_sha256: Option<String>,
    pub recipe: String,
    pub decoded_window_sha256: String,
    pub frame_spans: serde_json::Value,
    pub omitted_outside_clip: Vec<String>,
    pub runs: Vec<serde_json::Value>,
    pub logical_buffer_bytes: u64,
    pub actual_peak_rss_verified: bool,
    pub quality_approved: bool,
}
struct Image {
    run: ClipSubtitleRun,
}
pub struct PreparedClipOverlay<'a, 'source> {
    pub(super) plan: ManualClipPlan,
    pub(super) media: &'a PreparedClipMedia<NativeClipSourceLease>,
    pub(super) encoder: &'a PreparedClipEncoder<PinnedClipEncoder>,
    pub(super) boundaries: &'a NativeClipBoundaryLease<'source>,
    _subtitles: &'a Admitted,
    images: Vec<Image>,
    facts: ClipOverlayFacts,
    working: Option<ClipWorkingFile>,
}
pub(crate) struct RetainedClipOverlay {
    facts: ClipOverlayFacts,
    _images: Vec<Image>,
    _working: Option<ClipWorkingFile>,
}
impl RetainedClipOverlay {
    pub(crate) fn take_working_release(
        &mut self,
    ) -> Option<crate::infrastructure::clip_working_file::ClipWorkingFileRelease> {
        self._working.take().map(ClipWorkingFile::into_release)
    }
    pub(crate) fn facts(&self) -> &ClipOverlayFacts {
        &self.facts
    }
}
impl PreparedClipOverlay<'_, '_> {
    pub(crate) fn into_retained(self) -> RetainedClipOverlay {
        RetainedClipOverlay {
            facts: self.facts,
            _images: self.images,
            _working: self.working,
        }
    }
    pub fn facts(&self) -> &ClipOverlayFacts {
        &self.facts
    }
    pub(super) fn append_inputs(&self, args: &mut Vec<OsString>) {
        if self.working.is_some() {
            args.extend(
                [
                    "-threads",
                    "1",
                    "-protocol_whitelist",
                    "file,fd",
                    "-f",
                    "image2pipe",
                    "-c:v",
                    "png",
                    "-framerate",
                    "1",
                    "-fd",
                    "0",
                    "-i",
                    "fd:",
                ]
                .into_iter()
                .map(Into::into),
            );
        }
    }
    pub(super) fn with_input<T>(
        &self,
        callback: impl FnOnce(Option<std::fs::File>) -> Result<T, ApiError>,
    ) -> Result<T, ApiError> {
        match &self.working {
            Some(working) => working.with_reader(|file| callback(Some(file))),
            None => callback(None),
        }
    }
    pub(super) fn finish_graph(&self, mut graph: String, plan: &ManualClipPlan) -> String {
        let mut previous = "subtitle-base".to_owned();
        if !self.images.is_empty() {
            graph.push_str(&format!(";[1:v]split={}", self.images.len()));
            for index in 0..self.images.len() {
                graph.push_str(&format!("[subtitle-source-{index}]"));
            }
        }
        for (index, image) in self.images.iter().enumerate() {
            let selected = format!("subtitle-selected-{index}");
            // Generic enable n is zero-based. A real pinned-engine pixel test binds it.
            graph.push_str(&format!(";[subtitle-source-{index}]select=\'eq(n,{index})\',format=rgba,setpts=PTS-STARTPTS[subtitle-image-{index}];[{previous}][subtitle-image-{index}]overlay=x=0:y=0:format=rgb:alpha=straight:shortest=0:eof_action=repeat:repeatlast=1:enable='gte(n,{})*lt(n,{})'[{selected}]",image.run.first_frame.0,image.run.end_frame.0));
            previous = selected;
        }
        graph.push_str(&format!(
            ";[{previous}]format=yuv420p,fps={}/{}[v]",
            plan.fps_numerator, plan.fps_denominator
        ));
        graph
    }
}
#[cfg(not(feature = "native-proposal-window"))]
pub fn prepare_subtitle_overlay<'a, 'source>(
    _media: &'a PreparedClipMedia<NativeClipSourceLease>,
    _encoder: &'a PreparedClipEncoder<PinnedClipEncoder>,
    _boundaries: &'a NativeClipBoundaryLease<'source>,
    _subtitles: &'a Admitted,
    _temporary: &Path,
    control: &dyn MutationControl,
) -> Result<PreparedClipOverlay<'a, 'source>, ApiError> {
    control.checkpoint()?;
    Err(ApiError::new(
        ErrorCode::Unsupported,
        "Common-font subtitle rendering requires native-proposal-window",
    ))
}
#[cfg(feature = "native-proposal-window")]
pub fn prepare_subtitle_overlay<'a, 'source>(
    media: &'a PreparedClipMedia<NativeClipSourceLease>,
    encoder: &'a PreparedClipEncoder<PinnedClipEncoder>,
    boundaries: &'a NativeClipBoundaryLease<'source>,
    subtitles: &'a Admitted,
    temporary: &Path,
    control: &dyn MutationControl,
) -> Result<PreparedClipOverlay<'a, 'source>, ApiError> {
    control.checkpoint()?;
    let plan = manual_clip_plan_with_subtitles(
        media,
        encoder,
        &boundaries.observation().boundaries,
        subtitles,
    )?;
    if !media
        .sources()
        .iter()
        .any(|source| std::ptr::eq(source, boundaries.source()))
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Subtitle decoder must retain the original prepared media owner",
        ));
    }
    let segment = &media.publication().document.segments[0];
    let spans = boundaries.subtitle_frames(segment, subtitles, control)?;
    let runs = active_subtitle_runs(
        &spans,
        boundaries.observation().boundaries.included_frames.0,
        control,
    )?;
    let logical_buffer_bytes = u64::from(plan.width)
        * u64::from(plan.height)
        * 4
        * (3 * plan.crop_sections.len() as u64 + 3 * runs.len() as u64 + 4);
    if logical_buffer_bytes > 512 * 1024 * 1024 {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Subtitle/crop logical buffer budget exceeded",
        ));
    }
    let mut png_bytes = Vec::new();
    let mut images = Vec::new();
    let mut records = Vec::new();
    let mut total_bytes = 0usize;
    for run in runs {
        control.checkpoint()?;
        let image = clip_subtitle_raster::render_cues(
            subtitles,
            run.cue_indices(),
            plan.width,
            plan.height,
            control,
        )?;
        total_bytes = total_bytes.checked_add(image.png().len()).ok_or_else(|| {
            ApiError::new(
                ErrorCode::ResourceExhausted,
                "Subtitle image byte budget overflow",
            )
        })?;
        if total_bytes > 64 * 1024 * 1024 {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Subtitle images exceed 64 MiB",
            ));
        }
        png_bytes.extend_from_slice(image.png());
        records.push(serde_json::json!({"frames":run,"image":image.facts()}));
        images.push(Image { run });
    }
    control.checkpoint()?;
    let (working, working_input_sha256) = if png_bytes.is_empty() {
        (None, None)
    } else {
        let mut working = ClipWorkingFile::create(temporary, 64 * 1024 * 1024, control)?;
        let facts = working.seal(&png_bytes, control)?;
        if facts.bytes != png_bytes.len() as u64
            || facts.sha256 != format!("{:x}", Sha256::digest(&png_bytes))
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Subtitle working input differs from original PNG sequence",
            ));
        }
        (Some(working), Some(facts.sha256))
    };
    let facts = ClipOverlayFacts {
        recipe: "source-frame-common-font-overlay/2".into(),
        input_transport: "owned_regular_stdin_png_sequence".into(),
        working_input_sha256,
        decoded_window_sha256: boundaries.observation().raw_frame_window_sha256.clone(),
        frame_spans: serde_json::to_value(&spans)
            .map_err(|_| ApiError::new(ErrorCode::Internal, "Invalid subtitle span record"))?,
        omitted_outside_clip: subtitles.omitted_outside_clip().to_vec(),
        runs: records,
        logical_buffer_bytes,
        actual_peak_rss_verified: false,
        quality_approved: false,
    };
    Ok(PreparedClipOverlay {
        plan,
        media,
        encoder,
        boundaries,
        _subtitles: subtitles,
        images,
        facts,
        working,
    })
}
pub fn encode_candidate_with_subtitles(
    overlay: &PreparedClipOverlay<'_, '_>,
    color: &DeclaredBt709Limited,
    timeout: Duration,
    max_output_bytes: usize,
    control: &dyn MutationControl,
) -> Result<EncodedClipCandidate, ApiError> {
    super::encode_plan(super::CandidateEncoding {
        media: overlay.media,
        encoder: overlay.encoder,
        boundaries: overlay.boundaries,
        color,
        plan: overlay.plan.clone(),
        overlay: Some(overlay),
        timeout,
        max_output_bytes,
        control,
    })
}
#[cfg(all(test, feature = "native-proposal-window"))]
mod tests;
