use super::*;
use kinekino_player::domain::{
    api_values::DecimalU64,
    projects::{
        ProjectWorkState, WatchingSelection, WatchingSource, WorkingState, canonical_sha256,
    },
};

fn project() -> ProjectRevision {
    let (_, _, stored) = super::super::scoped_panel::tests::fixture();
    let mut project = ProjectRevision {
        schema_version: "1".into(),
        project_id: "card-project".into(),
        revision: DecimalU64(9_007_199_254_740_993),
        parent_revision: Some(DecimalU64(9_007_199_254_740_992)),
        title: "Saved project".into(),
        watching: None,
        proposals: vec![ProjectProposalSnapshot {
            capture_sha256: canonical_sha256(&stored).unwrap(),
            captured: stored,
            working_draft: None,
            working_state: WorkingState::Absent,
            proposal_origin: None,
        }],
        work_state: ProjectWorkState::default(),
    };
    project.work_state = project.derived_work_state();
    project.validate().unwrap();
    project
}

#[test]
fn explicit_second_card_preserves_capture_identity_and_separate_working_draft() {
    let mut project = project();
    let mut working = project.proposals[0]
        .captured
        .document
        .draft
        .clone()
        .unwrap();
    let ProposalPayload::Cards { cards } = &mut working else {
        panic!("Card fixture required")
    };
    cards[1].front = "Unsaved project wording".into();
    project.proposals[0].working_draft = Some(working);
    project.proposals[0].working_state = WorkingState::Valid;
    project.work_state = project.derived_work_state();
    let choices = ProjectCardChoices::new(&project, NonZeroU64::new(5).unwrap()).unwrap();
    assert_eq!(choices.rows().len(), 2);
    let selected = choices.resolve(choices.rows()[1].key()).unwrap();
    assert_eq!(selected.card().front, "second");
    assert_eq!(selected.item_index(), 1);
    assert!(std::ptr::eq(selected.capture(), &project.proposals[0]));
    assert!(selected.capture().working_draft.is_some());
    assert_eq!(choices.project().revision.0, 9_007_199_254_740_993);
    let selection = selected.live_inspection_selection();
    let original = &project.proposals[0].captured.document;
    assert_eq!(selection.proposal_id, original.proposal_id);
    assert_eq!(selection.document_revision, original.draft_revision);
    assert_eq!(selection.source_id, original.source.source_id());
    assert_eq!(selection.source_revision, original.source.source_revision());
    assert_eq!(selection.source_sha256, original.source.source_sha256());
    assert_eq!(selection.target, NativeProposalTarget::Card { index: 1 });
}

#[test]
fn unknown_payload_and_previous_frame_generation_never_select_first_card() {
    let project = project();
    let old = ProjectCardChoices::new(&project, NonZeroU64::new(7).unwrap()).unwrap();
    let current = ProjectCardChoices::new(&project, NonZeroU64::new(8).unwrap()).unwrap();
    for key in ["", "0", "scoped-card", old.rows()[1].key()] {
        assert!(matches!(
            current.resolve(key),
            Err(CardChoiceError::UnknownChoice)
        ));
    }
    assert_eq!(
        current
            .resolve(current.rows()[1].key())
            .unwrap()
            .item_index(),
        1
    );
}

#[test]
fn domain_capture_integrity_failure_is_not_repaired_by_presentation() {
    let mut project = project();
    project.proposals[0].capture_sha256 = "b".repeat(64);
    assert!(matches!(
        ProjectCardChoices::new(&project, NonZeroU64::new(1).unwrap()),
        Err(CardChoiceError::InvalidProject(_))
    ));
    assert_eq!(project.proposals[0].capture_sha256, "b".repeat(64));
}

#[test]
fn watching_only_project_has_no_implicit_card_selection() {
    let mut project = project();
    project.proposals.clear();
    project.watching = Some(WatchingSelection {
        label: "Selected episode".into(),
        source: WatchingSource::EpisodePage {
            source_id: "episode-source".into(),
            episode_url: "https://catalog.example.test/e/selected".into(),
        },
    });
    project.work_state = project.derived_work_state();
    let choices = ProjectCardChoices::new(&project, NonZeroU64::new(1).unwrap()).unwrap();
    assert!(choices.rows().is_empty());
    assert!(matches!(
        choices.resolve("0"),
        Err(CardChoiceError::UnknownChoice)
    ));
}
