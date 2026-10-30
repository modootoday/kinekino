use super::*;
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::api_values::DecimalU64,
};

#[test]
fn long_review_keeps_save_and_accept_inside_the_fixed_footer() {
    let ctx = egui::Context::default();
    let client: ApplicationClientHandle =
        std::sync::Arc::new(crate::application::api::PlayerSdk::default());
    let mut state = State::new(ProposalTask::Cards);
    state.apply(worker::Outcome::Document(Ok(Box::new(document(
        ProposalTask::Cards,
    )))));
    state.draft = Some(ProposalPayload::Cards {
        cards: (0..24)
            .map(|index| Card {
                front: format!("편집 문구 {index} · 原文比較"),
                back: "긴 내용을 편집해도 저장 동작은 접근 가능해야 합니다.".repeat(4),
                source_refs: vec!["cue-1".into()],
            })
            .collect(),
    });
    state.dirty = true;
    let viewport = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(300., 600.));
    let mut action_positions = Vec::new();
    for _ in 0..12 {
        let output = crate::presentation::test_support::run(
            &ctx,
            egui::RawInput {
                screen_rect: Some(viewport),
                ..Default::default()
            },
            |ui| view::show(ui, &mut state, true, &client),
        );
        action_positions.clear();
        for shape in output.shapes {
            if let egui::Shape::Text(text) = shape.shape
                && matches!(text.galley.text(), "수정본 저장" | "명시적 채택")
            {
                action_positions.push(text.pos);
            }
        }
    }
    assert_eq!(action_positions.len(), 2);
    for position in action_positions {
        assert!(viewport.contains(position));
        assert!(
            position.y
                >= viewport.bottom() - crate::presentation::design_tokens::size::INSPECTOR_FOOTER
        );
    }
    assert!(state.dirty);
    assert!(state.pending.is_none());
    assert!(
        matches!(&state.draft, Some(ProposalPayload::Cards { cards }) if cards.len() == 24 && cards[0].source_refs == ["cue-1"])
    );
}

fn document(task: ProposalTask) -> ProposalDocument {
    ProposalDocument {
        schema_version: "1".into(),
        proposal_id: "proposal-test".into(),
        source: SourceSnapshot::Cue(CueSourceSnapshot {
            source_id: "source-test".into(),
            source_revision: DecimalU64(1),
            source_sha256: "a".repeat(64),
            cues: vec![SourceCue {
                cue_id: "cue-1".into(),
                text: "原文 · 원문".into(),
                start_us: None,
                end_us: None,
            }],
        }),
        generated: GeneratedResult {
            task,
            raw_text: "malformed original bytes".into(),
        },
        draft_revision: DecimalU64(1),
        state: DraftState::NeedsRepair,
        draft: None,
        import_error: Some("invalid import".into()),
    }
}

#[test]
fn manual_repair_reset_keeps_a_structured_editor_and_immutable_source() {
    let mut state = State::new(ProposalTask::Cards);
    state.apply(worker::Outcome::Document(Ok(Box::new(document(
        ProposalTask::Cards,
    )))));
    let original = state.document.as_ref().unwrap().generated.raw_text.clone();
    state.draft = Some(ProposalPayload::Cards {
        cards: vec![Card {
            front: "修正".into(),
            back: "수정".into(),
            source_refs: vec!["cue-1".into()],
        }],
    });
    state.dirty = true;
    state.reset_buffer();
    assert!(!state.dirty);
    assert!(matches!(state.draft,Some(ProposalPayload::Cards{ref cards}) if cards.is_empty()));
    assert_eq!(
        state.document.as_ref().unwrap().generated.raw_text,
        original
    );
    assert_eq!(
        state.document.as_ref().unwrap().source.cues()[0].text,
        "原文 · 원문"
    );
}

#[test]
fn revision_conflict_and_wrong_task_do_not_erase_unsaved_input() {
    let mut state = State::new(ProposalTask::Highlights);
    state.apply(worker::Outcome::Document(Ok(Box::new(document(
        ProposalTask::Highlights,
    )))));
    state.draft = Some(ProposalPayload::Highlights {
        highlights: vec![Highlight {
            title: "編集 · 편집".into(),
            source_refs: vec!["cue-1".into()],
        }],
    });
    state.dirty = true;
    state.apply(worker::Outcome::Document(Err(ApiError::new(
        ErrorCode::RevisionConflict,
        "changed",
    ))));
    assert!(state.dirty);
    state.apply(worker::Outcome::Document(Ok(Box::new(document(
        ProposalTask::Cards,
    )))));
    assert!(state.dirty);
    assert!(
        matches!(state.draft,Some(ProposalPayload::Highlights{ref highlights}) if highlights[0].title=="編集 · 편집")
    );
    assert_eq!(
        state.document.as_ref().unwrap().generated.task,
        ProposalTask::Highlights
    );
}

#[test]
fn inspect_restores_explicit_acceptance_without_guessing_from_revision() {
    let mut doc = document(ProposalTask::Cards);
    doc.draft_revision = DecimalU64(3);
    let output = ProposalPayload::Cards {
        cards: vec![Card {
            front: "질문".into(),
            back: "답변".into(),
            source_refs: vec!["cue-1".into()],
        }],
    };
    doc.draft = Some(output.clone());
    doc.state = DraftState::Editable;
    let receipt = AcceptanceReceipt {
        proposal_id: doc.proposal_id.clone(),
        draft_revision: DecimalU64(2),
        source_id: doc.source.source_id().to_owned(),
        source_revision: doc.source.source_revision(),
        source_sha256: doc.source.source_sha256().to_owned(),
        output_revision: DecimalU64(3),
        output,
        automatic_application_enabled: false,
    };
    let mut state = State::new(ProposalTask::Cards);
    state.apply(worker::Outcome::Inspected(Ok(Box::new(
        crate::application::proposals::StoredProposal {
            document: doc.clone(),
            acceptance: Some(receipt),
            historical: None,
        },
    ))));
    assert_eq!(state.receipt.as_ref().unwrap().output_revision.0, 3);
    state.apply(worker::Outcome::Inspected(Ok(Box::new(
        crate::application::proposals::StoredProposal {
            document: doc,
            acceptance: None,
            historical: None,
        },
    ))));
    assert!(state.receipt.is_none());
    assert_eq!(state.document.as_ref().unwrap().draft_revision.0, 3);
}

#[test]
fn document_repair_uses_typed_document_references_without_inventing_cues() {
    use sha2::{Digest, Sha256};
    let content = "Edit this ASR draft. Correct punctuation only. Do not guess unclear words. JSON only: {\"proposed\":string,\"uncertain_words\":[string]}. Under100words. /no_think\n[\"원문입니다。\",\"第二文です。\"]";
    let asset = serde_json::json!({"messages":[{"role":"user","content":content}]}).to_string();
    let hash = format!("{:x}", Sha256::digest(asset.as_bytes()));
    let source =
        DocumentSourceSnapshot::from_asset("document-source".into(), DecimalU64(1), hash, asset)
            .unwrap();
    let reference = source.text_reference("/1").unwrap();
    let mut doc = document(ProposalTask::Correction);
    doc.source = SourceSnapshot::Document(source);
    doc.schema_version = "2".into();
    let mut state = State::new(ProposalTask::Correction);
    state.apply(worker::Outcome::Document(Ok(Box::new(doc))));
    assert!(
        matches!(state.draft, Some(ProposalPayload::DocumentCorrection { ref edits }) if edits.is_empty())
    );
    assert!(state.document.as_ref().unwrap().source.cues().is_empty());
    assert_eq!(reference.inner_pointer, "/1");
    assert_eq!(
        state
            .document
            .as_ref()
            .unwrap()
            .source
            .as_document()
            .unwrap()
            .texts[1]
            .text,
        "第二文です。"
    );
    state.dirty = true;
    state.reset_buffer();
    assert!(
        matches!(state.draft, Some(ProposalPayload::DocumentCorrection { ref edits }) if edits.is_empty())
    );
}
