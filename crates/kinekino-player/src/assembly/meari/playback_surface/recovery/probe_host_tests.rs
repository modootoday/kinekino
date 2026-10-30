use super::{KeyError, ReleaseObservation, ReleaseState, probe_host::RetainedProbeHost};
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
#[derive(Debug)]
struct Resource(Rc<Cell<usize>>);
impl Drop for Resource {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
type Host = RetainedProbeHost<Resource, Resource, (), ()>;

#[test]
fn released_entry_cannot_reenter_owner_initialization_before_retire() {
    let drops = Rc::new(Cell::new(0));
    let mut host = Host::new(1).unwrap();
    let key = host.reserve(Resource(Rc::clone(&drops))).unwrap();
    host.construct(&key, |_| Ok(Resource(Rc::clone(&drops))))
        .unwrap();
    host.release(&key, |_| Ok(ReleaseObservation::Complete))
        .unwrap();
    assert_eq!(
        host.with_owned(&key, |_, _| panic!("released owner callback")),
        Err::<(), _>(KeyError::WrongPhase)
    );
    assert_eq!(host.state(&key), Ok(ReleaseState::Released));
    assert_eq!(drops.get(), 0);
    host.retire(&key).unwrap();
    assert_eq!(drops.get(), 2);
}

#[test]
fn failed_and_unobserved_entries_allow_cleanup_retry_but_no_initialization_callback() {
    let drops = Rc::new(Cell::new(0));
    let mut host = Host::new(1).unwrap();
    let key = host.reserve(Resource(Rc::clone(&drops))).unwrap();
    host.construct(&key, |_| Ok(Resource(Rc::clone(&drops))))
        .unwrap();
    host.release(&key, |_| Err(())).unwrap();
    assert_eq!(
        host.with_owned(&key, |_, _| panic!("failed owner callback")),
        Err::<(), _>(KeyError::WrongPhase)
    );
    host.release(&key, |_| Ok(ReleaseObservation::Unobserved))
        .unwrap();
    assert_eq!(
        host.with_owned(&key, |_, _| panic!("unobserved owner callback")),
        Err::<(), _>(KeyError::WrongPhase)
    );
    assert_eq!(drops.get(), 0);
    host.release(&key, |_| Ok(ReleaseObservation::Complete))
        .unwrap();
    host.retire(&key).unwrap();
    assert_eq!(drops.get(), 2);
}
#[test]
fn panic_after_owner_storage_retains_both_resources_and_blocks_parent_close() {
    let drops = Rc::new(Cell::new(0));
    let mut host = Host::new(1).unwrap();
    let key = host.reserve(Resource(Rc::clone(&drops))).unwrap();
    host.construct(&key, |_| Ok(Resource(Rc::clone(&drops))))
        .unwrap();
    let panic = catch_unwind(AssertUnwindSafe(|| {
        host.with_owned(&key, |_, _| panic!("fake OS construction interruption"))
            .unwrap();
    }));
    assert!(panic.is_err());
    assert_eq!(drops.get(), 0);
    assert_eq!(host.state(&key), Ok(ReleaseState::Owned));
    assert!(host.try_shutdown().is_err());
    assert_eq!(host.state(&key), Ok(ReleaseState::Owned));
    assert_eq!(drops.get(), 0);
    // Fake cleanup receipt only; this is not Windows callback/Destroy evidence.
    host.release(&key, |_| Ok(ReleaseObservation::Complete))
        .unwrap();
    host.retire(&key).unwrap();
    assert_eq!(drops.get(), 2);
    assert!(host.try_shutdown().is_ok());
}
#[test]
fn failed_or_unobserved_release_keeps_host_and_original_entry() {
    let drops = Rc::new(Cell::new(0));
    let mut host = Host::new(1).unwrap();
    let key = host.reserve(Resource(Rc::clone(&drops))).unwrap();
    host.construct(&key, |_| Ok(Resource(Rc::clone(&drops))))
        .unwrap();
    host.release(&key, |_| Err(())).unwrap();
    assert!(host.try_shutdown().is_err());
    host.release(&key, |_| Ok(ReleaseObservation::Unobserved))
        .unwrap();
    assert!(host.try_shutdown().is_err());
    assert_eq!(host.state(&key), Ok(ReleaseState::Unobserved));
    assert_eq!(drops.get(), 0);
}
