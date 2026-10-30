use super::*;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Focus {
    pub(super) keyboard: HWND,
    pub(super) active: HWND,
    pub(super) foreground: HWND,
    pub(super) capture: HWND,
}
impl Focus {
    pub(super) fn read() -> Self {
        // SAFETY: Observation only on the original UI thread; no activation or focus mutation.
        unsafe {
            Self {
                keyboard: GetFocus(),
                active: GetActiveWindow(),
                foreground: GetForegroundWindow(),
                capture: GetCapture(),
            }
        }
    }
    pub(super) fn require_unchanged(self, current: Self, child: HWND) -> Result<()> {
        if self != current {
            return Err(VisibleError::FocusChanged);
        }
        current.require_child_unfocused(child)
    }
    pub(super) fn require_child_unfocused(self, child: HWND) -> Result<()> {
        if [self.keyboard, self.active, self.foreground, self.capture].contains(&child) {
            Err(VisibleError::FocusChanged)
        } else {
            Ok(())
        }
    }
}

impl VisibleRenderOnlyChild {
    pub(super) fn observe_visible(&self, expected: [i32; 4]) -> Result<()> {
        let child = self.child()?;
        events::require_original(child, self.native.generation, &self.input_observation)?;
        let message = self.input_observation.load(Ordering::Acquire);
        if message != 0 {
            return Err(VisibleError::InputObserved(message));
        }
        // SAFETY: Called only within fresh original checked parent callback on original UI thread.
        unsafe {
            preflight::require_thread(child, self.native.thread).map_err(VisibleError::Child)?;
            let mut process = 0;
            let thread = GetWindowThreadProcessId(child, Some(&mut process));
            let mut class = [0u16; 256];
            let length = GetClassNameW(child, &mut class);
            if length == 0 {
                return Err(VisibleError::Os(windows::core::Error::from_thread()));
            }
            let events = &self.native.events;
            if thread != GetCurrentThreadId()
                || process != GetCurrentProcessId()
                || GetParent(child).map_err(VisibleError::Os)? != self.native.parent
                || length <= 0
                || class[..length as usize] != self.native.class[..self.native.class.len() - 1]
                || read_long(child, GWLP_USERDATA)? != Arc::as_ptr(events) as isize
                || read_long(child, GWLP_WNDPROC)? != visible_proc as *const () as isize
                || !events.installed.load(Ordering::Acquire)
                || events.generation != self.native.generation
                || events.destroy_entered.load(Ordering::Acquire)
                || events.destroyed.load(Ordering::Acquire)
                || events.detached.load(Ordering::Acquire)
                || events.callback_error.load(Ordering::Acquire) != 0
            {
                return Err(VisibleError::Identity);
            }
            let style = read_long(child, GWL_STYLE)? as u32;
            let extended = read_long(child, GWL_EXSTYLE)? as u32;
            #[cfg(test)]
            style_trace::record_style_readback(extended);
            let mut rect = RECT::default();
            GetWindowRect(child, &mut rect).map_err(VisibleError::Os)?;
            let mut corner = POINT {
                x: rect.left,
                y: rect.top,
            };
            ScreenToClient(self.native.parent, &mut corner)
                .ok()
                .map_err(VisibleError::Os)?;
            let measured = [
                corner.x,
                corner.y,
                rect.right - rect.left,
                rect.bottom - rect.top,
            ];
            let evidence = Readback {
                child_style: style & WS_CHILD.0 != 0,
                disabled: style & WS_DISABLED.0 != 0 && !IsWindowEnabled(child).as_bool(),
                visible: style & WS_VISIBLE.0 != 0 && IsWindowVisible(child).as_bool(),
                nonactivating: extended & WS_EX_NOACTIVATE.0 != 0,
                geometry: measured == expected,
            };
            evidence.require()?;
        }
        events::require_available()?;
        self.focus.ok_or(VisibleError::Phase)?;
        // User focus may change after transition; the render-only child must never own it.
        Focus::read().require_child_unfocused(child)
    }
}
pub(super) fn require_parent(
    native: &HiddenSurfaceProbe,
    input: &NativeSurfaceInput,
    parent: &meari_native::infrastructure::window::NativeScopedParent<'_>,
) -> Result<()> {
    if native.binding != (input.window(), input.child()) {
        return Err(VisibleError::Identity);
    }
    let actual = preflight::parent(parent).map_err(VisibleError::Child)?;
    if native.construction.phase == super::super::construction_guard::Phase::Prepared
        && native.hwnd.is_none()
        && !native.class_registered
    {
        return Ok(());
    }
    if actual != native.parent {
        return Err(VisibleError::Identity);
    }
    Ok(())
}
pub(super) fn require_child_identity(native: &HiddenSurfaceProbe, child: HWND) -> Result<()> {
    // SAFETY: Original UI owner and cleanup parent were checked immediately before this call.
    unsafe {
        preflight::require_thread(child, native.thread).map_err(VisibleError::Child)?;
        if GetParent(child).map_err(VisibleError::Os)? != native.parent
            || read_long(child, GWLP_USERDATA)? != Arc::as_ptr(&native.events) as isize
            || native.events.destroy_entered.load(Ordering::Acquire)
            || native.events.detached.load(Ordering::Acquire)
            || native.events.callback_error.load(Ordering::Acquire) != 0
        {
            return Err(VisibleError::Identity);
        }
    }
    Ok(())
}

pub(super) fn read_long(child: HWND, index: WINDOW_LONG_PTR_INDEX) -> Result<isize> {
    // SAFETY: Private helper receives only the already scoped original owner handle.
    unsafe {
        window_events::checked_write(|| {
            SetLastError(WIN32_ERROR(0));
            let value = GetWindowLongPtrW(child, index);
            (value, GetLastError().0)
        })
        .map_err(VisibleError::WindowLongRead)
    }
}

#[derive(Clone, Copy)]
pub(super) struct Readback {
    pub(super) child_style: bool,
    pub(super) disabled: bool,
    pub(super) visible: bool,
    pub(super) nonactivating: bool,
    pub(super) geometry: bool,
}
impl Readback {
    pub(super) fn require(self) -> Result<()> {
        if self.child_style && self.disabled && self.visible && self.nonactivating && self.geometry
        {
            Ok(())
        } else {
            Err(VisibleError::Identity)
        }
    }
}
