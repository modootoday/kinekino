use super::*;
use crate::{domain::identifiers::valid_sha256, infrastructure::content_digest};
/// Exact local FFprobe identity; no PATH search, caller manifest or deserialization.
pub(super) struct PinnedTimelineTool {
    /// Verified local path, for evidence.
    pub(super) path: PathBuf,
    /// What to spawn (ADR 20261116151137); valid while `_guards` are held.
    pub(super) program: PathBuf,
    sha256: String,
    identity: String,
    _guards: Vec<File>,
}
impl PinnedTimelineTool {
    /// `stem` is the tool name without this OS's executable suffix.
    fn open(path: &Path, expected: &str, stem: &str) -> Result<Self, ApiError> {
        let name = &format!("{stem}{}", std::env::consts::EXE_SUFFIX);
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
        let program =
            crate::infrastructure::frame_capture::tools::pinned_program(&file, &file_path)?;
        guards.push(file);
        Ok(Self {
            path: file_path,
            program,
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
pinned_tool!(PinnedTimelineProbe, "ffprobe");
pinned_tool!(PinnedTimelineAudioDecoder, "ffmpeg");
pinned_tool!(PinnedTimelineCaptureDecoder, "ffmpeg");

pub(super) fn file_identity(file: &File) -> Result<String, ApiError> {
    // Same rendering as before: 8-hex volume serial, 16-hex file index.
    let id = crate::infrastructure::os_policy::file_identity(file).map_err(io_error)?;
    Ok(format!("{:08x}-{:016x}", id.volume, id.index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_timeline_tool_uses_this_os_name_and_spawns_its_program() {
        let _ = crate::infrastructure::os_policy::host();
        let name = format!("ffprobe{}", std::env::consts::EXE_SUFFIX);
        let Some(installed) = std::env::var_os("PATH").and_then(|paths| {
            std::env::split_paths(&paths)
                .map(|dir| dir.join(&name))
                .find(|path| path.is_file())
        }) else {
            eprintln!("ffprobe not installed; pinned timeline tool check skipped");
            return;
        };
        let path = installed.canonicalize().unwrap();
        if path.file_name().and_then(|n| n.to_str()) != Some(name.as_str()) {
            eprintln!("ffprobe resolves to another file name; check skipped");
            return;
        }
        let (_, sha) =
            content_digest::sha256(&mut File::open(&path).unwrap(), 512 * 1024 * 1024).unwrap();
        let tool = PinnedTimelineProbe::open(&path, &sha).unwrap();
        // Evidence keeps the verified path; the program is what runs.
        assert_eq!(tool.0.path, path);
        let output = std::process::Command::new(&tool.0.program)
            .arg("-version")
            .output()
            .unwrap();
        assert!(String::from_utf8_lossy(&output.stdout).starts_with("ffprobe version "));
        // A name for another tool is refused before anything is hashed.
        let error = PinnedTimelineAudioDecoder::open(&path, &sha).err().unwrap();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
    }
}
