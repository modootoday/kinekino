use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::io::Read;

pub(super) struct Payload {
    pub bytes: Vec<u8>,
    pub sha256: String,
}
pub(super) fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
pub(super) fn read(reader: impl Read, declared: u64, maximum: u64) -> Result<Payload, ApiError> {
    if declared == 0 {
        return Err(invalid("Empty wire payload declaration"));
    }
    if declared > maximum {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Wire declaration exceeds bound",
        ));
    }
    let length =
        usize::try_from(declared).map_err(|_| invalid("Wire length cannot be represented"))?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| ApiError::new(ErrorCode::ResourceExhausted, "Wire allocation failed"))?;
    let mut reader = reader.take(declared + 1);
    let mut chunk = [0u8; 8192];
    loop {
        let count = reader
            .read(&mut chunk)
            .map_err(|_| ApiError::new(ErrorCode::EngineFailed, "Wire read failed"))?;
        if count == 0 {
            break;
        }
        if count > length - bytes.len() {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Actual wire exceeds declaration",
            ));
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    if bytes.len() != length {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Actual wire shorter than declaration",
        ));
    }
    Ok(Payload {
        sha256: format!("{:x}", Sha256::digest(&bytes)),
        bytes,
    })
}
pub(super) fn parse<T: DeserializeOwned>(payload: &[u8]) -> Result<T, ApiError> {
    let mut decoder = serde_json::Deserializer::from_slice(payload);
    let value: T = serde::Deserialize::deserialize(&mut decoder)
        .map_err(|_| invalid("Invalid strict wire JSON"))?;
    decoder.end().map_err(|_| invalid("Trailing wire data"))?;
    Ok(value)
}
pub(super) fn optional_text(value: &Option<String>, maximum: usize) -> bool {
    value.as_ref().is_none_or(|text| {
        !text.is_empty() && text.len() <= maximum && !text.chars().any(char::is_control)
    })
}
