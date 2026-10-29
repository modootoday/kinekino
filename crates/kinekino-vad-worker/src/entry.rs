//! Single bounded wire path. Execution authority is supplied by composition.
mod launch;
pub use launch::WorkerLaunch;

use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::domain::ArtifactPin;
use kinekino_vad::protocol::{
    application::{
        ValidatedWorkerRequest, decode_worker_outcome, decode_worker_request,
        encode_worker_response,
    },
    domain::WorkerResult,
};
use std::io::{Read, Write};

/// Validates inherited stdin against the original guarded request before dispatch.
/// Does not create a control/deadline, authorize a profile or acknowledge cleanup.
pub fn serve_request(
    reader: impl Read,
    declared_bytes: u64,
    descriptor_raw: &[u8],
    original_request: &ArtifactPin,
    mut writer: impl Write,
    execute: impl FnOnce(&ValidatedWorkerRequest) -> Result<Vec<u8>, ApiError>,
) -> Result<(), ApiError> {
    let request = decode_worker_request(reader, declared_bytes, descriptor_raw)?;
    if request.expected().request() != original_request {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Inherited stdin differs from original request",
        ));
    }
    let response = match execute(&request) {
        Ok(response) => response,
        Err(error) => encode_worker_response(
            &request,
            &WorkerResult::Failure {
                code: error.code,
                message: error.message,
            },
        )?,
    };
    decode_worker_outcome(
        response.as_slice(),
        response.len() as u64,
        request.expected(),
    )?;
    writer
        .write_all(&response)
        .and_then(|()| writer.flush())
        .map_err(|_| {
            ApiError::new(
                ErrorCode::EngineFailed,
                "Worker response write/flush failed",
            )
        })
}

/// Both default and native-ort builds remain closed until trusted release and
/// native runtime admission are concretely connected; control is not admission.
pub fn unsupported_execution(_: &ValidatedWorkerRequest) -> Result<Vec<u8>, ApiError> {
    Err(ApiError::new(
        ErrorCode::Unsupported,
        "Worker trusted release and native runtime admission are not integrated",
    ))
}
