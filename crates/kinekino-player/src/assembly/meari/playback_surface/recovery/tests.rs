use super::*;
use std::{cell::Cell, rc::Rc};
#[derive(Debug)]
struct Owner {
    drops: Rc<Cell<usize>>,
    calls: usize,
}
impl Drop for Owner {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}
type Registry = RecoveryRegistry<Owner, Owner, &'static str, &'static str>;
fn owner(drops: &Rc<Cell<usize>>) -> Owner {
    Owner {
        drops: Rc::clone(drops),
        calls: 0,
    }
}
fn registry(capacity: usize) -> Registry {
    Registry::new(capacity).unwrap()
}

#[test]
fn capacity_exhaustion_and_reservation_cancel_return_original_input() {
    let drops = Rc::new(Cell::new(0));
    let mut registry = registry(1);
    let key = registry.reserve(owner(&drops)).unwrap();
    let failed = registry.reserve(owner(&drops)).unwrap_err();
    assert_eq!(failed.error, ReservationError::Capacity);
    assert_eq!(drops.get(), 0);
    let returned = registry.cancel_reservation(&key).unwrap();
    assert_eq!(registry.state(&key), Err(KeyError::Stale));
    assert_eq!(drops.get(), 0);
    drop((returned, failed.input));
    assert_eq!(drops.get(), 2);
}

#[test]
fn exhausted_generation_does_not_consume_or_drop_input() {
    let drops = Rc::new(Cell::new(0));
    let mut registry = registry(1);
    registry.exhaust_generation();
    let failure = registry.reserve(owner(&drops)).unwrap_err();
    assert_eq!(failure.error, ReservationError::CounterExhausted);
    assert_eq!(drops.get(), 0);
    drop(failure.input);
    assert_eq!(drops.get(), 1);
}

#[test]
fn foreign_and_stale_keys_never_call_factory_or_release() {
    let drops = Rc::new(Cell::new(0));
    let mut first = registry(1);
    let mut second = registry(1);
    let key = first.reserve(owner(&drops)).unwrap();
    assert_eq!(
        second.construct(&key, |_| panic!("foreign factory")),
        Err(KeyError::Foreign)
    );
    assert_eq!(
        second.release(&key, |_| panic!("foreign release")),
        Err(KeyError::Foreign)
    );
    let returned = first.cancel_reservation(&key).unwrap();
    let fresh = first.reserve(returned).unwrap();
    assert_eq!(
        first.construct(&key, |_| panic!("stale factory")),
        Err(KeyError::Stale)
    );
    drop(first.cancel_reservation(&fresh).unwrap());
}

#[test]
fn release_failure_and_unobserved_retain_same_owner_until_explicit_complete() {
    let drops = Rc::new(Cell::new(0));
    let mut registry = registry(1);
    let key = registry.reserve(owner(&drops)).unwrap();
    registry.construct(&key, |_| Ok(owner(&drops))).unwrap();
    assert_eq!(
        registry.release(&key, |owner| {
            owner.calls += 1;
            Err("destroy failed")
        }),
        Ok(ReleaseState::Failed)
    );
    assert_eq!(registry.release_error(&key), Ok(Some(&"destroy failed")));
    assert_eq!(registry.retire(&key), Err(KeyError::WrongPhase));
    assert_eq!(
        registry.release(&key, |owner| {
            assert_eq!(owner.calls, 1);
            owner.calls += 1;
            Ok(ReleaseObservation::Unobserved)
        }),
        Ok(ReleaseState::Unobserved)
    );
    assert_eq!(drops.get(), 0);
    assert_eq!(
        registry.release(&key, |owner| {
            assert_eq!(owner.calls, 2);
            Ok(ReleaseObservation::Complete)
        }),
        Ok(ReleaseState::Released)
    );
    assert_eq!(
        registry.release(&key, |_| panic!("duplicate release")),
        Ok(ReleaseState::Released)
    );
    registry.retire(&key).unwrap();
    assert_eq!(drops.get(), 2);
    assert!(registry.shutdown().is_ok());
}

#[test]
fn partial_creation_failure_keeps_input_and_owner_through_incomplete_shutdown() {
    let drops = Rc::new(Cell::new(0));
    let mut registry = registry(1);
    let key = registry.reserve(owner(&drops)).unwrap();
    registry
        .construct(&key, |_| {
            Err(ConstructionFailure {
                partial: Some(owner(&drops)),
                error: "create partial",
            })
        })
        .unwrap();
    assert_eq!(
        registry.creation_failure(&key).unwrap().unwrap().error,
        "create partial"
    );
    assert!(matches!(
        registry.recover_creation_failure(&key),
        Err(KeyError::WrongPhase)
    ));
    let mut returned = match registry.shutdown() {
        Err(registry) => registry,
        Ok(()) => panic!("incomplete shutdown"),
    };
    assert_eq!(drops.get(), 0);
    returned
        .release(&key, |_| Ok(ReleaseObservation::Complete))
        .unwrap();
    returned.retire(&key).unwrap();
    assert_eq!(drops.get(), 2);
}

#[test]
fn input_only_creation_failure_is_returned_without_silent_drop() {
    let drops = Rc::new(Cell::new(0));
    let mut registry = registry(1);
    let key = registry.reserve(owner(&drops)).unwrap();
    registry
        .construct(&key, |_| {
            Err(ConstructionFailure {
                partial: None,
                error: "preflight",
            })
        })
        .unwrap();
    let failure = registry.recover_creation_failure(&key).unwrap();
    assert_eq!(failure.error, "preflight");
    assert_eq!(drops.get(), 0);
    assert!(registry.shutdown().is_ok());
    drop(failure.input);
    assert_eq!(drops.get(), 1);
}

#[test]
fn interrupted_reservation_survives_shutdown_until_cancelled() {
    let drops = Rc::new(Cell::new(0));
    let mut registry = registry(1);
    let key = registry.reserve(owner(&drops)).unwrap();
    let mut returned = match registry.shutdown() {
        Err(registry) => registry,
        Ok(()) => panic!("reservation lost"),
    };
    assert_eq!(drops.get(), 0);
    drop(returned.cancel_reservation(&key).unwrap());
    assert_eq!(drops.get(), 1);
}

#[test]
fn factory_panic_keeps_original_input_key_and_incomplete_shutdown() {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let drops = Rc::new(Cell::new(0));
    let mut registry = registry(1);
    let key = registry.reserve(owner(&drops)).unwrap();
    let original = registry.input(&key).unwrap() as *const Owner;
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = registry.construct(&key, |input| {
            assert_eq!(input as *const Owner, original);
            panic!("factory failure before external allocation")
        });
    }));
    assert!(panic.is_err());
    assert_eq!(drops.get(), 0);
    assert_eq!(registry.state(&key), Ok(ReleaseState::Constructing));
    assert_eq!(registry.input(&key).unwrap() as *const Owner, original);
    assert!(matches!(
        registry.cancel_reservation(&key),
        Err(KeyError::WrongPhase)
    ));
    assert_eq!(
        registry.construct(&key, |_| panic!("must not restart")),
        Err(KeyError::WrongPhase)
    );
    assert_eq!(
        registry.release(&key, |_| panic!("no owner")),
        Err(KeyError::WrongPhase)
    );
    assert_eq!(registry.retire(&key), Err(KeyError::WrongPhase));
    let retained = match registry.shutdown() {
        Err(registry) => registry,
        Ok(()) => panic!("false shutdown success"),
    };
    assert_eq!(retained.state(&key), Ok(ReleaseState::Constructing));
    assert_eq!(drops.get(), 0);
    // Dropping the test registry is not a production recovery guarantee.
    drop(retained);
    assert_eq!(drops.get(), 1);
}
