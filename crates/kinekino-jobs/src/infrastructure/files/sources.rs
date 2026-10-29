use super::*;
use crate::{
    application::SourceBlobStore,
    domain::{MAX_SOURCE_BYTES, SourceBlobRef, valid_sha256},
};
use sha2::{Digest, Sha256};
use std::io::Read;
pub(super) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
impl SourceBlobStore for JobFiles {
    fn put_utf8(&self, source: &str) -> Result<SourceBlobRef, ApiError> {
        let _local = self.local_gate()?;
        if source.is_empty() || source.len() as u64 > MAX_SOURCE_BYTES {
            return Err(invalid("Source snapshot must contain bounded UTF8 text"));
        }
        let reference = SourceBlobRef {
            sha256: digest(source.as_bytes()),
            bytes: DecimalU64(source.len() as u64),
        };
        let _writer = self.writer_lock()?;
        let path = self.sources.join(format!("{}.utf8", reference.sha256));
        match kinekino_core::file_publication::publish(
            path.to_str()
                .ok_or_else(|| invalid("Job source path is not UTF8"))?,
            source.as_bytes(),
        ) {
            Ok(()) => {}
            Err(error) if error.code == ErrorCode::RevisionConflict => {}
            Err(error) => return Err(error),
        }
        self.read_utf8(&reference)?;
        Ok(reference)
    }
    fn read_utf8(&self, source: &SourceBlobRef) -> Result<String, ApiError> {
        if !valid_sha256(&source.sha256) || source.bytes.0 == 0 || source.bytes.0 > MAX_SOURCE_BYTES
        {
            return Err(invalid("Invalid source blob reference"));
        }
        let path = self.sources.join(format!("{}.utf8", source.sha256));
        let mut file = self.paths.file(&path)?;
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(MAX_SOURCE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        if bytes.len() as u64 != source.bytes.0
            || bytes.len() as u64 > MAX_SOURCE_BYTES
            || digest(&bytes) != source.sha256
        {
            return Err(corrupt());
        }
        String::from_utf8(bytes).map_err(|_| corrupt())
    }
}
