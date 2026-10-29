use crate::staged_files::immutable_storage_paths;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::{
    application::{AnalysisControl, ArtifactReader, read_verified_artifact},
    domain::ArtifactPin,
};
use kinekino_vad_executor::application::{
    MAX_RELEASE_MANIFEST_BYTES, VadReleaseManifest, decode_release_manifest,
};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
struct Reader {
    path: PathBuf,
    pin: ArtifactPin,
    _file: File,
}
impl ArtifactReader for Reader {
    fn open(&self, pin: &ArtifactPin) -> Result<Box<dyn Read + '_>, ApiError> {
        if pin != &self.pin {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original release pin differs",
            ));
        }
        Ok(Box::new(File::open(&self.path).map_err(|e| {
            ApiError::new(ErrorCode::EngineFailed, e.to_string())
        })?))
    }
}
pub(super) struct OriginalRelease {
    release: VadReleaseManifest,
    _directories: Vec<File>,
    _reader: Reader,
}
impl OriginalRelease {
    pub(super) fn open(
        root: &Path,
        pin: &ArtifactPin,
        control: &dyn AnalysisControl,
    ) -> Result<Self, ApiError> {
        if pin.bytes() > MAX_RELEASE_MANIFEST_BYTES {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Original release exceeds bound",
            ));
        }
        let (_, directories) = immutable_storage_paths::root(root)?;
        let path = root.join("release.manifest.json");
        let reader = Reader {
            _file: immutable_storage_paths::file(&path)?,
            path,
            pin: pin.clone(),
        };
        let bytes = read_verified_artifact(&reader, pin, MAX_RELEASE_MANIFEST_BYTES, control)?;
        let release = decode_release_manifest(&bytes, pin)?;
        Ok(Self {
            release,
            _directories: directories,
            _reader: reader,
        })
    }
    pub(super) fn release(&self) -> &VadReleaseManifest {
        &self.release
    }
}
