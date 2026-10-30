use super::{ApiError, ProposalFiles, WriterReservation, paths, records};
use crate::{
    application::proposals::output_review::StoredOutputDecision,
    domain::proposals::output_review::OutputDecision,
    infrastructure::{immutable_storage_paths as rooted, new_file, proposals::io_error},
};
use std::fs;

#[derive(Clone, Copy, Debug)]
pub(super) enum Phase {
    Reserved,
    Published,
}
pub(super) fn publish(
    store: &ProposalFiles,
    writer: &WriterReservation<'_>,
    decision: &OutputDecision,
) -> Result<StoredOutputDecision, ApiError> {
    publish_with(store, writer, decision, |_| Ok(()))
}
pub(super) fn publish_with(
    store: &ProposalFiles,
    writer: &WriterReservation<'_>,
    decision: &OutputDecision,
    mut checkpoint: impl FnMut(Phase) -> Result<(), ApiError>,
) -> Result<StoredOutputDecision, ApiError> {
    // Encode before any namespace/identity reservation.
    let (body, sha) = records::encode(decision)?;
    let id = &decision.input.target.proposal_id;
    let directory = paths::history(store, writer, id, true)?.ok_or_else(super::incomplete)?;
    let target = paths::decision_path(&directory, &decision.input.decision_id)?;
    fs::create_dir(&target).map_err(io_error)?;
    let mut phase = "reserved";
    let result = (|| {
        let _guard = rooted::directory(&target)?;
        checkpoint(Phase::Reserved)?;
        let file = target.join(format!("{sha}.json"));
        phase = "file_publication";
        new_file::publish(file.to_str().ok_or_else(super::incomplete)?, &body)?;
        phase = "published";
        checkpoint(Phase::Published)?;
        phase = "readback";
        let record = records::read(&target, id)?;
        if record.decision_sha256 != sha || record.decision != *decision {
            return Err(super::incomplete());
        }
        Ok(record)
    })();
    result.map_err(|mut error: ApiError| {
        error.details = serde_json::json!({"cause":error.details,"publication_may_exist":true,"phase":phase,"reserved_path":target.to_string_lossy(),"recovery":"preserve_and_inspect"});
        error
    })
}
