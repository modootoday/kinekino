use super::*;
use crate::domain::model_review::{AutomatedCheck, CheckOutcome, ReviewPackage};
pub(super) fn derive(
    base: &ReviewPackage,
    case: &ReviewCase,
    prefix: &str,
    assets: Vec<EvidenceAsset>,
) -> Result<Vec<u8>, ApiError> {
    let mut package = base.clone();
    if assets
        .iter()
        .any(|asset| !crate::domain::review_source::valid_review_object_id(&asset.asset_id))
    {
        return Err(invalid(
            "Derived evidence must obey the case reference ID contract",
        ));
    }
    let item = package
        .items
        .iter_mut()
        .find(|item| item.item_id == case.item_id)
        .ok_or_else(|| invalid("Derived package item missing"))?;
    if item.source.sha256 != case.source_request_sha256 {
        return Err(conflict("Derived source request differs"));
    }
    let existing = base
        .assets()
        .into_iter()
        .map(|asset| asset.asset_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if assets
        .iter()
        .any(|asset| existing.contains(asset.asset_id.as_str()))
    {
        return Err(conflict(
            "Derived evidence ID collides with original package",
        ));
    }
    for (index, chunk) in assets.chunks(8).enumerate() {
        item.candidate.automated_checks.push(AutomatedCheck { check_id:format!("{prefix}-evidence-{index}"),outcome:CheckOutcome::Passed,
            summary:"Live source timeline technical custody only; model outputs, historical host results and user quality approval unchanged".into(),evidence:chunk.to_vec() });
    }
    package
        .validate()
        .map_err(|_| invalid("Derived package exceeds existing contract or resource bounds"))?;
    let body = serde_json::to_vec(&package)
        .map_err(|e| ApiError::new(ErrorCode::EngineFailed, e.to_string()))?;
    if body.len() > 524288 {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Derived package exceeds existing 512KiB bound",
        ));
    }
    Ok(body)
}
