use crate::{
    application::api::{ApiError, ErrorCode},
    domain::{api_values::DecimalU64, proposals::*},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestCue {
    id: String,
    in_us: u64,
    out_us: u64,
    text: String,
}
/// Declared request coordinates, never a verified media clock or playback lease.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredCueTime {
    pub cue_id: String,
    pub declared_in_us: DecimalU64,
    pub declared_out_us: DecimalU64,
}
/// Additive inspection context. Canonical snapshots and acceptance hashes retain
/// their existing representation; no conversion to verified SourceCue times exists.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportedRequestContext {
    pub source: SourceSnapshot,
    pub declared_cue_times: Vec<DeclaredCueTime>,
}

pub fn from_request_asset_with_context(
    source_id: String,
    revision: DecimalU64,
    source_sha256: String,
    asset_utf8: String,
    task: ProposalTask,
) -> Result<ImportedRequestContext, ApiError> {
    if asset_utf8.len() > MAX_DOCUMENT_BYTES {
        return Err(invalid("Source asset exceeds the document byte bound"));
    }
    // Validate hash, byte bound, selector, cue identities and range pairs through
    // the original importer before exposing any supplemental declared metadata.
    let source = from_request_asset(source_id, revision, source_sha256, asset_utf8.clone(), task)?;
    let declared_cue_times = if task == ProposalTask::Correction {
        Vec::new()
    } else {
        let suffix = request_source_payload(&asset_utf8, task).map_err(invalid)?;
        let cues: Vec<RequestCue> = serde_json::from_str(&suffix)
            .map_err(|_| invalid("Source suffix must be one complete cue array"))?;
        cues.into_iter()
            .map(|cue| DeclaredCueTime {
                cue_id: cue.id,
                declared_in_us: DecimalU64(cue.in_us),
                declared_out_us: DecimalU64(cue.out_us),
            })
            .collect()
    };
    Ok(ImportedRequestContext {
        source,
        declared_cue_times,
    })
}
fn invalid(m: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, m)
}
/// Snapshot adaptation only. Does not rewrite model output, infer times or record quality decisions.
pub fn from_request_asset(
    source_id: String,
    revision: DecimalU64,
    source_sha256: String,
    asset_utf8: String,
    task: ProposalTask,
) -> Result<SourceSnapshot, ApiError> {
    if asset_utf8.len() > MAX_DOCUMENT_BYTES
        || format!("{:x}", Sha256::digest(asset_utf8.as_bytes())) != source_sha256
    {
        return Err(invalid(
            "Source asset bytes differ from the supplied SHA256 or bound",
        ));
    }
    let snapshot = if task == ProposalTask::Correction {
        SourceSnapshot::Document(
            DocumentSourceSnapshot::from_asset(source_id, revision, source_sha256, asset_utf8)
                .map_err(invalid)?,
        )
    } else {
        let suffix = request_source_payload(&asset_utf8, task).map_err(invalid)?;
        let cues: Vec<RequestCue> = serde_json::from_str(&suffix)
            .map_err(|_| invalid("Source suffix must be one complete cue array"))?;
        if cues.iter().any(|c| c.in_us >= c.out_us) {
            return Err(invalid("Invalid declared source cue time range"));
        }
        SourceSnapshot::Cue(CueSourceSnapshot {
            source_id,
            source_revision: revision,
            source_sha256,
            cues: cues
                .into_iter()
                .map(|c| SourceCue {
                    cue_id: c.id,
                    text: c.text,
                    start_us: None,
                    end_us: None,
                })
                .collect(),
        })
    };
    snapshot.validate().map_err(invalid)?;
    Ok(snapshot)
}
