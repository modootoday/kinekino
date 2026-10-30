use super::preserve_callback_error;
use std::cell::Cell;
#[test]
fn style_callback_preserves_zero_and_delegates_once() {
    let error = Cell::new(0);
    let calls = Cell::new(0);
    let seen = Cell::new(None);
    let result = preserve_callback_error(
        0,
        || {
            calls.set(calls.get() + 1);
            error.set(6);
            42
        },
        || error.get(),
        |value| error.set(value),
        |entry, delegated| seen.set(Some((entry, delegated))),
    );
    assert_eq!(calls.get(), 1);
    assert_eq!(result, 42);
    assert_eq!(error.get(), 0);
    assert_eq!(seen.get(), Some((0, 6)));
}
#[test]
fn style_callback_preserves_nonzero_and_delegates_once() {
    let error = Cell::new(6);
    let calls = Cell::new(0);
    let seen = Cell::new(None);
    let result = preserve_callback_error(
        6,
        || {
            calls.set(calls.get() + 1);
            error.set(0);
            -17
        },
        || error.get(),
        |value| error.set(value),
        |entry, delegated| seen.set(Some((entry, delegated))),
    );
    assert_eq!(calls.get(), 1);
    assert_eq!(result, -17);
    assert_eq!(error.get(), 6);
    assert_eq!(seen.get(), Some((6, 0)));
}
