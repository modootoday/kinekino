use super::*;
use meari::application::{Language, plugins::PluginScope, state_program::StateProgram};

fn review(task: ProposalTask) -> ReviewSession {
    let component = meari::parse(
        "type Props={};export function Scope(props:Props){return <main/>;}",
        Language::Tsx,
    )
    .unwrap();
    let program = StateProgram::mount(component, "manual-repair-test".into(), 1).unwrap();
    let view = PluginScope {
        instance: program.snapshot().unwrap().instance,
        source_generation: 1,
    };
    let stored = StoredProposal {
        document: ProposalDocument {
            schema_version: "1".into(),
            proposal_id: "repair-proposal".into(),
            source: SourceSnapshot::Cue(CueSourceSnapshot {
                source_id: "repair-source".into(),
                source_revision: DecimalU64(1),
                source_sha256: "a".repeat(64),
                cues: vec![SourceCue {
                    cue_id: "chosen-cue".into(),
                    text: "원문".into(),
                    start_us: None,
                    end_us: None,
                }],
            }),
            generated: GeneratedResult {
                task,
                raw_text: "{truncated model output".into(),
            },
            draft_revision: DecimalU64(1),
            state: DraftState::NeedsRepair,
            draft: None,
            import_error: Some("typed schema mismatch".into()),
        },
        acceptance: None,
        historical: None,
    };
    ReviewSession::new(view, stored).unwrap()
}

#[test]
fn cue_tasks_require_explicit_selection_and_start_preserving_raw_and_stored_state() {
    for task in [
        ProposalTask::Correction,
        ProposalTask::Cards,
        ProposalTask::Highlights,
    ] {
        let mut review = review(task);
        let scope = review.scope().clone();
        let before = serde_json::to_vec(review.stored()).unwrap();
        assert!(
            review
                .manual_repair_entry()
                .unwrap()
                .selected_source
                .is_none()
        );
        assert_eq!(
            review.begin_manual_repair(&scope),
            Err(ViewError::InvalidSelection)
        );
        review.choose_manual_source(&scope, "1".into()).unwrap();
        assert_eq!(
            review.manual_repair_entry().unwrap().selected_source,
            Some("원문")
        );
        assert!(review.draft.is_none());
        assert!(!review.dirty());
        review.begin_manual_repair(&scope).unwrap();
        assert!(review.dirty());
        assert_eq!(serde_json::to_vec(review.stored()).unwrap(), before);
        assert_eq!(
            review.begin_manual_repair(&scope),
            Err(ViewError::ExistingDraft)
        );
        let fields = review.text_fields();
        assert_eq!(
            fields.len(),
            if task == ProposalTask::Cards { 2 } else { 1 }
        );
        assert!(fields.iter().all(|field| field.text.is_empty()));
        match review.draft.as_ref().unwrap() {
            ProposalPayload::Correction { edits } => assert_eq!(edits[0].cue_id, "chosen-cue"),
            ProposalPayload::Cards { cards } => assert_eq!(cards[0].source_refs, ["chosen-cue"]),
            ProposalPayload::Highlights { highlights } => {
                assert_eq!(highlights[0].source_refs, ["chosen-cue"])
            }
            _ => panic!("cue source cannot create a document edit"),
        }
        review
            .change_text(&scope, &fields[0].input, "사람 입력".into())
            .unwrap();
        assert_eq!(
            review.begin_save(&scope).unwrap().expected_revision,
            DecimalU64(1)
        );
        assert_eq!(review.begin_manual_repair(&scope), Err(ViewError::Busy));
        assert_eq!(serde_json::to_vec(review.stored()).unwrap(), before);
    }
}

#[test]
fn invalid_or_foreign_selection_cannot_create_a_draft() {
    let mut review = review(ProposalTask::Cards);
    let scope = review.scope().clone();
    for choice in ["", "0", "2", "chosen-cue", "1,2"] {
        assert_eq!(
            review.choose_manual_source(&scope, choice.into()),
            Err(ViewError::InvalidSelection)
        );
        assert_eq!(
            review.begin_manual_repair(&scope),
            Err(ViewError::InvalidSelection)
        );
        assert!(review.draft.is_none());
    }
    let mut foreign = scope.clone();
    foreign.source_id = "foreign-source".into();
    assert_eq!(
        review.choose_manual_source(&foreign, "1".into()),
        Err(ViewError::Stale)
    );
    assert_eq!(review.begin_manual_repair(&foreign), Err(ViewError::Stale));
    assert!(!review.dirty());
}

#[test]
fn manual_panel_bootstrap_and_bound_projection_match_without_implicit_draft() {
    use super::super::super::{
        IntentPins,
        bootstrap::{BootstrapLabels, BootstrapSelection, BootstrapTarget, project_initial},
        panel_model::{PanelBudget, PanelLabels},
        repair_panel::prepare_repair_panel,
        review_feedback::FeedbackLabels,
    };
    use meari::domain::{Value, build::ContentHash};
    for task in [
        ProposalTask::Correction,
        ProposalTask::Cards,
        ProposalTask::Highlights,
    ] {
        let mut review = review(task);
        let scope = review.scope().clone();
        let labels = || PanelLabels {
            feedback: FeedbackLabels::korean(),
            revision: "문서1".into(),
            source_identity: "원문".into(),
            quality_unknown: "미판정".into(),
            selected_item: "직접 입력".into(),
            range: "시간 미확인".into(),
            result: String::new(),
        };
        let prepare = |review: &ReviewSession| {
            prepare_repair_panel(
                review,
                review.scope(),
                labels(),
                PanelBudget {
                    entries: 64,
                    text_bytes: 65536,
                },
                IntentPins {
                    contract: ContentHash::of(b"contract"),
                    provider: ContentHash::of(b"provider"),
                    request: ContentHash::of(b"request"),
                    reply: ContentHash::of(b"reply"),
                },
            )
            .unwrap()
        };
        let selected = BootstrapSelection {
            proposal_id: scope.proposal_id.clone(),
            document_revision: DecimalU64(1),
            source_id: scope.source_id.clone(),
            source_revision: scope.source_revision,
            source_sha256: scope.source_sha256.clone(),
            target: BootstrapTarget::ManualRepair { task },
        };
        let initial = project_initial(
            review.stored(),
            &selected,
            BootstrapLabels {
                panel: labels(),
                source_count_label: String::new(),
                unselected_source_text: String::new(),
            },
        )
        .unwrap();
        assert_eq!(initial.props, prepare(&review).props);
        assert!(review.draft.is_none());
        review.choose_manual_source(&scope, "1".into()).unwrap();
        let selected_props = prepare(&review).props;
        assert_eq!(selected_props["startDisabled"], Value::Bool(false));
        assert_eq!(selected_props["canSave"], Value::Bool(false));
        review.begin_manual_repair(&scope).unwrap();
        assert_eq!(prepare(&review).props["canAccept"], Value::Bool(false));
        review.begin_save(&scope).unwrap();
        let pending = prepare(&review).props;
        for name in [
            "cardDisabled",
            "highlightDisabled",
            "correctionDisabled",
            "selectionDisabled",
            "startDisabled",
        ] {
            assert_eq!(pending[name], Value::Bool(true));
        }
        assert_eq!(pending["canSave"], Value::Bool(false));
        assert_eq!(pending["canAccept"], Value::Bool(false));
    }
}

#[test]
fn document_repair_reuses_explicit_canonical_text_reference() {
    use meari::domain::build::ContentHash;
    let initial = review(ProposalTask::Correction);
    let mut stored = initial.stored().clone();
    let prefix = "Edit this ASR draft. Correct punctuation only. Do not guess unclear words. JSON only: {\"proposed\":string,\"uncertain_words\":[string]}. Under100words. /no_think\n";
    let asset = serde_json::json!({"messages":[{"role":"user","content":format!("{prefix}[\"original\"]")}]}).to_string();
    let hash = ContentHash::of(asset.as_bytes());
    stored.document.source = SourceSnapshot::Document(
        DocumentSourceSnapshot::from_asset(
            "repair-source".into(),
            DecimalU64(1),
            hash.as_str().into(),
            asset,
        )
        .unwrap(),
    );
    stored.document.schema_version = "2".into();
    let mut review = ReviewSession::new(initial.scope().view.clone(), stored).unwrap();
    let scope = review.scope().clone();
    let before = serde_json::to_vec(review.stored()).unwrap();
    assert_eq!(
        review.begin_manual_repair(&scope),
        Err(ViewError::InvalidSelection)
    );
    review.choose_manual_source(&scope, "1".into()).unwrap();
    assert!(review.draft.is_none());
    let chosen = review.document_selection().unwrap().clone();
    review.begin_manual_repair(&scope).unwrap();
    let Some(ProposalPayload::DocumentCorrection { edits }) = review.draft.as_ref() else {
        panic!("document repair must keep the typed document reference");
    };
    assert_eq!(edits[0].source_ref, chosen);
    assert!(edits[0].text.is_empty());
    assert_eq!(serde_json::to_vec(review.stored()).unwrap(), before);
}
