mod decision;
mod package;
pub use crate::domain::identifiers::{valid_id, valid_sha256};
pub use decision::*;
pub use package::*;
fn text(value: &str, maximum: usize) -> bool {
    !value.trim().is_empty()
        && value.len() <= maximum
        && !value
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
}
