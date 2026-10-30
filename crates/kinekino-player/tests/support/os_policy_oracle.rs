//! The library's OS policy plus the retained-guard oracle, which is `cfg(test)` inside the
//! library and therefore invisible to integration tests that compile its sources by path.
pub(crate) use kinekino_player::infrastructure::os_policy::*;

/// Windows share modes make `mutation` fail; POSIX sharing is advisory, so it applies.
/// Returns whether the mutation was applied.
pub(crate) fn guarded_mutation(mutation: std::io::Result<()>) -> bool {
    let mandatory = has_mandatory_share_modes();
    assert_eq!(mutation.is_err(), mandatory, "{mutation:?}");
    !mandatory
}
