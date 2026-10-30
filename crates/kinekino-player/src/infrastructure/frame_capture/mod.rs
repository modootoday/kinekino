pub(crate) mod plan;
mod png_validation;
pub(crate) use png_validation::validate as validate_png;
mod probe;
pub(crate) use probe::selected_pts;
mod discovery;
pub(crate) mod process;
mod seek;
pub(crate) mod tools;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        frame_capture::{CapturedFrame, FrameCaptureCapability, FrameCapturePort},
    },
    domain::{
        api_values::DecimalU64,
        frame_capture::{CaptureFrame, MAX_MEDIA_BYTES},
    },
};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

pub struct FfmpegFrameCapture {
    capability: FrameCaptureCapability,
    _tool_guards: Vec<File>,
}
impl FfmpegFrameCapture {
    pub fn discover() -> Self {
        let ffmpeg = find_tool("ffmpeg");
        let ffprobe = find_tool("ffprobe");
        Self::inspect_tools(ffmpeg, ffprobe, Vec::new())
    }

    /// Trusted host configuration only; no search or fallback on a rejected pin.
    #[cfg(windows)]
    pub fn pinned(
        ffmpeg: &Path,
        ffmpeg_sha256: &str,
        ffprobe: &Path,
        ffprobe_sha256: &str,
    ) -> Result<Self, ApiError> {
        let (ffmpeg, mut guards) = tools::open(ffmpeg, ffmpeg_sha256)?;
        let (ffprobe, probe_guards) = tools::open(ffprobe, ffprobe_sha256)?;
        guards.extend(probe_guards);
        Ok(Self::inspect_tools(Some(ffmpeg), Some(ffprobe), guards))
    }

    fn inspect_tools(ffmpeg: Option<PathBuf>, ffprobe: Option<PathBuf>, guards: Vec<File>) -> Self {
        let mut failures = Vec::new();
        let mut version = |path: &PathBuf, stage: &str| {
            discovery::version(
                process::run(
                    Command::new(path).arg("-version"),
                    Instant::now() + Duration::from_secs(2),
                    16384,
                ),
                stage,
            )
            .map_err(|reason| failures.push(reason))
            .ok()
        };
        let ffmpeg_version = ffmpeg
            .as_ref()
            .and_then(|path| version(path, "ffmpeg_version"));
        let ffprobe_version = ffprobe
            .as_ref()
            .and_then(|path| version(path, "ffprobe_version"));
        let features_available = ffmpeg.as_ref().is_some_and(|tool| {
            let encoders = process::run(
                Command::new(tool).args(["-hide_banner", "-encoders"]),
                Instant::now() + Duration::from_secs(2),
                262144,
            );
            let filters = process::run(
                Command::new(tool).args(["-hide_banner", "-filters"]),
                Instant::now() + Duration::from_secs(2),
                262144,
            );
            let contains = |output: &process::ProcessOutput, name: &str| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .any(|line| line.split_whitespace().nth(1) == Some(name))
            };
            let encoders = discovery::observe(encoders, "ffmpeg_encoders", &mut failures);
            let filters = discovery::observe(filters, "ffmpeg_filters", &mut failures);
            encoders.is_some_and(|output| contains(&output, "png"))
                && filters.is_some_and(|output| {
                    contains(&output, "select") && contains(&output, "showinfo")
                })
        });
        let available = features_available
            && ffmpeg_version
                .as_ref()
                .is_some_and(|v| v.starts_with("ffmpeg version "))
            && ffprobe_version
                .as_ref()
                .is_some_and(|v| v.starts_with("ffprobe version "));
        Self {
            _tool_guards: guards,
            capability: FrameCaptureCapability {
                available,
                adapter: "external_ffmpeg_cpu".into(),
                ffmpeg_path: ffmpeg.map(|p| p.to_string_lossy().into()),
                ffprobe_path: ffprobe.map(|p| p.to_string_lossy().into()),
                ffmpeg_version,
                ffprobe_version,
                unavailable_reason: (!available).then(|| discovery::reason(&failures)),
            },
        }
    }
}

impl FrameCapturePort for FfmpegFrameCapture {
    fn capture_recoverable(
        &self,
        request: &CaptureFrame,
        timeout_ms: u64,
        context: crate::application::frame_capture::FrameExecutionContext<'_>,
    ) -> Result<CapturedFrame, ApiError> {
        self.capture_inner(request, timeout_ms, context.control, context.recovery)
    }
    fn capability(&self) -> FrameCaptureCapability {
        self.capability.clone()
    }
    fn capture(&self, request: &CaptureFrame, timeout_ms: u64) -> Result<CapturedFrame, ApiError> {
        self.capture_inner(request, timeout_ms, None, None)
    }
    fn cooperative_cancel_supported(&self) -> bool {
        cfg!(windows)
    }
    fn capture_controlled(
        &self,
        request: &CaptureFrame,
        timeout_ms: u64,
        control: &dyn crate::application::review_source::MutationControl,
    ) -> Result<CapturedFrame, ApiError> {
        self.capture_inner(request, timeout_ms, Some(control), None)
    }
}

impl FfmpegFrameCapture {
    fn capture_inner(
        &self,
        request: &CaptureFrame,
        timeout_ms: u64,
        control: Option<&dyn crate::application::review_source::MutationControl>,
        recovery: Option<&dyn crate::application::sdk_execution::ReceiptRecoveryControl>,
    ) -> Result<CapturedFrame, ApiError> {
        if let Some(control) = control {
            control.checkpoint()?;
        }
        request
            .validate()
            .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
        if !(1..=crate::domain::frame_capture::MAX_CAPTURE_TIMEOUT_MS).contains(&timeout_ms) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid capture timeout",
            ));
        }
        if !self.capability.available {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "FFmpeg/FFprobe adapter unavailable",
            ));
        }
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        let input = Path::new(&request.source_path)
            .canonicalize()
            .map_err(super::new_file::io_error)?;
        if input.to_string_lossy().starts_with(r"\\?\UNC\") {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Network filesystem sources are unsupported",
            ));
        }
        let (mut source, demuxer) = open_source(input.to_str().ok_or_else(|| {
            ApiError::new(ErrorCode::InvalidArgument, "Source path is not Unicode")
        })?)?;
        let mut header = [0; 12];
        source
            .read_exact(&mut header)
            .map_err(|_| ApiError::new(ErrorCode::InvalidArgument, "Media header is incomplete"))?;
        if (demuxer == "mov" && &header[4..8] != b"ftyp")
            || (demuxer == "matroska" && header[..4] != [0x1a, 0x45, 0xdf, 0xa3])
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Media signature does not match the supported container",
            ));
        }
        if Path::new(&request.output_path).symlink_metadata().is_ok() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Output already exists; choose a new path",
            ));
        }
        let info = probe::inspect(
            self.capability.ffprobe_path.as_ref().unwrap(),
            &input,
            demuxer,
            deadline,
            control,
            recovery,
        )?;
        info.validate(request.source_pts_us.0)?;
        let threshold = info.time_base.threshold(request.source_pts_us.0)?;
        let mut command = Command::new(self.capability.ffmpeg_path.as_ref().unwrap());
        let seek_timestamp = seek::input_timestamp(info.start, request.source_pts_us.0);
        command.args(plan::arguments(
            &input,
            demuxer,
            plan::Selection::AtOrAfter(threshold),
            seek_timestamp.as_deref(),
            2,
        )?);
        let decoded = process::run_with_recovery(
            &mut command,
            deadline,
            crate::domain::frame_capture::MAX_FRAME_PNG_BYTES as usize,
            control,
            recovery,
        )?;
        let (width, height) = png_validation::validate(&decoded.stdout)?;
        let (ticks, time_base, actual) = probe::selected_pts(&decoded.stderr)?;
        if actual < request.source_pts_us.0
            || !info.time_base.equivalent(&time_base)
            || width != info.width
            || height != info.height
        {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Decoded frame metadata does not match the requested source",
            ));
        }
        let output_sha256 = format!("{:x}", Sha256::digest(&decoded.stdout));
        if Instant::now() >= deadline {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Frame verification exceeded timeout; output not published",
            ));
        }
        super::new_file::publish_guarded(&request.output_path, &decoded.stdout, || {
            if let Some(control) = control {
                control.checkpoint()?;
            }
            if Instant::now() >= deadline {
                return Err(super::new_file::io_error(
                    std::io::ErrorKind::TimedOut.into(),
                ));
            }
            Ok(())
        })?;
        Ok(CapturedFrame{output_path:request.output_path.clone(),requested_source_pts_us:request.source_pts_us,actual_source_pts_us:DecimalU64(actual),source_pts_ticks:ticks,source_time_base:time_base,width,height,output_bytes:decoded.stdout.len() as u64,output_sha256:Some(output_sha256),source_color_space:info.color_space,source_color_transfer:info.color_transfer,rotation_applied:false,source_rotation_degrees:info.rotation,subtitle_mode:"separate_tracks_excluded_burn_in_preserved".into(),limitations:vec!["First decoded source frame at or after requested PTS; not GUI current-frame capture".into(),"CPU RGB24 conversion; HDR tone mapping, ICC/sRGB tagging and player color parity are not validated".into(),"Source rotation is not applied; burn-in text cannot be removed".into()],dependency:self.capability.clone()})
    }
}

fn open_source(path: &str) -> Result<(File, &'static str), ApiError> {
    let extension = Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let demuxer = if extension == "mp4" {
        "mov"
    } else {
        "matroska"
    };
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    crate::infrastructure::os_policy::apply_share_mode(
        &mut options,
        crate::infrastructure::os_policy::ShareMode::ReadOnly,
    );
    let file = options.open(path).map_err(super::new_file::io_error)?;
    let metadata = file.metadata().map_err(super::new_file::io_error)?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Source must be a nonempty regular media file",
        ));
    }
    if metadata.len() > MAX_MEDIA_BYTES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Source media exceeds 16 GiB limit",
        ));
    }
    Ok((file, demuxer))
}

fn find_tool(name: &str) -> Option<PathBuf> {
    let executable = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.into()
    };
    let mut candidates = Vec::new();
    if let Ok(current) = std::env::current_exe()
        && let Some(parent) = current.parent()
    {
        candidates.push(parent.join(&executable));
    }
    #[cfg(windows)]
    candidates.push(
        PathBuf::from(r"C:\ProgramData\chocolatey\lib\ffmpeg\tools\ffmpeg\bin").join(&executable),
    );
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&path).map(|p| p.join(&executable)));
    }
    candidates
        .into_iter()
        .find(|path| {
            path.metadata()
                .is_ok_and(|m| m.is_file() && m.len() > 100_000)
        })
        .and_then(|p| p.canonicalize().ok())
}
