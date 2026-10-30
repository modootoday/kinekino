use super::window_events::Events;
use std::sync::atomic::Ordering;

#[test]
fn original_generation_requires_install_and_destroy_observation() {
    let events = Events::new(7);
    assert!(!events.complete(7));
    events.destroyed.store(true, Ordering::Release);
    assert!(!events.complete(7));
    events.installed.store(true, Ordering::Release);
    events.detached.store(true, Ordering::Release);
    assert!(events.complete(7));
    assert!(!events.complete(8));
}

#[test]
fn zero_userdata_return_requires_last_error_check() {
    use super::window_events::checked_write;
    assert_eq!(checked_write(|| (0, 0)), Ok(0));
    assert_eq!(checked_write(|| (0, 5)), Err(5));
    assert_eq!(checked_write(|| (42, 5)), Ok(42));
}

#[test]
fn installation_failure_keeps_creator_ownership_and_no_window_identity() {
    use super::window_events::install_events;
    use std::sync::Arc;
    let events = Arc::new(Events::new(7));
    let transferred = Arc::into_raw(Arc::clone(&events));
    assert!(!install_events(&events, 42, || Err(5)));
    assert!(!events.installed.load(Ordering::Acquire));
    assert_eq!(events.hwnd.load(Ordering::Acquire), 0);
    assert_eq!(events.callback_error.load(Ordering::Acquire), 5);
    unsafe {
        drop(Arc::from_raw(transferred));
    }
    assert_eq!(Arc::strong_count(&events), 1);
}

#[test]
fn destroy_failure_never_observes_or_completes_and_return_success_is_not_ack() {
    use super::release::destroy_then_observe;
    assert_eq!(
        destroy_then_observe(|| Err(5), || panic!("observation after failed destroy")),
        Err(5)
    );
    assert_eq!(
        destroy_then_observe(|| Ok::<_, u32>(()), || false),
        Ok(false)
    );
    assert_eq!(destroy_then_observe(|| Ok::<_, u32>(()), || true), Ok(true));
}

#[test]
fn successful_detach_reclaims_one_arc_after_downstream_and_tolerates_reentry() {
    use super::window_events::finish_destroy;
    use std::sync::Arc;
    use windows::Win32::Foundation::LRESULT;
    let events = Arc::new(Events::new(7));
    events.installed.store(true, Ordering::Release);
    let transferred = Arc::into_raw(Arc::clone(&events));
    unsafe {
        finish_destroy(
            transferred,
            || Ok(0),
            || {
                assert!(!events.complete(7));
                finish_destroy(transferred, || panic!("duplicate detach"), || LRESULT(0));
                assert_eq!(Arc::strong_count(&events), 2);
                LRESULT(0)
            },
        );
    }
    assert!(events.complete(7));
    assert_eq!(Arc::strong_count(&events), 1);
}

#[test]
fn failed_detach_keeps_callback_reference_and_refuses_complete() {
    use super::window_events::finish_destroy;
    use std::sync::Arc;
    use windows::Win32::Foundation::LRESULT;
    let events = Arc::new(Events::new(7));
    events.installed.store(true, Ordering::Release);
    let transferred = Arc::into_raw(Arc::clone(&events));
    unsafe {
        finish_destroy(transferred, || Err(5), || LRESULT(0));
    }
    assert!(!events.complete(7));
    assert!(events.destroyed.load(Ordering::Acquire));
    assert_eq!(events.callback_error.load(Ordering::Acquire), 5);
    assert_eq!(Arc::strong_count(&events), 2);
    // Test-only fake callback has no OS userdata. Production cannot assume this recovery is safe.
    unsafe {
        drop(Arc::from_raw(transferred));
    }
}

#[test]
fn class_release_failure_retains_retry_and_success_is_once() {
    use super::release::release_class;
    let mut registered = true;
    assert_eq!(release_class(&mut registered, || Err(5)), Err(5));
    assert!(registered);
    assert_eq!(release_class(&mut registered, || Ok::<_, u32>(())), Ok(()));
    assert!(!registered);
    assert_eq!(
        release_class(&mut registered, || panic!("duplicate unregister")),
        Ok::<_, u32>(())
    );
}

#[test]
fn installation_and_wake_are_not_destroy_acknowledgements() {
    let events = Events::new(7);
    events.installed.store(true, Ordering::Release);
    events.wake.store(true, Ordering::Release);
    events.hwnd.store(42, Ordering::Release);
    assert!(!events.complete(7));
}
