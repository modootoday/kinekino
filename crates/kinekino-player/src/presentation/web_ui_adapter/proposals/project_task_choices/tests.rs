use super::*;
use kinekino_player::domain::{
    api_values::DecimalU64,
    projects::{ProjectWorkState, WorkingState, canonical_sha256},
    proposals::{
        CueSourceSnapshot, DocumentSourceSnapshot, DraftState, GeneratedResult, ProposalDocument,
        ProposalTask, SourceCue, SourceSnapshot, StoredProposal,
    },
};
use sha2::{Digest, Sha256};

fn cue_source() -> SourceSnapshot {
    SourceSnapshot::Cue(CueSourceSnapshot {
        source_id: "captions".into(),
        source_revision: DecimalU64(9_007_199_254_740_993),
        source_sha256: "a".repeat(64),
        cues: vec![SourceCue {
            cue_id: "cue-original".into(),
            text: "원문 日本語".into(),
            start_us: None,
            end_us: None,
        }],
    })
}

fn capture(id: &str, source: SourceSnapshot, payload: ProposalPayload) -> ProjectProposalSnapshot {
    let stored = StoredProposal {
        document: ProposalDocument {
            schema_version: source.schema_version().into(),
            proposal_id: id.into(),
            source,
            generated: GeneratedResult {
                task: payload.task(),
                raw_text: serde_json::to_string(&payload).unwrap(),
            },
            draft_revision: DecimalU64(9_007_199_254_740_995),
            state: DraftState::Editable,
            draft: Some(payload),
            import_error: None,
        },
        acceptance: None,
        historical: None,
    };
    stored.validate().unwrap();
    ProjectProposalSnapshot {
        capture_sha256: canonical_sha256(&stored).unwrap(),
        captured: stored,
        working_draft: None,
        working_state: WorkingState::Absent,
        proposal_origin: None,
    }
}

fn project() -> ProjectRevision {
    let content = concat!(
        "Edit this ASR draft. Correct punctuation only. Do not guess unclear words. ",
        "JSON only: {\"proposed\":string,\"uncertain_words\":[string]}. Under100words. ",
        "/no_think\n[\"원본\",\"둘째\"]"
    );
    let asset = serde_json::json!({"messages":[{"role":"user","content":content}]}).to_string();
    let source = DocumentSourceSnapshot::from_asset(
        "document-source".into(),
        DecimalU64(7),
        format!("{:x}", Sha256::digest(asset.as_bytes())),
        asset,
    )
    .unwrap();
    let reference = source.text_reference("/1").unwrap();
    let mut project = ProjectRevision {
        schema_version: "1".into(),
        project_id: "mixed-project".into(),
        revision: DecimalU64(2),
        parent_revision: Some(DecimalU64(1)),
        title: "Mixed tasks".into(),
        watching: None,
        proposals: vec![
            capture(
                "cards",
                cue_source(),
                ProposalPayload::Cards {
                    cards: vec![
                        Card {
                            front: "first".into(),
                            back: "back".into(),
                            source_refs: vec!["cue-original".into()],
                        },
                        Card {
                            front: "second".into(),
                            back: "back".into(),
                            source_refs: vec!["cue-original".into()],
                        },
                    ],
                },
            ),
            capture(
                "highlights",
                cue_source(),
                ProposalPayload::Highlights {
                    highlights: vec![Highlight {
                        title: "Title".into(),
                        source_refs: vec!["cue-original".into()],
                    }],
                },
            ),
            capture(
                "correction",
                cue_source(),
                ProposalPayload::Correction {
                    edits: vec![Correction {
                        cue_id: "cue-original".into(),
                        text: "수정".into(),
                    }],
                },
            ),
            capture(
                "document",
                SourceSnapshot::Document(source),
                ProposalPayload::DocumentCorrection {
                    edits: vec![DocumentCorrection {
                        source_ref: reference,
                        text: "문서 수정".into(),
                    }],
                },
            ),
        ],
        work_state: ProjectWorkState::default(),
    };
    project.work_state = project.derived_work_state();
    project.validate().unwrap();
    project
}

#[test]
fn mixed_tasks_preserve_exact_identity_and_explicit_targets() {
    let project = project();
    let choices = ProjectTaskChoices::new(&project, NonZeroU64::new(3).unwrap()).unwrap();
    assert!(std::ptr::eq(choices.project(), &project));
    assert_eq!(choices.rows().len(), 5);
    let mut keys = std::collections::BTreeSet::new();
    for row in choices.rows() {
        assert!(keys.insert(row.key()));
        let selected = choices.resolve(row.key()).unwrap();
        assert!(std::ptr::eq(selected, row));
        let selection = row.live_inspection_selection();
        let document = &row.capture().captured.document;
        assert_eq!(selection.proposal_id, document.proposal_id);
        assert_eq!(selection.document_revision, document.draft_revision);
        assert_eq!(selection.source_id, document.source.source_id());
        assert_eq!(selection.source_revision, document.source.source_revision());
        assert_eq!(selection.source_sha256, document.source.source_sha256());
        let expected = match row.item() {
            ProjectTaskItem::Card(_) => NativeProposalTarget::Card {
                index: row.item_index(),
            },
            ProjectTaskItem::Highlight(_) => NativeProposalTarget::Highlight {
                index: row.item_index(),
            },
            ProjectTaskItem::CueCorrection(edit) => NativeProposalTarget::CueCorrection {
                index: row.item_index(),
                cue_id: edit.cue_id.clone(),
            },
            ProjectTaskItem::DocumentCorrection(edit) => NativeProposalTarget::DocumentCorrection {
                index: row.item_index(),
                source_ref: edit.source_ref.clone(),
            },
        };
        assert_eq!(selection.target, expected);
    }
    assert_eq!(
        choices.rows()[1].live_inspection_selection().target,
        NativeProposalTarget::Card { index: 1 }
    );
    assert!(
        matches!(choices.rows()[4].live_inspection_selection().target, NativeProposalTarget::DocumentCorrection { source_ref, .. } if source_ref.inner_pointer == "/1")
    );
}

#[test]
fn old_generation_foreign_card_keys_and_unknown_input_never_select_first() {
    let project = project();
    let old = ProjectTaskChoices::new(&project, NonZeroU64::new(3).unwrap()).unwrap();
    let current = ProjectTaskChoices::new(&project, NonZeroU64::new(4).unwrap()).unwrap();
    for key in [
        "",
        "0",
        "project-card-4-0-0",
        "project-task-4-99-0",
        old.rows()[1].key(),
    ] {
        assert!(matches!(
            current.resolve(key),
            Err(TaskChoiceError::UnknownChoice)
        ));
    }
}

#[test]
fn captured_content_stays_separate_from_working_draft_and_existing_card_choices() {
    let mut project = project();
    let mut working = project.proposals[0]
        .captured
        .document
        .draft
        .clone()
        .unwrap();
    let ProposalPayload::Cards { cards } = &mut working else {
        panic!("cards")
    };
    cards[1].front = "working only".into();
    project.proposals[0].working_draft = Some(working);
    project.proposals[0].working_state = WorkingState::Valid;
    project.work_state = project.derived_work_state();
    let generation = NonZeroU64::new(8).unwrap();
    let choices = ProjectTaskChoices::new(&project, generation).unwrap();
    let old =
        super::super::project_card_choices::ProjectCardChoices::new(&project, generation).unwrap();
    assert_eq!(old.rows().len(), 2);
    let row = &choices.rows()[1];
    assert!(matches!(row.item(), ProjectTaskItem::Card(card) if card.front == "second"));
    assert!(std::ptr::eq(row.capture(), old.rows()[1].capture()));
    assert!(row.capture().working_draft.is_some());
    assert_eq!(
        row.live_inspection_selection().target,
        old.rows()[1].live_inspection_selection().target
    );
}

#[test]
fn repair_without_draft_has_no_invented_source_reference_or_selection() {
    let mut project = project();
    project
        .proposals
        .retain(|capture| capture.captured.document.source.as_document().is_some());
    let capture = &mut project.proposals[0];
    capture.captured.document.state = DraftState::NeedsRepair;
    capture.captured.document.draft = None;
    capture.captured.document.import_error = Some("repair required".into());
    capture.captured.document.generated.task = ProposalTask::Correction;
    capture.capture_sha256 = canonical_sha256(&capture.captured).unwrap();
    project.work_state = project.derived_work_state();
    let choices = ProjectTaskChoices::new(&project, NonZeroU64::new(1).unwrap()).unwrap();
    assert!(choices.rows().is_empty());
    assert!(matches!(
        choices.resolve("0"),
        Err(TaskChoiceError::UnknownChoice)
    ));
}

#[test]
fn invalid_capture_is_rejected_without_repairing_domain_state() {
    let mut project = project();
    project.proposals[0].capture_sha256 = "b".repeat(64);
    assert!(matches!(
        ProjectTaskChoices::new(&project, NonZeroU64::new(1).unwrap()),
        Err(TaskChoiceError::InvalidProject(_))
    ));
    assert_eq!(project.proposals[0].capture_sha256, "b".repeat(64));
}

#[test]
fn item_index_conversion_never_truncates() {
    assert_eq!(item_index(u32::MAX as usize), Ok(u32::MAX));
    #[cfg(target_pointer_width = "64")]
    assert_eq!(
        item_index(u32::MAX as usize + 1),
        Err(TaskChoiceError::ItemIndexOverflow)
    );
}
