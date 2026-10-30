use super::*;
use crate::domain::api_values::DecimalU64;

fn association() -> AssociationRevision {
    AssociationRevision {
        association_id: "binding-1".into(),
        case: ReviewCase {
            package_sha256: "a".repeat(64),
            item_id: "sample_1".into(),
            source_request_sha256: "b".repeat(64),
        },
        media: MediaReference {
            asset_id: "original-media".into(),
            sha256: "c".repeat(64),
        },
        revision: DecimalU64(1),
        timeline_status: TimelineStatus::Unverified,
        evidence_refs: Vec::new(),
        note: String::new(),
    }
}
fn draft() -> RangeDraft {
    RangeDraft {
        draft_id: "draft-1".into(),
        association_id: "binding-1".into(),
        association_revision: DecimalU64(1),
        draft_revision: DecimalU64(1),
        authored_by: DraftAuthor::User,
        ranges: vec![],
        note: String::new(),
    }
}
#[test]
fn independent_reassociation_preserves_the_old_drafts_exact_binding() {
    let original = association();
    let old = draft();
    let mut next = original.clone();
    next.revision = DecimalU64(2);
    next.media.sha256 = "d".repeat(64);
    assert_eq!(next.validate_successor(&original), Ok(()));
    assert_eq!(old.validate(&original, None), Ok(()));
    assert_eq!(
        old.validate(&next, None),
        Err(ValidationError::BindingChanged)
    );
    let mut changed_case = next.clone();
    changed_case.case.item_id = "different_item".into();
    assert_eq!(
        changed_case.validate_successor(&original),
        Err(ValidationError::BindingChanged)
    );
    let mut relinked = old.clone();
    relinked.draft_revision = DecimalU64(2);
    relinked.association_revision = DecimalU64(2);
    assert!(relinked.validate_successor(&old, &next, None).is_err());
}
#[test]
fn user_ranges_preserve_empty_order_and_overlap_without_converting_unknown_duration_to_success() {
    let binding = association();
    let mut value = draft();
    assert_eq!(value.validate(&binding, None), Ok(()));
    value.ranges = vec![
        UserRange {
            start_us: DecimalU64(9),
            end_us: DecimalU64(20),
        },
        UserRange {
            start_us: DecimalU64(1),
            end_us: DecimalU64(12),
        },
    ];
    let original = value.ranges.clone();
    assert_eq!(value.validate(&binding, None), Ok(()));
    assert_eq!(value.ranges, original);
    assert!(!binding.playback_ready());
    assert_eq!(
        value.validate(&binding, Some(19)),
        Err(ValidationError::RangeExceedsVerifiedDuration)
    );
    assert_eq!(value.validate(&binding, Some(20)), Ok(()));
    assert!(!binding.playback_ready());
    value.ranges[0].start_us = DecimalU64(20);
    assert_eq!(
        value.validate(&binding, None),
        Err(ValidationError::InvalidRange)
    );
}
#[test]
fn revisions_counts_utf8_byte_limits_and_ids_fail_closed_at_boundaries() {
    assert_eq!(next_revision(None), Ok(DecimalU64(1)));
    assert_eq!(
        next_revision(Some(DecimalU64(0))),
        Err(ValidationError::InvalidRevision)
    );
    assert_eq!(
        next_revision(Some(DecimalU64(u64::MAX))),
        Err(ValidationError::InvalidRevision)
    );
    for bad in ["", "../asset", "a.b", "https://media", "한글"] {
        assert!(!valid_review_object_id(bad));
    }
    assert!(valid_review_object_id(&"x".repeat(128)));
    assert!(!valid_review_object_id(&"x".repeat(129)));
    let binding = association();
    let mut value = draft();
    value.note = "가".repeat(2730) + "ab";
    assert_eq!(value.note.len(), 8192);
    assert_eq!(value.validate(&binding, None), Ok(()));
    value.note.push('x');
    assert_eq!(
        value.validate(&binding, None),
        Err(ValidationError::NoteTooLarge)
    );
    value.note.clear();
    value.ranges = vec![
        UserRange {
            start_us: DecimalU64(0),
            end_us: DecimalU64(u64::MAX)
        };
        64
    ];
    assert_eq!(value.validate(&binding, None), Ok(()));
    value.ranges.push(value.ranges[0].clone());
    assert_eq!(
        value.validate(&binding, None),
        Err(ValidationError::TooManyRanges)
    );
}
#[test]
fn wire_preserves_u64_and_rejects_forged_verification_authorship_unknown_fields_and_numbers() {
    let mut value = draft();
    value.draft_revision = DecimalU64(u64::MAX);
    let wire = serde_json::to_value(&value).unwrap();
    assert_eq!(wire["draft_revision"], u64::MAX.to_string());
    assert_eq!(
        serde_json::from_value::<RangeDraft>(wire.clone()).unwrap(),
        value
    );
    for mutation in [
        serde_json::json!(1),
        serde_json::json!(-1),
        serde_json::json!(1.5),
        serde_json::json!("18446744073709551616"),
    ] {
        let mut invalid = wire.clone();
        invalid["draft_revision"] = mutation;
        assert!(serde_json::from_value::<RangeDraft>(invalid).is_err());
    }
    let mut invalid = wire.clone();
    invalid["authored_by"] = serde_json::json!("model");
    assert!(serde_json::from_value::<RangeDraft>(invalid).is_err());
    let mut invalid = wire;
    invalid["media_path"] = serde_json::json!("C:/private/video");
    assert!(serde_json::from_value::<RangeDraft>(invalid).is_err());
    let mut binding_wire = serde_json::to_value(association()).unwrap();
    binding_wire["timeline_status"] = serde_json::json!({"verified":{"timebase":"1/1000"}});
    assert!(serde_json::from_value::<AssociationRevision>(binding_wire).is_err());
    let mut binding = association();
    binding.case.package_sha256 = "A".repeat(64);
    assert_eq!(binding.validate(), Err(ValidationError::InvalidIdentity));
    binding = association();
    binding.evidence_refs.push(EvidenceReference {
        asset_id: "claimed-proof".into(),
        sha256: "d".repeat(64),
    });
    assert_eq!(
        binding.validate(),
        Err(ValidationError::UnsupportedTimelineEvidence)
    );
}
#[test]
fn draft_updates_need_the_next_revision_and_validate_new_contents() {
    let binding = association();
    let previous = draft();
    let mut next = previous.clone();
    assert_eq!(
        next.validate_successor(&previous, &binding, None),
        Err(ValidationError::RevisionConflict)
    );
    next.draft_revision = DecimalU64(2);
    next.note = "user correction".into();
    assert_eq!(next.validate_successor(&previous, &binding, None), Ok(()));
    next.ranges.push(UserRange {
        start_us: DecimalU64(3),
        end_us: DecimalU64(2),
    });
    assert_eq!(
        next.validate_successor(&previous, &binding, None),
        Err(ValidationError::InvalidRange)
    );
}
