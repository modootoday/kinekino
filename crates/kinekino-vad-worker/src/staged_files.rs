//! Fixed-role files under an already owned attempt root. No id becomes a path.
#[path = "../../kinekino-player/src/infrastructure/immutable_storage_paths.rs"]
#[allow(dead_code)] // Exact shared module; its writer-lock helper is not used here.
pub(crate) mod immutable_storage_paths;

use crate::payload::{PayloadRole, PreparedWorkerPayload};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::protocol::domain::DecodedDescriptor;
use kinekino_vad::{
    application::{AnalysisControl, ArtifactReader, read_verified_artifact},
    domain::{ArtifactPin, VadProfile},
};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

fn io(error: std::io::Error) -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, error.to_string())
}
fn check_stage(control: &dyn AnalysisControl) -> Result<(), ApiError> {
    if control.checkpoint()? {
        return Err(ApiError::new(
            ErrorCode::Cancelled,
            "Payload staging stopped; no cleanup ACK",
        ));
    }
    if Instant::now() >= control.deadline() {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Original payload staging cutoff expired",
        ));
    }
    Ok(())
}
struct GuardedFile {
    path: PathBuf,
    pin: ArtifactPin,
    _original: File,
}
impl ArtifactReader for GuardedFile {
    fn open(&self, pin: &ArtifactPin) -> Result<Box<dyn Read + '_>, ApiError> {
        if pin != &self.pin {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Staged artifact pin differs",
            ));
        }
        // Original guard forbids write/delete; this is an independent cursor.
        Ok(Box::new(File::open(&self.path).map_err(io)?))
    }
}

pub struct StagedWorkerPayload<L: Send + Sync + ?Sized> {
    _directories: Vec<File>,
    _files: Vec<GuardedFile>,
    _original_lease: Arc<L>,
}
impl<L: Send + Sync + ?Sized> PreparedWorkerPayload<L> {
    /// Leaves partial evidence intact on error. Caller retains the original
    /// admission owner on that path; success owns the same Arc through cleanup.
    pub fn stage(
        &self,
        owned_attempt_root: &Path,
        control: &dyn AnalysisControl,
    ) -> Result<StagedWorkerPayload<L>, ApiError> {
        let (_, directories) = immutable_storage_paths::root(owned_attempt_root)?;
        let mut files = Vec::new();
        let mut write = |name: &str, pin: ArtifactPin, bytes: &[u8], maximum: u64| {
            check_stage(control)?;
            let path = owned_attempt_root.join(name);
            {
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .map_err(io)?;
                for chunk in bytes.chunks(8192) {
                    check_stage(control)?;
                    file.write_all(chunk).map_err(io)?;
                }
                file.sync_all().map_err(io)?;
            }
            let guarded = GuardedFile {
                _original: immutable_storage_paths::file(&path)?,
                path,
                pin,
            };
            // Verify the reopened original under its immutable guard, closing
            // the write-to-read gap with the shared bounded digest reader.
            read_verified_artifact(&guarded, &guarded.pin, maximum, control)?;
            files.push(guarded);
            Ok::<_, ApiError>(())
        };
        write(
            "descriptor.json",
            self.descriptor_pin.clone(),
            &self.descriptor,
            kinekino_vad::protocol::application::MAX_DESCRIPTOR_BYTES,
        )?;
        for artifact in &self.artifacts {
            write(
                artifact.role.name(),
                artifact.pin.clone(),
                &artifact.bytes,
                artifact.role.maximum(),
            )?;
        }
        Ok(StagedWorkerPayload {
            _directories: directories,
            _files: files,
            _original_lease: self.original_lease.clone(),
        })
    }
}

/// Child opens fixed-role files and its actual trusted executable location.
/// The parent remains responsible for approved release and original process
/// guard/identity. This constructor creates no ProfileAdmission credential.
pub struct WorkerArtifactFiles {
    _directories: Vec<File>,
    files: Vec<GuardedFile>,
}
impl WorkerArtifactFiles {
    pub fn open(
        owned_attempt_root: &Path,
        executable: &Path,
        descriptor: &DecodedDescriptor,
        approved: &VadProfile,
    ) -> Result<Self, ApiError> {
        if descriptor.profile() != approved {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Worker release differs",
            ));
        }
        if !executable.is_absolute()
            || executable
                .file_name()
                .and_then(|name| name.to_str())
                .is_none_or(|name| name.contains(':') || name.chars().any(char::is_control))
        {
            return Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "Ordinary executable filename required",
            ));
        }
        let (_, mut directories) = immutable_storage_paths::root(owned_attempt_root)?;
        let executable_parent = executable.parent().ok_or_else(|| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Worker executable parent absent",
            )
        })?;
        let (_, executable_directories) = immutable_storage_paths::root(executable_parent)?;
        directories.extend(executable_directories);
        let roles = [
            (PayloadRole::Model.name(), approved.weights()),
            (PayloadRole::License.name(), approved.license_document()),
            (PayloadRole::Wav.name(), descriptor.input().wav()),
        ];
        let mut files = Vec::new();
        for (name, pin) in roles {
            let path = owned_attempt_root.join(name);
            files.push(GuardedFile {
                _original: immutable_storage_paths::file(&path)?,
                path,
                pin: pin.clone(),
            });
        }
        files.push(GuardedFile {
            _original: immutable_storage_paths::file(executable)?,
            path: executable.to_owned(),
            pin: approved.runtime().clone(),
        });
        Ok(Self {
            _directories: directories,
            files,
        })
    }
}
impl ArtifactReader for WorkerArtifactFiles {
    fn open(&self, pin: &ArtifactPin) -> Result<Box<dyn Read + '_>, ApiError> {
        self.files
            .iter()
            .find(|file| &file.pin == pin)
            .ok_or_else(|| {
                ApiError::new(ErrorCode::NotFound, "Artifact outside fixed worker payload")
            })?
            .open(pin)
    }
}

/// Ordinary original request/descriptor handles, not release admission.
/// No model/runtime is opened and no operation budget is created here.
pub struct WorkerRequestFiles {
    _directories: Vec<File>,
    _request: File,
    _descriptor: File,
    descriptor_raw: Vec<u8>,
    original_request: ArtifactPin,
}
impl WorkerRequestFiles {
    pub fn open(request: &Path) -> Result<Self, ApiError> {
        if request.file_name().and_then(|name| name.to_str()) != Some("stdin.raw") {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Original stdin.raw required",
            ));
        }
        let root = request
            .parent()
            .ok_or_else(|| ApiError::new(ErrorCode::InvalidArgument, "Request root absent"))?;
        let (_, directories) = immutable_storage_paths::root(root)?;
        let mut request_file = immutable_storage_paths::file(request)?;
        let mut descriptor_file = immutable_storage_paths::file(&root.join("descriptor.json"))?;
        let mut descriptor_raw = Vec::new();
        Read::by_ref(&mut descriptor_file)
            .take(kinekino_vad::protocol::application::MAX_DESCRIPTOR_BYTES + 1)
            .read_to_end(&mut descriptor_raw)
            .map_err(io)?;
        if descriptor_raw.len() as u64 > kinekino_vad::protocol::application::MAX_DESCRIPTOR_BYTES {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Original descriptor exceeds bound",
            ));
        }
        let declared = request_file.metadata().map_err(io)?.len();
        let decoded = kinekino_vad::protocol::application::decode_worker_request(
            &mut request_file,
            declared,
            &descriptor_raw,
        )?;
        Ok(Self {
            _directories: directories,
            _request: request_file,
            _descriptor: descriptor_file,
            descriptor_raw,
            original_request: decoded.expected().request().clone(),
        })
    }
    pub fn declared_bytes(&self) -> u64 {
        self.original_request.bytes()
    }
    pub fn descriptor_raw(&self) -> &[u8] {
        &self.descriptor_raw
    }
    pub fn original_request(&self) -> &ArtifactPin {
        &self.original_request
    }
}
