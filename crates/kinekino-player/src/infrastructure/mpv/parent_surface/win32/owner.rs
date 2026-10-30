use super::{ProbeError, construction_guard::ConstructionGuard, preflight, window_events::Events};
use crate::infrastructure::mpv::parent_surface::NativeSurfaceInput;
use std::{
    marker::PhantomData,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use windows::Win32::{
    Foundation::{HINSTANCE, HWND, RECT},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::*,
};

static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);
pub struct ProbeFailure {
    pub error: ProbeError,
    pub partial: Option<HiddenSurfaceProbe>,
    pub input: NativeSurfaceInput,
}
impl std::fmt::Debug for ProbeFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProbeFailure")
            .field("error", &self.error)
            .field("partial", &self.partial.is_some())
            .finish_non_exhaustive()
    }
}
/// Diagnostic readback only. Copying this does not authorize display or media open.
#[derive(Debug)]
pub struct ProbeObservation {
    pub generation: u64,
    pub thread: u32,
    pub client_width: i32,
    pub client_height: i32,
    pub disabled: bool,
    pub visible: bool,
}
pub struct HiddenSurfaceProbe {
    input: Option<NativeSurfaceInput>,
    pub(super) binding: (
        meari_native::application::window_lifecycle::WindowStamp,
        u64,
    ),
    pub(super) construction: ConstructionGuard,
    pub(super) hwnd: Option<HWND>,
    pub(super) parent: HWND,
    pub(super) thread: u32,
    pub(super) generation: u64,
    pub(super) events: Arc<Events>,
    pub(super) class: Vec<u16>,
    pub(super) instance: HINSTANCE,
    pub(super) class_registered: bool,
    _ui: PhantomData<Rc<()>>,
}
impl HiddenSurfaceProbe {
    /// Consumes the original pair, and returns that same pair on every failure.
    /// Success retains it alongside a hidden zero-area HWND, with no display/route/engine authority.
    pub fn create(input: NativeSurfaceInput) -> Result<Self, Box<ProbeFailure>> {
        let mut owner = match Self::prepare_retained(&input) {
            Ok(owner) => owner,
            Err(error) => {
                return Err(Box::new(ProbeFailure {
                    error,
                    partial: None,
                    input,
                }));
            }
        };
        match owner.construct_retained(&input) {
            Ok(()) => {
                owner.input = Some(input);
                Ok(owner)
            }
            Err(error) => Err(Box::new(ProbeFailure {
                error,
                input,
                partial: Some(owner),
            })),
        }
    }
    /// Inert allocation only. The host must store this owner before invoking construction.
    pub fn prepare_retained(input: &NativeSurfaceInput) -> Result<Self, ProbeError> {
        let generation = NEXT_GENERATION
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| ProbeError::CounterExhausted)?;
        let class: Vec<u16> = format!("player.native.surface.probe.{generation}\0")
            .encode_utf16()
            .collect();
        Ok(Self {
            input: None,
            binding: (input.window(), input.child()),
            construction: ConstructionGuard::default(),
            hwnd: None,
            parent: HWND::default(),
            thread: 0,
            generation,
            events: Arc::new(Events::new(generation)),
            class,
            instance: HINSTANCE::default(),
            class_registered: false,
            _ui: PhantomData,
        })
    }
    pub fn observe_hidden(&self) -> Result<ProbeObservation, ProbeError> {
        if self.events.complete(self.generation) {
            return Err(ProbeError::ObservationMismatch);
        }
        let hwnd = self.hwnd.ok_or(ProbeError::ObservationMismatch)?;
        // SAFETY: Owner is UI-only. Original generation, UI thread, parent and style are rechecked.
        unsafe {
            if GetCurrentThreadId() != self.thread {
                return Err(ProbeError::ForeignThread);
            }
            preflight::require_thread(hwnd, self.thread)?;
            if GetParent(hwnd).map_err(ProbeError::Os)? != self.parent
                || GetWindowLongPtrW(hwnd, GWLP_USERDATA) != Arc::as_ptr(&self.events) as isize
            {
                return Err(ProbeError::ObservationMismatch);
            }
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
            let mut rect = RECT::default();
            GetClientRect(hwnd, &mut rect).map_err(ProbeError::Os)?;
            let disabled = style & WS_DISABLED.0 != 0;
            let visible = style & WS_VISIBLE.0 != 0;
            if !disabled || visible || style & WS_CHILD.0 == 0 || rect != RECT::default() {
                return Err(ProbeError::ObservationMismatch);
            }
            Ok(ProbeObservation {
                generation: self.generation,
                thread: self.thread,
                client_width: rect.right,
                client_height: rect.bottom,
                disabled,
                visible,
            })
        }
    }
    /// No borrowed client-frame authority exists in the pinned producer yet.
    pub fn require_display(&self) -> Result<(), ProbeError> {
        let input = self.input.as_ref().ok_or(ProbeError::ObservationMismatch)?;
        input.inspect().map_err(ProbeError::Input)?;
        self.observe_hidden()?;
        Err(ProbeError::DisplayAdmissionUnavailable)
    }
}
impl Drop for HiddenSurfaceProbe {
    fn drop(&mut self) {
        // Best effort only: Drop never emits a release receipt or a lifecycle completion.
        let _ = self.release();
    }
}
