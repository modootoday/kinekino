use super::*;
#[test]
fn duplicate_reorder_replace_and_undo_keep_source_and_advance_revision() {
    let initial = document();
    let sources = initial.sources.clone();
    let original_segment = initial.segments[0].clone();
    let mut editor = ClipEditor::open(initial).unwrap();
    editor
        .apply(
            DecimalU64(1),
            ClipEdit::Duplicate {
                segment_id: "segment-1".into(),
                new_segment_id: "segment-2".into(),
                position: 1,
            },
        )
        .unwrap();
    editor
        .apply(
            DecimalU64(2),
            ClipEdit::Reorder {
                segment_ids: vec!["segment-2".into(), "segment-1".into()],
            },
        )
        .unwrap();
    let mut changed = editor.document().segments[0].clone();
    changed.audio = AudioSelection::Disabled;
    changed.subtitles = None;
    editor
        .apply(
            DecimalU64(3),
            ClipEdit::Replace {
                segment_id: "segment-2".into(),
                segment: changed,
            },
        )
        .unwrap();
    editor.undo(DecimalU64(4)).unwrap();
    assert_eq!(editor.document().revision.0, 5);
    assert_eq!(editor.document().segments[0].audio, original_segment.audio);
    assert_eq!(editor.document().sources, sources);
    editor.undo(DecimalU64(5)).unwrap();
    editor.undo(DecimalU64(6)).unwrap();
    assert_eq!(editor.document().segments, vec![original_segment]);
    assert_eq!(editor.document().revision.0, 7);
}
#[test]
fn stale_or_invalid_edits_leave_document_and_history_unchanged() {
    let initial = document();
    let mut editor = ClipEditor::open(initial.clone()).unwrap();
    assert_eq!(
        editor.apply(
            DecimalU64(2),
            ClipEdit::SetCutMode {
                cut_mode: CutMode::FastKeyframe
            }
        ),
        Err(ClipEditError::RevisionConflict)
    );
    assert!(
        editor
            .apply(
                DecimalU64(1),
                ClipEdit::Delete {
                    segment_id: "segment-1".into()
                }
            )
            .is_err()
    );
    assert!(
        editor
            .apply(
                DecimalU64(1),
                ClipEdit::Duplicate {
                    segment_id: "segment-1".into(),
                    new_segment_id: "segment-1".into(),
                    position: 0
                }
            )
            .is_err()
    );
    let mut invalid = initial.segments[0].clone();
    invalid.range.end_us = DecimalU64(0);
    assert!(
        editor
            .apply(
                DecimalU64(1),
                ClipEdit::Replace {
                    segment_id: "segment-1".into(),
                    segment: invalid
                }
            )
            .is_err()
    );
    assert_eq!(editor.document(), &initial);
    assert_eq!(editor.undo(DecimalU64(1)), Err(ClipEditError::NoUndo));
}
#[test]
fn undo_history_is_bounded_and_reopening_does_not_invent_history() {
    let mut editor = ClipEditor::open(document()).unwrap();
    for index in 0..MAX_UNDO + 2 {
        let mut preset = editor.document().preset.clone();
        preset.width = 1080 + index as u32;
        editor
            .apply(editor.document().revision, ClipEdit::SetPreset { preset })
            .unwrap();
    }
    for _ in 0..MAX_UNDO {
        editor.undo(editor.document().revision).unwrap();
    }
    assert_eq!(
        editor.undo(editor.document().revision),
        Err(ClipEditError::NoUndo)
    );
    let mut reopened = ClipEditor::open(editor.document().clone()).unwrap();
    assert_eq!(
        reopened.undo(reopened.document().revision),
        Err(ClipEditError::NoUndo)
    );
}
#[test]
fn exhausted_revision_never_changes_working_state() {
    let mut initial = document();
    initial.revision = DecimalU64(u64::MAX);
    let mut editor = ClipEditor::open(initial.clone()).unwrap();
    assert_eq!(
        editor.apply(
            initial.revision,
            ClipEdit::SetCutMode {
                cut_mode: CutMode::FastKeyframe
            }
        ),
        Err(ClipEditError::RevisionExhausted)
    );
    assert_eq!(editor.document(), &initial);
}
