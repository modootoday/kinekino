use super::{ChildProfile, WS_EX_NOACTIVATE};
#[test]
fn creation_profiles_keep_legacy_zero_and_explicit_visible_nonactivation() {
    assert_eq!(ChildProfile::LegacyHidden.extended_style().0, 0);
    assert_eq!(
        ChildProfile::VisibleRenderOnly.extended_style(),
        WS_EX_NOACTIVATE
    );
}
