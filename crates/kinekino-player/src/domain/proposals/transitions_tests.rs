use super::*;
use crate::domain::{projects::tests::proposal, proposals::Correction};

fn draft() -> ProposalPayload {
    ProposalPayload::Correction {
        edits: vec![Correction {
            cue_id: "cue-1".into(),
            text: "Repaired.".into(),
        }],
    }
}

#[test]
fn edit_accept_retry_and_edit_again_keep_the_original_source() {
    let original = proposal();
    let immutable = serde_json::to_vec(&original.document.source).unwrap();
    let edited = edit_state(original, draft(), 1).unwrap();
    assert_eq!(edited.document.draft_revision.0, 2);
    let accepted = accept_state(edited, DecimalU64(1), 2).unwrap();
    assert!(accepted.publication_required);
    accepted.state.validate().unwrap();
    let raw = serde_json::to_vec(&accepted.state).unwrap();
    let retry = accept_state(accepted.state, DecimalU64(1), 2).unwrap();
    assert!(!retry.publication_required);
    assert_eq!(serde_json::to_vec(&retry.state).unwrap(), raw);
    assert_eq!(retry.receipt.output_revision.0, 3);
    let edited_again = edit_state(retry.state, draft(), 3).unwrap();
    assert_eq!(edited_again.document.draft_revision.0, 4);
    assert!(edited_again.acceptance.is_none());
    assert_eq!(
        serde_json::to_vec(&edited_again.document.source).unwrap(),
        immutable
    );
    edited_again.validate().unwrap();
}

#[test]
fn stale_revision_and_wrong_target_cannot_become_a_retry() {
    let edited = edit_state(proposal(), draft(), 1).unwrap();
    assert!(matches!(
        edit_state(edited.clone(), draft(), 1),
        Err(ProposalMutationError::RevisionConflict(_))
    ));
    assert!(matches!(
        accept_state(edited.clone(), DecimalU64(2), 2),
        Err(ProposalMutationError::RevisionConflict(_))
    ));
    let accepted = accept_state(edited, DecimalU64(1), 2).unwrap();
    assert!(matches!(
        accept_state(accepted.state, DecimalU64(2), 2),
        Err(ProposalMutationError::RevisionConflict(_))
    ));
    assert!(matches!(
        accept_state(proposal(), DecimalU64(1), 1),
        Err(ProposalMutationError::NeedsRepair)
    ));
}
