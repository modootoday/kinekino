use super::super::panel_model::PanelLabels;
use super::*;
use kinekino_player::domain::api_values::DecimalU64;
use meari::{
    application::{Language, plugins::PluginScope, state_program::StateProgram},
    domain::build::ContentHash,
};

fn labels() -> CorrectionLabels {
    CorrectionLabels {
        panel: PanelLabels {
            feedback: crate::presentation::web_ui_adapter::proposals::review_feedback::FeedbackLabels::korean(),
            revision: "문서 2".into(),
            source_identity: "명시 원본".into(),
            quality_unknown: "품질 미판정".into(),
            selected_item: "두 번째 원문".into(),
            range: String::new(),
            result: String::new(),
        },
        source_count: "원문 2개".into(),
        unselected_source: "원문을 선택하세요".into(),
    }
}
fn pins() -> IntentPins {
    IntentPins {
        contract: ContentHash::of(b"contract"),
        provider: ContentHash::of(b"provider"),
        request: ContentHash::of(b"request"),
        reply: ContentHash::of(b"reply"),
    }
}
fn cue_source() -> SourceSnapshot {
    SourceSnapshot::Cue(CueSourceSnapshot {
        source_id: "source-a".into(),
        source_revision: DecimalU64(7),
        source_sha256: "a".repeat(64),
        cues: ["a", "b"]
            .into_iter()
            .map(|cue_id| SourceCue {
                cue_id: cue_id.into(),
                text: format!("원문 {cue_id}"),
                start_us: None,
                end_us: None,
            })
            .collect(),
    })
}
fn document_source() -> SourceSnapshot {
    let prefix = "Edit this ASR draft. Correct punctuation only. Do not guess unclear words. JSON only: {\"proposed\":string,\"uncertain_words\":[string]}. Under100words. /no_think\n";
    let asset=serde_json::json!({"messages":[{"role":"user","content":format!("{prefix}[\"원문 1\",\"原文 2\"]")}]}).to_string();
    SourceSnapshot::Document(
        DocumentSourceSnapshot::from_asset(
            "source-doc".into(),
            DecimalU64(7),
            ContentHash::of(asset.as_bytes()).as_str().into(),
            asset,
        )
        .unwrap(),
    )
}
fn review(source: SourceSnapshot, draft: Option<ProposalPayload>) -> ReviewSession {
    let state = if draft.is_some() {
        "editable"
    } else {
        "needs_repair"
    };
    let stored=serde_json::from_value(serde_json::json!({"document":{"schema_version":source.schema_version(),"proposal_id":"correction-a","source":source,"generated":{"task":"correction","raw_text":"preserved raw"},"draft_revision":"2","state":state,"draft":draft,"import_error":"preserved error"},"acceptance":null})).unwrap();
    let program = StateProgram::mount(
        meari::parse(
            "type Props={};export function Scope(props:Props){return <main/>;}",
            Language::Tsx,
        )
        .unwrap(),
        "correction-panel".into(),
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
fn prepare(
    review: &ReviewSession,
    selected: &CorrectionSelection,
) -> Result<PreparedPanel, PanelPreparationError> {
    prepare_correction(
        review,
        review.scope(),
        selected,
        labels(),
        PanelBudget {
            entries: 8,
            text_bytes: 8192,
        },
        pins(),
    )
}
#[test]
fn selected_initial_repair_matches_live_full_reference_and_unselected_stays_empty() {
    let mut review = review(document_source(), None);
    let scope = review.scope().clone();
    let source = review.stored().document.source.as_document().unwrap();
    let reference = source.text_reference("/1").unwrap();
    let unselected =
        initial_correction_props_with_selection(review.stored(), None, None, labels()).unwrap();
    assert_eq!(unselected["sourceChoice"], Value::String(String::new()));
    assert_eq!(unselected["startDisabled"], Value::Bool(true));
    let selected =
        initial_correction_props_with_selection(review.stored(), None, Some(&reference), labels())
            .unwrap();
    assert_eq!(selected["sourceChoice"], Value::String("2".into()));
    assert_eq!(selected["startDisabled"], Value::Bool(false));
    review.choose_document_text(&scope, "2".into()).unwrap();
    let live = prepare(
        &review,
        &CorrectionSelection::repair(&review, &scope).unwrap(),
    )
    .unwrap()
    .props;
    assert_eq!(selected, live);
    let mut foreign = reference;
    foreign.source_asset_sha256 = "f".repeat(64);
    assert!(
        initial_correction_props_with_selection(review.stored(), None, Some(&foreign), labels())
            .is_err()
    );
}
#[test]
fn selected_second_cue_preserves_original_and_independent_working_field() {
    let mut review = review(
        cue_source(),
        Some(ProposalPayload::Correction {
            edits: vec![
                Correction {
                    cue_id: "a".into(),
                    text: "수정 a".into(),
                },
                Correction {
                    cue_id: "b".into(),
                    text: "수정 b".into(),
                },
            ],
        }),
    );
    let selected = CorrectionSelection::at(&review, review.scope(), 1).unwrap();
    let props = prepare(&review, &selected).unwrap().props;
    assert_eq!(props["sourceText"], Value::String("원문 b".into()));
    assert_eq!(props["workingText"], Value::String("수정 b".into()));
    let scope = review.scope().clone();
    let field = review.text_fields().remove(1);
    review
        .change_text(&scope, &field.input, "日本語 수정".into())
        .unwrap();
    assert!(matches!(
        prepare(&review, &selected),
        Err(PanelPreparationError::View(ViewError::Stale))
    ));
    let selected = CorrectionSelection::at(&review, &scope, 1).unwrap();
    let props = prepare(&review, &selected).unwrap().props;
    assert_eq!(props["workingText"], Value::String("日本語 수정".into()));
    assert_eq!(props["canSave"], Value::Bool(true));
    assert_eq!(props["canAccept"], Value::Bool(false));
    assert_eq!(review.stored().document.source.cues()[1].text, "원문 b");
}
#[test]
fn old_field_generation_cannot_be_reused_after_replacing_draft() {
    let draft = ProposalPayload::Correction {
        edits: vec![Correction {
            cue_id: "b".into(),
            text: "수정 b".into(),
        }],
    };
    let mut review = review(cue_source(), Some(draft.clone()));
    let selected = CorrectionSelection::at(&review, review.scope(), 0).unwrap();
    let scope = review.scope().clone();
    review.set_draft(&scope, draft).unwrap();
    assert!(matches!(
        prepare(&review, &selected),
        Err(PanelPreparationError::View(ViewError::Stale))
    ));
}
#[test]
fn document_repair_requires_explicit_choice_then_retains_full_reference() {
    let mut review = review(document_source(), None);
    let scope = review.scope().clone();
    let selected = CorrectionSelection::repair(&review, &scope).unwrap();
    let props = prepare(&review, &selected).unwrap().props;
    assert_eq!(props["sourceChoice"], Value::String(String::new()));
    assert_eq!(
        props["sourceText"],
        Value::String("원문을 선택하세요".into())
    );
    assert_eq!(props["startDisabled"], Value::Bool(true));
    review.choose_document_text(&scope, "2".into()).unwrap();
    let props = prepare(&review, &selected).unwrap().props;
    assert_eq!(props["sourceText"], Value::String("原文 2".into()));
    assert_eq!(props["startDisabled"], Value::Bool(false));
    review.begin_document_repair(&scope).unwrap();
    let selected = CorrectionSelection::at(&review, &scope, 0).unwrap();
    let props = prepare(&review, &selected).unwrap().props;
    let expected = review
        .stored()
        .document
        .source
        .as_document()
        .unwrap()
        .text_reference("/1")
        .unwrap();
    assert_eq!(
        props["sourceRefs"],
        vec![serde_json::to_string(&expected).unwrap()].into_value()
    );
    assert_eq!(props["generatedRaw"], Value::String("preserved raw".into()));
    assert_eq!(
        props["importError"],
        Value::String("preserved error".into())
    );
}
#[test]
fn async_transport_locks_followup_inspect_without_erasing_quality_or_reference() {
    let review = review(
        cue_source(),
        Some(ProposalPayload::Correction {
            edits: vec![Correction {
                cue_id: "b".into(),
                text: "수정".into(),
            }],
        }),
    );
    let selected = CorrectionSelection::at(&review, review.scope(), 0).unwrap();
    let props = prepare(&review, &selected)
        .unwrap()
        .with_transport_pending(true)
        .props;
    for key in [
        "inputDisabled",
        "saveDisabled",
        "acceptDisabled",
        "refreshDisabled",
        "selectionDisabled",
        "startDisabled",
        "busy",
    ] {
        assert_eq!(props[key], Value::Bool(true));
    }
    assert_eq!(props["qualityLabel"], Value::String("품질 미판정".into()));
    assert_eq!(props["sourceText"], Value::String("원문 b".into()));
}
