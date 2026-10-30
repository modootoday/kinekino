use crate::application::api::{ApiError, ErrorCode};
use sha2::{Digest, Sha256};
use std::io::{self, Read};
fn failed() -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        "Content identity verification failed",
    )
}
#[derive(Debug)]
pub enum DigestFailure {
    Engine,
    Read(io::Error),
    ByteCountOverflow,
    ByteBoundExceeded { maximum: u64, observed: u64 },
    Checkpoint(ApiError),
}
impl DigestFailure {
    /// Preserve the existing sha256 API's public errors. Typed consumers can
    /// instead retain Read's original kind/OS code and their own bound policy.
    pub fn into_api_error(self) -> ApiError {
        match self {
            Self::Checkpoint(error) => error,
            Self::ByteBoundExceeded { .. } => ApiError::new(
                ErrorCode::ResourceExhausted,
                "Content exceeds verification byte bound",
            ),
            Self::Engine | Self::Read(_) | Self::ByteCountOverflow => failed(),
        }
    }
}
pub fn sha256(reader: &mut impl Read, maximum: u64) -> Result<(u64, String), ApiError> {
    sha256_with_checkpoint(reader, maximum, || Ok(())).map_err(DigestFailure::into_api_error)
}
/// OS-neutral SHA-256 (`sha2`) over 64KiB reads. Check before EVERY read, including
/// the EOF read. The caller retains its original guards/control/deadline.
pub fn sha256_with_checkpoint(
    reader: &mut impl Read,
    maximum: u64,
    mut checkpoint: impl FnMut() -> Result<(), ApiError>,
) -> Result<(u64, String), DigestFailure> {
    let mut hash = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        checkpoint().map_err(DigestFailure::Checkpoint)?;
        let n = reader.read(&mut buffer).map_err(DigestFailure::Read)?;
        if n == 0 {
            break;
        }
        bytes = bytes
            .checked_add(n as u64)
            .ok_or(DigestFailure::ByteCountOverflow)?;
        if bytes > maximum {
            return Err(DigestFailure::ByteBoundExceeded {
                maximum,
                observed: bytes,
            });
        }
        hash.update(&buffer[..n]);
    }
    let output = hash.finalize();
    Ok((bytes, output.iter().map(|b| format!("{b:02x}")).collect()))
}

#[cfg(test)]
mod tests;
