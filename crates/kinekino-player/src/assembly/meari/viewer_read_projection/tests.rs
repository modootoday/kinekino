use super::*;
#[test]
fn projection_failure_preserves_candidate_for_reprojection_without_sdk_work() {
    let mut owner = ReadProjection {
        dirty: true,
        ..Default::default()
    };
    owner.projection_failed("preflight".into());
    assert!(owner.dirty);
    assert_eq!(owner.panel.projection_error.as_deref(), Some("preflight"));
    owner.committed();
    assert!(!owner.dirty);
}
