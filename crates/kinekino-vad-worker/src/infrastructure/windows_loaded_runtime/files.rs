use super::{
    check,
    deployment::{FixedNativeDeployment, PinnedFile},
};
use crate::staged_files::immutable_storage_paths;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::{
    application::{AnalysisControl, ArtifactReader, read_verified_artifact},
    domain::{ArtifactPin, MAX_RUNTIME_BYTES, MAX_RUNTIME_NOTICE_BYTES},
};
use kinekino_vad_executor::application::{MAX_RELEASE_MANIFEST_BYTES, decode_release_manifest};
use std::{fs::File, io::Read, path::PathBuf};
pub(super) struct GuardedFile {
    pub canonical: PathBuf,
    pin: ArtifactPin,
    path: PathBuf,
    _file: File,
    _directories: Vec<File>,
}
impl ArtifactReader for GuardedFile {
    fn open(&self, pin: &ArtifactPin) -> Result<Box<dyn Read + '_>, ApiError> {
        if pin != &self.pin {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Native file pin changed",
            ));
        }
        Ok(Box::new(File::open(&self.path).map_err(|e| {
            ApiError::new(ErrorCode::EngineFailed, e.to_string())
        })?))
    }
}
fn file(
    expected: &PinnedFile,
    max: u64,
    pe: bool,
    control: &dyn AnalysisControl,
) -> Result<(GuardedFile, Vec<u8>), ApiError> {
    check(control)?;
    let parent = expected
        .path
        .parent()
        .ok_or_else(|| ApiError::new(ErrorCode::InvalidArgument, "Native file parent missing"))?;
    let (_, directories) = immutable_storage_paths::root(parent)?;
    let original = immutable_storage_paths::file(&expected.path)?;
    let guard = GuardedFile {
        canonical: expected
            .path
            .canonicalize()
            .map_err(|e| ApiError::new(ErrorCode::EngineFailed, e.to_string()))?,
        pin: expected.pin.clone(),
        path: expected.path.clone(),
        _file: original,
        _directories: directories,
    };
    let bytes = read_verified_artifact(&guard, &expected.pin, max, control)?;
    if pe {
        let offset = bytes
            .get(60..64)
            .and_then(|n| <[u8; 4]>::try_from(n).ok())
            .map(u32::from_le_bytes)
            .map(|n| n as usize)
            .ok_or_else(|| ApiError::new(ErrorCode::SourceExpired, "Native PE header missing"))?;
        let end = offset
            .checked_add(6)
            .ok_or_else(|| ApiError::new(ErrorCode::SourceExpired, "Native PE offset overflow"))?;
        if bytes.get(..2) != Some(b"MZ".as_slice())
            || bytes.get(offset..end) != Some(b"PE\0\0\x64\x86".as_slice())
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Native artifact is not the pinned x64 PE",
            ));
        }
    }
    Ok((guard, bytes))
}
pub(super) struct GuardedDeployment {
    pub worker: GuardedFile,
    pub modules: Vec<GuardedFile>,
    _manifest: GuardedFile,
    _notices: Vec<GuardedFile>,
}
impl GuardedDeployment {
    pub fn open(
        config: &FixedNativeDeployment,
        control: &dyn AnalysisControl,
    ) -> Result<Self, ApiError> {
        let (manifest, raw) = file(&config.manifest, MAX_RELEASE_MANIFEST_BYTES, false, control)?;
        let release = decode_release_manifest(&raw, &config.manifest.pin)?;
        if release.profile() != &config.profile
            || release.artifacts().len() != 3 + config.native_modules.len() + config.notices.len()
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Native deployment differs from original complete release",
            ));
        }
        for expected in config
            .native_modules
            .iter()
            .map(|m| &m.file)
            .chain(config.notices.iter())
        {
            if !release.artifacts().contains(&expected.pin) {
                return Err(ApiError::new(
                    ErrorCode::SourceExpired,
                    "Native file is outside the original manifest",
                ));
            }
        }
        let (worker, _) = file(&config.worker, MAX_RUNTIME_BYTES, true, control)?;
        let mut modules = Vec::new();
        for expected in &config.native_modules {
            modules.push(file(&expected.file, MAX_RUNTIME_BYTES, true, control)?.0);
        }
        let mut notices = Vec::new();
        for expected in &config.notices {
            notices.push(file(expected, MAX_RUNTIME_NOTICE_BYTES, false, control)?.0);
        }
        Ok(Self {
            worker,
            modules,
            _manifest: manifest,
            _notices: notices,
        })
    }
}

#[cfg(test)]
#[path = "files/tests.rs"]
mod tests;
