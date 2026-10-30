//! Explicit installed Rust verifier adapter. No implicit tool or workspace search.
mod process;
#[cfg(test)]
mod tests;
use crate::{
    application::{
        ai_provenance::{CompanionReference, HistoricalEvidenceReader, VerifiedHistoricalSample},
        api::{ApiError, ErrorCode},
    },
    domain::{identifiers::valid_sha256, proposals::ProposalTask},
    infrastructure::{content_digest, immutable_storage_paths as paths},
};
use serde::Deserialize;
use std::{
    fs::File,
    io::Cursor,
    path::{Path, PathBuf},
};

pub struct HistoricalVerifier {
    executable: PathBuf,
    executable_sha256: String,
    root: PathBuf,
    label: String,
    process: process::Runner,
    _guards: Vec<File>,
}

fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid historical verifier input",
    )
}
fn mismatch() -> ApiError {
    ApiError::new(
        ErrorCode::NeedsInput,
        "Historical evidence is missing or differs",
    )
}

impl HistoricalVerifier {
    pub fn open(
        executable: &Path,
        sha256: String,
        root: &Path,
        label: String,
    ) -> Result<Self, ApiError> {
        if !executable.is_absolute()
            || !valid_sha256(&sha256)
            || label.is_empty()
            || label.len() > 80
            || !label
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        {
            return Err(invalid());
        }
        let (directory, mut guards) = paths::root(executable.parent().ok_or_else(invalid)?)?;
        let executable = directory.join(executable.file_name().ok_or_else(invalid)?);
        let (root, root_guards) = paths::root(root)?;
        guards.extend(root_guards);
        let verifier = Self {
            executable,
            executable_sha256: sha256,
            root,
            label,
            process: process::Runner::default(),
            _guards: guards,
        };
        verifier.pinned_executable()?;
        Ok(verifier)
    }
    fn pinned_executable(&self) -> Result<File, ApiError> {
        let mut file = paths::file(&self.executable)?;
        if content_digest::sha256(&mut file, 134_217_728)?.1 != self.executable_sha256 {
            return Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "Historical verifier executable pin differs",
            ));
        }
        Ok(file)
    }
}

#[derive(Deserialize)]
struct Asset {
    sha256: String,
    bytes: u64,
}
#[derive(Deserialize)]
struct Evidence {
    item_id: String,
    task: String,
    source: Asset,
    candidate: Asset,
}
#[derive(Deserialize)]
struct Imported {
    companion: Asset,
    evidence: Evidence,
    source_request_utf8: String,
    candidate_raw_utf8: String,
}

impl HistoricalEvidenceReader for HistoricalVerifier {
    fn verify(&self, reference: &CompanionReference) -> Result<VerifiedHistoricalSample, ApiError> {
        self.process.recover()?;
        reference.validate().map_err(|_| invalid())?;
        let _executable_guard = self.pinned_executable()?;
        let bytes = self.process.inspect(
            &self.executable,
            &self.root,
            &self.label,
            &reference.item_id,
        )?;
        let imported: Imported = serde_json::from_slice(&bytes).map_err(|_| mismatch())?;
        let e = imported.evidence;
        let task = match e.task.as_str() {
            "correction" => ProposalTask::Correction,
            "cards" => ProposalTask::Cards,
            "highlights" => ProposalTask::Highlights,
            _ => return Err(mismatch()),
        };
        if imported.companion.sha256 != reference.sha256
            || imported.companion.bytes != reference.bytes
            || e.item_id != reference.item_id
            || content_digest::sha256(
                &mut Cursor::new(imported.source_request_utf8.as_bytes()),
                1_048_576,
            )? != (e.source.bytes, e.source.sha256.clone())
            || content_digest::sha256(
                &mut Cursor::new(imported.candidate_raw_utf8.as_bytes()),
                262_144,
            )? != (e.candidate.bytes, e.candidate.sha256.clone())
        {
            return Err(mismatch());
        }
        Ok(VerifiedHistoricalSample {
            reference: reference.clone(),
            task,
            source_sha256: e.source.sha256,
            source_bytes: e.source.bytes,
            source_request_utf8: imported.source_request_utf8,
            candidate_sha256: e.candidate.sha256,
            candidate_bytes: e.candidate.bytes,
            candidate_raw_utf8: imported.candidate_raw_utf8,
        })
    }
}
