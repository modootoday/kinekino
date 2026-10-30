use super::*;

#[test]
fn parent_alias_close_is_observed_from_the_original_slot_and_drop_grants_no_close_permission() {
    let lifetime = TransitionLifetime::new();
    let alias = lifetime.slot();
    assert!(!lifetime.admission_closed());
    alias.close_admission();
    assert!(lifetime.admission_closed());
    assert!(lifetime.end_result().is_none());

    let mut other = TransitionLifetime::new();
    let other_alias = other.slot();
    drop(lifetime);
    assert!(!other_alias.admission_closed());
    other.close_admission();
    other.finish_once();
    other.finish_once();
    assert!(matches!(other.end_result(), Some(Ok(None))));
    drop(other);

    let unclosed = TransitionLifetime::new();
    let original = unclosed.slot();
    drop(unclosed);
    assert!(!original.admission_closed());
    assert!(original.end().unwrap().is_none());
}
