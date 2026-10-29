//! CDP adapter: drives installed Google Chrome (codecs + Widevine) via chromiumoxide.
mod intercept;
mod launch;

pub use launch::CdpLaunch;

use crate::engine::port::{
    AudioControl, BrowserEngine, BrowserEvent, EngineError, EngineKind, EngineResult, FrameCapture,
    NetworkInterceptor,
};
use chromiumoxide::page::ScreenshotParams;
use chromiumoxide::{Browser, Page};
use futures::StreamExt;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

/// One Chrome process with one page, exposed through the sync port traits.
pub struct CdpEngine {
    runtime: Runtime,
    browser: Browser,
    page: Page,
    events: Arc<Mutex<Vec<BrowserEvent>>>,
    muted: bool,
    /// Driving a pre-existing (`--app`) target: chromiumoxide gets no lifecycle events for it,
    /// so `Page.navigate` never resolves and navigation is done and awaited from the page.
    attached: bool,
}

fn call<E: std::fmt::Display>(e: E) -> EngineError {
    EngineError::Call(e.to_string())
}

impl CdpEngine {
    pub fn launch(spec: &CdpLaunch) -> EngineResult<Self> {
        let runtime = Runtime::new().map_err(|e| EngineError::Launch(e.to_string()))?;
        let config = spec.config()?;
        let (browser, page) = runtime.block_on(async {
            let (mut browser, mut handler) = Browser::launch(config)
                .await
                .map_err(|e| EngineError::Launch(e.to_string()))?;
            tokio::spawn(async move { while handler.next().await.is_some() {} });
            let page = match &spec.app_url {
                // `--app` opens its own window/page; drive that one instead of a new tab.
                Some(_) => app_page(&mut browser).await?,
                None => browser.new_page("about:blank").await.map_err(call)?,
            };
            Ok::<_, EngineError>((browser, page))
        })?;
        Ok(Self {
            runtime,
            browser,
            page,
            events: Arc::default(),
            muted: false,
            attached: spec.app_url.is_some(),
        })
    }

    /// Closes the browser process.
    pub fn close(mut self) -> EngineResult<()> {
        self.runtime.block_on(self.browser.close()).map_err(call)?;
        let _ = self.runtime.block_on(self.browser.wait());
        Ok(())
    }
}

async fn app_page(browser: &mut Browser) -> EngineResult<Page> {
    let mut seen = Vec::new();
    for _ in 0..50 {
        // Attaches targets that existed before the CDP connection (the --app window).
        let targets = browser.fetch_targets().await.map_err(call)?;
        if let Some(t) = targets.iter().find(|t| t.r#type == "page") {
            for _ in 0..50 {
                if let Ok(page) = browser.get_page(t.target_id.clone()).await {
                    // Init scripts only fire once the Page domain is enabled on this session.
                    page.execute(
                        chromiumoxide::cdp::browser_protocol::page::EnableParams::default(),
                    )
                    .await
                    .map_err(call)?;
                    return Ok(page);
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
        seen = targets
            .into_iter()
            .map(|t| format!("{}:{}", t.r#type, t.url))
            .collect();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    Err(EngineError::Launch(format!(
        "chrome --app page never attached; targets={seen:?}"
    )))
}

impl BrowserEngine for CdpEngine {
    fn kind(&self) -> EngineKind {
        EngineKind::ChromeCdp
    }

    fn navigate(&mut self, url: &str) -> EngineResult<()> {
        self.push(BrowserEvent::NavigationStarted { url: url.into() });
        if self.attached {
            self.navigate_in_page(url)?;
        } else {
            self.runtime.block_on(async {
                self.page.goto(url).await.map_err(call)?;
                self.page.wait_for_navigation().await.map_err(call)?;
                Ok::<_, EngineError>(())
            })?;
        }
        self.push(BrowserEvent::Loaded { url: url.into() });
        Ok(())
    }

    fn eval(&mut self, script: &str) -> EngineResult<serde_json::Value> {
        let result = self
            .runtime
            .block_on(self.page.evaluate(script))
            .map_err(call)?;
        Ok(result.value().cloned().unwrap_or(serde_json::Value::Null))
    }

    fn inject_script(&mut self, source: &str) -> EngineResult<()> {
        self.runtime
            .block_on(self.page.evaluate_on_new_document(source))
            .map_err(call)?;
        Ok(())
    }

    fn set_interceptor(&mut self, interceptor: Box<dyn NetworkInterceptor>) -> EngineResult<()> {
        let page = self.page.clone();
        let events = self.events.clone();
        self.runtime
            .block_on(intercept::install(page, interceptor.into(), events))
    }

    fn drain_events(&mut self) -> Vec<BrowserEvent> {
        std::mem::take(&mut *self.events.lock().unwrap_or_else(|p| p.into_inner()))
    }

    fn audio(&mut self) -> Option<&mut dyn AudioControl> {
        Some(self)
    }

    fn frame_capture(&mut self) -> Option<&mut dyn FrameCapture> {
        Some(self)
    }
}

impl CdpEngine {
    fn navigate_in_page(&mut self, url: &str) -> EngineResult<()> {
        let target = serde_json::to_string(url).map_err(call)?;
        let marker = "window.__kinekinoNavFrom";
        self.eval(&format!("{marker}=1;location.href={target};true"))?;
        for _ in 0..300 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            // Errors are expected while the old document is torn down.
            let done = self.eval(&format!(
                "!{marker} && document.readyState === 'complete' && location.href === new URL({target}).href"
            ));
            if matches!(done, Ok(serde_json::Value::Bool(true))) {
                return Ok(());
            }
        }
        Err(EngineError::Call(format!("navigation to {url} timed out")))
    }

    fn push(&self, event: BrowserEvent) {
        self.events
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(event);
    }
}

/// CDP has no per-page mute; mutes every media element and new ones via an init script.
impl AudioControl for CdpEngine {
    fn set_muted(&mut self, muted: bool) -> EngineResult<()> {
        let js = format!(
            "window.__kinekinoMuted={muted};document.querySelectorAll('audio,video').forEach(m=>m.muted={muted});true"
        );
        self.eval(&js)?;
        self.muted = muted;
        Ok(())
    }

    fn is_muted(&mut self) -> EngineResult<bool> {
        Ok(self.muted)
    }
}

impl FrameCapture for CdpEngine {
    fn capture_png(&mut self) -> EngineResult<Vec<u8>> {
        let params = ScreenshotParams::builder().build();
        self.runtime
            .block_on(self.page.screenshot(params))
            .map_err(call)
    }
}
