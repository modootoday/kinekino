//! Process-level CEF lifecycle: sub-process dispatch, initialize, pumping, browser creation.
use super::config::CefLaunch;
use super::handlers::{self, Shared};
use crate::engine::port::{EngineError, EngineResult};
use cef::args::Args;
use cef::{
    App, Browser, BrowserSettings, CefString, ImplCommandLine, Rect, RuntimeStyle, Settings,
    WindowInfo,
};
use std::time::{Duration, Instant};

/// Call first in `main`. Returns `Some(exit_code)` when this process is a CEF sub-process
/// (renderer, GPU, ...) that must exit now; `None` in the browser process.
pub fn subprocess_entry() -> Option<i32> {
    let _ = cef::api_hash(cef::sys::CEF_API_VERSION_LAST, 0);
    let args = Args::new();
    let cmd = args.as_cmd_line()?;
    let is_browser = cmd.has_switch(Some(&CefString::from("type"))) != 1;
    let code = cef::execute_process(
        Some(args.as_main_args()),
        None::<&mut App>,
        std::ptr::null_mut(),
    );
    (!is_browser).then_some(code)
}

pub(super) fn initialize(spec: &CefLaunch, shared: &Shared) -> EngineResult<()> {
    let resources = CefLaunch::resources_dir()?;
    let path = |p: &std::path::Path| CefString::from(p.to_string_lossy().as_ref());
    let settings = Settings {
        // Sandbox needs a setuid chrome-sandbox; tracked as an open item in ADR 0002.
        no_sandbox: 1,
        external_message_pump: 1,
        root_cache_path: path(&spec.cache_dir),
        cache_path: path(&spec.cache_dir.join("default")),
        resources_dir_path: path(&resources),
        locales_dir_path: path(&resources.join("locales")),
        ..Default::default()
    };
    let args = Args::new();
    let mut app = handlers::app(shared.clone());
    let ok = cef::initialize(
        Some(args.as_main_args()),
        Some(&settings),
        Some(&mut app),
        std::ptr::null_mut(),
    );
    if ok != 1 {
        return Err(EngineError::Launch("cef initialize failed".into()));
    }
    if pump_until(Duration::from_secs(10), || shared.context_ready()) {
        Ok(())
    } else {
        Err(EngineError::Launch("cef context never initialized".into()))
    }
}

pub(super) fn create_browser(spec: &CefLaunch, shared: &Shared) -> EngineResult<Browser> {
    let info = WindowInfo {
        parent_window: spec.parent as _,
        bounds: Rect {
            x: 0,
            y: 0,
            width: spec.size.0,
            height: spec.size.1,
        },
        runtime_style: RuntimeStyle::ALLOY,
        ..Default::default()
    };
    let mut client = handlers::client(shared.clone());
    cef::browser_host_create_browser_sync(
        Some(&info),
        Some(&mut client),
        Some(&CefString::from(spec.initial_url.as_str())),
        Some(&BrowserSettings::default()),
        None,
        None,
    )
    .ok_or_else(|| EngineError::Launch("cef browser creation failed".into()))
}

/// Pumps CEF work until `done` holds or `timeout` passes; returns whether it held.
pub(super) fn pump_until(timeout: Duration, mut done: impl FnMut() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        cef::do_message_loop_work();
        if done() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    done()
}
