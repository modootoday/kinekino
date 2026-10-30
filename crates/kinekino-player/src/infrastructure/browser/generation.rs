//! Zero permanently ends a browser's identity space; exhausted generations never wrap.
use std::cell::Cell;
pub(super) fn advance(generation: &Cell<u64>) -> Option<u64> {
    let current = generation.get();
    if current == 0 {
        return None;
    }
    let Some(next) = current.checked_add(1) else {
        generation.set(0);
        return None;
    };
    generation.set(next);
    Some(next)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhausted_generation_is_permanently_invalid_and_never_reuses_an_old_identity() {
        let value = Cell::new(u64::MAX);
        assert_eq!(advance(&value), None);
        assert_eq!(value.get(), 0);
        assert_eq!(advance(&value), None);
        assert_eq!(value.get(), 0);
    }
}
