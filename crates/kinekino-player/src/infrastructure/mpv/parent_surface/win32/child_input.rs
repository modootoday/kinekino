use super::{HiddenSurfaceProbe, ProbeError, ProbeRelease, construction_guard::Phase, preflight};
use crate::infrastructure::mpv::parent_surface::{NativeSurfaceInput, SurfaceInputError};
use meari_native::infrastructure::window::NativeParentClientError;
use std::{marker::PhantomData, rc::Rc, sync::atomic::Ordering};
use windows::Win32::{
    System::Threading::{GetCurrentProcessId, GetCurrentThreadId},
    UI::{Input::KeyboardAndMouse::IsWindowEnabled, WindowsAndMessaging::*},
};

#[derive(Debug)]
pub enum ChildInputError {
    Input(SurfaceInputError),
    Parent(NativeParentClientError),
    Child(ProbeError),
}
impl std::fmt::Display for ChildInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "quarantine child: {self:?}")
    }
}
impl std::error::Error for ChildInputError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Input(e) => e,
            Self::Parent(e) => e,
            Self::Child(e) => e,
        })
    }
}

/// Callback-scoped read observation only. No display, media or future-input guarantee.
pub struct DisabledChildProof<'a> {
    verify: &'a dyn Fn() -> Result<(), ChildInputError>,
    _ui: PhantomData<Rc<()>>,
}
impl DisabledChildProof<'_> {
    /// Revalidates the same checked frame and original child immediately before use.
    pub fn verify(&self) -> Result<(), ChildInputError> {
        (self.verify)()
    }
}

#[derive(Clone, Copy)]
struct InputReadback {
    installed: bool,
    destroying: bool,
    callback_fault: bool,
    same_process: bool,
    same_thread: bool,
    enabled: bool,
    visible: bool,
}
impl InputReadback {
    fn accepts(self) -> bool {
        self.installed
            && !self.destroying
            && !self.callback_fault
            && self.same_process
            && self.same_thread
            && !self.enabled
            && !self.visible
    }
}
fn release_observation(
    class_retry_ready: bool,
    complete: bool,
    generation: u64,
    inert: bool,
) -> Result<ProbeRelease, ProbeError> {
    if class_retry_ready {
        Err(ProbeError::ClassFinalizationRequired)
    } else if complete {
        Ok(ProbeRelease::Complete { generation })
    } else if inert {
        Ok(ProbeRelease::NotCreated)
    } else {
        Ok(ProbeRelease::Unobserved)
    }
}
impl HiddenSurfaceProbe {
    /// Separate NoMedia profile. The legacy display/start gate is deliberately unchanged.
    pub(crate) fn construct_quarantine(
        &mut self,
        input: &NativeSurfaceInput,
    ) -> Result<(), ChildInputError> {
        if self.binding != (input.window(), input.child())
            || self.construction.phase != Phase::Prepared
        {
            return Err(ChildInputError::Child(ProbeError::ObservationMismatch));
        }
        input
            .with_checked_parent(|checked| {
                checked
                    .with_parent(|parent, _| self.construct_on_parent(parent))
                    .map_err(ChildInputError::Parent)?
                    .map_err(ChildInputError::Child)
            })
            .map_err(ChildInputError::Input)?
    }

    pub(crate) fn with_disabled_child<R>(
        &self,
        input: &NativeSurfaceInput,
        read: impl for<'a> FnOnce(&DisabledChildProof<'a>) -> R,
    ) -> Result<R, ChildInputError> {
        if self.binding != (input.window(), input.child()) {
            return Err(ChildInputError::Child(ProbeError::ObservationMismatch));
        }
        input
            .with_checked_parent(|checked| {
                let verify = || {
                    checked
                        .with_parent(|parent, _| {
                            if preflight::parent(parent)? != self.parent {
                                return Err(ProbeError::ObservationMismatch);
                            }
                            self.observe_hidden()?;
                            let hwnd = self.hwnd.ok_or(ProbeError::ObservationMismatch)?;
                            // SAFETY: Original UI owner, parent, callback pointer and styles were just rechecked.
                            let observation = unsafe {
                                let mut process = 0;
                                let thread = GetWindowThreadProcessId(hwnd, Some(&mut process));
                                InputReadback {
                                    installed: self.events.installed.load(Ordering::Acquire),
                                    destroying: self.events.destroy_entered.load(Ordering::Acquire)
                                        || self.events.destroyed.load(Ordering::Acquire)
                                        || self.events.detached.load(Ordering::Acquire),
                                    callback_fault: self
                                        .events
                                        .callback_error
                                        .load(Ordering::Acquire)
                                        != 0,
                                    same_process: process == GetCurrentProcessId(),
                                    same_thread: thread == self.thread
                                        && thread == GetCurrentThreadId(),
                                    enabled: IsWindowEnabled(hwnd).as_bool(),
                                    visible: IsWindowVisible(hwnd).as_bool(),
                                }
                            };
                            if observation.accepts() {
                                Ok(())
                            } else {
                                Err(ProbeError::ObservationMismatch)
                            }
                        })
                        .map_err(ChildInputError::Parent)?
                        .map_err(ChildInputError::Child)
                };
                verify()?;
                Ok(read(&DisabledChildProof {
                    verify: &verify,
                    _ui: PhantomData,
                }))
            })
            .map_err(ChildInputError::Input)?
    }

    /// Observation never reissues DestroyWindow or unregisters a class.
    pub(crate) fn observe_quarantine_release(&self) -> Result<ProbeRelease, ProbeError> {
        let inert = self.construction.release_observable()
            && !self.class_registered
            && self.hwnd.is_none()
            && !self.events.installed.load(Ordering::Acquire)
            && self.construction.pending.is_none();
        release_observation(
            self.quarantine_class_retry_ready(),
            self.events.complete(self.generation) && !self.class_registered,
            self.generation,
            inert,
        )
    }

    pub(crate) fn take_quarantine_notice(&self) -> bool {
        self.events.wake.swap(false, Ordering::AcqRel)
    }

    /// No Destroy remains in flight: explicit coordinator retry may finish class release.
    pub(crate) fn quarantine_class_retry_ready(&self) -> bool {
        self.construction.release_observable()
            && self.class_registered
            && (self.events.complete(self.generation)
                || (!self.events.installed.load(Ordering::Acquire)
                    && self.hwnd.is_none()
                    && self.construction.pending.is_none()))
    }
}

#[cfg(test)]
mod tests;
