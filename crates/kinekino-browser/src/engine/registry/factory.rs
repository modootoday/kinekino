//! Per-adapter factories. Each adapter's construction stays inside its own cfg gate.
use super::BrowserBuildConfig;
use crate::engine::port::{BrowserEngine, EngineError, EngineKind, EngineResult};

pub(super) fn build(
    kind: EngineKind,
    cfg: &BrowserBuildConfig,
) -> EngineResult<Box<dyn BrowserEngine>> {
    match kind {
        EngineKind::ChromeCdp => chrome_cdp(cfg),
        EngineKind::SystemChrome => system_chrome(cfg),
        EngineKind::Cef => cef(cfg),
        EngineKind::WebView2 => Err(EngineError::Unsupported(
            "webview2 is created from the player's host window (WebView2Engine::from_view)",
        )),
    }
}

pub(super) fn missing_reason(kind: EngineKind) -> String {
    match kind {
        EngineKind::WebView2 => "needs a Windows build with feature `webview2`".into(),
        EngineKind::Cef => "needs feature `cef` and KINEKINO_CEF_DIR with libcef".into(),
        EngineKind::SystemChrome => {
            "needs Linux, feature `chrome-embed` and Google Chrome (KINEKINO_CHROME_PATH)".into()
        }
        EngineKind::ChromeCdp => {
            "needs feature `cdp` and Google Chrome (KINEKINO_CHROME_PATH)".into()
        }
    }
}

fn parent(cfg: &BrowserBuildConfig) -> EngineResult<u64> {
    cfg.parent
        .ok_or_else(|| EngineError::Launch("embedded engine needs `parent` host window".into()))
}

#[cfg(feature = "cdp")]
fn chrome_cdp(cfg: &BrowserBuildConfig) -> EngineResult<Box<dyn BrowserEngine>> {
    use crate::engine::chromium::cdp::{CdpEngine, CdpLaunch};
    let spec = CdpLaunch {
        headless: cfg.headless,
        profile_dir: cfg.profile_dir.clone(),
        window: cfg.window,
        app_url: cfg.initial_url.clone(),
    };
    Ok(Box::new(CdpEngine::launch(&spec)?))
}

#[cfg(not(feature = "cdp"))]
fn chrome_cdp(_: &BrowserBuildConfig) -> EngineResult<Box<dyn BrowserEngine>> {
    Err(EngineError::Launch("built without feature `cdp`".into()))
}

#[cfg(all(target_os = "linux", feature = "chrome-embed"))]
fn system_chrome(cfg: &BrowserBuildConfig) -> EngineResult<Box<dyn BrowserEngine>> {
    use crate::engine::chromium::chrome_embed::{ChromeEmbedLaunch, SystemChromeEngine};
    let parent = u32::try_from(parent(cfg)?)
        .map_err(|_| EngineError::Launch("X11 parent window id exceeds 32 bits".into()))?;
    let spec = ChromeEmbedLaunch {
        parent,
        size: cfg.window,
        profile_dir: cfg.profile_dir.clone(),
    };
    Ok(Box::new(SystemChromeEngine::launch(&spec)?))
}

#[cfg(not(all(target_os = "linux", feature = "chrome-embed")))]
fn system_chrome(cfg: &BrowserBuildConfig) -> EngineResult<Box<dyn BrowserEngine>> {
    parent(cfg)?;
    Err(EngineError::Launch(
        "built without Linux feature `chrome-embed`".into(),
    ))
}

#[cfg(feature = "cef")]
fn cef(cfg: &BrowserBuildConfig) -> EngineResult<Box<dyn BrowserEngine>> {
    use crate::engine::chromium::cef::{CefEngine, CefLaunch};
    let size = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
    let spec = CefLaunch {
        parent: parent(cfg)?,
        size: (size(cfg.window.0), size(cfg.window.1)),
        cache_dir: cfg.profile_dir.clone(),
        initial_url: cfg
            .initial_url
            .clone()
            .unwrap_or_else(|| "about:blank".into()),
    };
    Ok(Box::new(CefEngine::launch(&spec)?))
}

#[cfg(not(feature = "cef"))]
fn cef(cfg: &BrowserBuildConfig) -> EngineResult<Box<dyn BrowserEngine>> {
    parent(cfg)?;
    Err(EngineError::Launch("built without feature `cef`".into()))
}
