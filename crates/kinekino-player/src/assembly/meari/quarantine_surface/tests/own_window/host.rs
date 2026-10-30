//! Concrete original NoMedia scenario; startup/host mechanism is shared.
use super::fixture::{Evidence, Fixture, Stage};
use crate::{
    assembly::meari::quarantine_surface::QuarantineChild,
    test_support::native_window_host::Scenario,
};
use meari_native::{
    application::window_lifecycle::WindowLifecycle,
    infrastructure::{native_host::*, window::*},
};
use std::{cell::RefCell, io, rc::Rc, time::Instant};
pub(super) type Controller = crate::test_support::native_window_host::Controller<Fixture>;
impl Scenario for Fixture {
    type Child = QuarantineChild;
    type Evidence = Evidence;
    fn prepare(
        document: NativeDocumentHandle,
        video: String,
        children: Rc<WindowLifecycle<QuarantineChild>>,
        surfaces: NativeSurfaceRegistrar<QuarantineChild>,
        deadline: Instant,
        evidence: Rc<RefCell<Evidence>>,
    ) -> Self {
        Self {
            document,
            video,
            children,
            surfaces: Some(surfaces),
            registration: None,
            registration_failure: None,
            wake: None,
            stage: Stage::Register,
            redraw: false,
            deadline,
            evidence,
        }
    }
    fn submitted(&mut self, notice: NativeWindowNotice) -> Result<(), String> {
        Fixture::submitted(self, notice)
    }
    fn failed(&mut self, error: String) {
        self.evidence.borrow_mut().failure.get_or_insert(error);
        if let Some(wake) = &self.wake {
            let _ = wake.wake();
        }
    }
    fn after_dispatch(&mut self, turn: HostTurn) -> Result<HostEffects, io::Error> {
        if Instant::now() >= self.deadline || self.evidence.borrow().failure.is_some() {
            self.evidence.borrow_mut().failure.get_or_insert_with(|| {
                "own-window deadline reached; original cleanup required".into()
            });
            self.wake
                .as_ref()
                .ok_or_else(|| io::Error::other("missing close wake"))?
                .request_close()
                .map_err(io::Error::other)?;
            return Ok(HostEffects::default());
        }
        if matches!(self.stage, Stage::Finished) {
            let reflected = self.evidence.borrow_mut().reflect(&turn);
            if let Err(error) = reflected {
                self.evidence.borrow_mut().failure.get_or_insert(error);
            }
            self.wake
                .as_ref()
                .ok_or_else(|| io::Error::other("missing close wake"))?
                .request_close()
                .map_err(io::Error::other)?;
            return Ok(HostEffects::default());
        }
        if std::mem::take(&mut self.redraw) {
            return Ok(HostEffects {
                committed: None,
                redraw: true,
            });
        }
        Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "waiting for original frame",
        ))
    }
    fn set_wake(&mut self, wake: NativeWindowWake) {
        self.wake = Some(wake);
    }
    fn close(&self) {
        self.evidence.borrow_mut().close_seen = true;
    }
}
