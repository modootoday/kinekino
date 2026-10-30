use super::*;
use crate::{domain::identifiers::valid_sha256, infrastructure::content_digest};
/// Exact local FFprobe identity; no PATH search, caller manifest or deserialization.
pub(super) struct PinnedTimelineTool {
    pub(super) path: PathBuf,
    sha256: String,
    identity: String,
    _guards: Vec<File>,
}
impl PinnedTimelineTool {
    fn open(path: &Path, expected: &str, name: &str) -> Result<Self, ApiError> {
        if !valid_sha256(expected) || path.file_name().and_then(|n| n.to_str()) != Some(name) {
            return Err(invalid("Explicit local tool filename and SHA256 required"));
        }
        let (parent, mut guards) = rooted::root(
            path.parent()
                .ok_or_else(|| invalid("Probe parent missing"))?,
        )?;
        let file_path = parent.join(name);
        let mut file = rooted::file(&file_path)?;
        let (_, sha256) = content_digest::sha256(&mut file, 512 * 1024 * 1024)?;
        if sha256 != expected {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Timeline tool pin differs",
            ));
        }
        let identity = file_identity(&file)?;
        guards.push(file);
        Ok(Self {
            path: file_path,
            sha256,
            identity,
            _guards: guards,
        })
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn file_identity(&self) -> &str {
        &self.identity
    }
}

macro_rules! pinned_tool {
    ($name:ident, $file:literal) => {
        pub struct $name(pub(super) PinnedTimelineTool);
        impl $name {
            pub fn open(path: &Path, expected: &str) -> Result<Self, ApiError> {
                PinnedTimelineTool::open(path, expected, $file).map(Self)
            }
            pub fn sha256(&self) -> &str {
                self.0.sha256()
            }
            pub fn file_identity(&self) -> &str {
                self.0.file_identity()
            }
        }
    };
}
pinned_tool!(PinnedTimelineProbe, "ffprobe.exe");
pinned_tool!(PinnedTimelineAudioDecoder, "ffmpeg.exe");
pinned_tool!(PinnedTimelineCaptureDecoder, "ffmpeg.exe");

pub(super) fn file_identity(file: &File) -> Result<String, ApiError> {
    // Same rendering as before: 8-hex volume serial, 16-hex file index.
    let id = crate::infrastructure::os_policy::file_identity(file).map_err(io_error)?;
    Ok(format!("{:08x}-{:016x}", id.volume, id.index))
}
