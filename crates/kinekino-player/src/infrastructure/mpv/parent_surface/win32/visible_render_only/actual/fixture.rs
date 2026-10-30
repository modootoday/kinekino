//! Real submitted-frame scenario; copied metadata never creates parent or ready authority.
use super::super::{NativeSurfaceInput, VisibleError};
use super::{
    Evidence,
    child::{Child, State},
    explain,
};
use crate::test_support::native_window_host::Scenario;
use meari_native::{
    application::window_lifecycle::{ChildCreationFailure, ChildLease, WindowLifecycle},
    infrastructure::{native_host::*, window::*},
};
use std::{cell::RefCell, io, rc::Rc, time::Instant};
enum Stage {
    Register,
    Show,
    ChangedFrame,
    Cleanup,
    Finished,
}
pub(crate) struct Fixture {
    document: NativeDocumentHandle,
    video: String,
    children: Rc<WindowLifecycle<Child>>,
    surfaces: NativeSurfaceRegistrar<Child>,
    lease: Option<ChildLease>,
    owner: Option<Rc<RefCell<State>>>,
    registration_failure: Option<NativeSurfaceRegistrationFailure<std::sync::Arc<VisibleError>>>,
    wake: Option<NativeWindowWake>,
    stage: Stage,
    redraw: bool,
    deadline: Instant,
    evidence: Rc<RefCell<Evidence>>,
    expected_error: Option<std::sync::Arc<VisibleError>>,
}
impl Fixture {
    fn redraw(&mut self) -> Result<(), String> {
        self.redraw = true;
        self.wake
            .as_ref()
            .ok_or("missing original wake")?
            .wake()
            .map_err(explain)?;
        Ok(())
    }
    fn finish_cleanup(&mut self) -> Result<(), String> {
        let report = self.children.poll().map_err(explain)?;
        if report.remaining == 0 {
            let state = self
                .owner
                .as_ref()
                .ok_or("missing original diagnostic owner")?
                .borrow();
            let retained = state
                .first_error
                .as_ref()
                .ok_or("missing original first error after cleanup")?;
            let expected = self
                .expected_error
                .as_ref()
                .ok_or("missing expected original frame error")?;
            if !std::sync::Arc::ptr_eq(retained, expected) {
                return Err("cleanup replaced original first error".into());
            }
            self.evidence.borrow_mut().first_error_retained = true;
            drop(state);
            self.evidence.borrow_mut().remaining_zero = true;
            self.owner.take();
            self.lease.take();
            self.stage = Stage::Finished;
            self.wake
                .as_ref()
                .ok_or("missing close wake")?
                .request_close()
                .map_err(explain)?;
        }
        Ok(())
    }
}
impl Scenario for Fixture {
    type Child = Child;
    type Evidence = Evidence;
    fn prepare(
        document: NativeDocumentHandle,
        video: String,
        children: Rc<WindowLifecycle<Child>>,
        surfaces: NativeSurfaceRegistrar<Child>,
        deadline: Instant,
        evidence: Rc<RefCell<Evidence>>,
    ) -> Self {
        Self {
            document,
            video,
            children,
            surfaces,
            lease: None,
            owner: None,
            registration_failure: None,
            wake: None,
            stage: Stage::Register,
            redraw: false,
            deadline,
            evidence,
            expected_error: None,
        }
    }
    fn submitted(&mut self, notice: NativeWindowNotice) -> Result<(), String> {
        if !matches!(notice.kind, NativeWindowNoticeKind::FrameSubmitted)
            || matches!(self.stage, Stage::Finished | Stage::Cleanup)
        {
            return Ok(());
        }
        if Instant::now() >= self.deadline {
            return Err("original visible fixture deadline".into());
        }
        let frame = notice.frame.ok_or("missing actual frame")?;
        if frame.document != self.document.stamp().map_err(explain)?
            || notice.window != Some(self.children.stamp().map_err(explain)?)
        {
            return Err("foreign/delayed original frame".into());
        }
        match self.stage {
            Stage::Register => {
                let reservation = self
                    .surfaces
                    .reserve_video(&frame.document, &self.video)
                    .map_err(explain)?;
                let evidence = self.evidence.clone();
                let mut retained = None;
                let result = self
                    .surfaces
                    .register_reserved(reservation, |_, parent, slot| {
                        let state = Rc::new(RefCell::new(State::prepare(
                            NativeSurfaceInput::new(parent, slot),
                            evidence,
                        )));
                        let error = state.borrow().first_error.clone();
                        retained = Some(state.clone());
                        let child = Child::new(state);
                        match error {
                            Some(error) => Err(ChildCreationFailure {
                                error,
                                partial: Some(child),
                            }),
                            None => Ok(child),
                        }
                    });
                self.owner = retained;
                match result {
                    Ok(lease) => self.lease = Some(lease),
                    Err(failure) => {
                        let text = explain(&failure);
                        self.registration_failure = Some(failure);
                        return Err(text);
                    }
                }
                {
                    let mut state = self
                        .owner
                        .as_ref()
                        .ok_or("missing registered original owner")?
                        .borrow_mut();
                    let owner = state.ready_mut().map_err(explain)?;
                    if owner.phase != super::super::Phase::Prepared
                        || owner.native.hwnd.is_some()
                        || owner.native.class_registered
                        || owner
                            .native
                            .events
                            .installed
                            .load(std::sync::atomic::Ordering::Acquire)
                    {
                        return Err("registration was not inert before original fresh frame".into());
                    }
                    self.evidence.borrow_mut().original_window =
                        Some(explain(owner.input.window()));
                    self.evidence.borrow_mut().original_child = Some(owner.input.child());
                }
                self.evidence.borrow_mut().inert_registered = true;
                self.stage = Stage::Show;
                self.redraw()?;
            }
            Stage::Show => {
                let retained = self.owner.as_ref().ok_or("original child lost")?;
                let mut state = retained.borrow_mut();
                let owner = state.ready_mut().map_err(explain)?;
                owner.construct_retained().map_err(explain)?;
                owner.transition_visible().map_err(explain)?;
                owner
                    .with_visible_render_only(|proof| proof.verify())
                    .map_err(explain)?
                    .map_err(explain)?;
                self.evidence.borrow_mut().fresh_visible_input_focus = true;
                self.evidence.borrow_mut().show_frame =
                    owner.frame.as_ref().map(|s| format!("{:?}", s.frame));
                drop(state);
                self.stage = Stage::ChangedFrame;
                self.redraw()?;
            }
            Stage::ChangedFrame => {
                self.evidence.borrow_mut().changed_frame_notice = Some(explain(&frame.document));
                let retained = self.owner.as_ref().ok_or("original child lost")?;
                let mut state = retained.borrow_mut();
                let owner = state.ready_mut().map_err(explain)?;
                let denied = owner.with_visible_render_only(|proof| proof.verify());
                match denied {
                    Err(error) if matches!(error.as_ref(), VisibleError::OriginalFrameChanged) => {
                        self.evidence.borrow_mut().changed_frame_rejected = true;
                        self.evidence.borrow_mut().expected_first_error = Some(explain(&error));
                        self.expected_error = Some(error);
                    }
                    other => {
                        return Err(format!(
                            "changed original frame not rejected as expected: {other:?}"
                        ));
                    }
                }
                drop(state);
                let lease = self.lease.as_ref().ok_or("original cleanup lease lost")?;
                self.children.release_child(lease).map_err(explain)?;
                self.stage = Stage::Cleanup;
                self.finish_cleanup()?;
            }
            Stage::Cleanup | Stage::Finished => {}
        }
        Ok(())
    }
    fn failed(&mut self, error: String) {
        self.evidence.borrow_mut().failure.get_or_insert(error);
        if let Some(wake) = &self.wake {
            let _ = wake.wake();
        }
    }
    fn after_dispatch(&mut self, _: HostTurn) -> Result<HostEffects, io::Error> {
        if Instant::now() >= self.deadline || self.evidence.borrow().failure.is_some() {
            self.evidence.borrow_mut().failure.get_or_insert_with(|| {
                "original visible fixture failed/deadline; original cleanup required".into()
            });
            self.wake
                .as_ref()
                .ok_or_else(|| io::Error::other("missing close wake"))?
                .request_close()
                .map_err(io::Error::other)?;
            return Ok(HostEffects::default());
        }
        if matches!(self.stage, Stage::Cleanup) {
            self.finish_cleanup().map_err(io::Error::other)?;
        }
        if matches!(self.stage, Stage::Finished) {
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
            "waiting original submitted frame/cleanup",
        ))
    }
    fn set_wake(&mut self, wake: NativeWindowWake) {
        self.wake = Some(wake)
    }
    fn close(&self) {
        self.evidence.borrow_mut().close_seen = true;
    }
}
