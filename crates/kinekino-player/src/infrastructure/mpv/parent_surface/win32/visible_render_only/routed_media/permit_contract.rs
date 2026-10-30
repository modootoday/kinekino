//! Positive source-admission baseline for separately allocated compiler rejection fixtures.
use super::CheckedMediaParent;

#[test]
fn checked_parent_contract_type_is_in_scope() {
    let _ = std::mem::size_of::<CheckedMediaParent<'_>>();
}
