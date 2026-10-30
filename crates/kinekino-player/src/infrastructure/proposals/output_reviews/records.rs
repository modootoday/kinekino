use super::{
    ApiError, ErrorCode, MAX_HISTORY, MAX_RECORD, ProposalFiles, WriterReservation, incomplete,
    paths,
};
use crate::{
    application::proposals::output_review::StoredOutputDecision,
    domain::{identifiers::valid_sha256, proposals::output_review::OutputDecision},
    infrastructure::{content_digest, immutable_storage_paths as rooted, proposals::io_error},
};
use std::{
    fs,
    io::{Cursor, Read},
    path::Path,
};

pub(super) fn encode(decision: &OutputDecision) -> Result<(Vec<u8>, String), ApiError> {
    decision
        .validate_stored()
        .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
    let bytes = serde_json::to_vec(decision).map_err(|_| incomplete())?;
    let (_, sha) = content_digest::sha256(&mut Cursor::new(&bytes), MAX_RECORD)?;
    Ok((bytes, sha))
}
pub(super) fn read(path: &Path, proposal_id: &str) -> Result<StoredOutputDecision, ApiError> {
    let id = paths::decision_id(path)?;
    let _guard = rooted::directory(path)?;
    let entries = fs::read_dir(path)
        .map_err(io_error)?
        .take(2)
        .collect::<Result<Vec<_>, _>>()
        .map_err(io_error)?;
    if entries.len() != 1 {
        return Err(incomplete());
    }
    let file_path = entries[0].path();
    let sha = file_path
        .file_stem()
        .and_then(|name| name.to_str())
        .filter(|sha| valid_sha256(sha))
        .ok_or_else(incomplete)?;
    if file_path.extension().and_then(|s| s.to_str()) != Some("json") {
        return Err(incomplete());
    }
    let mut file = rooted::file(&file_path)?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(MAX_RECORD + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if content_digest::sha256(&mut Cursor::new(&bytes), MAX_RECORD)?.1 != sha {
        return Err(incomplete());
    }
    let decision: OutputDecision = serde_json::from_slice(&bytes).map_err(|_| incomplete())?;
    decision.validate_stored().map_err(|_| incomplete())?;
    if decision.input.decision_id != id || decision.input.target.proposal_id != proposal_id {
        return Err(incomplete());
    }
    Ok(StoredOutputDecision {
        decision_sha256: sha.into(),
        decision,
    })
}
pub(super) fn history(
    store: &ProposalFiles,
    writer: &WriterReservation<'_>,
    id: &str,
) -> Result<Vec<StoredOutputDecision>, ApiError> {
    let Some(directory) = paths::history(store, writer, id, false)? else {
        return Ok(Vec::new());
    };
    let mut entries = Vec::new();
    for entry in fs::read_dir(&directory.path).map_err(io_error)? {
        let path = entry.map_err(io_error)?.path();
        let decision_id = paths::decision_id(&path)?;
        entries.push((decision_id, path));
        if entries.len() > MAX_HISTORY {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Output decision history exceeds its bound",
            ));
        }
    }
    entries.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    entries
        .into_iter()
        .map(|(_, path)| read(&path, id))
        .collect()
}
