//! Minimal libmpv C ABI (client.h / render.h, API 2.x) resolved with `dlopen`.
use std::ffi::{c_char, c_int, c_void};

pub type Handle = *mut c_void;
pub type RenderCtx = *mut c_void;

#[repr(C)]
pub struct RenderParam {
    pub kind: c_int,
    pub data: *mut c_void,
}

pub const PARAM_INVALID: c_int = 0;
pub const PARAM_API_TYPE: c_int = 1;
pub const PARAM_SW_SIZE: c_int = 17;
pub const PARAM_SW_FORMAT: c_int = 18;
pub const PARAM_SW_STRIDE: c_int = 19;
pub const PARAM_SW_POINTER: c_int = 20;
pub const UPDATE_FRAME: u64 = 1;
pub const FORMAT_STRING: c_int = 1;
pub const FORMAT_FLAG: c_int = 3;
pub const FORMAT_INT64: c_int = 4;
pub const FORMAT_DOUBLE: c_int = 5;
/// `MPV_ERROR_PROPERTY_FORMAT`: the property exists but not in the asked format.
pub const ERROR_PROPERTY_FORMAT: c_int = -9;
pub const EVENT_NONE: c_int = 0;
pub const EVENT_START_FILE: c_int = 6;
pub const EVENT_END_FILE: c_int = 7;
pub const EVENT_PLAYBACK_RESTART: c_int = 21;
/// `mpv_end_file_reason`: the old file was replaced (`loadfile replace`).
pub const END_FILE_REASON_STOP: c_int = 2;
/// `mpv_end_file_reason`: the file redirected to another (playlist, ytdl).
pub const END_FILE_REASON_REDIRECT: c_int = 5;

/// Leading fields of `mpv_event_end_file`.
#[repr(C)]
pub struct EndFile {
    pub reason: c_int,
    pub error: c_int,
    pub playlist_entry_id: i64,
}

/// `mpv_event_start_file`.
#[repr(C)]
pub struct StartFile {
    pub playlist_entry_id: i64,
}

#[repr(C)]
pub struct Event {
    pub event_id: c_int,
    pub error: c_int,
    pub reply_userdata: u64,
    pub data: *mut c_void,
}

pub struct Api {
    /// Never unloaded: libmpv/ffmpeg leave thread-local destructors and helper threads
    /// (audio, decoder pools) that may still run after `mpv_terminate_destroy`, so
    /// `dlclose` there can jump into unmapped code (intermittent SIGSEGV at teardown).
    _lib: std::mem::ManuallyDrop<libloading::Library>,
    pub create: unsafe extern "C" fn() -> Handle,
    pub initialize: unsafe extern "C" fn(Handle) -> c_int,
    pub terminate_destroy: unsafe extern "C" fn(Handle),
    pub set_option_string: unsafe extern "C" fn(Handle, *const c_char, *const c_char) -> c_int,
    pub command: unsafe extern "C" fn(Handle, *mut *const c_char) -> c_int,
    pub get_property: unsafe extern "C" fn(Handle, *const c_char, c_int, *mut c_void) -> c_int,
    pub free: unsafe extern "C" fn(*mut c_void),
    pub wait_event: unsafe extern "C" fn(Handle, f64) -> *mut Event,
    pub error_string: unsafe extern "C" fn(c_int) -> *const c_char,
    pub render_create: unsafe extern "C" fn(*mut RenderCtx, Handle, *mut RenderParam) -> c_int,
    pub render_update: unsafe extern "C" fn(RenderCtx) -> u64,
    pub render_render: unsafe extern "C" fn(RenderCtx, *mut RenderParam) -> c_int,
    pub render_free: unsafe extern "C" fn(RenderCtx),
}

/// Windows `LoadLibraryExW` flags for `path`: search the DLL's own folder, then the default
/// folders, so dependencies shipped beside `libmpv-2.dll` (e.g. `vulkan-1.dll`) load. The
/// DLL-folder search only accepts a fully qualified path, so a bare name (the default
/// `libmpv-2.dll`) keeps the standard search order (flags 0). Pure, so it is tested on every OS.
#[cfg_attr(
    all(not(windows), not(test)),
    expect(
        dead_code,
        reason = "only the Windows loader calls it; tests cover it everywhere"
    )
)]
pub(crate) fn windows_load_flags(path: &std::path::Path) -> u32 {
    // Values from libloaderapi.h.
    const LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR: u32 = 0x0000_0100;
    const LOAD_LIBRARY_SEARCH_DEFAULT_DIRS: u32 = 0x0000_1000;
    if path.is_absolute() {
        LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS
    } else {
        0
    }
}

impl Api {
    /// # Safety
    /// `path` must name a genuine libmpv (API 2.x); its initialisers run on load.
    pub unsafe fn load(path: &std::path::Path) -> Result<Self, libloading::Error> {
        // SAFETY: caller guarantees the library is libmpv; symbol types match client.h/render.h.
        unsafe {
            #[cfg(windows)]
            let lib: libloading::Library =
                libloading::os::windows::Library::load_with_flags(path, windows_load_flags(path))?
                    .into();
            #[cfg(not(windows))]
            let lib = libloading::Library::new(path)?;
            macro_rules! sym {
                ($n:literal) => {
                    *lib.get($n)?
                };
            }
            Ok(Self {
                create: sym!(b"mpv_create\0"),
                initialize: sym!(b"mpv_initialize\0"),
                terminate_destroy: sym!(b"mpv_terminate_destroy\0"),
                set_option_string: sym!(b"mpv_set_option_string\0"),
                command: sym!(b"mpv_command\0"),
                get_property: sym!(b"mpv_get_property\0"),
                free: sym!(b"mpv_free\0"),
                wait_event: sym!(b"mpv_wait_event\0"),
                error_string: sym!(b"mpv_error_string\0"),
                render_create: sym!(b"mpv_render_context_create\0"),
                render_update: sym!(b"mpv_render_context_update\0"),
                render_render: sym!(b"mpv_render_context_render\0"),
                render_free: sym!(b"mpv_render_context_free\0"),
                _lib: std::mem::ManuallyDrop::new(lib),
            })
        }
    }
}
