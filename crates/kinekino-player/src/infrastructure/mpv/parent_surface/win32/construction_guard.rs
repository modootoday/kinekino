use super::{
    HiddenSurfaceProbe, ProbeError, preflight,
    window_events::{Events, window_proc},
};
use crate::infrastructure::mpv::NativeSurfaceInput;
use std::sync::{Arc, atomic::Ordering};
use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND},
        System::Threading::GetCurrentThreadId,
        UI::WindowsAndMessaging::*,
    },
    core::{PCWSTR, w},
};

#[derive(Clone, Copy)]
enum ChildProfile {
    LegacyHidden,
    VisibleRenderOnly,
}
impl ChildProfile {
    fn extended_style(self) -> WINDOW_EX_STYLE {
        match self {
            Self::LegacyHidden => WINDOW_EX_STYLE::default(),
            Self::VisibleRenderOnly => WS_EX_NOACTIVATE,
        }
    }
}
#[cfg(test)]
mod profile_tests;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub(super) enum Phase {
    #[default]
    Prepared,
    Registering,
    ClassOwned,
    Creating,
    Returned,
}
#[derive(Default)]
pub(super) struct ConstructionGuard {
    pub phase: Phase,
    pub pending: Option<*const Events>,
}
impl ConstructionGuard {
    pub fn release_observable(&self) -> bool {
        matches!(
            self.phase,
            Phase::Prepared | Phase::ClassOwned | Phase::Returned
        )
    }
}
impl HiddenSurfaceProbe {
    /// Must be invoked only after this inert owner is centrally retained. WndProc never borrows it.
    pub fn construct_retained(&mut self, input: &NativeSurfaceInput) -> Result<(), ProbeError> {
        if self.binding != (input.window(), input.child())
            || self.construction.phase != Phase::Prepared
        {
            return Err(ProbeError::ObservationMismatch);
        }
        input
            .with_probe_parent(|parent| self.construct_on_parent(parent))
            .map_err(ProbeError::Input)?
    }
    /// Shared OS ownership primitive only. Callers retain the owner before entering here.
    pub(super) fn construct_on_parent(
        &mut self,
        parent: &meari_native::infrastructure::window::NativeScopedParent<'_>,
    ) -> Result<(), ProbeError> {
        self.construct_on_parent_with_proc(parent, Some(window_proc))
    }
    pub(super) fn construct_on_parent_with_proc(
        &mut self,
        parent: &meari_native::infrastructure::window::NativeScopedParent<'_>,
        procedure: WNDPROC,
    ) -> Result<(), ProbeError> {
        self.construct_profile(parent, procedure, ChildProfile::LegacyHidden)
    }
    pub(super) fn construct_visible_on_parent(
        &mut self,
        parent: &meari_native::infrastructure::window::NativeScopedParent<'_>,
        procedure: WNDPROC,
    ) -> Result<(), ProbeError> {
        self.construct_profile(parent, procedure, ChildProfile::VisibleRenderOnly)
    }
    fn construct_profile(
        &mut self,
        parent: &meari_native::infrastructure::window::NativeScopedParent<'_>,
        procedure: WNDPROC,
        profile: ChildProfile,
    ) -> Result<(), ProbeError> {
        self.parent = preflight::parent(parent)?;
        // SAFETY: Same live original parent borrow, UI thread verified above. No registry callback.
        unsafe {
            self.thread = GetCurrentThreadId();
            self.instance =
                HINSTANCE(GetWindowLongPtrW(self.parent, GWLP_HINSTANCE) as *mut std::ffi::c_void);
            let descriptor = WNDCLASSW {
                lpfnWndProc: procedure,
                hInstance: self.instance,
                lpszClassName: PCWSTR(self.class.as_ptr()),
                ..Default::default()
            };
            self.construction.phase = Phase::Registering;
            let registered = RegisterClassW(&descriptor);
            if registered == 0 {
                self.construction.phase = Phase::Returned;
                return Err(ProbeError::Os(windows::core::Error::from_thread()));
            }
            self.class_registered = true;
            self.construction.phase = Phase::ClassOwned;
            let transferred = Arc::into_raw(Arc::clone(&self.events));
            self.construction.pending = Some(transferred);
            self.construction.phase = Phase::Creating;
            let result = CreateWindowExW(
                profile.extended_style(),
                PCWSTR(self.class.as_ptr()),
                w!(""),
                WS_CHILD | WS_DISABLED | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
                0,
                0,
                0,
                0,
                Some(self.parent),
                None,
                Some(self.instance),
                Some(transferred.cast()),
            );
            self.construction.phase = Phase::Returned;
            self.synchronize_construction()?;
            self.hwnd = Some(result.map_err(ProbeError::Os)?);
            self.observe_hidden()?;
            Ok(())
        }
    }
    pub(super) fn synchronize_construction(&mut self) -> Result<(), ProbeError> {
        if !self.construction.release_observable() {
            return Err(ProbeError::ObservationMismatch);
        }
        if let Some(pending) = self.construction.pending.take() {
            // Only a known normal OS return permits reclaiming an unadopted creator reference.
            if !self.events.installed.load(Ordering::Acquire) {
                unsafe {
                    drop(Arc::from_raw(pending));
                }
            }
        }
        let recorded = self.events.hwnd.load(Ordering::Acquire);
        if recorded != 0 {
            self.hwnd = Some(HWND(recorded as *mut std::ffi::c_void));
        }
        Ok(())
    }
}
