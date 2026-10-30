use super::*;
use std::cell::Cell;

fn absent_marker() -> PathBuf {
    std::env::temp_dir().join(format!(
        "kinekino-source-owner-no-file-{}-close.request",
        std::process::id()
    ))
}

#[test]
fn close_scope_rejects_nonfresh_or_unbounded_identity_without_an_attachment() {
    let deadline = Instant::now() + Duration::from_secs(1);
    assert!(Scope::enter(PathBuf::from("relative.request"), "run-1", deadline).is_err());
    let existing = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join("Cargo.toml");
    assert!(Scope::enter(existing, "run-1", deadline).is_err());
    for nonce in ["", "run\n1", "run/1", &"x".repeat(129)] {
        assert!(Scope::enter(absent_marker(), nonce, deadline).is_err());
    }
    let mut scope = Scope::enter(absent_marker(), "run-1", deadline).unwrap();
    let evidence = scope.finish();
    assert!(!evidence.requested);
    assert!(evidence.delivered.is_none());
    assert!(evidence.watcher.is_err());
}

#[test]
fn missing_request_and_drop_cannot_invoke_close_and_nested_scope_restores_original() {
    let deadline = Instant::now() + Duration::from_secs(1);
    let outer = Scope::enter(absent_marker(), "outer", deadline).unwrap();
    let invoked = Rc::new(Cell::new(false));
    let observe = invoked.clone();
    outer.state.borrow_mut().close = Some(Box::new(move || {
        observe.set(true);
        Ok(true)
    }));
    after_dispatch();
    assert!(!invoked.get());
    let inner = Scope::enter(absent_marker(), "inner", deadline).unwrap();
    assert!(ACTIVE.with(|active| Rc::ptr_eq(active.borrow().as_ref().unwrap(), &inner.state)));
    drop(inner);
    assert!(ACTIVE.with(|active| Rc::ptr_eq(active.borrow().as_ref().unwrap(), &outer.state)));
    drop(outer);
    assert!(!invoked.get());
}
