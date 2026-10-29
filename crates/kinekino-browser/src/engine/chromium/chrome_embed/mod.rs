//! Installed Chrome embed adapter (Linux/X11): Chrome in `--app` mode is reparented into the
//! host window with x11rb, and CDP ([`CdpEngine`]) stays the control channel.
use crate::engine::chromium::cdp::{CdpEngine, CdpLaunch};
use crate::engine::embed::x11::X11Host;
use crate::engine::port::{
    AudioControl, BrowserEngine, BrowserEvent, EngineError, EngineKind, EngineResult, FrameCapture,
    NetworkInterceptor,
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ChromeEmbedLaunch {
    /// Host X11 window (XID) the Chrome window becomes a child of.
    pub parent: u32,
    pub size: (u32, u32),
    pub profile_dir: PathBuf,
}

pub struct SystemChromeEngine {
    cdp: CdpEngine,
    host: X11Host,
    child: u32,
}

impl SystemChromeEngine {
    pub fn launch(spec: &ChromeEmbedLaunch) -> EngineResult<Self> {
        // Unique title lets us find exactly this Chrome's top-level window.
        let tag = format!("kinekino-embed-{}", std::process::id());
        let mut cdp = CdpEngine::launch(&CdpLaunch {
            headless: false,
            profile_dir: spec.profile_dir.clone(),
            window: spec.size,
            // about: URLs are not honored by --app (Chrome opens a tabbed window instead).
            app_url: Some("data:text/html,".into()),
        })?;
        // The --app window title follows document.title.
        cdp.eval(&format!("document.title = '{tag}'"))?;
        let host = X11Host::attach(spec.parent)?;
        let child = host
            .find_by_title(&tag)
            .ok_or_else(|| EngineError::Launch("chrome --app window not found".into()))?;
        host.embed(child)?;
        host.resize(spec.size.0, spec.size.1)?;
        Ok(Self { cdp, host, child })
    }

    /// Follows a host resize (Chrome's viewport tracks its window; see headed `viewport(None)`).
    pub fn resize(&self, w: u32, h: u32) -> EngineResult<()> {
        self.host.resize(w, h)
    }

    pub fn focus(&self) -> EngineResult<()> {
        self.host.focus(self.child)
    }

    /// Screen offset between the host window and the embedded Chrome window. (0, 0) means
    /// clicks on the host land at identical page coordinates (no title-bar/frame offset).
    pub fn click_offset(&self) -> (i32, i32) {
        let (hx, hy) = self.host.abs_origin(self.host.window);
        let (cx, cy) = self.host.abs_origin(self.child);
        (cx - hx, cy - hy)
    }

    pub fn close(self) -> EngineResult<()> {
        self.cdp.close()
    }
}

impl BrowserEngine for SystemChromeEngine {
    fn kind(&self) -> EngineKind {
        EngineKind::SystemChrome
    }
    fn navigate(&mut self, url: &str) -> EngineResult<()> {
        self.cdp.navigate(url)
    }
    fn eval(&mut self, script: &str) -> EngineResult<serde_json::Value> {
        self.cdp.eval(script)
    }
    fn inject_script(&mut self, source: &str) -> EngineResult<()> {
        self.cdp.inject_script(source)
    }
    fn set_interceptor(&mut self, i: Box<dyn NetworkInterceptor>) -> EngineResult<()> {
        self.cdp.set_interceptor(i)
    }
    fn drain_events(&mut self) -> Vec<BrowserEvent> {
        self.cdp.drain_events()
    }
    fn audio(&mut self) -> Option<&mut dyn AudioControl> {
        Some(&mut self.cdp)
    }
    fn frame_capture(&mut self) -> Option<&mut dyn FrameCapture> {
        Some(&mut self.cdp)
    }
}
