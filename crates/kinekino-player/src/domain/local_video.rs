use std::path::{Path, PathBuf};

/// A filesystem selection, never constructed from browser messages or remote URLs.
pub struct LocalVideo(PathBuf);
impl LocalVideo {
    pub fn selected(path: PathBuf) -> anyhow::Result<Self> {
        anyhow::ensure!(path.is_absolute(), "Select an absolute local video path");
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        anyhow::ensure!(
            matches!(
                extension.as_str(),
                "mp4" | "mkv" | "webm" | "mov" | "avi" | "m4v" | "ts" | "m2ts" | "mpg" | "mpeg"
            ),
            "Unsupported local video format"
        );
        Ok(Self(path))
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
}
#[cfg(all(test, windows))]
mod tests {
    use super::*;
    #[test]
    fn rejects_programs_playlists_and_relative_sources() {
        assert!(LocalVideo::selected(PathBuf::from(r"C:\media\sample.MKV")).is_ok());
        for path in [
            r"C:\media\run.exe",
            r"C:\media\remote.m3u",
            "relative.mp4",
            "https://example.com/a.mp4",
        ] {
            assert!(LocalVideo::selected(PathBuf::from(path)).is_err());
        }
    }
}
