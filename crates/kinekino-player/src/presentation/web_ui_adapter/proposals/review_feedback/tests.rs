use super::*;
use crate::presentation::web_ui_adapter::proposals::{
    IntentPins, bootstrap::*, correction_panel::*, panel_model::*,
};
use crate::{application::api::ErrorCode, domain::proposals::*};
use meari::{
    application::{Language, plugins::PluginScope, state_program::StateProgram},
    domain::build::ContentHash,
};

const LARGE: u64 = 9_007_199_254_740_993;
fn labels() -> PanelLabels {
    PanelLabels {
        feedback: FeedbackLabels::english(),
        revision: "Custom document".into(),
        result: "Custom context".into(),
        source_identity: "Custom source".into(),
        quality_unknown: "Unknown".into(),
        selected_item: "Explicit item".into(),
        range: "No range".into(),
    }
}
fn stored(task: ProposalTask) -> StoredProposal {
    let source = SourceSnapshot::Cue(CueSourceSnapshot {
        source_id: "feedback-source".into(),
        source_revision: DecimalU64(7),
        source_sha256: "a".repeat(64),
        cues: vec![SourceCue {
            cue_id: "a".into(),
            text: "Original source".into(),
            start_us: None,
            end_us: None,
        }],
    });
    let draft = match task {
        ProposalTask::Cards => ProposalPayload::Cards {
            cards: vec![Card {
                front: "Front".into(),
                back: "Back".into(),
                source_refs: vec!["a".into()],
            }],
        },
        ProposalTask::Highlights => ProposalPayload::Highlights {
            highlights: vec![Highlight {
                title: "Title".into(),
                source_refs: vec!["a".into()],
            }],
        },
        ProposalTask::Correction => ProposalPayload::Correction {
            edits: vec![Correction {
                cue_id: "a".into(),
                text: "Correction".into(),
            }],
        },
    };
    let value = serde_json::json!({"document":{"schema_version":source.schema_version(),"proposal_id":"feedback-a","source":source,"generated":{"task":task,"raw_text":"Unmodified raw"},"draft_revision":LARGE.to_string(),"state":"editable","draft":draft,"import_error":null},"acceptance":null});
    let stored: StoredProposal = serde_json::from_value(value).unwrap();
    stored.validate().unwrap();
    stored
}
fn review(stored: StoredProposal) -> ReviewSession {
    let program = StateProgram::mount(
        meari::parse(
            "type Props={};export function Scope(props:Props){return <main/>;}",
            Language::Tsx,
        )
        .unwrap(),
        "feedback-owner".into(),
        1,
    )
    .unwrap();
    ReviewSession::new(
        PluginScope {
            instance: program.snapshot().unwrap().instance,
            source_generation: 1,
        },
        stored,
    )
    .unwrap()
}
fn pins() -> IntentPins {
    IntentPins {
        contract: ContentHash::of(b"contract"),
        provider: ContentHash::of(b"provider"),
        request: ContentHash::of(b"request"),
        reply: ContentHash::of(b"reply"),
    }
}
fn text<'a>(props: &'a BTreeMap<String, Value>, key: &str) -> &'a str {
    match &props[key] {
        Value::String(text) => text,
        _ => panic!("Expected text"),
    }
}

#[test]
fn exact_decimal_revisions_and_custom_prefixes_survive_injected_catalog() {
    let mut s = stored(ProposalTask::Cards);
    s.acceptance = Some(AcceptanceReceipt {
        proposal_id: s.document.proposal_id.clone(),
        draft_revision: DecimalU64(LARGE - 1),
        source_id: s.document.source.source_id().into(),
        source_revision: s.document.source.source_revision(),
        source_sha256: s.document.source.source_sha256().into(),
        output_revision: DecimalU64(LARGE),
        output: s.document.draft.clone().unwrap(),
        automatic_application_enabled: false,
    });
    s.validate().unwrap();
    let mut props = BTreeMap::new();
    ReviewFeedback::from_stored(&s).apply(&mut props, &labels());
    assert_eq!(
        text(&props, "revisionLabel"),
        "Custom document 9007199254740993 | Output revision 9007199254740993"
    );
    assert_eq!(
        text(&props, "resultLabel"),
        "Custom context | Stored acceptance"
    );
}
#[test]
fn confirmed_response_with_inspect_error_keeps_verified_and_confirmed_revisions_distinct() {
    let s = stored(ProposalTask::Cards);
    let error = ApiError::new(ErrorCode::Internal, "Original inspect error");
    let mut facts = ReviewFeedback::from_stored(&s);
    facts.confirmed = Some(ConfirmedResult {
        kind: ConfirmedKind::Save,
        revision: DecimalU64(LARGE + 1),
    });
    facts.pending = true;
    facts.sdk_error = Some(&error);
    let mut props = BTreeMap::new();
    facts.apply(&mut props, &labels());
    assert_eq!(
        text(&props, "revisionLabel"),
        "Custom document 9007199254740993"
    );
    assert_eq!(
        text(&props, "resultLabel"),
        "Custom context | Save response received 9007199254740994 | Result verification required | SDK error: Original inspect error"
    );
    assert!(!text(&props, "resultLabel").contains("Stored draft"));
}
#[test]
fn sdk_and_local_errors_preserve_source_and_do_not_rewrite_action_guards() {
    let s = stored(ProposalTask::Highlights);
    let error = ApiError::new(ErrorCode::Internal, "SDK original");
    let mut facts =
        ReviewFeedback::from_stored(&s).with_local_error(Some(ViewError::InvalidSelection));
    facts.sdk_error = Some(&error);
    facts.dirty = true;
    let mut props = BTreeMap::from([
        ("canSave".into(), Value::Bool(false)),
        ("canAccept".into(), Value::Bool(false)),
        ("errorText".into(), Value::String("previous".into())),
    ]);
    facts.apply(&mut props, &labels());
    assert_eq!(
        text(&props, "resultLabel"),
        "Custom context | SDK error: SDK original | Input error: Invalid selection | Unsaved changes"
    );
    assert_eq!(
        text(&props, "errorText"),
        "SDK error: SDK original | Input error: Invalid selection"
    );
    assert_eq!(props["canSave"], Value::Bool(false));
    assert_eq!(props["canAccept"], Value::Bool(false));
    assert_eq!(error.message, "SDK original");
}
#[test]
fn pending_dirty_and_repair_are_facts_without_invented_success_history() {
    let s = stored(ProposalTask::Correction);
    let mut props = BTreeMap::new();
    let mut facts = ReviewFeedback::from_stored(&s);
    facts.dirty = true;
    facts.pending = true;
    facts.apply(&mut props, &labels());
    assert_eq!(
        text(&props, "resultLabel"),
        "Custom context | Request pending"
    );
    facts.pending = false;
    facts.apply(&mut props, &labels());
    assert_eq!(
        text(&props, "resultLabel"),
        "Custom context | Unsaved changes"
    );
    facts.dirty = false;
    facts.draft_state = DraftState::NeedsRepair;
    facts.apply(&mut props, &labels());
    assert_eq!(
        text(&props, "resultLabel"),
        "Custom context | Draft repair required"
    );
    facts.confirmed = Some(ConfirmedResult {
        kind: ConfirmedKind::Accept,
        revision: DecimalU64(LARGE + 1),
    });
    facts.apply(&mut props, &labels());
    assert!(
        text(&props, "resultLabel")
            .contains("Accept response received 9007199254740994 | Result verification required")
    );
}
#[test]
fn labels_are_explicitly_injected_without_locale_inference_from_legacy_prefix() {
    let s = stored(ProposalTask::Cards);
    let mut custom = labels();
    custom.revision = "제안 문서".into();
    custom.feedback.stored = "Borrador guardado".into();
    custom.feedback.separator = " / ".into();
    let mut props = BTreeMap::new();
    ReviewFeedback::from_stored(&s).apply(&mut props, &custom);
    assert_eq!(
        text(&props, "resultLabel"),
        "Custom context / Borrador guardado"
    );
    assert_eq!(text(&props, "revisionLabel"), "제안 문서 9007199254740993");
}

#[test]
fn non_error_status_does_not_fill_correction_alert_and_empty_prefix_uses_injected_name() {
    let s = stored(ProposalTask::Correction);
    let mut custom = labels();
    custom.revision.clear();
    custom.feedback.document_revision = "Documento".into();
    let mut props = BTreeMap::from([("errorText".into(), Value::String("old error".into()))]);
    ReviewFeedback::from_stored(&s).apply(&mut props, &custom);
    assert_eq!(text(&props, "revisionLabel"), "Documento 9007199254740993");
    assert_eq!(text(&props, "errorText"), "");
    assert_eq!(text(&props, "resultLabel"), "Custom context | Stored draft");
}
#[test]
fn review_projection_reads_existing_dirty_owner_without_sdk_or_mutation_commands() {
    let mut owner = review(stored(ProposalTask::Cards));
    let scope = owner.scope().clone();
    let draft = owner.stored().document.draft.clone().unwrap();
    owner.set_draft(&scope, draft).unwrap();
    let facts = ReviewFeedback::from_review(&owner);
    assert!(facts.dirty);
    assert!(!facts.pending);
    assert!(facts.confirmed.is_none());
    assert_eq!(facts.document_revision, DecimalU64(LARGE));
    assert_eq!(owner.scope(), &scope);
}
#[test]
fn initial_and_current_three_task_feedback_preserve_original_callback_references() {
    for task in [
        ProposalTask::Cards,
        ProposalTask::Highlights,
        ProposalTask::Correction,
    ] {
        let s = stored(task);
        let target = match task {
            ProposalTask::Cards => BootstrapTarget::Card { index: 0 },
            ProposalTask::Highlights => BootstrapTarget::Highlight { index: 0 },
            ProposalTask::Correction => BootstrapTarget::CueCorrection {
                index: 0,
                cue_id: "a".into(),
            },
        };
        let selection = BootstrapSelection {
            proposal_id: s.document.proposal_id.clone(),
            document_revision: s.document.draft_revision,
            source_id: s.document.source.source_id().into(),
            source_revision: s.document.source.source_revision(),
            source_sha256: s.document.source.source_sha256().into(),
            target,
        };
        let initial = project_initial(
            &s,
            &selection,
            BootstrapLabels {
                panel: labels(),
                source_count_label: "Count".into(),
                unselected_source_text: "Choose".into(),
            },
        )
        .unwrap();
        let owner = review(s.clone());
        let current = if task == ProposalTask::Correction {
            prepare_correction(
                &owner,
                owner.scope(),
                &CorrectionSelection::at(&owner, owner.scope(), 0).unwrap(),
                CorrectionLabels {
                    panel: labels(),
                    source_count: "Count".into(),
                    unselected_source: "Choose".into(),
                },
                PanelBudget {
                    entries: 8,
                    text_bytes: 8192,
                },
                pins(),
            )
            .unwrap()
        } else {
            prepare_panel(
                &owner,
                owner.scope(),
                &PanelSelection::at(&owner, owner.scope(), 0).unwrap(),
                labels(),
                PanelBudget {
                    entries: 8,
                    text_bytes: 8192,
                },
                pins(),
            )
            .unwrap()
        };
        for key in ["revisionLabel", "resultLabel"] {
            assert_eq!(initial.props[key], current.props[key]);
        }
        for key in ["onSave", "onAccept", "onRefresh"] {
            assert_eq!(initial.props[key], current.props[key]);
        }
        assert_eq!(current.props["canSave"], Value::Bool(false));
        assert_eq!(current.props["canAccept"], Value::Bool(true));
    }
}
