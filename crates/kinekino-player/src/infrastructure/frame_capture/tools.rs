//! Keep the verified local files and their ancestor directories guarded for the adapter lifetime.
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::identifiers::valid_sha256,
    infrastructure::{content_digest, immutable_storage_paths as paths, os_policy},
};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

/// A hash-verified pinned tool. `path` is the verified local path, for evidence and messages.
/// `program` is what to spawn (ADR 20261116151137): on Linux it names the verified handle and
/// is valid only while `guards` are held, so keep both together.
pub(crate) struct PinnedTool {
    pub path: PathBuf,
    pub program: PathBuf,
    pub guards: Vec<File>,
}

pub(crate) fn open(path: &Path, expected: &str) -> Result<PinnedTool, ApiError> {
    if !valid_sha256(expected) || !path.is_absolute() {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Frame tool requires an absolute local path and SHA256",
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| ApiError::new(ErrorCode::InvalidArgument, "Frame tool parent is missing"))?;
    let name = path.file_name().ok_or_else(|| {
        ApiError::new(ErrorCode::InvalidArgument, "Frame tool filename is missing")
    })?;
    let (parent, mut guards) = paths::root(parent)?;
    let path = parent.join(name);
    let mut file = paths::file(&path)?;
    let (_, actual) = content_digest::sha256(&mut file, 512 * 1024 * 1024)?;
    if actual != expected {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Frame tool SHA256 differs from the explicit host configuration",
        ));
    }
    let program = pinned_program(&file, &path)?;
    guards.push(file);
    Ok(PinnedTool {
        path,
        program,
        guards,
    })
}

/// Program to spawn for a verified tool file (ADR 20261116151137), as an API error on refusal.
pub(crate) fn pinned_program(verified: &File, path: &Path) -> Result<PathBuf, ApiError> {
    os_policy::pinned_program(verified, path).map_err(program_error)
}

fn program_error(error: std::io::Error) -> ApiError {
    let code = match error.kind() {
        std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
        std::io::ErrorKind::InvalidData | std::io::ErrorKind::NotFound => ErrorCode::SourceExpired,
        std::io::ErrorKind::InvalidInput => ErrorCode::InvalidArgument,
        _ => ErrorCode::EngineFailed,
    };
    ApiError::new(code, format!("Pinned tool cannot run: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_pin_rejects_mismatch_and_holds_verified_file_against_writes() {
        let root = std::env::temp_dir().join(format!("frame-pin-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("tool.exe");
        std::fs::write(&path, b"not executed").unwrap();
        let wrong = open(&path, &"0".repeat(64)).err().unwrap();
        assert_eq!(wrong.code, ErrorCode::SourceExpired);
        let (_, expected) = content_digest::sha256(&mut File::open(&path).unwrap(), 4096).unwrap();
        let PinnedTool { guards, .. } = open(&path, &expected).unwrap();
        let mandatory = crate::infrastructure::os_policy::has_mandatory_share_modes();
        assert_eq!(
            std::fs::OpenOptions::new().write(true).open(&path).is_err(),
            mandatory
        );
        // Mandatory share modes keep the verified file from being removed while guarded; POSIX
        // unlink only drops the name, and the guarded original stays readable.
        assert_eq!(std::fs::remove_file(&path).is_err(), mandatory);
        if !mandatory {
            use std::io::{Read, Seek};
            let mut original = &guards[guards.len() - 1];
            original.seek(std::io::SeekFrom::Start(0)).unwrap();
            let mut held = Vec::new();
            original.read_to_end(&mut held).unwrap();
            assert_eq!(held, b"not executed");
            std::fs::write(&path, b"replacement").unwrap();
        }
        drop(guards);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(root).unwrap();
    }

    #[test]
    fn evidence_path_stays_the_verified_path_and_program_is_absolute() {
        let root = std::env::temp_dir().join(format!("frame-pin-program-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir(&root).unwrap();
        let path = root.join("tool.exe");
        std::fs::write(&path, b"not executed").unwrap();
        let (_, expected) = content_digest::sha256(&mut File::open(&path).unwrap(), 4096).unwrap();
        let tool = open(&path, &expected).unwrap();
        // Evidence records the verified local path, never a handle path.
        assert_eq!(tool.path, root.canonicalize().unwrap().join("tool.exe"));
        assert!(tool.program.is_absolute());
        drop(tool);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn pinned_frame_capture_reports_verified_paths_and_runs_its_programs() {
        use crate::application::frame_capture::FrameCapturePort;
        use crate::infrastructure::frame_capture::FfmpegFrameCapture;
        let _ = crate::infrastructure::os_policy::host();
        let discovered = FfmpegFrameCapture::discover().capability();
        let (Some(ffmpeg), Some(ffprobe)) = (discovered.ffmpeg_path, discovered.ffprobe_path)
        else {
            eprintln!("FFmpeg/FFprobe not installed; pinned frame capture check skipped");
            return;
        };
        let real = |path: String| PathBuf::from(path).canonicalize().unwrap();
        let (ffmpeg, ffprobe) = (real(ffmpeg), real(ffprobe));
        let sha = |path: &Path| {
            content_digest::sha256(&mut File::open(path).unwrap(), 512 * 1024 * 1024)
                .unwrap()
                .1
        };
        let pinned =
            FfmpegFrameCapture::pinned(&ffmpeg, &sha(&ffmpeg), &ffprobe, &sha(&ffprobe)).unwrap();
        let capability = pinned.capability();
        // The version and feature probes ran through the pinned programs.
        assert!(capability.available, "{:?}", capability.unavailable_reason);
        // Evidence names the verified files, never a handle path.
        assert_eq!(
            capability.ffmpeg_path.as_deref(),
            Some(ffmpeg.to_string_lossy().as_ref())
        );
        assert_eq!(
            capability.ffprobe_path.as_deref(),
            Some(ffprobe.to_string_lossy().as_ref())
        );
    }
}
