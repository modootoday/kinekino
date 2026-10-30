//! Product preparation from actual producers, never an imported authority DTO.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        review_playback::inspect_review_input,
        review_source::{MutationControl, ReviewCaseEvidence},
    },
    domain::review_source::{MediaReference, ReviewCase},
    infrastructure::{
        model_review::ReviewFiles,
        review_content_classification::LocalReviewContentClassifier,
        timeline_evidence::{
            production::{
                PcmClockRange, PendingTimelineProcess, PinnedTimelineAudioDecoder,
                PinnedTimelineCaptureDecoder, PinnedTimelineProbe, TimelineBundleInputs,
                TimelineCapturePoint, TimelineProcessFailure, TimelineProductionConfig,
                TimelineProductionSession, VideoProbeWindow,
            },
            publication::{PublishedTimelinePackage, TimelinePackageDestination, publish},
        },
    },
};
use std::{
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

/// Explicit pre-opened and hash-pinned tools. No PATH discovery or download.
pub struct TimelinePreparationTools {
    pub video: PinnedTimelineProbe,
    pub audio: PinnedTimelineAudioDecoder,
    pub capture: PinnedTimelineCaptureDecoder,
}
/// Selection and limits are configuration, not evidence or an authority grant.
pub struct TimelinePreparation {
    pub source_root: PathBuf,
    pub case: ReviewCase,
    pub media: MediaReference,
    pub runtime: TimelineProductionConfig,
    pub destination: TimelinePackageDestination,
    pub video_window: VideoProbeWindow,
    pub pcm_range: PcmClockRange,
    pub start_us: u64,
    pub end_us: u64,
}
/// If bounded observation expires, this callback must retain the original stack
/// and owner. It cannot return a fabricated cleanup success or replacement PID.
pub trait IncompleteTimelineCleanup {
    fn incomplete(
        &mut self,
        original_error: &ApiError,
        pending: &mut PendingTimelineProcess<'_, '_>,
    ) -> !;
}

fn physically_complete<T>(
    result: Result<T, Box<TimelineProcessFailure<'_, '_>>>,
    incomplete: &mut impl IncompleteTimelineCleanup,
) -> Result<T, ApiError> {
    match result {
        Ok(value) => Ok(value),
        Err(mut failure) => {
            if let Some(pending) = failure.pending.as_mut() {
                // A cleanup observation budget, not a renewed operation deadline.
                let cutoff = Instant::now() + Duration::from_secs(10);
                loop {
                    if pending.poll_cleanup() {
                        break;
                    }
                    if Instant::now() >= cutoff {
                        incomplete.incomplete(&failure.error, pending);
                    }
                    thread::sleep(Duration::from_millis(20));
                }
            }
            Err(failure.error)
        }
    }
}

/// Serial video/decode/replay/capture/bundle/publication. All actual component
/// leases coexist until publish has copied and committed their original bytes.
/// The returned private token is created solely by the actual publisher.
pub fn prepare_timeline(
    config: TimelinePreparation,
    tools: &TimelinePreparationTools,
    control: &dyn MutationControl,
    incomplete: &mut impl IncompleteTimelineCleanup,
) -> Result<PublishedTimelinePackage, ApiError> {
    control.checkpoint()?;
    if config.start_us >= config.end_us
        || config.end_us > 86_400_000_000
        || !(1..=4_294_967_296).contains(&config.destination.max_total_bytes)
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Explicit positive capture range required",
        ));
    }
    // Reject root conflicts before creating any producer child or output.
    let mut roots = Vec::new();
    let mut guards = Vec::new();
    for path in [
        &config.source_root,
        &config.runtime.runtime_root,
        &config.destination.root,
    ]
    .into_iter()
    .chain(config.destination.excluded_roots.iter())
    {
        let (root, retained) = crate::infrastructure::immutable_storage_paths::root(path)?;
        roots.push(PathBuf::from(root.to_string_lossy().to_lowercase()));
        guards.extend(retained);
    }
    for (index, root) in roots.iter().enumerate() {
        if roots
            .iter()
            .skip(index + 1)
            .any(|other| root.starts_with(other) || other.starts_with(root))
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Preparation roots must be disjoint",
            ));
        }
    }
    let files = ReviewFiles::open(&config.source_root)?;
    let reader = ReviewCaseEvidence::new(ReviewFiles::open(&config.source_root)?);
    let selected = inspect_review_input(
        &reader,
        &config.case,
        &config.media,
        &LocalReviewContentClassifier,
    )?;
    let source = files.bind_playback_media(selected)?;
    let session = TimelineProductionSession::reserve(&source, config.runtime)?;
    let video = physically_complete(
        session.video(&tools.video, config.video_window, control),
        incomplete,
    )?;
    let pcm = physically_complete(session.audio_pcm(&tools.audio, control), incomplete)?;
    let lineage = physically_complete(
        session.pcm_lineage(&pcm, &tools.audio, config.pcm_range, control),
        incomplete,
    )?;
    let origin = physically_complete(
        session.capture_frame(
            &tools.capture,
            &video,
            TimelineCapturePoint::Origin,
            control,
        ),
        incomplete,
    )?;
    let start = physically_complete(
        session.capture_frame(
            &tools.capture,
            &video,
            TimelineCapturePoint::AtOrAfter {
                source_us: config.start_us,
            },
            control,
        ),
        incomplete,
    )?;
    let end = physically_complete(
        session.capture_frame(
            &tools.capture,
            &video,
            TimelineCapturePoint::AtOrAfter {
                source_us: config.end_us,
            },
            control,
        ),
        incomplete,
    )?;
    let bundle = session.bundle(
        TimelineBundleInputs {
            video: &video,
            pcm: &lineage,
            origin: &origin,
            start: &start,
            end: &end,
        },
        control,
    )?;
    publish(&bundle, &files, config.destination, control)
}
