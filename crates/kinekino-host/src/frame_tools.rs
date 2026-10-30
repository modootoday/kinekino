//! Explicit host tool configuration; never sourced from request payload or browser metadata.
use kinekino_player::{
    application::api::{ApiError, ErrorCode},
    infrastructure::frame_capture::FfmpegFrameCapture,
};

pub fn options(args: &[String]) -> Result<Option<[&str; 4]>, ApiError> {
    let flags = [
        "--frame-ffmpeg",
        "--frame-ffmpeg-sha256",
        "--frame-ffprobe",
        "--frame-ffprobe-sha256",
    ];
    let mut values = [None; 4];
    for (index, arg) in args.iter().enumerate() {
        if !is_option(arg) {
            continue;
        }
        if let Some(slot) = flags.iter().position(|flag| *flag == arg) {
            let value = args
                .get(index + 1)
                .filter(|value| !value.starts_with("--"))
                .ok_or_else(|| {
                    ApiError::new(ErrorCode::InvalidArgument, "Frame option requires a value")
                })?;
            if values[slot].replace(value.as_str()).is_some() {
                return Err(ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Duplicate frame tool option",
                ));
            }
        }
    }
    if values.iter().all(Option::is_none) {
        return Ok(None);
    }
    let [
        Some(ffmpeg),
        Some(ffmpeg_sha),
        Some(ffprobe),
        Some(ffprobe_sha),
    ] = values
    else {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Frame tool paths and SHA256 pins must be supplied together",
        ));
    };
    if !std::path::Path::new(ffmpeg).is_absolute()
        || !std::path::Path::new(ffprobe).is_absolute()
        || !kinekino_player::domain::identifiers::valid_sha256(ffmpeg_sha)
        || !kinekino_player::domain::identifiers::valid_sha256(ffprobe_sha)
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid explicit frame tool configuration",
        ));
    }
    Ok(Some([ffmpeg, ffmpeg_sha, ffprobe, ffprobe_sha]))
}

pub fn adapter(args: &[String]) -> Result<FfmpegFrameCapture, ApiError> {
    Ok(configured(args)?.unwrap_or_else(FfmpegFrameCapture::discover))
}

pub fn is_option(flag: &str) -> bool {
    matches!(
        flag,
        "--frame-ffmpeg" | "--frame-ffmpeg-sha256" | "--frame-ffprobe" | "--frame-ffprobe-sha256"
    )
}

pub fn configured(args: &[String]) -> Result<Option<FfmpegFrameCapture>, ApiError> {
    match options(args)? {
        None => Ok(None),
        #[cfg(windows)]
        Some([ffmpeg, ffmpeg_sha, ffprobe, ffprobe_sha]) => FfmpegFrameCapture::pinned(
            std::path::Path::new(ffmpeg),
            ffmpeg_sha,
            std::path::Path::new(ffprobe),
            ffprobe_sha,
        )
        .map(Some),
        #[cfg(not(windows))]
        Some(_) => Err(ApiError::new(
            ErrorCode::Unsupported,
            "Pinned frame tools are not implemented on this target",
        )),
    }
}
