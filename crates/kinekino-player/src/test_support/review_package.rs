//! Synthetic review-package construction, separate from historical provenance.
//! Integration-test support; no model execution or human quality verdict.
use kinekino_player::domain::{api_values::DecimalU64, model_review::*};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub(crate) const SOURCE: &[u8] = b"Original fixed cue";
pub(crate) const BASELINE: &[u8] = b"Reference fixed cue";
pub(crate) const CANDIDATE: &[u8] = b"Candidate fixed cue";

fn asset(id: &str, bytes: &[u8]) -> EvidenceAsset {
    EvidenceAsset {
        asset_id: id.into(),
        sha256: format!("{:x}", Sha256::digest(bytes)),
        bytes: DecimalU64(bytes.len() as u64),
        media_type: "text/plain".into(),
    }
}

fn result(id: &str, bytes: &[u8]) -> ReviewResult {
    ReviewResult {
        condition: ModelCondition {
            model_id: "synthetic-fixture".into(),
            revision: "synthetic-v1".into(),
            precision: "synthetic-text".into(),
            engine_id: "synthetic-fixture".into(),
            engine_revision: "synthetic-v1".into(),
            input_conditions: BTreeMap::from([("origin".into(), "synthetic".into())]),
        },
        assets: vec![asset(id, bytes)],
        automated_checks: vec![AutomatedCheck {
            check_id: "fixture-integrity".into(),
            outcome: CheckOutcome::Passed,
            summary: "Synthetic integrity fixture; not a model quality result.".into(),
            evidence: vec![],
        }],
        measured_resources: BTreeMap::from([("execution".into(), "not-performed".into())]),
    }
}

pub(crate) fn package() -> ReviewPackage {
    ReviewPackage {
        schema_version: "1".into(),
        package_id: "synthetic-review-v1".into(),
        items: vec![ReviewItem {
            item_id: "sample-001".into(),
            task: "correction".into(),
            source: asset("source", SOURCE),
            range: None,
            baseline: Some(result("baseline", BASELINE)),
            candidate: result("candidate", CANDIDATE),
            criteria: vec![ReviewCriterion {
                criterion_id: "text-integrity".into(),
                how_to_review: "Compare the explicitly supplied synthetic text assets.".into(),
                expected: "Content identities match; no automatic quality approval.".into(),
            }],
            known_issues: vec![],
        }],
    }
}
