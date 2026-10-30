use super::Mpv;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_session::{CaptureEpisodeFrame, CapturedEpisodeFrame},
    },
    domain::api_values::DecimalU64,
    infrastructure::{immutable_storage_paths as paths, new_file},
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
static NEXT: AtomicU64 = AtomicU64::new(0);
const MAX_PNG_BYTES: u64 = 64 * 1024 * 1024;

impl Mpv {
    pub(super) fn capture_episode_frame(
        &mut self,
        artifact: &str,
        input: CaptureEpisodeFrame,
    ) -> Result<CapturedEpisodeFrame, ApiError> {
        let started = Instant::now();
        let deadline = started + Duration::from_secs(4);
        let before = self.snapshot().clone();
        if !before.connected
            || before.error.is_some()
            || !before.idle_observed
            || before.idle
            || before.width.is_none()
            || before.height.is_none()
        {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "No current native video frame is available",
            ));
        }
        let destination = Path::new(&input.output_path);
        let (parent, parent_guards) = paths::root(destination.parent().ok_or_else(failed)?)?;
        let destination = parent.join(destination.file_name().ok_or_else(failed)?);
        if destination.try_exists().map_err(new_file::io_error)? {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native frame output already exists",
            ));
        }
        let staging = parent.join(format!(
            ".native-frame-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&staging).map_err(new_file::io_error)?;
        let staging_guard = paths::directory(&staging)?;
        let png_path = staging.join("frame.png");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| failed())?;
        runtime.block_on(async {
            tokio::time::timeout(deadline.saturating_duration_since(Instant::now()), screenshot(&self.pipe, png_path.to_str().ok_or_else(failed)?)).await.map_err(|_| ApiError::new(ErrorCode::Timeout, "Native capture completion was not observed; no output published and no automatic retry"))?
        })?;
        let screenshot_finished = Instant::now();
        let mut file = paths::file(&png_path)?;
        if file.metadata().map_err(new_file::io_error)?.len() > MAX_PNG_BYTES {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Native PNG exceeds capture byte budget",
            ));
        }
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(MAX_PNG_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(new_file::io_error)?;
        if bytes.len() as u64 > MAX_PNG_BYTES {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Native PNG exceeds capture byte budget",
            ));
        }
        let (width, height) = crate::infrastructure::frame_capture::validate_png(&bytes)?;
        let after = self.snapshot().clone();
        if !after.connected || after.error.is_some() {
            return Err(failed());
        }
        let verified = Instant::now();
        new_file::publish_before(
            destination.to_str().ok_or_else(failed)?,
            &bytes,
            Some(deadline),
        ).map_err(|mut error| {
            error.details = serde_json::json!({"stage":"native_capture_publication","deadline_ms":"4000","screenshot_elapsed_ms":screenshot_finished.duration_since(started).as_millis().to_string(),"read_and_verify_elapsed_ms":verified.duration_since(screenshot_finished).as_millis().to_string(),"publication_elapsed_ms":verified.elapsed().as_millis().to_string(),"total_elapsed_ms":started.elapsed().as_millis().to_string(),"output_published":false});
            error
        })?;
        let result = CapturedEpisodeFrame {
            engine_instance_id: input.engine_instance_id,
            resolution_artifact_id: artifact.into(),
            output_path: input.output_path,
            output_sha256: format!("{:x}", Sha256::digest(&bytes)),
            output_bytes: DecimalU64(bytes.len() as u64),
            width,
            height,
            position_before_us: micros(before.time),
            position_after_us: micros(after.time),
            frame_time_exact: false,
            subtitles_included: false,
            window_capture: false,
            color_parity_verified: false,
        };
        drop(file);
        drop(staging_guard);
        let _ = fs::remove_file(png_path);
        let _ = fs::remove_dir(staging);
        drop(parent_guards);
        Ok(result)
    }
}
fn micros(value: Option<f64>) -> Option<DecimalU64> {
    let value = value? * 1_000_000.;
    (value.is_finite() && value >= 0. && value < u64::MAX as f64)
        .then(|| DecimalU64(value.floor() as u64))
}
async fn screenshot(pipe: &str, destination: &str) -> Result<(), ApiError> {
    let mut connection = crate::infrastructure::os_policy::ipc()
        .connect_address(pipe)
        .map_err(|_| failed())?;
    let command =
        serde_json::json!({"command":["screenshot-to-file",destination,"video"],"request_id":1});
    connection
        .write_all(format!("{command}\n").as_bytes())
        .await
        .map_err(|_| failed())?;
    let mut reader = BufReader::new(connection);
    for _ in 0..32 {
        let mut line = Vec::new();
        let count = (&mut reader)
            .take(65537)
            .read_until(b'\n', &mut line)
            .await
            .map_err(|_| failed())?;
        if count == 0 || count > 65536 || !line.ends_with(b"\n") {
            return Err(failed());
        }
        let response: serde_json::Value = serde_json::from_slice(&line).map_err(|_| failed())?;
        if response["request_id"] == 1 {
            return if response["error"] == "success" {
                Ok(())
            } else {
                Err(failed())
            };
        }
    }
    Err(failed())
}
fn failed() -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        "Native frame capture could not be verified; output not published",
    )
}
