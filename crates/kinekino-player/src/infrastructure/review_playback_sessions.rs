//! Trusted fixed-root local review adapter. Request selections contain no paths or tools.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        review_playback::{ReviewRuntimeProfile, inspect_review_input, sessions::*},
        review_source::{
            CaseLease, CaseResolver, ReviewCaseEvidence, ReviewCases, ReviewPackageMedia,
            StoreAccess, UserReviewStore, timeline_evidence,
        },
    },
    domain::{
        api_values::DecimalU64,
        review_source::{ReviewCase, TimelineStatus},
    },
    infrastructure::{
        immutable_storage_paths,
        model_review::ReviewFiles,
        mpv::{ReviewPlaybackWorker, ReviewWorkerLimits, TrustedMpvExecutable},
        review_content_classification::LocalReviewContentClassifier,
        review_runtime::ReviewRuntimePool,
        review_source::UserReviewFiles,
        timeline_evidence::{ReviewTimelineEvidence, TrustedTimelineAdmission},
    },
};
use kinekino_process::infrastructure::family::JobBudget;
use std::{
    fs::{self, File},
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Required trusted assembly inputs. No Default/Deserialize or global configuration changes.
pub struct LocalReviewSessionConfig {
    pub review_root: PathBuf,
    pub user_root: PathBuf,
    pub runtime_root: PathBuf,
    pub executable: PathBuf,
    pub profile: ReviewRuntimeProfile,
    pub budget: JobBudget,
    pub admission: TrustedTimelineAdmission,
    pub total: Duration,
    pub cleanup: Duration,
    pub raw_output_bytes: u64,
}
/// Runtime inputs independent of archive/live evidence format. Trusted Rust
/// assembly must supply the original resolver separately; there is no default.
pub struct LocalReviewRuntimeConfig {
    pub review_root: PathBuf,
    pub user_root: PathBuf,
    pub runtime_root: PathBuf,
    pub executable: PathBuf,
    pub profile: ReviewRuntimeProfile,
    pub budget: JobBudget,
    pub total: Duration,
    pub cleanup: Duration,
    pub raw_output_bytes: u64,
}
impl LocalReviewSessionConfig {
    pub(crate) fn into_parts(self) -> (LocalReviewRuntimeConfig, TrustedTimelineAdmission) {
        (
            LocalReviewRuntimeConfig {
                review_root: self.review_root,
                user_root: self.user_root,
                runtime_root: self.runtime_root,
                executable: self.executable,
                profile: self.profile,
                budget: self.budget,
                total: self.total,
                cleanup: self.cleanup,
                raw_output_bytes: self.raw_output_bytes,
            },
            self.admission,
        )
    }
}
struct Fixed {
    config: LocalReviewRuntimeConfig,
    evidence: Arc<dyn timeline_evidence::TimelineEvidenceResolver>,
    cases: ReviewCases<ReviewFiles>,
    _guards: Vec<File>,
}
pub struct LocalReviewSessionLauncher {
    fixed: Arc<Fixed>,
}
impl LocalReviewSessionLauncher {
    pub fn new(config: LocalReviewSessionConfig) -> Result<Self, ApiError> {
        let (config, admission) = config.into_parts();
        let _guards = retain_root_layout(&config)?;
        let evidence = Arc::new(ReviewTimelineEvidence::new(
            ReviewFiles::open(&config.review_root)?,
            admission,
        )?);
        Self::with_evidence(config, evidence)
    }
    pub fn with_evidence(
        config: LocalReviewRuntimeConfig,
        evidence: Arc<dyn timeline_evidence::TimelineEvidenceResolver>,
    ) -> Result<Self, ApiError> {
        let guards = retain_root_layout(&config)?;
        // Reuse the existing disjoint evidence/user-root boundary and readonly policy.
        super::review_source::open_api(&config.review_root, &config.user_root, false)?;
        // Canonical paths are for comparison only. Root admission deliberately
        // requires the original ordinary Disk path, not a VerbatimDisk prefix.
        let tool = TrustedMpvExecutable::open(&config.executable, &config.profile.engine_sha256)?;
        ReviewRuntimePool::open(
            &config.runtime_root,
            tool,
            config.profile.clone(),
            config.budget,
        )?;
        kinekino_process::domain::deadline::Deadline::new(
            std::time::Instant::now(),
            config.total,
            config.cleanup,
        )
        .map_err(|error| ApiError::new(ErrorCode::InvalidArgument, error.to_string()))?;
        if config.raw_output_bytes == 0 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Positive raw output limit required",
            ));
        }
        let cases = ReviewCases::new(ReviewFiles::open(&config.review_root)?);
        Ok(Self {
            fixed: Arc::new(Fixed {
                config,
                evidence,
                cases,
                _guards: guards,
            }),
        })
    }
}
/// Retain all original ordinary roots before any optional user-store creation.
pub(crate) fn retain_root_layout(config: &LocalReviewRuntimeConfig) -> Result<Vec<File>, ApiError> {
    let mut guards = Vec::new();
    let mut roots = Vec::new();
    for path in [&config.review_root, &config.user_root, &config.runtime_root] {
        let (canonical, files) = immutable_storage_paths::root(path)?;
        guards.extend(files);
        roots.push(PathBuf::from(canonical.to_string_lossy().to_lowercase()));
    }
    for (index, root) in roots.iter().enumerate() {
        for other in roots.iter().skip(index + 1) {
            if root.starts_with(other) || other.starts_with(root) {
                return Err(ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Review evidence, user and runtime roots must be disjoint",
                ));
            }
        }
    }
    Ok(guards)
}
impl SessionLauncher for LocalReviewSessionLauncher {
    fn controls_available(&self) -> bool {
        true
    }
    fn retain_case(&self, case: &ReviewCase) -> Result<Box<dyn CaseLease + '_>, ApiError> {
        self.fixed.cases.retain(case)
    }
    fn start(&self, selection: &Selection) -> Result<Box<dyn SessionOwner>, ApiError> {
        let fixed = self.fixed.clone();
        let selected = selection.clone();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| ApiError::new(ErrorCode::EngineFailed, error.to_string()))?
            .as_nanos();
        let output = fixed
            .config
            .runtime_root
            .join(format!("review-session-{}-{nonce}", std::process::id()));
        fs::create_dir(&output)
            .map_err(|error| ApiError::new(ErrorCode::EngineFailed, error.to_string()))?;
        let limits = ReviewWorkerLimits {
            total: fixed.config.total,
            cleanup: fixed.config.cleanup,
            raw_output_bytes: fixed.config.raw_output_bytes,
        };
        let worker = ReviewPlaybackWorker::spawn_scoped(
            move |launch| {
                let config = &fixed.config;
                let cases = ReviewCases::new(ReviewFiles::open(&config.review_root)?);
                let media = ReviewPackageMedia::new(ReviewFiles::open(&config.review_root)?);
                let case = cases.retain(&selected.case)?;
                if case.case() != &selected.case {
                    return Err(binding());
                }
                case.check_current()?;
                let store = UserReviewFiles::open(&config.user_root, false)?;
                let transaction = UserReviewStore::open(&store, &selected.case, StoreAccess::Read)?;
                let association = transaction
                    .association(
                        &selected.association_id,
                        Some(selected.association_revision),
                    )?
                    .ok_or_else(|| {
                        ApiError::new(ErrorCode::NotFound, "Exact review association missing")
                    })?;
                association.validate().map_err(|_| binding())?;
                if association.case != selected.case
                    || association.association_id != selected.association_id
                    || association.revision != selected.association_revision
                {
                    return Err(binding());
                }
                let TimelineStatus::Verified(record) = &association.timeline_status else {
                    return Err(ApiError::new(
                        ErrorCode::NeedsInput,
                        "A retained verified timeline is required for product review playback",
                    ));
                };
                let tool =
                    TrustedMpvExecutable::open(&config.executable, &config.profile.engine_sha256)?;
                let reservation = ReviewRuntimePool::open(
                    &config.runtime_root,
                    tool,
                    config.profile.clone(),
                    config.budget,
                )?
                .reserve_owned()?;
                let proof = timeline_evidence::retain(
                    &cases,
                    &media,
                    fixed.evidence.as_ref(),
                    &selected.case,
                    &association.media,
                    &record.evidence,
                )?;
                proof.check_record(record)?;
                let reader = ReviewCaseEvidence::new(ReviewFiles::open(&config.review_root)?);
                let input = inspect_review_input(
                    &reader,
                    &selected.case,
                    &association.media,
                    &LocalReviewContentClassifier,
                )?;
                let source = ReviewFiles::open(&config.review_root)?.bind_playback_media(input)?;
                let admitted = source.admit(reservation)?;
                let window = crate::application::review_playback::window::ReviewWindow::new(
                    record.source_start_us.0,
                    record.source_end_us.0,
                )?;
                let result = launch.run_with_policy(
                    admitted,
                    &|| {
                        case.check_current()?;
                        proof.check_record(record)
                    },
                    &|intent, observed| {
                        proof.check_record(record)?;
                        window.check_control(
                            intent,
                            &crate::application::playback_status::project_native_status(observed),
                        )
                    },
                    &|observed| {
                        proof.check_record(record)?;
                        window.should_pause(
                            &crate::application::playback_status::project_native_status(observed),
                        )
                    },
                );
                // Original immutable envelope, case, media and proof remain here until run returns.
                drop(proof);
                drop(transaction);
                drop(case);
                result
            },
            output,
            limits,
        )?;
        Ok(Box::new(LocalOwner {
            selection: selection.clone(),
            worker,
        }))
    }
}
struct LocalOwner {
    selection: Selection,
    worker: ReviewPlaybackWorker,
}
impl SessionOwner for LocalOwner {
    fn selection(&self) -> &Selection {
        &self.selection
    }
    fn observe(&self) -> Result<OwnerObservation, ApiError> {
        let state = self.worker.snapshot()?;
        let clock = crate::application::playback_status::project_native_status(&state.playback);
        Ok(OwnerObservation {
            process_created: state.process_created,
            connected: state.playback.connected,
            closing: state.closing,
            physical_closed: state.physical_closed,
            family_empty: state.family_empty,
            ipc_exited: state.ipc_exited,
            owner_finished: self.worker.is_finished(),
            engine_instance_id: state.engine_instance_id.map(DecimalU64),
            has_error: state.first_error.is_some(),
            failure: state.first_error.map(|message| SessionFailure {
                code: state.first_error_code.unwrap_or(ErrorCode::EngineFailed),
                message,
            }),
            position_us: clock.position_us,
            duration_us: clock.duration_us,
            paused: clock.paused,
            volume_percent: clock.volume_percent,
            muted: clock.muted,
        })
    }
    fn request_close(&self) -> Result<(), ApiError> {
        self.worker.close();
        Ok(())
    }
    fn submit(
        &self,
        instance: DecimalU64,
        intent: crate::domain::playback_control::PlaybackControl,
    ) -> Result<crate::application::playback_control::ControlReceipt, ApiError> {
        use crate::application::playback_control::PlaybackControlPort;
        let source =
            self.worker.snapshot()?.source.ok_or_else(|| {
                ApiError::new(ErrorCode::NeedsInput, "Review source is not ready")
            })?;
        self.worker
            .scoped_control(source.case, source.media, instance.0)?
            .submit(intent)
    }
}
fn binding() -> ApiError {
    ApiError::new(
        ErrorCode::RevisionConflict,
        "Stored association or retained playback proof binding differs",
    )
}
