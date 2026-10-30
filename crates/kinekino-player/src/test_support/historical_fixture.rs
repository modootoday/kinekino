//! Synthetic mock-reader data, unrelated to preserved model observations.
//! Available only through the root cfg(test) support module.
use crate::{
    application::ai_provenance::VerifiedHistoricalSample,
    domain::proposals::{CompanionReference, ProposalTask},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{io::Read, path::Path};

/// Explicit runtime input for optional actual-observation tests; no path search.
pub(crate) fn read_companion(
    path: Option<&Path>,
    expected_sha256: &str,
    expected_bytes: u64,
) -> Result<serde_json::Value, String> {
    let path = path.ok_or("HISTORICAL_COMPANION_PATH must be explicitly supplied")?;
    if !path.is_absolute() || expected_bytes > 1_048_576 {
        return Err("Companion requires an absolute path and bounded size".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|error| error.to_string())?
        .take(expected_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 != expected_bytes
        || format!("{:x}", Sha256::digest(&bytes)) != expected_sha256
    {
        return Err("Actual companion differs from its preserved identity".into());
    }
    serde_json::from_slice(&bytes).map_err(|error| error.to_string())
}

#[test]
fn actual_input_requires_an_explicit_absolute_path_before_io() {
    assert!(read_companion(None, &"0".repeat(64), 1).is_err());
    assert!(read_companion(Some(Path::new("relative.json")), &"0".repeat(64), 1).is_err());
}

/// Deliberately invalid candidate preserves the NeedsRepair exercise.
/// This constructs mock data; it does not verify a historical evidence package.
pub(crate) fn correction() -> VerifiedHistoricalSample {
    let instruction = r#"Edit this ASR draft. Correct punctuation only. Do not guess unclear words. JSON only: {"proposed":string,"uncertain_words":[string]}. Under100words. "#;
    let source = serde_json::to_string(&json!({
        "messages": [{"role": "user", "content": format!(
            "{instruction}/no_think\n[\"Synthetic sentence without punctuation\"]"
        )}]
    }))
    .unwrap();
    let candidate = "Synthetic malformed candidate; no model inference or quality approval.";
    let digest = |bytes: &[u8]| format!("{:x}", Sha256::digest(bytes));
    let source_sha256 = digest(source.as_bytes());
    let candidate_sha256 = digest(candidate.as_bytes());
    let companion = serde_json::to_vec(&json!({
        "fixture_version": "1",
        "synthetic": true,
        "items": [{
            "item_id": "synthetic-correction",
            "source": {"sha256": source_sha256, "bytes": source.len()},
            "candidate": {"sha256": candidate_sha256, "bytes": candidate.len()}
        }]
    }))
    .unwrap();
    VerifiedHistoricalSample {
        reference: CompanionReference {
            sha256: digest(&companion),
            bytes: companion.len() as u64,
            item_id: "synthetic-correction".into(),
        },
        task: ProposalTask::Correction,
        source_sha256,
        source_bytes: source.len() as u64,
        source_request_utf8: source,
        candidate_sha256,
        candidate_bytes: candidate.len() as u64,
        candidate_raw_utf8: candidate.into(),
    }
}
