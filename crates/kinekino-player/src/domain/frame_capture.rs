use crate::domain::api_values::DecimalU64;
use serde::{Deserialize, Serialize};

pub const MAX_MEDIA_BYTES: u64 = 16 * 1024 * 1024 * 1024;
pub const MAX_FRAME_PIXELS: u64 = 3840 * 2160;
pub const MAX_FRAME_PNG_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_CAPTURE_TIMEOUT_MS: u64 = 120_000;

pub fn is_absolute_local_path(path: &str) -> bool {
    std::path::Path::new(path).is_absolute()
        && !path.starts_with("\\\\")
        && !path.contains('\0')
        && !path
            .chars()
            .enumerate()
            .any(|(index, c)| c == ':' && index != 1)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureFrame {
    pub source_path: String,
    pub source_pts_us: DecimalU64,
    pub output_path: String,
}

impl CaptureFrame {
    pub fn validate(&self) -> Result<(), &'static str> {
        for path in [&self.source_path, &self.output_path] {
            if !is_absolute_local_path(path) {
                return Err("Only absolute local filesystem paths are supported");
            }
        }
        if self.source_path == self.output_path
            || !self.output_path.to_ascii_lowercase().ends_with(".png")
        {
            return Err("Output must be a distinct new PNG path");
        }
        if ![".mp4", ".mkv", ".webm"]
            .iter()
            .any(|extension| self.source_path.to_ascii_lowercase().ends_with(extension))
        {
            return Err("Only local MP4, MKV and WebM media are supported");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blocks_remote_protocols_playlists_and_non_png_outputs() {
        let base = std::env::temp_dir();
        let mut request = CaptureFrame {
            source_path: base.join("local.mp4").to_string_lossy().into(),
            source_pts_us: DecimalU64(0),
            output_path: base.join("new.png").to_string_lossy().into(),
        };
        assert!(request.validate().is_ok());
        for source in [
            "https://host/video.mp4",
            "file:///tmp/video.mp4",
            "concat:a.mp4|b.mp4",
            r"\\host\share\v.mp4",
            "relative.mp4",
        ] {
            request.source_path = source.into();
            assert!(request.validate().is_err());
        }
        request.source_path = base.join("playlist.m3u8").to_string_lossy().into();
        assert!(request.validate().is_err());
        request.source_path = base.join("local.mp4").to_string_lossy().into();
        request.output_path = base.join("output.jpg").to_string_lossy().into();
        assert!(request.validate().is_err());
    }
}
