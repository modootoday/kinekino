use kinekino_player::{
    application::{
        api::{ApiError, ErrorCode},
        review_playback::{ReviewChildOwnership, ReviewRuntimeBackend, ReviewRuntimeProfile},
    },
    assembly::review_playback::{TimelinePreparation, TimelinePreparationTools},
    domain::review_source::{MediaReference, ReviewCase},
    infrastructure::{
        mpv::TrustedMpvExecutable,
        review_playback_sessions::LocalReviewRuntimeConfig,
        review_startup_settings::RetainedReviewStartupSettings,
        timeline_evidence::{production::*, publication::TimelinePackageDestination},
    },
};
use kinekino_process::infrastructure::family::JobBudget;
use serde::Deserialize;
use std::{path::PathBuf, time::Duration};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Tool {
    path: PathBuf,
    sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    #[serde(default = "default_transport_ms")]
    transport_timeout_ms: u64,
    source_root: PathBuf,
    published_root: PathBuf,
    user_root: PathBuf,
    runtime_root: PathBuf,
    case: ReviewCase,
    media: MediaReference,
    video: Tool,
    audio: Tool,
    capture: Tool,
    playback: Tool,
    video_start_us: u64,
    video_end_us: u64,
    pcm_start_pts: i64,
    pcm_end_pts: i64,
    start_us: u64,
    end_us: u64,
    job_memory_bytes: usize,
    affinity_mask: usize,
    production_total_ms: u64,
    playback_total_ms: u64,
    cleanup_ms: u64,
    max_source_bytes: u64,
    max_output_bytes: u64,
    max_package_bytes: u64,
    playback_raw_output_bytes: u64,
    startup_timeout_ms: u64,
    max_history: usize,
}
fn default_transport_ms() -> u64 {
    kinekino_player::application::local_host::DEFAULT_TIMEOUT_MS
}
pub(super) struct Settings {
    config: Config,
    guard: RetainedReviewStartupSettings,
}
pub(super) fn read(args: &[String]) -> Result<Settings, ApiError> {
    let mut values: [Option<&str>; 2] = [None, None];
    let mut seen = false;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--serve-review" {
            if seen {
                return Err(invalid());
            }
            seen = true;
            index += 1;
            continue;
        }
        let slot = match args[index].as_str() {
            "--review-host-config" => 0,
            "--review-host-config-sha256" => 1,
            _ => return Err(invalid()),
        };
        index += 1;
        let value = args
            .get(index)
            .filter(|s| !s.starts_with("--"))
            .ok_or_else(invalid)?;
        if values[slot].replace(value).is_some() {
            return Err(invalid());
        }
        index += 1;
    }
    let [Some(path), Some(sha)] = values else {
        return Err(invalid());
    };
    if !seen {
        return Err(invalid());
    }
    let path = PathBuf::from(path);
    let guard = RetainedReviewStartupSettings::open(&path, sha)?;
    let config = serde_json::from_slice(guard.bytes()).map_err(|_| invalid())?;
    Ok(Settings { config, guard })
}
type Built = (
    TimelinePreparation,
    TimelinePreparationTools,
    LocalReviewRuntimeConfig,
    usize,
    RetainedReviewStartupSettings,
);
impl Settings {
    pub(super) fn transport_options(
        &self,
    ) -> Result<kinekino_player::infrastructure::local_host::TransportOptions, ApiError> {
        transport_options(self.config.transport_timeout_ms)
    }
    pub(super) fn build(self) -> Result<Built, ApiError> {
        self.transport_options()?;
        let c = self.config;
        if !(1..=64).contains(&c.max_history) {
            return Err(invalid());
        }
        let tools = TimelinePreparationTools {
            video: PinnedTimelineProbe::open(&c.video.path, &c.video.sha256)?,
            audio: PinnedTimelineAudioDecoder::open(&c.audio.path, &c.audio.sha256)?,
            capture: PinnedTimelineCaptureDecoder::open(&c.capture.path, &c.capture.sha256)?,
        };
        let mpv = TrustedMpvExecutable::open(&c.playback.path, &c.playback.sha256)?;
        let budget = JobBudget {
            max_committed_bytes: c.job_memory_bytes,
            affinity_mask: c.affinity_mask,
        };
        let preparation = TimelinePreparation {
            source_root: c.source_root,
            case: c.case,
            media: c.media,
            runtime: TimelineProductionConfig {
                runtime_root: c.runtime_root.clone(),
                budget,
                total: Duration::from_millis(c.production_total_ms),
                cleanup: Duration::from_millis(c.cleanup_ms),
                max_source_bytes: c.max_source_bytes,
                max_output_bytes: c.max_output_bytes,
            },
            destination: TimelinePackageDestination {
                root: c.published_root.clone(),
                max_total_bytes: c.max_package_bytes,
                excluded_roots: vec![c.user_root.clone()],
            },
            video_window: VideoProbeWindow::new(c.video_start_us, c.video_end_us)?,
            pcm_range: PcmClockRange::new(c.pcm_start_pts, c.pcm_end_pts)?,
            start_us: c.start_us,
            end_us: c.end_us,
        };
        let runtime = LocalReviewRuntimeConfig {
            review_root: c.published_root,
            user_root: c.user_root,
            runtime_root: c.runtime_root,
            executable: c.playback.path,
            budget,
            profile: ReviewRuntimeProfile {
                backend: ReviewRuntimeBackend::OwnedLocalMpv,
                engine_sha256: c.playback.sha256,
                max_source_bytes: c.max_source_bytes,
                max_job_committed_bytes: c.job_memory_bytes as u64,
                cpu_threads: 1,
                startup_timeout_ms: c.startup_timeout_ms,
                child_ownership: ReviewChildOwnership::OwnedJobUntilPhysicalExit,
            },
            total: Duration::from_millis(c.playback_total_ms),
            cleanup: Duration::from_millis(c.cleanup_ms),
            raw_output_bytes: c.playback_raw_output_bytes,
        };
        kinekino_process::domain::deadline::Deadline::new(
            std::time::Instant::now(),
            runtime.total,
            runtime.cleanup,
        )
        .map_err(|_| invalid())?;
        if runtime.raw_output_bytes == 0 {
            return Err(invalid());
        }
        kinekino_player::infrastructure::review_runtime::ReviewRuntimePool::open(
            &runtime.runtime_root,
            mpv,
            runtime.profile.clone(),
            runtime.budget,
        )?;
        Ok((preparation, tools, runtime, c.max_history, self.guard))
    }
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Explicit review startup config, unique unmixed flags and complete settings required",
    )
}
fn transport_options(
    ms: u64,
) -> Result<kinekino_player::infrastructure::local_host::TransportOptions, ApiError> {
    if !(10..=30_000).contains(&ms) {
        return Err(invalid());
    }
    Ok(
        kinekino_player::infrastructure::local_host::TransportOptions {
            timeout: Duration::from_millis(ms),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn startup_rejects_duplicate_missing_and_mixed_modes_before_file_access() {
        for args in [
            vec!["--serve-review", "--serve-review"],
            vec!["--serve-review", "--endpoint", "untrusted"],
            vec!["--serve-review", "--serve-native"],
            vec!["--serve-review", "--review-host-config", "missing.json"],
            vec![
                "--serve-review",
                "--review-host-config",
                "one",
                "--review-host-config",
                "two",
            ],
        ] {
            let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
            let error = match read(&args) {
                Ok(_) => panic!("invalid startup accepted"),
                Err(error) => error,
            };
            assert_eq!(error.code, ErrorCode::InvalidArgument);
        }
    }
    #[test]
    fn incomplete_settings_never_default_a_producer_or_authority() {
        assert!(serde_json::from_value::<Config>(serde_json::json!({})).is_err());
    }

    #[test]
    fn host_transport_default_and_explicit_bounds_preserve_operation_separation() {
        assert_eq!(default_transport_ms(), 5000);
        for ms in [10, 5000, 30000] {
            assert_eq!(
                transport_options(ms).unwrap().timeout.as_millis(),
                u128::from(ms)
            );
        }
        for ms in [0, 9, 30001, u64::MAX] {
            assert!(transport_options(ms).is_err());
        }
    }
}
