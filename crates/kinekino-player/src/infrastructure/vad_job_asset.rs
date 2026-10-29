//! Original file guards and bounded artifact reads, independent of input lookup.
use super::vad_job_files::VadJobFile;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::application::{AnalysisControl, ArtifactReader, read_verified_artifact};
use kinekino_vad_executor::{
    ArtifactPin, application::SnapshotLease, infrastructure::GuardedOutputFile,
};
use sha2::{Digest, Sha256};
use std::{cell::RefCell, fs::File, io::Read, path::PathBuf, sync::Arc};

pub(super) struct FixedVadFile {
    pub path: PathBuf,
    pub pin: ArtifactPin,
}
pub(super) struct GuardedAsset {
    path: Arc<VadJobFile>,
    pin: ArtifactPin,
    maximum: u64,
}
struct GuardedArtifactReader<'a> {
    asset: &'a GuardedAsset,
    original: RefCell<Option<File>>,
}
impl ArtifactReader for GuardedArtifactReader<'_> {
    fn open(&self, pin: &ArtifactPin) -> Result<Box<dyn Read + '_>, ApiError> {
        if pin != self.asset.pin() {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Controlled artifact pin differs",
            ));
        }
        let file = self.asset.path.open_actual()?;
        let retained = file
            .try_clone()
            .map_err(kinekino_core::file_publication::io_error)?;
        *self.original.borrow_mut() = Some(retained);
        Ok(Box::new(file))
    }
}
struct FileSnapshotLease {
    _files: Vec<File>,
    _paths: Vec<Arc<VadJobFile>>,
}
impl SnapshotLease for FileSnapshotLease {}
impl GuardedAsset {
    pub(super) fn new(file: FixedVadFile, maximum: u64) -> Result<Self, ApiError> {
        if file.pin.bytes() > maximum {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Fixed VAD artifact exceeds bound",
            ));
        }
        Ok(Self {
            path: Arc::new(VadJobFile::from_application_path(&file.path)?),
            pin: file.pin,
            maximum,
        })
    }
    pub(super) fn pin(&self) -> &ArtifactPin {
        &self.pin
    }
    fn read(&self) -> Result<(Vec<u8>, File), ApiError> {
        let mut original = self.path.open_actual()?;
        let mut bytes = Vec::new();
        Read::by_ref(&mut original)
            .take(self.pin.bytes() + 1)
            .read_to_end(&mut bytes)
            .map_err(kinekino_core::file_publication::io_error)?;
        if bytes.len() as u64 > self.maximum
            || bytes.len() as u64 != self.pin.bytes()
            || format!("{:x}", Sha256::digest(&bytes)) != self.pin.sha256()
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Fixed VAD original file bytes/SHA changed",
            ));
        }
        Ok((bytes, original))
    }
    fn leased(&self, bytes: Vec<u8>, file: File) -> (Vec<u8>, Arc<dyn SnapshotLease>) {
        (
            bytes,
            Arc::new(FileSnapshotLease {
                _files: vec![file],
                _paths: vec![self.path.clone()],
            }),
        )
    }
    pub(super) fn snapshot(&self) -> Result<(Vec<u8>, Arc<dyn SnapshotLease>), ApiError> {
        let (bytes, file) = self.read()?;
        Ok(self.leased(bytes, file))
    }
    pub(super) fn snapshot_with_control(
        &self,
        control: Option<&dyn AnalysisControl>,
    ) -> Result<(Vec<u8>, Arc<dyn SnapshotLease>), ApiError> {
        let Some(control) = control else {
            return self.snapshot();
        };
        let reader = GuardedArtifactReader {
            asset: self,
            original: RefCell::new(None),
        };
        let bytes = read_verified_artifact(&reader, &self.pin, self.maximum, control)?;
        let file = reader.original.into_inner().ok_or_else(|| {
            ApiError::new(
                ErrorCode::Internal,
                "Verified artifact lost its original guard",
            )
        })?;
        Ok(self.leased(bytes, file))
    }
}
