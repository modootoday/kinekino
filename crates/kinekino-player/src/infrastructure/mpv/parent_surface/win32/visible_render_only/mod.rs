//! Private product transition candidate. No route, backend or stored readiness permission.
#[cfg(test)]
mod actual;
#[cfg(test)]
mod borrow_contract;
mod error;
mod events;
mod geometry;
mod observation;
pub(crate) mod routed_media;
#[cfg(test)]
mod style_trace;
use error::*;
#[cfg(test)]
use events::unexpected_input;
use events::visible_proc;
use geometry::{geometry, same_frame};
#[cfg(test)]
use observation::Readback;
use observation::{Focus, read_long, require_child_identity, require_parent};

use super::{HiddenSurfaceProbe, ProbeError, ProbeRelease, preflight, window_events};
use crate::infrastructure::mpv::parent_surface::{NativeSurfaceInput, SurfaceInputError};
use meari_native::{
    application::window_lifecycle::CleanupRequest,
    domain::surface_slot::{SurfaceClip, SurfacePlane, SurfaceVisibility},
    infrastructure::window::{NativeParentClientError, NativeSurfaceSnapshot},
};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    marker::PhantomData,
    rc::Rc,
    sync::{
        Arc, Weak,
        atomic::{AtomicU32, Ordering},
    },
};
use windows::Win32::{
    Foundation::{
        GetLastError, HWND, LPARAM, LRESULT, POINT, RECT, SetLastError, WIN32_ERROR, WPARAM,
    },
    Graphics::Gdi::ScreenToClient,
    System::Threading::{GetCurrentProcessId, GetCurrentThreadId},
    UI::{
        Input::KeyboardAndMouse::{GetActiveWindow, GetCapture, GetFocus, IsWindowEnabled},
        WindowsAndMessaging::*,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Prepared,
    Hidden,
    Transitioning,
    Visible,
    Uncertain,
    Revoked,
    Released,
}

/// Observation at each verify call, never stored/returned as backend startup authority.
pub(super) struct VisibleRenderOnlyProof<'a> {
    verify: &'a dyn Fn() -> SharedResult<()>,
    _ui: PhantomData<Rc<()>>,
}
impl VisibleRenderOnlyProof<'_> {
    pub(super) fn verify(&self) -> SharedResult<()> {
        (self.verify)()
    }
}

pub(super) struct PrepareFailure {
    pub error: VisibleError,
    pub input: NativeSurfaceInput,
}

/// Caller centrally retains this inert owner before construct/transition. All failures keep it.
pub(super) struct VisibleRenderOnlyChild {
    input: NativeSurfaceInput,
    native: HiddenSurfaceProbe,
    phase: Phase,
    frame: Option<NativeSurfaceSnapshot>, // Correlation only: every use reacquires the checked callback.
    focus: Option<Focus>,
    revoked: bool,
    first_error: RefCell<Option<Arc<VisibleError>>>,
    input_observation: Arc<AtomicU32>,
}
impl VisibleRenderOnlyChild {
    pub(super) fn prepare(
        input: NativeSurfaceInput,
    ) -> std::result::Result<Self, Box<PrepareFailure>> {
        let native = match HiddenSurfaceProbe::prepare_retained(&input) {
            Ok(owner) => owner,
            Err(error) => {
                return Err(Box::new(PrepareFailure {
                    error: VisibleError::Child(error),
                    input,
                }));
            }
        };
        Ok(Self {
            input,
            native,
            phase: Phase::Prepared,
            frame: None,
            focus: None,
            revoked: false,
            first_error: RefCell::new(None),
            input_observation: Arc::new(AtomicU32::new(0)),
        })
    }
    fn finish_attempt(
        &mut self,
        result: Result<()>,
        success: Phase,
    ) -> std::result::Result<(), Arc<VisibleError>> {
        match result {
            Ok(()) => {
                self.phase = success;
                Ok(())
            }
            Err(error) => {
                let error = remember(&self.first_error, error);
                self.phase = Phase::Uncertain;
                Err(error)
            }
        }
    }
    pub(super) fn first_error(&self) -> Option<Arc<VisibleError>> {
        self.first_error.borrow().clone()
    }

    pub(super) fn construct_retained(&mut self) -> std::result::Result<(), Arc<VisibleError>> {
        let result = if self.phase != Phase::Prepared || self.revoked {
            Err(VisibleError::Phase)
        } else {
            self.input
                .with_checked_parent(|checked| {
                    checked
                        .with_parent(|parent, snapshot| {
                            geometry(snapshot)?;
                            let focus = Focus::read();
                            self.native
                                .construct_visible_on_parent(parent, Some(visible_proc))
                                .map_err(VisibleError::Child)?;
                            let child = self.native.hwnd.ok_or(VisibleError::Identity)?;
                            let extended = read_long(child, GWL_EXSTYLE)? as u32;
                            #[cfg(test)]
                            style_trace::record_creation(style_trace::CreationStyleTrace {
                                hwnd: child.0 as isize,
                                parent: self.native.parent.0 as isize,
                                generation: self.native.generation,
                                owner_thread: self.native.thread,
                                current_thread: unsafe { GetCurrentThreadId() },
                                requested: WS_EX_NOACTIVATE.0,
                                observed: extended,
                                installed: self.native.events.installed.load(Ordering::Acquire),
                                destroyed: self.native.events.destroyed.load(Ordering::Acquire),
                                detached: self.native.events.detached.load(Ordering::Acquire),
                                callback_error: self
                                    .native
                                    .events
                                    .callback_error
                                    .load(Ordering::Acquire),
                            });
                            if extended & WS_EX_NOACTIVATE.0 == 0 {
                                return Err(VisibleError::CreationStyleMissing);
                            }
                            focus.require_unchanged(Focus::read(), child)?;
                            Ok(())
                        })
                        .map_err(VisibleError::Parent)?
                })
                .map_err(VisibleError::Input)
                .and_then(|result| result)
        };
        let result = result.and_then(|()| {
            let child = self.child()?;
            events::register(child, self.native.generation, &self.input_observation)
        });
        self.finish_attempt(result, Phase::Hidden)
    }

    /// Real OS transition candidate. A later failed check leaves residual visibility unknown.
    pub(super) fn transition_visible(&mut self) -> std::result::Result<(), Arc<VisibleError>> {
        if self.phase != Phase::Hidden || self.revoked {
            return self.finish_attempt(Err(VisibleError::Phase), Phase::Visible);
        }
        self.phase = Phase::Transitioning;
        let result = (|| {
            // First checked visit: observe the original hidden child, creation style, frame and focus.
            let (frame, focus) = self
                .input
                .with_checked_parent(|checked| {
                    checked
                        .with_parent(|parent, snapshot| {
                            geometry(snapshot)?;
                            self.native.observe_hidden().map_err(VisibleError::Child)?;
                            self.require_parent(parent)?;
                            let child = self.child()?;
                            let focus = Focus::read();
                            focus.require_unchanged(focus, child)?;
                            // Style is established at original creation, not mutated here.
                            if read_long(child, GWL_EXSTYLE)? as u32 & WS_EX_NOACTIVATE.0 == 0 {
                                return Err(VisibleError::CreationStyleMissing);
                            }
                            Ok((snapshot.clone(), focus))
                        })
                        .map_err(VisibleError::Parent)?
                })
                .map_err(VisibleError::Input)??;
            self.frame = Some(frame);
            self.focus = Some(focus);
            // A previous observation is not permission for another OS operation: reacquire original frame.
            self.input
                .with_checked_parent(|checked| {
                    checked
                        .with_parent(|parent, snapshot| {
                            self.require_frame(snapshot)?;
                            self.require_parent(parent)?;
                            self.native.observe_hidden().map_err(VisibleError::Child)?;
                            let child = self.child()?;
                            focus.require_unchanged(Focus::read(), child)?;
                            let [x, y, width, height] = geometry(snapshot)?;
                            // SAFETY: Original child/parent and current physical client geometry; no extra DPI conversion.
                            unsafe {
                                SetWindowPos(
                                    child,
                                    None,
                                    x,
                                    y,
                                    width,
                                    height,
                                    SWP_NOACTIVATE | SWP_NOZORDER | SWP_SHOWWINDOW,
                                )
                                .map_err(VisibleError::Os)?;
                            }
                            Ok(())
                        })
                        .map_err(VisibleError::Parent)?
                })
                .map_err(VisibleError::Input)??;
            // Side effects may reenter/change frame. Never substitute a newer frame silently.
            self.verify_visible()?;
            // Transition side effects must preserve focus; later user activity need not.
            focus.require_unchanged(Focus::read(), self.child()?)
        })();
        self.finish_attempt(result, Phase::Visible)
    }
    fn child(&self) -> Result<HWND> {
        self.native.hwnd.ok_or(VisibleError::Identity)
    }
    fn require_parent(
        &self,
        parent: &meari_native::infrastructure::window::NativeScopedParent<'_>,
    ) -> Result<()> {
        require_parent(&self.native, &self.input, parent)
    }
    fn require_frame(&self, snapshot: &NativeSurfaceSnapshot) -> Result<()> {
        if !self
            .frame
            .as_ref()
            .is_some_and(|expected| same_frame(expected, snapshot))
        {
            return Err(VisibleError::OriginalFrameChanged);
        }
        geometry(snapshot).map(|_| ())
    }
    fn verify_visible(&self) -> Result<()> {
        if self.revoked {
            return Err(VisibleError::Phase);
        }
        self.input
            .with_checked_parent(|checked| {
                checked
                    .with_parent(|parent, snapshot| {
                        self.require_parent(parent)?;
                        self.require_frame(snapshot)?;
                        self.observe_visible(geometry(snapshot)?)
                    })
                    .map_err(VisibleError::Parent)?
            })
            .map_err(VisibleError::Input)?
    }
    pub(super) fn with_visible_render_only<R>(
        &self,
        visit: impl for<'a> FnOnce(&VisibleRenderOnlyProof<'a>) -> R,
    ) -> SharedResult<R> {
        if self.phase != Phase::Visible {
            return Err(remember(&self.first_error, VisibleError::Phase));
        }
        self.input
            .with_checked_parent(|checked| {
                let verify = || {
                    checked
                        .with_parent(|parent, snapshot| {
                            self.require_parent(parent)?;
                            self.require_frame(snapshot)?;
                            self.observe_visible(geometry(snapshot)?)
                        })
                        .map_err(VisibleError::Parent)
                        .and_then(|result| result)
                        .map_err(|error| remember(&self.first_error, error))
                };
                verify()?;
                Ok(visit(&VisibleRenderOnlyProof {
                    verify: &verify,
                    _ui: PhantomData,
                }))
            })
            .map_err(|error| remember(&self.first_error, VisibleError::Input(error)))?
    }
    /// Admission is revoked first. Hide failure remains an error with retained original owner.
    pub(super) fn revoke(
        &mut self,
        request: CleanupRequest,
    ) -> std::result::Result<(), Arc<VisibleError>> {
        self.revoked = true;
        self.input.revoke();
        let result = self
            .input
            .with_cleanup_parent(request, |parent| {
                self.require_parent(parent)?;
                if let Some(child) = self.native.hwnd {
                    require_child_identity(&self.native, child)?;
                    // SAFETY: Original cleanup parent/stamp and retained child; never a startup path.
                    unsafe {
                        SetWindowPos(
                            child,
                            None,
                            0,
                            0,
                            0,
                            0,
                            SWP_NOACTIVATE
                                | SWP_NOZORDER
                                | SWP_NOMOVE
                                | SWP_NOSIZE
                                | SWP_HIDEWINDOW,
                        )
                        .map_err(VisibleError::Os)?;
                        if IsWindowVisible(child).as_bool() {
                            return Err(VisibleError::Identity);
                        }
                    }
                }
                Ok(())
            })
            .map_err(VisibleError::Input)
            .and_then(|result| result);
        self.finish_attempt(result, Phase::Revoked)
    }
    pub(super) fn release(
        &mut self,
        request: CleanupRequest,
    ) -> std::result::Result<ProbeRelease, Arc<VisibleError>> {
        self.revoked = true;
        self.input.revoke();
        let input = &self.input;
        let native = &mut self.native;
        let result = input
            .with_cleanup_parent(request, |parent| {
                require_parent(native, input, parent)?;
                native.release().map_err(VisibleError::Child)
            })
            .map_err(VisibleError::Input)
            .and_then(|result| result);
        match result {
            Ok(ProbeRelease::Complete { generation }) => {
                self.phase = Phase::Released;
                Ok(ProbeRelease::Complete { generation })
            }
            Ok(other) => Ok(other), // Unobserved/NotCreated is not a Destroy acknowledgement.
            Err(error) => {
                self.phase = Phase::Uncertain;
                Err(remember(&self.first_error, error))
            }
        }
    }
}

fn remember(slot: &RefCell<Option<Arc<VisibleError>>>, error: VisibleError) -> Arc<VisibleError> {
    let error = Arc::new(error);
    let mut first = slot.borrow_mut();
    if first.is_none() {
        *first = Some(error.clone());
    }
    error
}
#[cfg(test)]
mod tests;
