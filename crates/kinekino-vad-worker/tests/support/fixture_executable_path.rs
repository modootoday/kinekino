//! Test-artifact current_exe boundary only; shared ordinary-path rules unchanged.
use super::fixture_data::{io, ordinary_paths};
use kinekino_core::platform::{FileIdentity128, OpenFlags, ShareMode};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::{
    fs::{File, OpenOptions},
    path::{Component, Path, PathBuf},
};
fn denied() -> ApiError {
    ApiError::new(
        ErrorCode::PermissionDenied,
        "Fixture executable requires the same ordinary local drive file",
    )
}
/// The OS port supplies the ordinary local spelling (Windows: drive path, no
/// UNC/device prefix); every remaining component must be a plain, unambiguous name.
fn local_drive_path(raw: &Path) -> Result<PathBuf, ApiError> {
    let path = kinekino_core::platform::os()
        .normalize_local_path(raw)
        .ok_or_else(denied)?;
    let mut count = 0;
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir if count == 0 => {}
            Component::Normal(name) => {
                let text = name.to_str().ok_or_else(denied)?;
                if text.is_empty()
                    || text.ends_with('.')
                    || text.ends_with(' ')
                    || text.chars().any(|c| c == ':' || c.is_control())
                {
                    return Err(denied());
                }
                count += 1;
            }
            _ => return Err(denied()),
        }
    }
    if count == 0 {
        return Err(denied());
    }
    Ok(path)
}
fn identity(file: &File) -> Result<FileIdentity128, ApiError> {
    kinekino_core::platform::os()
        .file_identity_128(file)
        .map_err(|error| {
            ApiError::new(
                ErrorCode::Unsupported,
                format!("Fixture physical file identity unavailable: {error}"),
            )
        })
}
fn open(path: &Path, share: ShareMode, directory: bool) -> Result<File, ApiError> {
    let os = kinekino_core::platform::os();
    let mut options = OpenOptions::new();
    options.read(true);
    os.apply_share_mode(&mut options, share);
    os.apply_open_flags(
        &mut options,
        OpenFlags {
            no_follow: true,
            directory_handle: directory,
            ..OpenFlags::default()
        },
    );
    options.open(path).map_err(io)
}
pub struct OrdinaryFixtureExecutable {
    pub path: PathBuf,
    _raw: File,
    _ordinary: File,
    _raw_parent: File,
    _directories: Vec<File>,
}
pub fn open_current_executable(raw: &Path) -> Result<OrdinaryFixtureExecutable, ApiError> {
    let path = local_drive_path(raw)?;
    let parent = path.parent().ok_or_else(denied)?;
    let raw_parent = raw.parent().ok_or_else(denied)?;
    let (_, directories) = ordinary_paths::root(parent)?;
    let ordinary = ordinary_paths::file(&path)?;
    let original = open(raw, ShareMode::ReadOnly, false)?;
    let original_parent = open(raw_parent, ShareMode::ReadWrite, true)?;
    let ordinary_parent = directories.last().ok_or_else(denied)?;
    if identity(&ordinary)? != identity(&original)?
        || identity(ordinary_parent)? != identity(&original_parent)?
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Fixture path representation changed original file/root identity",
        ));
    }
    Ok(OrdinaryFixtureExecutable {
        path,
        _raw: original,
        _ordinary: ordinary,
        _raw_parent: original_parent,
        _directories: directories,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn ordinary_local_path_preserves_the_exact_local_components() {
        kinekino_platform::install();
        let raw = std::env::temp_dir().join("bin").join("fixture.exe");
        assert_eq!(super::local_drive_path(&raw).unwrap(), raw);
    }
    #[test]
    fn relative_parent_and_ambiguous_normalization_are_rejected() {
        kinekino_platform::install();
        let base = std::env::temp_dir().join("bin");
        for raw in [
            std::path::PathBuf::from("bin").join("fixture.exe"),
            base.join("..").join("fixture.exe"),
            base.join("root.").join("fixture.exe"),
            base.join("fixture.exe:stream"),
        ] {
            assert!(super::local_drive_path(&raw).is_err(), "{}", raw.display());
        }
    }
}
