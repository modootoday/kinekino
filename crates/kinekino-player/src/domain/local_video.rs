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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_programs_playlists_and_relative_sources() {
        // The OS temp directory is absolute on every target, so the rule is
        // checked without platform-specific path literals.
        let root = std::env::temp_dir();
        assert!(root.is_absolute());
        assert!(LocalVideo::selected(root.join("sample.MKV")).is_ok());
        for path in [
            root.join("run.exe"),
            root.join("remote.m3u"),
            root.join("no_extension"),
            PathBuf::from("relative.mp4"),
            PathBuf::from("https://example.com/a.mp4"),
        ] {
            assert!(LocalVideo::selected(path).is_err());
        }
    }
}
