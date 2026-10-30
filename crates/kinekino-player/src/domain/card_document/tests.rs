use super::*;
use crate::domain::proposals::{Card, SourceCue};

fn entry(id: &str) -> CardEntry {
    CardEntry {
        card_id: id.into(),
        role: CardRole::Body,
        text_kind: CardTextKind::Summary,
        content: Card {
            front: "제목".into(),
            back: "요약 초안".into(),
            source_refs: vec!["cue-1".into()],
        },
        frame: None,
        alt_text_draft: "검토 전 설명 초안".into(),
    }
}
fn document() -> CardDocument {
    CardDocument {
        schema_version: "1".into(),
        document_id: "cards-project-1".into(),
        revision: DecimalU64(1),
        source: CueSourceSnapshot {
            source_id: "original".into(),
            source_revision: DecimalU64(3),
            source_sha256: "a".repeat(64),
            cues: vec![SourceCue {
                cue_id: "cue-1".into(),
                text: "원문 대사".into(),
                start_us: None,
                end_us: None,
            }],
        },
        layout: CardLayout {
            canvas: CardCanvas {
                width: 1080,
                height: 1080,
                template_id: "card.frame-two-slot".into(),
                template_version: "0.1.0-proposal.1".into(),
                template_sha256: "b".repeat(64),
            },
            cards: vec![entry("first")],
            caption_draft: "게시문 초안".into(),
        },
    }
}
fn bytes(editor: &CardEditor) -> Vec<u8> {
    serde_json::to_vec(editor.document()).unwrap()
}

#[test]
fn edit_sequence_preserves_source_and_undo_restores_content_with_new_revision() {
    let original = document();
    let source = serde_json::to_value(&original.source).unwrap();
    let mut editor = CardEditor::open(original.clone()).unwrap();
    editor
        .apply(
            DecimalU64(1),
            CardEdit::Duplicate {
                card_id: "first".into(),
                new_card_id: "second".into(),
                position: 1,
            },
        )
        .unwrap();
    editor
        .apply(
            DecimalU64(2),
            CardEdit::Add {
                position: 2,
                card: entry("third"),
            },
        )
        .unwrap();
    editor
        .apply(
            DecimalU64(3),
            CardEdit::Reorder {
                card_ids: vec!["third".into(), "second".into(), "first".into()],
            },
        )
        .unwrap();
    let mut changed = entry("second");
    changed.content.back = "수정된 요약".into();
    changed.frame = Some(CardFrameReference {
        artifact_id: "frame-1".into(),
        png_sha256: "c".repeat(64),
        source_sha256: "a".repeat(64),
        declared_source_pts_us: Some(DecimalU64(1000)),
    });
    editor
        .apply(
            DecimalU64(4),
            CardEdit::Replace {
                card_id: "second".into(),
                card: changed,
            },
        )
        .unwrap();
    editor
        .apply(
            DecimalU64(5),
            CardEdit::Delete {
                card_id: "first".into(),
            },
        )
        .unwrap();
    assert_eq!(
        editor
            .document()
            .layout
            .cards
            .iter()
            .map(|c| c.card_id.as_str())
            .collect::<Vec<_>>(),
        ["third", "second"]
    );
    assert_eq!(
        editor.document().layout.cards[1].content.back,
        "수정된 요약"
    );
    for revision in 6..11 {
        editor.undo(DecimalU64(revision)).unwrap();
    }
    assert_eq!(editor.document().revision, DecimalU64(11));
    assert_eq!(
        serde_json::to_value(&editor.document().layout).unwrap(),
        serde_json::to_value(&original.layout).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&editor.document().source).unwrap(),
        source
    );
    assert_eq!(editor.undo(DecimalU64(11)), Err(CardDocumentError::NoUndo));
}

#[test]
fn rejected_edits_do_not_mutate_document_or_consume_undo() {
    let mut editor = CardEditor::open(document()).unwrap();
    editor
        .apply(
            DecimalU64(1),
            CardEdit::SetCaption {
                caption_draft: "다음".into(),
            },
        )
        .unwrap();
    let before = bytes(&editor);
    let edits = [
        CardEdit::Duplicate {
            card_id: "first".into(),
            new_card_id: "first".into(),
            position: 0,
        },
        CardEdit::Reorder {
            card_ids: vec!["missing".into()],
        },
        CardEdit::Reorder { card_ids: vec![] },
        CardEdit::Replace {
            card_id: "first".into(),
            card: entry("different"),
        },
        CardEdit::Add {
            position: usize::MAX,
            card: entry("new"),
        },
        CardEdit::SetCaption {
            caption_draft: "bad\0caption".into(),
        },
    ];
    for edit in edits {
        assert!(editor.apply(DecimalU64(2), edit).is_err());
        assert_eq!(bytes(&editor), before);
    }
    assert_eq!(
        editor.apply(
            DecimalU64(1),
            CardEdit::Delete {
                card_id: "first".into()
            }
        ),
        Err(CardDocumentError::RevisionConflict)
    );
    assert_eq!(
        editor.undo(DecimalU64(1)),
        Err(CardDocumentError::RevisionConflict)
    );
    assert_eq!(bytes(&editor), before);
    editor.undo(DecimalU64(2)).unwrap();
    assert_eq!(editor.document().layout.caption_draft, "게시문 초안");
}

#[test]
fn quotation_and_frame_refs_cannot_claim_foreign_or_changed_source() {
    let mut valid = document();
    valid.layout.cards[0].text_kind = CardTextKind::Quotation;
    assert_eq!(valid.validate(), Err(CardDocumentError::InvalidQuotation));
    valid.layout.cards[0].content.back = "원문 대사".into();
    valid.validate().unwrap();
    valid.layout.cards[0]
        .content
        .source_refs
        .push("cue-1".into());
    assert_eq!(valid.validate(), Err(CardDocumentError::InvalidReference));
    valid.layout.cards[0].content.source_refs = vec!["foreign-cue".into()];
    assert_eq!(valid.validate(), Err(CardDocumentError::InvalidReference));
    valid.layout.cards[0].content.source_refs = vec!["cue-1".into()];
    valid.layout.cards[0].frame = Some(CardFrameReference {
        artifact_id: "frame-1".into(),
        png_sha256: "c".repeat(64),
        source_sha256: "d".repeat(64),
        declared_source_pts_us: None,
    });
    assert_eq!(valid.validate(), Err(CardDocumentError::InvalidReference));
}

#[test]
fn empty_manual_draft_is_valid_without_becoming_render_or_quality_approval() {
    let mut draft = document();
    draft.layout.cards[0].text_kind = CardTextKind::Manual;
    draft.layout.cards[0].content = Card {
        front: String::new(),
        back: String::new(),
        source_refs: vec![],
    };
    draft.validate().unwrap();
    let mut editor = CardEditor::open(draft).unwrap();
    editor
        .apply(
            DecimalU64(1),
            CardEdit::Delete {
                card_id: "first".into(),
            },
        )
        .unwrap();
    assert!(editor.document().layout.cards.is_empty());
    editor.undo(DecimalU64(2)).unwrap();
    assert_eq!(editor.document().layout.cards.len(), 1);
    let serialized = serde_json::to_value(editor.document()).unwrap();
    assert!(serialized.get("quality_approved").is_none());
    assert!(serialized.get("accepted_output").is_none());
}

#[test]
fn undo_is_bounded_and_revision_never_wraps() {
    let mut editor = CardEditor::open(document()).unwrap();
    for revision in 1..=40 {
        editor
            .apply(
                DecimalU64(revision),
                CardEdit::SetCaption {
                    caption_draft: revision.to_string(),
                },
            )
            .unwrap();
    }
    for revision in 41..73 {
        editor.undo(DecimalU64(revision)).unwrap();
    }
    assert_eq!(editor.document().layout.caption_draft, "8");
    assert_eq!(editor.undo(DecimalU64(73)), Err(CardDocumentError::NoUndo));
    let mut max = document();
    max.revision = DecimalU64(u64::MAX);
    let mut editor = CardEditor::open(max).unwrap();
    let before = bytes(&editor);
    assert_eq!(
        editor.apply(
            DecimalU64(u64::MAX),
            CardEdit::Delete {
                card_id: "first".into()
            }
        ),
        Err(CardDocumentError::RevisionExhausted)
    );
    assert_eq!(bytes(&editor), before);
}

#[test]
fn limits_and_strict_saved_document_are_checked_on_reopen() {
    let mut draft = document();
    draft.layout.canvas.width = 4097;
    assert_eq!(draft.validate(), Err(CardDocumentError::InvalidDocument));
    draft = document();
    draft.layout.cards = (0..MAX_CARDS)
        .map(|i| entry(&format!("card-{i}")))
        .collect();
    let mut editor = CardEditor::open(draft).unwrap();
    assert_eq!(
        editor.apply(
            DecimalU64(1),
            CardEdit::Duplicate {
                card_id: "card-0".into(),
                new_card_id: "extra".into(),
                position: 0
            }
        ),
        Err(CardDocumentError::LimitExceeded)
    );
    let mut value = serde_json::to_value(editor.document()).unwrap();
    value["source"]["cues"][0]["cue_id"] = serde_json::json!("foreign");
    let corrupt: CardDocument = serde_json::from_value(value).unwrap();
    assert!(CardEditor::open(corrupt).is_err());
    let mut value = serde_json::to_value(document()).unwrap();
    value["quality_approved"] = serde_json::json!(true);
    assert!(serde_json::from_value::<CardDocument>(value).is_err());
    let mut large = document();
    large.layout.cards = (0..40)
        .map(|i| {
            let mut card = entry(&format!("card-{i}"));
            card.content.back = "x".repeat(16_384);
            card
        })
        .collect();
    assert_eq!(large.validate(), Err(CardDocumentError::LimitExceeded));
}

#[test]
fn canvas_change_is_undoable_and_saved_document_reopens() {
    let mut editor = CardEditor::open(document()).unwrap();
    let mut canvas = editor.document().layout.canvas.clone();
    canvas.width = 1920;
    canvas.height = 1080;
    editor
        .apply(DecimalU64(1), CardEdit::SetCanvas { canvas })
        .unwrap();
    let reopened: CardDocument = serde_json::from_slice(&bytes(&editor)).unwrap();
    CardEditor::open(reopened).unwrap();
    assert_eq!(editor.document().layout.canvas.width, 1920);
    editor.undo(DecimalU64(2)).unwrap();
    assert_eq!(editor.document().layout.canvas.width, 1080);
}
