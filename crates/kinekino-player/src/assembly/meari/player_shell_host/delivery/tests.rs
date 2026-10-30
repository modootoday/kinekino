use super::*;

#[test]
fn pending_wake_is_one_shot_and_accepts_coalesced_delivery() {
    let wake = PendingWake::default();
    let sent = Cell::new(0);
    wake.mark();
    wake.mark();
    wake.deliver(|| {
        sent.set(sent.get() + 1);
        Ok::<_, ()>(false)
    })
    .unwrap();
    wake.deliver(|| {
        sent.set(sent.get() + 1);
        Ok::<_, ()>(true)
    })
    .unwrap();
    assert_eq!(sent.get(), 1);
}
#[test]
fn failed_wake_preserves_pending_progress_until_explicit_delivery_retry() {
    let wake = PendingWake::default();
    let sent = Cell::new(0);
    wake.mark();
    assert_eq!(
        wake.deliver(|| {
            sent.set(sent.get() + 1);
            Err::<bool, _>("original wake failed")
        }),
        Err("original wake failed")
    );
    assert!(wake.0.get());
    wake.deliver(|| {
        sent.set(sent.get() + 1);
        Ok::<_, &str>(true)
    })
    .unwrap();
    assert!(!wake.0.get());
    assert_eq!(sent.get(), 2);
}
#[test]
fn unchanged_observation_does_not_request_wake() {
    let wake = PendingWake::default();
    wake.deliver(|| panic!("Idle delivery must not send a wake"))
        .unwrap_or_else(|_: ()| panic!("Unexpected error"));
}
#[test]
fn terminal_cancellation_prevents_late_progress_wake() {
    let wake = PendingWake::default();
    wake.mark();
    wake.cancel();
    wake.deliver(|| panic!("Terminal delivery must not send a wake"))
        .unwrap_or_else(|_: ()| panic!("Unexpected error"));
}
use crate::{
    application::api::{ApplicationClientHandle, PlayerSdk},
    assembly::meari::{
        player_shell_document::{DocumentOptions, mount_prepared_shell},
        player_shell_startup::{ShellInputs, prepare_shell},
    },
    presentation::web_ui_adapter::player_shell::document::StartLabels,
};
use meari_native::{
    application::window_lifecycle::{
        CleanupProgress, CleanupRequest, CompletionSignal, LifecycleLimits, MonotonicClock,
    },
    domain::Viewport,
    infrastructure::window::{
        NativeNoticeError,
        test_support::{MetadataProbeKind, ObserverDeliveryHarness},
    },
};
use std::{convert::Infallible, sync::Arc, task::Waker, time::Duration};

// An uninhabited child type: the test cannot create a fake playback child or native parent.
#[derive(Clone, Copy)]
enum NoChild {}
impl NativeChild for NoChild {
    type Error = Infallible;
    fn parent_lost(&mut self) {
        match *self {}
    }
    fn begin_cleanup(
        &mut self,
        _: CleanupRequest,
        _: CompletionSignal,
    ) -> Result<CleanupProgress, Self::Error> {
        match *self {}
    }
    fn observe_cleanup(&mut self, _: CleanupRequest) -> Result<CleanupProgress, Self::Error> {
        match *self {}
    }
}
struct Clock;
impl MonotonicClock for Clock {
    fn now(&self) -> Duration {
        Duration::ZERO
    }
}

struct Fixture {
    document: NativeDocumentHandle,
    route: Rc<RefCell<StartRouteObservation>>,
    failure: Rc<Cell<Option<ObservationFailure>>>,
    children: ChildrenSlot<NoChild>,
    calls: Rc<RefCell<Vec<(u64, bool)>>>,
}
fn fixture() -> Fixture {
    let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
    let prepared = match prepare_shell(
        ShellInputs {
            client,
            host_owner: (),
        },
        StartLabels {
            connection: "관측 없음".into(),
            time: "관측 없음".into(),
        },
    ) {
        Ok(prepared) => prepared,
        Err(_) => panic!("fixed graph preparation failed"),
    };
    let mounted = match mount_prepared_shell(
        prepared,
        DocumentOptions {
            namespace: "observer-delivery-test".into(),
            epoch: 1,
            source_generation: 1,
            viewport: Viewport {
                width: 320,
                height: 180,
                dark: true,
            },
        },
    ) {
        Ok(mounted) => mounted,
        Err(failure) => panic!("{:?}: {}", failure.stage, failure.diagnostic),
    };
    let children = WindowLifecycle::new(
        LifecycleLimits::new(1, Duration::from_secs(1)).unwrap(),
        Clock,
        Waker::noop().clone(),
    )
    .unwrap();
    Fixture {
        document: mounted.document,
        route: Rc::new(RefCell::new(StartRouteObservation::new(
            mounted.state,
            mounted.ticket,
        ))),
        failure: Rc::new(Cell::new(None)),
        children: Rc::new(RefCell::new(Some(Rc::new(children)))),
        calls: Rc::new(RefCell::new(Vec::new())),
    }
}
fn observer(fixture: &Fixture) -> NativeWindowObserver {
    let mut production = make_observer(
        fixture.document.clone(),
        fixture.children.clone(),
        fixture.route.clone(),
        fixture.failure.clone(),
    );
    let calls = fixture.calls.clone();
    Box::new(move |notice| {
        let revision = notice.revision;
        let result = production(notice);
        calls.borrow_mut().push((revision, result.is_ok()));
        result
    })
}

#[test]
fn real_route_borrow_failure_pauses_original_queue_until_explicit_retry_once() {
    let fixture = fixture();
    let harness = ObserverDeliveryHarness::new(fixture.document.id(), observer(&fixture));
    let guard = fixture.route.borrow_mut();
    assert!(matches!(
        harness.publish_metadata(MetadataProbeKind::Created),
        Err(NativeNoticeError::Callback(_))
    ));
    let failure = fixture
        .failure
        .get()
        .expect("actual route borrow must fail");
    assert_eq!(failure.stage, ObservationFailureStage::RouteBorrow);
    let paused = harness.state().unwrap();
    assert!(paused.paused);
    assert_eq!(paused.pending.len(), 1);
    assert_eq!(paused.pending[0].host_id, fixture.document.id());
    assert_eq!(paused.pending[0].revision, failure.notice_revision);
    assert!(matches!(harness.flush(), Err(NativeNoticeError::Paused)));
    assert_eq!(*fixture.calls.borrow(), [(failure.notice_revision, false)]);
    drop(guard);
    harness.retry_observer().unwrap();
    assert!(!harness.state().unwrap().paused);
    assert!(harness.state().unwrap().pending.is_empty());
    assert_eq!(
        *fixture.calls.borrow(),
        [
            (failure.notice_revision, false),
            (failure.notice_revision, true)
        ]
    );
    assert!(fixture.failure.get().is_none());
    harness.flush().unwrap();
    harness.retry_observer().unwrap();
    assert_eq!(fixture.calls.borrow().len(), 2);
    assert!(fixture.route.borrow().state.observed_route().is_none());
}

#[test]
fn paused_same_kind_replacement_delivers_only_latest_probe_without_route_authority() {
    let fixture = fixture();
    let harness = ObserverDeliveryHarness::new(fixture.document.id(), observer(&fixture));
    let guard = fixture.route.borrow_mut();
    assert!(matches!(
        harness.publish_metadata(MetadataProbeKind::Created),
        Err(NativeNoticeError::Callback(_))
    ));
    let failed = fixture.failure.get().unwrap();
    assert_eq!(failed.stage, ObservationFailureStage::RouteBorrow);
    assert!(matches!(
        harness.publish_metadata(MetadataProbeKind::Created),
        Err(NativeNoticeError::Paused)
    ));
    let paused = harness.state().unwrap();
    assert!(paused.paused);
    assert_eq!(paused.pending.len(), 1);
    let latest = paused.pending[0].revision;
    assert!(latest > failed.notice_revision);
    assert_eq!(paused.pending[0].host_id, fixture.document.id());
    assert_eq!(fixture.calls.borrow().len(), 1);
    drop(guard);
    harness.retry_observer().unwrap();
    assert_eq!(
        *fixture.calls.borrow(),
        [(failed.notice_revision, false), (latest, true)]
    );
    assert!(harness.state().unwrap().pending.is_empty());
    assert!(!harness.state().unwrap().paused);
    harness.retry_observer().unwrap();
    harness.flush().unwrap();
    assert_eq!(fixture.calls.borrow().len(), 2);
    assert!(fixture.route.borrow().state.observed_route().is_none());
}
