//! Explicit trusted desktop launch values; never sourced from browser/plugin messages.
use crate::application::api::{ApiError, ErrorCode};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct NativePlaybackConfiguration {
    pub catalog: PathBuf,
    pub executable: PathBuf,
    pub sha256: String,
}
impl NativePlaybackConfiguration {
    pub fn from_explicit(
        catalog: PathBuf,
        executable: PathBuf,
        sha256: String,
    ) -> Result<Self, ApiError> {
        if !catalog.is_absolute()
            || !executable.is_absolute()
            || !crate::domain::identifiers::valid_sha256(&sha256)
        {
            return Err(invalid());
        }
        Ok(Self {
            catalog,
            executable,
            sha256,
        })
    }
}
pub fn desktop_configuration(
    args: &[String],
) -> Result<Option<NativePlaybackConfiguration>, ApiError> {
    let mut values: [Option<String>; 3] = [None, None, None];
    let mut index = 0;
    while index < args.len() {
        let argument = &args[index];
        let found = ["--catalog-root", "--mpv-executable", "--mpv-sha256"]
            .iter()
            .enumerate()
            .find_map(|(slot, name)| {
                if argument == name {
                    Some((slot, None))
                } else {
                    argument
                        .strip_prefix(&format!("{name}="))
                        .map(|value| (slot, Some(value)))
                }
            });
        let Some((slot, inline)) = found else {
            index += 1;
            continue;
        };
        let value = match inline {
            Some(value) => value,
            None => {
                index += 1;
                args.get(index)
                    .filter(|value| !value.starts_with("--"))
                    .ok_or_else(invalid)?
                    .as_str()
            }
        };
        if value.is_empty() || values[slot].replace(value.into()).is_some() {
            return Err(invalid());
        }
        index += 1;
    }
    if values.iter().all(Option::is_none) {
        return Ok(None);
    }
    let [Some(catalog), Some(executable), Some(sha256)] = values else {
        return Err(invalid());
    };
    if super::desktop_launch_plan(args)?.mode != super::DesktopMode::NativeViewer {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Native playback configuration requires Native Viewer",
        ));
    }
    NativePlaybackConfiguration::from_explicit(catalog.into(), executable.into(), sha256).map(Some)
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Native playback requires explicit absolute catalog/mpv paths and a SHA256 pin; duplicate or partial options are invalid",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_values_reject_partial_relative_unpinned_or_duplicate_inputs() {
        assert!(
            NativePlaybackConfiguration::from_explicit(
                "relative".into(),
                "relative".into(),
                "a".repeat(64)
            )
            .is_err()
        );
        assert!(desktop_configuration(&["--catalog-root=D:/catalog".into()]).is_err());
        assert!(
            desktop_configuration(&["--mpv-sha256=a".into(), "--mpv-sha256=b".into()]).is_err()
        );
        assert!(
            desktop_configuration(&["--mpv-executable".into(), "--mpv-sha256".into()]).is_err()
        );
        assert!(desktop_configuration(&[]).unwrap().is_none());
    }
}
