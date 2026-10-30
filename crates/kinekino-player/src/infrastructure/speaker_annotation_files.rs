//! Ordinary local JSON leases and create-only publication. No model/media execution.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        review_source::MutationControl,
        speaker_annotations::{SpeakerDocumentFiles, SpeakerInput},
    },
    domain::speaker_annotations::MAX_DOCUMENT_BYTES,
    infrastructure::{content_digest, immutable_storage_paths as rooted, new_file},
};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};
pub struct LocalSpeakerFiles;
struct Input {
    bytes: Vec<u8>,
    file: File,
    _guards: Vec<File>,
}
impl SpeakerInput for Input {
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    fn check_current(&self) -> Result<(), ApiError> {
        let mut reader = self.file.try_clone().map_err(io_error)?;
        reader.seek(SeekFrom::Start(0)).map_err(io_error)?;
        let (_, observed) = content_digest::sha256(&mut reader, MAX_DOCUMENT_BYTES as u64)?;
        if observed != format!("{:x}", Sha256::digest(&self.bytes)) {
            Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original speaker file changed",
            ))
        } else {
            Ok(())
        }
    }
}
impl SpeakerDocumentFiles for LocalSpeakerFiles {
    fn retain(
        &self,
        path: &str,
        control: &dyn MutationControl,
    ) -> Result<Box<dyn SpeakerInput>, ApiError> {
        control.checkpoint()?;
        let path = Path::new(path);
        let (_, guards) = rooted::root(path.parent().ok_or_else(invalid)?)?;
        let mut file = rooted::file(path)?;
        if file.metadata().map_err(io_error)?.len() > MAX_DOCUMENT_BYTES as u64 {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Speaker document exceeds byte capacity",
            ));
        }
        let mut bytes = Vec::new();
        let mut buffer = [0u8; 65536];
        loop {
            control.checkpoint()?;
            let count = file.read(&mut buffer).map_err(io_error)?;
            if count == 0 {
                break;
            }
            if bytes.len() + count > MAX_DOCUMENT_BYTES {
                return Err(ApiError::new(
                    ErrorCode::ResourceExhausted,
                    "Speaker document exceeds byte capacity",
                ));
            }
            bytes.extend_from_slice(&buffer[..count]);
        }
        Ok(Box::new(Input {
            bytes,
            file,
            _guards: guards,
        }))
    }
    fn publish_new(
        &self,
        path: &str,
        bytes: &[u8],
        input: &dyn SpeakerInput,
        control: &dyn MutationControl,
    ) -> Result<(), ApiError> {
        let output = Path::new(path);
        let (_, _guards) = rooted::root(output.parent().ok_or_else(invalid)?)?;
        if bytes.len() > MAX_DOCUMENT_BYTES {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Speaker output exceeds byte capacity",
            ));
        }
        input.check_current()?;
        control.checkpoint()?;
        new_file::publish_guarded(path, bytes, || {
            input.check_current()?;
            control.checkpoint()
        })
    }
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Absolute ordinary speaker JSON path required",
    )
}
fn io_error(error: std::io::Error) -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_handle_hash_rejects_same_length_fault_injection() {
        let root = std::env::temp_dir().join(format!(
            "speaker-recheck-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("source.json");
        std::fs::write(&path, b"before").unwrap();
        // Deliberately permissive test handle to inject a writer the production
        // read-only share guard ordinarily rejects. This proves the hash recheck.
        let input = Input {
            bytes: b"before".to_vec(),
            file: File::open(&path).unwrap(),
            _guards: Vec::new(),
        };
        input.check_current().unwrap();
        std::fs::write(&path, b"after!").unwrap();
        assert_eq!(
            input.check_current().unwrap_err().code,
            ErrorCode::RevisionConflict
        );
    }
}
