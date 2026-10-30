use super::*;

#[test]
fn real_two_handle_session_reservation_blocks_public_writer_until_drop() {
    let root = Temp::new();
    let first = ProposalFiles::open(&root.0).unwrap();
    seed(&first);
    let second = ProposalFiles::open(&root.0).unwrap();
    let session = first.open_output_review("proposal-1").unwrap().unwrap();
    assert_eq!(
        second.load("proposal-1").unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let state = session.current().clone();
    assert_eq!(
        second.publish(Some(2), &state).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    session.verify_current_evidence().unwrap();
    drop(session);
    assert_eq!(
        second.load("proposal-1").unwrap().document.draft_revision.0,
        2
    );
}

#[test]
fn selection_before_edit_is_stale_and_current_same_id_retry_is_not_rebound() {
    let root = Temp::new();
    let first = ProposalFiles::open(&root.0).unwrap();
    seed(&first);
    record(&first, "decision-1");
    let selected = prepared(&first.load("proposal-1").unwrap(), "decision-1");
    let second = ProposalFiles::open(&root.0).unwrap();
    proposals::edit(
        &second,
        EditProposal {
            proposal_id: "proposal-1".into(),
            draft: ProposalPayload::Correction {
                edits: vec![Correction {
                    cue_id: "cue-1".into(),
                    text: "Different synthetic text.".into(),
                }],
            },
        },
        2,
    )
    .unwrap();
    let mut structural = first.open_output_review("proposal-1").unwrap().unwrap();
    assert!(structural.current().acceptance.is_none());
    assert_eq!(
        structural.append(&selected).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    drop(structural);
    use crate::application::proposals::output_review::{
        InspectOutputReviewTarget, RecordOutputDecision, inspect_target, record_decision,
    };
    assert_eq!(
        inspect_target(
            &first,
            InspectOutputReviewTarget {
                proposal_id: "proposal-1".into()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::NeedsInput
    );
    assert_eq!(
        record_decision(
            &first,
            RecordOutputDecision {
                proposal_id: "proposal-1".into(),
                decision: selected.input().clone()
            },
            2
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    let mut unavailable = selected.input().clone();
    unavailable.target.draft_revision = DecimalU64(2);
    unavailable.target.output_revision = DecimalU64(3);
    assert_eq!(
        record_decision(
            &first,
            RecordOutputDecision {
                proposal_id: "proposal-1".into(),
                decision: unavailable
            },
            3
        )
        .unwrap_err()
        .code,
        ErrorCode::NeedsInput
    );
    assert_eq!(
        first.output_decisions("proposal-1").unwrap().records[0]
            .decision
            .input,
        *selected.input()
    );
    proposals::accept(
        &second,
        AcceptProposal {
            proposal_id: "proposal-1".into(),
            target_source_revision: DecimalU64(7),
        },
        3,
    )
    .unwrap();
    let mut session = first.open_output_review("proposal-1").unwrap().unwrap();
    assert_eq!(
        session.append(&selected).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    drop(session);
    assert_eq!(
        first.output_decisions("proposal-1").unwrap().records.len(),
        1
    );
}

#[test]
fn guard_from_other_root_is_rejected_before_private_reads_or_record_paths() {
    let a = Temp::new();
    let b = Temp::new();
    let first = ProposalFiles::open(&a.0).unwrap();
    let second = ProposalFiles::open(&b.0).unwrap();
    seed(&first);
    seed(&second);
    let guard = WriterReservation::acquire(&first).unwrap();
    assert_eq!(
        second
            .read_current_locked(&guard, "proposal-1")
            .unwrap_err()
            .code,
        ErrorCode::PermissionDenied
    );
    assert!(
        matches!(paths::history(&second,&guard,"proposal-1",true),Err(error) if error.code==ErrorCode::PermissionDenied)
    );
    assert!(!b.0.join("output-decisions").exists());
}
