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
pub const FORMAT_DOUBLE: c_int = 5;
pub const EVENT_END_FILE: c_int = 7;
pub const EVENT_PLAYBACK_RESTART: c_int = 21;

#[repr(C)]
pub struct Event {
    pub event_id: c_int,
    pub error: c_int,
    pub reply_userdata: u64,
    pub data: *mut c_void,
}

pub struct Api {
    _lib: libloading::Library,
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

impl Api {
    /// # Safety
    /// `path` must name a genuine libmpv (API 2.x); its initialisers run on load.
    pub unsafe fn load(path: &std::path::Path) -> Result<Self, libloading::Error> {
        // SAFETY: caller guarantees the library is libmpv; symbol types match client.h/render.h.
        unsafe {
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
                _lib: lib,
            })
        }
    }
}
