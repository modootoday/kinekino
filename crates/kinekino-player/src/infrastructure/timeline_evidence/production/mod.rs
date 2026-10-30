//! Live producer components. Partial observations never grant Verified authority.
mod audio;
mod bundle;
mod capture;
mod pcm;
mod pcm_lineage;
mod process;
mod tool;
mod video;
use crate::{
    application::api::{ApiError, ErrorCode},
    infrastructure::{immutable_storage_paths as rooted, model_review::GuardedReviewMedia},
};
pub use audio::ProducedAudioClock;
pub use audio::ProducedAudioPcm;
pub(crate) use bundle::ProducedArtifact;
pub use bundle::{ProducedTimelineBundle, TimelineBundleInputs};
pub use capture::{ProducedTimelineCapture, TimelineCapturePoint};
use kinekino_process::{domain::deadline::Deadline, infrastructure::family::JobBudget};
pub use pcm::RetainedProducedPcm;
pub use pcm_lineage::{PcmClockRange, ProducedPcmLineage};
pub use process::{PendingTimelineProcess, TimelineProcessFailure};
use std::{
    cell::Cell,
    fs::{File, OpenOptions},
    marker::PhantomData,
    path::{Path, PathBuf},
    rc::Rc,
    time::{Duration, Instant},
};
pub use tool::{PinnedTimelineAudioDecoder, PinnedTimelineCaptureDecoder, PinnedTimelineProbe};
pub use video::{PendingVideoProbe, ProducedVideoFrames, VideoProbeFailure, VideoProbeWindow};

/// Trusted local configuration only; source identity comes from its retained lease.
pub struct TimelineProductionConfig {
    pub runtime_root: PathBuf,
    pub budget: JobBudget,
    pub total: Duration,
    pub cleanup: Duration,
    pub max_source_bytes: u64,
    pub max_output_bytes: u64,
}
/// One serial producer slot and original deadline shared by every component.
pub struct TimelineProductionSession<'a> {
    source: &'a GuardedReviewMedia,
    root: PathBuf,
    budget: JobBudget,
    deadline: Deadline,
    max_output_bytes: u64,
    active: Cell<u8>,
    _guards: Vec<File>,
    _slot: File,
    _same_thread: PhantomData<Rc<()>>,
}
impl<'a> TimelineProductionSession<'a> {
    pub fn reserve(
        source: &'a GuardedReviewMedia,
        config: TimelineProductionConfig,
    ) -> Result<Self, ApiError> {
        if config.max_source_bytes == 0
            || source.identity().bytes > config.max_source_bytes
            || !(1..=64 * 1024 * 1024).contains(&config.max_output_bytes)
        {
            return Err(invalid("Timeline production resource bounds differ"));
        }
        let deadline =
            Deadline::new(Instant::now(), config.total, config.cleanup).map_err(io_error)?;
        source.check_current()?;
        let (root, guards) = rooted::root(&config.runtime_root)?;
        if root.join("ABANDONED.json").exists() {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Timeline producer has unresolved abandoned ownership",
            ));
        }
        let source_root = source
            .path()
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| invalid("Source storage root missing"))?;
        if overlaps(&root, source_root) {
            return Err(invalid(
                "Timeline runtime and source roots must be disjoint",
            ));
        }
        // Share the existing review runtime namespace, so a playback reservation
        // and a producer cannot bypass one another by using different lock names.
        let slot = rooted::lock(&root.join("review-runtime.lock"))?;
        // Check the real current allocation before any child can be created.
        let job =
            kinekino_process::infrastructure::family::PreparedFamilyJob::reserve(config.budget)
                .map_err(io_error)?;
        drop(job);
        Ok(Self {
            source,
            root: config.runtime_root,
            budget: config.budget,
            deadline,
            max_output_bytes: config.max_output_bytes,
            active: Cell::new(0),
            _guards: guards,
            _slot: slot,
            _same_thread: PhantomData,
        })
    }
}
fn overlaps(a: &Path, b: &Path) -> bool {
    let a = PathBuf::from(a.to_string_lossy().to_lowercase());
    let b = PathBuf::from(b.to_string_lossy().to_lowercase());
    a.starts_with(&b) || b.starts_with(&a)
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn io_error(error: std::io::Error) -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        format!("Timeline production I/O: {error}"),
    )
}
fn write_new(path: &Path, value: &serde_json::Value) -> Result<File, ApiError> {
    use std::io::Write;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(io_error)?;
    file.write_all(
        &serde_json::to_vec_pretty(value)
            .map_err(|error| ApiError::new(ErrorCode::EngineFailed, error.to_string()))?,
    )
    .map_err(io_error)?;
    file.sync_all().map_err(io_error)?;
    drop(file);
    rooted::file(path)
}
