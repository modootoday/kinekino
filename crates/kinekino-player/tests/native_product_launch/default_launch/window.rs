use super::{Failure, Stage};
use kinekino_process::domain::identity::ProcessIdentity;
use serde_json::{Value, json};
use std::path::Path;
use windows::{
    Win32::{
        Foundation::{GetLastError, HWND, LPARAM, SetLastError, WIN32_ERROR, WPARAM},
        UI::WindowsAndMessaging::{
            EnumWindows, GW_OWNER, GWL_EXSTYLE, GWL_STYLE, GetClassNameW, GetParent, GetWindow,
            GetWindowLongPtrW, GetWindowTextW, GetWindowThreadProcessId, IsWindow, IsWindowVisible,
            PostMessageW, WINDOW_LONG_PTR_INDEX, WM_CLOSE, WS_CAPTION, WS_CHILD, WS_EX_APPWINDOW,
            WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP, WS_SYSMENU,
        },
    },
    core::BOOL,
};

pub(super) fn check_identity(
    identity: &ProcessIdentity,
    expected: &Path,
    exit: Option<u32>,
) -> Result<(), Failure> {
    if identity.creation_filetime.is_none()
        || identity.pid == 0
        || exit.is_some()
        || identity.executable != expected
    {
        return Err(Failure::new(
            Stage::Identity,
            "Original process is not live or its image identity disagrees",
        ));
    }
    Ok(())
}
// Reviewed winit-win32 0.31.0-beta.3 defaults used by the product's default WindowOptions.
const APPLICATION_CLASS: &str = "Window Class";
const INTERNAL_TARGET_CLASS: &str = "Winit Thread Event Target";
const ROLE_STYLE_MASK: u32 = WS_CHILD.0 | WS_POPUP.0 | WS_CAPTION.0 | WS_SYSMENU.0;
const ROLE_EX_STYLE_MASK: u32 = WS_EX_APPWINDOW.0 | WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0;

#[derive(Clone, Debug)]
pub(super) struct Identity {
    pub(super) handle: usize,
    pub(super) pid: u32,
    pub(super) thread: u32,
    pub(super) visible: bool,
    pub(super) top_level: bool,
    pub(super) class: String,
    pub(super) title: String,
    pub(super) parent: usize,
    pub(super) owner: usize,
    pub(super) style: u32,
    pub(super) ex_style: u32,
}
pub(super) fn accepts(identity: &Identity, expected: u32) -> bool {
    identity.handle != 0
        && identity.pid == expected
        && expected != 0
        && identity.thread != 0
        && identity.visible
        && identity.top_level
        && identity.parent == 0
        && identity.owner == 0
        && identity.class != INTERNAL_TARGET_CLASS
        && identity.class == APPLICATION_CLASS
        && identity.title == meari_native::application::WindowOptions::default().title
        && identity.style & ROLE_STYLE_MASK == (WS_CAPTION.0 | WS_SYSMENU.0)
        && identity.ex_style & ROLE_EX_STYLE_MASK == WS_EX_APPWINDOW.0
}
pub(super) struct Window {
    identity: Identity,
}
fn window_style(handle: HWND, index: WINDOW_LONG_PTR_INDEX) -> Result<u32, Failure> {
    // SAFETY: bounded HWND query; zero is valid only when the API leaves no error.
    unsafe {
        SetLastError(WIN32_ERROR(0));
        let value = GetWindowLongPtrW(handle, index);
        let error = GetLastError();
        if value == 0 && error.0 != 0 {
            return Err(Failure::new(
                Stage::Window,
                std::io::Error::from_raw_os_error(error.0 as i32),
            ));
        }
        Ok(value as u32)
    }
}
fn identity(handle: HWND) -> Result<Identity, Failure> {
    let mut pid = 0;
    let mut class = [0u16; 256];
    let mut title = [0u16; 512];
    // SAFETY: read-only queries of an enumerated HWND, with live bounded output buffers.
    unsafe {
        let thread = GetWindowThreadProcessId(handle, Some(&mut pid));
        if thread == 0 {
            return Err(Failure::new(Stage::Window, std::io::Error::last_os_error()));
        }
        let count = GetClassNameW(handle, &mut class);
        if count == 0 {
            return Err(Failure::new(Stage::Window, std::io::Error::last_os_error()));
        }
        let title_count = GetWindowTextW(handle, &mut title);
        let parent = GetParent(handle).map_or(0, |value| value.0 as usize);
        let owner = GetWindow(handle, GW_OWNER).map_or(0, |value| value.0 as usize);
        let style = window_style(handle, GWL_STYLE)?;
        let ex_style = window_style(handle, GWL_EXSTYLE)?;
        Ok(Identity {
            handle: handle.0 as usize,
            pid,
            thread,
            visible: IsWindowVisible(handle).as_bool(),
            top_level: parent == 0 && owner == 0 && style & WS_CHILD.0 == 0,
            class: String::from_utf16_lossy(&class[..count.max(0) as usize]),
            title: String::from_utf16_lossy(&title[..title_count.max(0) as usize]),
            parent,
            owner,
            style,
            ex_style,
        })
    }
}
struct Query {
    pid: u32,
    windows: Vec<Identity>,
    error: Option<Failure>,
}
pub(super) fn retain_first_query_error(first: &mut Option<Failure>, error: Failure) {
    if first.is_none() {
        *first = Some(error);
    }
}
unsafe extern "system" fn collect(handle: HWND, parameter: LPARAM) -> BOOL {
    // SAFETY: synchronous EnumWindows retains this exclusively borrowed stack object.
    let query = unsafe { &mut *(parameter.0 as *mut Query) };
    let mut pid = 0;
    // SAFETY: valid writable PID output; a destroyed foreign HWND is simply not selected.
    unsafe {
        GetWindowThreadProcessId(handle, Some(&mut pid));
    }
    if pid == query.pid {
        match identity(handle) {
            Ok(window) if accepts(&window, query.pid) => query.windows.push(window),
            Ok(_) => {}
            Err(error) => retain_first_query_error(&mut query.error, error),
        }
    }
    true.into()
}
pub(super) fn find(pid: u32) -> Result<Option<Window>, Failure> {
    let mut query = Query {
        pid,
        windows: Vec::new(),
        error: None,
    };
    // SAFETY: query and callback remain live for this synchronous enumeration.
    unsafe { EnumWindows(Some(collect), LPARAM((&mut query as *mut Query) as isize)) }
        .map_err(|e| Failure::new(Stage::Window, e))?;
    if let Some(error) = query.error {
        return Err(error);
    }
    select(query.windows, pid).map(|selected| selected.map(|identity| Window { identity }))
}
pub(super) fn select(
    windows: impl IntoIterator<Item = Identity>,
    pid: u32,
) -> Result<Option<Identity>, Failure> {
    let mut eligible = windows.into_iter().filter(|window| accepts(window, pid));
    let first = eligible.next();
    if eligible.next().is_some() {
        return Err(Failure::new(
            Stage::Window,
            "Multiple owned visible root windows",
        ));
    }
    Ok(first)
}
pub(super) fn check_close_target(
    original: &Identity,
    current: Result<Identity, Failure>,
    pid: u32,
) -> Result<(), Failure> {
    let current = current?;
    if !accepts(&current, pid)
        || !accepts(original, pid)
        || current.handle != original.handle
        || current.pid != original.pid
        || current.thread != original.thread
        || current.class != original.class
        || current.title != original.title
        || current.parent != original.parent
        || current.owner != original.owner
        || current.style & ROLE_STYLE_MASK != original.style & ROLE_STYLE_MASK
        || current.ex_style & ROLE_EX_STYLE_MASK != original.ex_style & ROLE_EX_STYLE_MASK
    {
        return Err(Failure::new(Stage::Close, "Original HWND identity changed"));
    }
    Ok(())
}
impl Window {
    fn handle(&self) -> HWND {
        HWND(self.identity.handle as *mut _)
    }
    pub(super) fn evidence(&self) -> Value {
        json!({"kind":"owned-window-observation","hwnd":self.identity.handle,"pid":self.identity.pid,
            "thread":self.identity.thread,"class":self.identity.class,"title":self.identity.title,
            "visible":self.identity.visible,"top_level":self.identity.top_level,
            "parent":self.identity.parent,"owner":self.identity.owner,
            "style":self.identity.style,"ex_style":self.identity.ex_style,
            "renderer_frame_observed":false})
    }
    pub(super) fn still_present(&self) -> bool {
        // SAFETY: IsWindow is only an observation; no ownership is inferred from the handle alone.
        unsafe { IsWindow(Some(self.handle())).as_bool() }
    }
    pub(super) fn close(&self, pid: u32) -> Result<(), Failure> {
        check_close_target(&self.identity, identity(self.handle()), pid)?;
        // SAFETY: caller just polled its original process handle; HWND/PID/thread/class were rechecked.
        unsafe { PostMessageW(Some(self.handle()), WM_CLOSE, WPARAM(0), LPARAM(0)) }
            .map_err(|e| Failure::new(Stage::Close, e))
    }
}
