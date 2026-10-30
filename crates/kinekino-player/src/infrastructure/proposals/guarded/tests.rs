use super::*;
use crate::{
    application::{
        projects::{self, CreateProject, ProjectContentInput, ProposalRevisionRef},
        proposals::{EditProposal, guarded},
    },
    domain::{
        api_values::DecimalU64,
        proposals::{Correction, ProposalPayload},
    },
    infrastructure::projects::ProjectFiles,
};
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "proposal-guard-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    eprintln!("proposal_guard_fixture_root={}", root.display());
    root
}
fn draft(text: &str) -> ProposalPayload {
    ProposalPayload::Correction {
        edits: vec![Correction {
            cue_id: "cue-1".into(),
            text: text.into(),
        }],
    }
}
fn setup() -> (PathBuf, ProposalFiles, ProjectFiles, guarded::CaptureAnchor) {
    let root = root();
    let proposal_root = root.join("proposals");
    let project_root = root.join("projects");
    std::fs::create_dir(&proposal_root).unwrap();
    std::fs::create_dir(&project_root).unwrap();
    let proposals = ProposalFiles::open(&proposal_root).unwrap();
    proposals
        .publish(None, &crate::domain::projects::tests::proposal())
        .unwrap();
    let projects = ProjectFiles::open(&project_root).unwrap();
    let capture = projects::create(
        &projects,
        &proposals,
        CreateProject {
            project_id: "work".into(),
            title: "Work".into(),
            content: ProjectContentInput {
                watching: None,
                proposals: vec![ProposalRevisionRef::Live {
                    proposal_id: "draft-a".into(),
                    draft_revision: DecimalU64(1),
                    working_draft: None,
                }],
            },
        },
    )
    .unwrap();
    let anchor = guarded::CaptureAnchor {
        project_id: "work".into(),
        project_revision: DecimalU64(1),
        project_revision_sha256: capture.revision_sha256,
        proposal_id: "draft-a".into(),
        capture_sha256: capture.project.proposals[0].capture_sha256.clone(),
    };
    (root, proposals, projects, anchor)
}

#[test]
fn capture_edit_continuation_accept_retry_and_later_edit_share_one_policy() {
    let (_, proposals, projects, anchor) = setup();
    let project_before = projects::inspect(
        &projects,
        projects::InspectProject {
            project_id: "work".into(),
            revision: Some(DecimalU64(1)),
        },
    )
    .unwrap();
    let edited = guarded::edit(&projects, &proposals, &anchor, None, draft("First.")).unwrap();
    assert_eq!(edited.continuation.draft_revision.0, 2);
    assert!(guarded::edit(&projects, &proposals, &anchor, None, draft("Stale.")).is_err());
    let accepted = guarded::accept(
        &projects,
        &proposals,
        &anchor,
        Some(&edited.continuation),
        DecimalU64(1),
    )
    .unwrap();
    assert!(!accepted.reused_acceptance);
    let retry = guarded::accept(
        &projects,
        &proposals,
        &anchor,
        Some(&edited.continuation),
        DecimalU64(1),
    )
    .unwrap();
    assert!(retry.reused_acceptance);
    assert_eq!(
        serde_json::to_vec(&accepted.state).unwrap(),
        serde_json::to_vec(&retry.state).unwrap()
    );
    let later = guarded::edit(
        &projects,
        &proposals,
        &anchor,
        Some(&accepted.continuation),
        draft("Later."),
    )
    .unwrap();
    assert_eq!(later.continuation.draft_revision.0, 4);
    assert!(
        guarded::accept(
            &projects,
            &proposals,
            &anchor,
            Some(&edited.continuation),
            DecimalU64(1)
        )
        .is_err()
    );
    let after = projects::inspect(
        &projects,
        projects::InspectProject {
            project_id: "work".into(),
            revision: Some(DecimalU64(1)),
        },
    )
    .unwrap();
    assert_eq!(after.revision_sha256, project_before.revision_sha256);
    assert_eq!(
        serde_json::to_vec(&after.project).unwrap(),
        serde_json::to_vec(&project_before.project).unwrap()
    );
    // Project storage is never published by proposal mutations.
    assert_eq!(after.latest_revision.0, 1);
}

#[test]
fn identical_proposal_in_another_backend_cannot_use_the_project_anchor() {
    let (root, proposals, projects, anchor) = setup();
    let foreign = root.join("foreign");
    std::fs::create_dir(&foreign).unwrap();
    let foreign = ProposalFiles::open(&foreign).unwrap();
    foreign
        .publish(None, &proposals.load("draft-a").unwrap())
        .unwrap();
    foreign.capture_current("draft-a", 1).unwrap();
    let error = guarded::edit(&projects, &foreign, &anchor, None, draft("Foreign.")).unwrap_err();
    assert_eq!(error.code, ErrorCode::RevisionConflict);
    assert_eq!(
        foreign.load("draft-a").unwrap().document.draft_revision.0,
        1
    );
}

#[test]
fn retry_checks_full_pre_state_and_target_before_returning_receipt() {
    let (_, proposals, projects, anchor) = setup();
    let edited = guarded::edit(&projects, &proposals, &anchor, None, draft("First.")).unwrap();
    guarded::accept(
        &projects,
        &proposals,
        &anchor,
        Some(&edited.continuation),
        DecimalU64(1),
    )
    .unwrap();
    let mut wrong = edited.continuation.clone();
    wrong.state_sha256 = "0".repeat(64);
    assert!(guarded::accept(&projects, &proposals, &anchor, Some(&wrong), DecimalU64(1)).is_err());
    assert!(
        guarded::accept(
            &projects,
            &proposals,
            &anchor,
            Some(&edited.continuation),
            DecimalU64(2)
        )
        .is_err()
    );
    assert_eq!(
        proposals.load("draft-a").unwrap().document.draft_revision.0,
        3
    );
}

#[test]
fn owned_writer_excludes_competing_publication_without_reacquiring() {
    let (_, proposals, projects, anchor) = setup();
    let session = proposals.open_capture_mutation("draft-a").unwrap().unwrap();
    let error = crate::application::proposals::edit(
        &proposals,
        EditProposal {
            proposal_id: "draft-a".into(),
            draft: draft("Competing."),
        },
        1,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::RevisionConflict);
    drop(session);
    guarded::edit(&projects, &proposals, &anchor, None, draft("Owned.")).unwrap();
}

#[test]
fn forged_continuation_and_mismatched_project_anchor_do_not_refresh_current() {
    let (_, proposals, projects, anchor) = setup();
    let mut wrong_anchor = anchor.clone();
    wrong_anchor.project_revision_sha256 = "0".repeat(64);
    assert!(
        guarded::edit(
            &projects,
            &proposals,
            &wrong_anchor,
            None,
            draft("Wrong project.")
        )
        .is_err()
    );
    wrong_anchor = anchor.clone();
    wrong_anchor.project_id = "other-project".into();
    assert!(
        guarded::edit(
            &projects,
            &proposals,
            &wrong_anchor,
            None,
            draft("Other project.")
        )
        .is_err()
    );
    let forged = guarded::CaptureExpectation {
        draft_revision: DecimalU64(1),
        state_sha256: "0".repeat(64),
    };
    assert!(
        guarded::edit(
            &projects,
            &proposals,
            &anchor,
            Some(&forged),
            draft("Forged.")
        )
        .is_err()
    );
    assert_eq!(
        proposals.load("draft-a").unwrap().document.draft_revision.0,
        1
    );
}

#[test]
fn legacy_capture_is_read_only_even_when_live_state_matches() {
    use crate::application::projects::ProjectStore;
    let (_, proposals, projects, anchor) = setup();
    let mut legacy = projects.load("work", Some(1)).unwrap().project;
    legacy.project_id = "legacy".into();
    legacy.schema_version = "1".into();
    legacy.proposals[0].proposal_origin = None;
    let legacy = projects.publish(None, &legacy).unwrap();
    let anchor = guarded::CaptureAnchor {
        project_id: "legacy".into(),
        project_revision_sha256: legacy.revision_sha256,
        ..anchor
    };
    let error = guarded::edit(&projects, &proposals, &anchor, None, draft("Legacy.")).unwrap_err();
    assert_eq!(error.code, ErrorCode::NeedsInput);
    assert_eq!(error.remediation, "create_explicit_live_capture");
    assert_eq!(
        proposals.load("draft-a").unwrap().document.draft_revision.0,
        1
    );
}

#[test]
fn mutation_session_never_issues_missing_origin_metadata() {
    let root = root();
    let proposals = ProposalFiles::open(&root).unwrap();
    proposals
        .publish(None, &crate::domain::projects::tests::proposal())
        .unwrap();
    match proposals.open_capture_mutation("draft-a") {
        Err(error) => assert_eq!(error.code, ErrorCode::NeedsInput),
        Ok(_) => panic!("Missing origin must not open a mutation session"),
    }
    assert!(!root.join(".proposal-store-origin.v1.json").exists());
    assert_eq!(
        proposals.load("draft-a").unwrap().document.draft_revision.0,
        1
    );
}
