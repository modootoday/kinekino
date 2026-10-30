use super::*;
use crate::domain::proposals::ProposalStoreOrigin;
use sha2::{Digest, Sha256};

fn legacy_bytes() -> Vec<u8> {
    // Frozen v1 field order/shape, independent of the new typed serializer.
    let captured = r#"{"document":{"schema_version":"1","proposal_id":"draft-a","source":{"source_id":"source-a","source_revision":"1","source_sha256":"__SHA64__","cues":[{"cue_id":"cue-1","text":"original","start_us":null,"end_us":null}]},"generated":{"task":"correction","raw_text":"invalid model output"},"draft_revision":"1","state":"needs_repair","draft":null,"import_error":"Needs repair"},"acceptance":null}"#.replace("__SHA64__", &"a".repeat(64));
    assert_eq!(
        serde_json::to_vec(&tests::proposal()).unwrap(),
        captured.as_bytes()
    );
    let capture_sha = format!("{:x}", Sha256::digest(captured.as_bytes()));
    r#"{"schema_version":"1","project_id":"legacy-work","revision":"1","parent_revision":null,"title":"Legacy","watching":null,"proposals":[{"capture_sha256":"__CAPTURE_HASH__","captured":__CAPTURED__,"working_draft":null,"working_state":"absent"}],"work_state":{"needs_repair":1,"editable":0,"accepted_output":0,"working_needs_input":0}}"#
        .replace("__CAPTURE_HASH__", &capture_sha).replace("__CAPTURED__", &captured).into_bytes()
}
#[test]
fn legacy_v1_exact_bytes_and_canonical_sha_survive_absent_origin() {
    let raw = legacy_bytes();
    let expected = format!("{:x}", Sha256::digest(&raw));
    let project: ProjectRevision = serde_json::from_slice(&raw).unwrap();
    project.validate().unwrap();
    assert!(project.proposals[0].proposal_origin.is_none());
    assert_eq!(serde_json::to_vec(&project).unwrap(), raw);
    assert_eq!(canonical_sha256(&project).unwrap(), expected);
}
#[test]
fn v2_mixed_origin_keeps_legacy_capture_hash_and_rejects_v1_upgrade() {
    let mut project: ProjectRevision = serde_json::from_slice(&legacy_bytes()).unwrap();
    let original = project.proposals[0].capture_sha256.clone();
    let mut legacy = project.proposals[0].clone();
    legacy.captured.document.proposal_id = "draft-b".into();
    legacy.capture_sha256 = canonical_sha256(&legacy.captured).unwrap();
    project.proposals.push(legacy);
    project.proposals[0].proposal_origin =
        Some(ProposalStoreOrigin::issued("a".repeat(32)).unwrap());
    project.work_state = project.derived_work_state();
    assert!(project.validate().is_err());
    project.schema_version = "2".into();
    project.validate().unwrap();
    let restored: ProjectRevision =
        serde_json::from_slice(&serde_json::to_vec(&project).unwrap()).unwrap();
    restored.validate().unwrap();
    assert_eq!(restored.proposals[0].capture_sha256, original);
    assert!(restored.proposals[0].proposal_origin.is_some());
    assert!(restored.proposals[1].proposal_origin.is_none());
}
#[test]
fn frozen_legacy_snapshot_reader_rejects_new_origin_field() {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct LegacySnapshot {
        capture_sha256: String,
        captured: StoredProposal,
        working_draft: Option<ProposalPayload>,
        working_state: WorkingState,
    }
    let project: ProjectRevision = serde_json::from_slice(&legacy_bytes()).unwrap();
    let mut snapshot = project.proposals[0].clone();
    let old: LegacySnapshot =
        serde_json::from_value(serde_json::to_value(&snapshot).unwrap()).unwrap();
    assert_eq!(old.capture_sha256, canonical_sha256(&old.captured).unwrap());
    assert!(old.working_draft.is_none());
    assert_eq!(old.working_state, WorkingState::Absent);
    snapshot.proposal_origin = Some(ProposalStoreOrigin::issued("a".repeat(32)).unwrap());
    assert!(
        serde_json::from_value::<LegacySnapshot>(serde_json::to_value(snapshot).unwrap()).is_err()
    );
}
