//! libmpv software render adapter: implements [`VideoSurface`] by asking libmpv to draw RGBA
//! into a caller buffer that the Meari UI uploads as a texture. libmpv is loaded at runtime
//! (`$KINEKINO_LIBMPV`, else the platform soname) and never bundled; see
//! `.spec/guide/mpv-install.guide.md` for the pinned install and licence notes.
mod ffi;

use kinekino_core::playback::surface::check_target;
use kinekino_core::playback::{RgbaFrame, VideoSurface};
use kinekino_core::port::error::{ErrorCode, PortError, PortResult};
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[cfg(test)]
mod tests;

/// Platform soname chosen at runtime (no compile-time platform cfg outside adapter crates).
pub fn default_library() -> PathBuf {
    if let Some(p) = std::env::var_os("KINEKINO_LIBMPV").filter(|p| !p.is_empty()) {
        return p.into();
    }
    match std::env::consts::OS {
        "windows" => "libmpv-2.dll".into(),
        "macos" => "libmpv.2.dylib".into(),
        _ => "libmpv.so.2".into(),
    }
}

pub struct LibMpvSurface {
    api: ffi::Api,
    handle: ffi::Handle,
    render: ffi::RenderCtx,
    /// A decoded frame was reported by libmpv and not yet rendered.
    pending: bool,
}

fn c(s: &str) -> PortResult<CString> {
    CString::new(s).map_err(|_| PortError::new(ErrorCode::InvalidInput, "NUL in string"))
}

impl LibMpvSurface {
    /// Create a headless-capable player: `vo=libmpv`, `ao=null` when `mute_audio`.
    pub fn new(library: Option<PathBuf>, mute_audio: bool) -> PortResult<Self> {
        let path = library.unwrap_or_else(default_library);
        // SAFETY: we load the library the user configured as libmpv.
        let api = unsafe { ffi::Api::load(&path) }.map_err(|e| {
            PortError::new(
                ErrorCode::Unavailable,
                format!("libmpv {}: {e}", path.display()),
            )
        })?;
        // SAFETY: plain constructor; null checked.
        let handle = unsafe { (api.create)() };
        if handle.is_null() {
            return Err(PortError::new(ErrorCode::Unavailable, "mpv_create failed"));
        }
        let mut this = Self {
            api,
            handle,
            render: std::ptr::null_mut(),
            pending: false,
        };
        for (k, v) in [
            ("vo", "libmpv"),
            ("config", "no"),
            ("terminal", "no"),
            ("idle", "yes"),
            ("pause", "yes"),
            ("keep-open", "yes"),
        ] {
            this.option(k, v)?;
        }
        if mute_audio {
            this.option("ao", "null")?;
        }
        // SAFETY: handle is live and not yet initialised.
        this.check(
            unsafe { (this.api.initialize)(this.handle) },
            "mpv_initialize",
        )?;
        let mut sw = *b"sw\0";
        let mut params = [
            ffi::RenderParam {
                kind: ffi::PARAM_API_TYPE,
                data: sw.as_mut_ptr().cast(),
            },
            ffi::RenderParam {
                kind: ffi::PARAM_INVALID,
                data: std::ptr::null_mut(),
            },
        ];
        // SAFETY: params are valid for the duration of the call and terminated by INVALID.
        let rc =
            unsafe { (this.api.render_create)(&mut this.render, this.handle, params.as_mut_ptr()) };
        this.check(rc, "mpv_render_context_create(sw)")?;
        Ok(this)
    }

    fn check(&self, rc: c_int, what: &str) -> PortResult<()> {
        if rc >= 0 {
            return Ok(());
        }
        // SAFETY: mpv_error_string returns a static string for any code.
        let msg = unsafe { CStr::from_ptr((self.api.error_string)(rc)) }.to_string_lossy();
        Err(PortError::new(
            ErrorCode::Internal,
            format!("{what}: {msg}"),
        ))
    }

    fn option(&mut self, k: &str, v: &str) -> PortResult<()> {
        let (k2, v2) = (c(k)?, c(v)?);
        // SAFETY: valid NUL-terminated strings, live handle.
        let rc = unsafe { (self.api.set_option_string)(self.handle, k2.as_ptr(), v2.as_ptr()) };
        self.check(rc, &format!("option {k}"))
    }

    pub fn command(&mut self, args: &[&str]) -> PortResult<()> {
        let owned = args.iter().map(|a| c(a)).collect::<PortResult<Vec<_>>>()?;
        let mut ptrs: Vec<*const c_char> = owned.iter().map(|s| s.as_ptr()).collect();
        ptrs.push(std::ptr::null());
        // SAFETY: NULL-terminated array of live C strings.
        let rc = unsafe { (self.api.command)(self.handle, ptrs.as_mut_ptr()) };
        self.check(rc, &format!("command {}", args.first().unwrap_or(&"")))
    }

    pub fn property_f64(&mut self, name: &str) -> Option<f64> {
        let n = c(name).ok()?;
        let mut v = 0f64;
        // SAFETY: FORMAT_DOUBLE writes one f64 into `v`.
        let rc = unsafe {
            (self.api.get_property)(
                self.handle,
                n.as_ptr(),
                ffi::FORMAT_DOUBLE,
                (&mut v as *mut f64).cast::<c_void>(),
            )
        };
        (rc >= 0).then_some(v)
    }

    /// Drain events until `want` arrives; END_FILE first means the load failed.
    fn wait_for(&mut self, want: c_int, timeout: Duration) -> PortResult<()> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            // SAFETY: returned event is owned by mpv and valid until the next wait call.
            let ev = unsafe { &*(self.api.wait_event)(self.handle, 0.05) };
            match ev.event_id {
                id if id == want => return Ok(()),
                ffi::EVENT_END_FILE if want == ffi::EVENT_PLAYBACK_RESTART => {
                    return Err(PortError::new(
                        ErrorCode::InvalidInput,
                        "libmpv could not load the source",
                    ));
                }
                _ => {}
            }
        }
        Err(PortError::new(ErrorCode::Timeout, "libmpv event timeout"))
    }

    /// Load `source` paused at `start_secs` and wait until its first frame is decodable.
    pub fn open(&mut self, source: &str, start_secs: f64) -> PortResult<()> {
        let opts = format!("start={start_secs},pause=yes");
        self.command(&["loadfile", source, "replace", "-1", &opts])?;
        self.wait_for(ffi::EVENT_PLAYBACK_RESTART, Duration::from_secs(10))?;
        self.settle()
    }

    /// Waits until libmpv reports the decoded frame so later readiness is not stale.
    fn settle(&mut self) -> PortResult<()> {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !self.poll_update() {
            if std::time::Instant::now() > deadline {
                return Err(PortError::new(
                    ErrorCode::Timeout,
                    "libmpv produced no frame",
                ));
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    }

    fn poll_update(&mut self) -> bool {
        // SAFETY: live render context.
        if unsafe { (self.api.render_update)(self.render) } & ffi::UPDATE_FRAME != 0 {
            self.pending = true;
        }
        self.pending
    }

    pub fn seek(&mut self, secs: f64) -> PortResult<()> {
        self.command(&["seek", &secs.to_string(), "absolute+exact"])?;
        self.wait_for(ffi::EVENT_PLAYBACK_RESTART, Duration::from_secs(10))?;
        self.settle()
    }
}

impl VideoSurface for LibMpvSurface {
    fn frame_ready(&mut self) -> PortResult<bool> {
        Ok(self.poll_update())
    }

    fn render(&mut self, target: &mut RgbaFrame) -> PortResult<()> {
        check_target(target)?;
        // Drain pending update flags so `frame_ready` reports only frames newer than this one.
        self.poll_update();
        self.pending = false;
        let mut size = [target.width as c_int, target.height as c_int];
        let mut format = *b"rgb0\0";
        let mut stride = target.width as usize * 4;
        let mut params = [
            ffi::RenderParam {
                kind: ffi::PARAM_SW_SIZE,
                data: size.as_mut_ptr().cast(),
            },
            ffi::RenderParam {
                kind: ffi::PARAM_SW_FORMAT,
                data: format.as_mut_ptr().cast(),
            },
            ffi::RenderParam {
                kind: ffi::PARAM_SW_STRIDE,
                data: (&mut stride as *mut usize).cast(),
            },
            ffi::RenderParam {
                kind: ffi::PARAM_SW_POINTER,
                data: target.pixels.as_mut_ptr().cast(),
            },
            ffi::RenderParam {
                kind: ffi::PARAM_INVALID,
                data: std::ptr::null_mut(),
            },
        ];
        // SAFETY: buffer is width*height*4 bytes (checked), stride matches, list terminated.
        let rc = unsafe { (self.api.render_render)(self.render, params.as_mut_ptr()) };
        self.check(rc, "mpv_render_context_render")?;
        // rgb0 leaves the 4th byte undefined; the UI texture is RGBA, so make it opaque.
        for px in target.pixels.as_chunks_mut::<4>().0 {
            px[3] = 255;
        }
        Ok(())
    }
}

impl Drop for LibMpvSurface {
    fn drop(&mut self) {
        // SAFETY: render context must be freed before the core is destroyed (render.h).
        unsafe {
            if !self.render.is_null() {
                (self.api.render_free)(self.render);
            }
            (self.api.terminate_destroy)(self.handle);
        }
    }
}
