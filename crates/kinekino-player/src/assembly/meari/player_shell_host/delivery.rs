//! The same observer factory is used by Native startup and opt-in queue conformance tests.
use super::StartRouteObservation;
use meari_native::{
    application::window_lifecycle::{NativeChild, WindowLifecycle},
    infrastructure::{
        native_host::NativeDocumentHandle,
        window::{NativeWindowNotice, NativeWindowObserver, NativeWindowWake},
    },
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

pub(super) type ChildrenSlot<C> = Rc<RefCell<Option<Rc<WindowLifecycle<C>>>>>;
#[derive(Default)]
struct PendingWake(Cell<bool>);
impl PendingWake {
    fn mark(&self) {
        self.0.set(true);
    }
    fn cancel(&self) {
        self.0.set(false);
    }
    fn deliver<E>(&self, send: impl FnOnce() -> Result<bool, E>) -> Result<(), E> {
        if self.0.get() {
            send()?;
            self.0.set(false);
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationFailureStage {
    DocumentStamp,
    ChildrenBorrow,
    ChildrenMissing,
    WindowStamp,
    RouteBorrow,
    FrameConsumerBorrow,
    FrameConsumer,
    FrameWake,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservationFailure {
    pub stage: ObservationFailureStage,
    pub host_id: u64,
    pub notice_revision: u64,
}

pub(super) fn record_delivery(
    failure: &Cell<Option<ObservationFailure>>,
    notice: &NativeWindowNotice,
    result: Result<(), ObservationFailureStage>,
) -> Result<(), String> {
    match result {
        Err(stage) => {
            failure.set(Some(ObservationFailure {
                stage,
                host_id: notice.host_id,
                notice_revision: notice.revision,
            }));
            Err(format!("shell observation unavailable: {stage:?}"))
        }
        Ok(()) => {
            failure.set(None);
            Ok(())
        }
    }
}

#[cfg(all(test, feature = "native-observer-test-support"))]
pub(super) fn make_observer<C: NativeChild>(
    document: NativeDocumentHandle,
    children: ChildrenSlot<C>,
    route: Rc<RefCell<StartRouteObservation>>,
    failure: Rc<Cell<Option<ObservationFailure>>>,
) -> NativeWindowObserver {
    make_observer_with_frames(
        document,
        children,
        route,
        failure,
        Weak::new(),
        Rc::new(RefCell::new(None)),
    )
}

pub(super) fn make_observer_with_frames<C: NativeChild>(
    document: NativeDocumentHandle,
    children: ChildrenSlot<C>,
    route: Rc<RefCell<StartRouteObservation>>,
    failure: Rc<Cell<Option<ObservationFailure>>>,
    frames: Weak<RefCell<Option<Box<dyn super::ShellFrameConsumer<C>>>>>,
    wake: Rc<RefCell<Option<NativeWindowWake>>>,
) -> NativeWindowObserver {
    let pending_wake = PendingWake::default();
    Box::new(move |notice| {
        let result = (|| {
            let stamp = document
                .stamp()
                .map_err(|_| ObservationFailureStage::DocumentStamp)?;
            let children = children
                .try_borrow()
                .map_err(|_| ObservationFailureStage::ChildrenBorrow)?
                .clone()
                .ok_or(ObservationFailureStage::ChildrenMissing)?;
            let window = children
                .stamp()
                .map_err(|_| ObservationFailureStage::WindowStamp)?;
            let mut route = route
                .try_borrow_mut()
                .map_err(|_| ObservationFailureStage::RouteBorrow)?;
            let before = (route.state.phase(), route.state.observed_route().cloned());
            route.observe(&notice, &stamp, window);
            let closing = route.state.intent() == Some(crate::presentation::web_ui_adapter::player_shell::ShellIntent::CloseRequested)
                || matches!(notice.kind,meari_native::infrastructure::window::NativeWindowNoticeKind::Closed|meari_native::infrastructure::window::NativeWindowNoticeKind::ForcedIncomplete);
            if closing {
                pending_wake.cancel();
            } else if before != (route.state.phase(), route.state.observed_route().cloned()) {
                pending_wake.mark();
            }
            #[cfg(test)]
            let observed_route = route
                .state
                .observed_route()
                .map(|route| route.as_str().to_owned());
            drop(route);
            // Weak capture breaks registrar -> parent binding -> observer -> consumer cycles.
            let notify = if let Some(frames) = frames.upgrade() {
                let mut slot = frames
                    .try_borrow_mut()
                    .map_err(|_| ObservationFailureStage::FrameConsumerBorrow)?;
                match slot.as_mut() {
                    Some(consumer) => consumer
                        .observe(&notice)
                        .map_err(|_| ObservationFailureStage::FrameConsumer)?,
                    None => false,
                }
            } else {
                false
            };
            #[cfg(test)]
            let probe_notify = super::super::viewer_native_probe::current().is_some_and(|probe| {
                let jobs = probe.jobs_notice(&document, &notice, observed_route.as_deref());
                let live = probe.live_notice(&document, &notice, observed_route.as_deref());
                probe.input_diagnostic_once(
                    4,
                    "delivery-after-probe",
                    &format!("live={live} jobs={jobs} consumer_notify={notify} closing={closing}"),
                );
                jobs || live
            });
            #[cfg(test)]
            let notify = notify || probe_notify;
            // Native checked/document/consumer borrows have ended. Wake sends a bit only.
            if notify && !closing {
                pending_wake.mark();
            }
            pending_wake.deliver(|| {
                let wake = wake
                    .try_borrow()
                    .map_err(|_| ObservationFailureStage::FrameWake)?
                    .clone()
                    .ok_or(ObservationFailureStage::FrameWake)?;
                let result = wake.wake();
                #[cfg(test)]
                if let Some(probe) = super::super::viewer_native_probe::current() {
                    probe.input_diagnostic_once(
                        5,
                        "original-wake-result",
                        &format!(
                            "ok={} delivered={:?} error_variant={:?}",
                            result.is_ok(),
                            result.as_ref().ok(),
                            result.as_ref().err().map(std::mem::discriminant)
                        ),
                    );
                }
                result.map_err(|_| ObservationFailureStage::FrameWake)
            })?;
            Ok(())
        })();
        // Err pauses delivery; the binding retains its bounded latest-per-kind queue.
        record_delivery(&failure, &notice, result)
    })
}

#[cfg(all(test, feature = "native-observer-test-support"))]
mod tests;
