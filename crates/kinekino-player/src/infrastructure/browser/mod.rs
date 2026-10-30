mod audio_handoff_ports;
pub use audio_handoff_ports::LegacyAudioHandoffPorts;
mod audio_backend;
mod audio_handoff;
mod events;
mod generation;
mod location;
pub(crate) mod requests;
mod responses;
use crate::{application::session::BrowserEvent, domain::navigation};
use anyhow::Result;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::mpsc::Sender,
};
use wry::{NewWindowResponse, Rect, WebView, WebViewBuilder, WebViewExtWindows};
pub struct Browser {
    audio_silence: Option<audio_handoff::BrowserAudioSilence>,
    view: WebView,
    enabled: Rc<RefCell<bool>>,
    generation: Rc<Cell<u64>>,
    document: Rc<RefCell<String>>,
    visible: Cell<bool>,
}

impl Browser {
    pub fn new(
        parent: &impl wry::raw_window_handle::HasWindowHandle,
        tx: Sender<crate::application::session::BrowserMessage>,
        start_url: &str,
    ) -> Result<Self> {
        let generation = Rc::new(Cell::new(1u64));
        let nav_generation = generation.clone();
        let document = Rc::new(RefCell::new(url::Url::parse(start_url)?.to_string()));
        let nav_document = document.clone();
        let ipc_document = document.clone();
        let tx = events::EventSink::new(tx, generation.clone(), document.clone());
        let popup_tx = tx.clone();
        let nav_tx = tx.clone();
        let load_tx = tx.clone();
        let observation_tx = tx.clone();
        let view = WebViewBuilder::new()
            .with_focused(false)
            .with_visible(start_url != "about:blank")
            .with_initialization_script(include_str!("../../../../../assets/clean-page.js"))
            .with_initialization_script(include_str!("../../../../../assets/media-control.js"))
            .with_initialization_script(include_str!("../../../../../assets/playback-observer.js"))
            .with_initialization_script(include_str!("../../../../../assets/episode-observer.js"))
            .with_ipc_handler(move |request| {
                if request.body().len() > 512_000
                    || !navigation::navigation_allowed(&request.uri().to_string())
                    || request.uri().to_string() != *ipc_document.borrow()
                {
                    return;
                }
                #[derive(serde::Deserialize)]
                #[serde(deny_unknown_fields)]
                struct EpisodeMessage {
                    episodes: crate::domain::episodes::EpisodePage,
                }
                if let Ok(message) = serde_json::from_str::<EpisodeMessage>(request.body()) {
                    if message.episodes.valid()
                        && request.uri().to_string() == message.episodes.page
                    {
                        let _ = observation_tx.send(BrowserEvent::Episodes(message.episodes));
                    }
                    return;
                }
                if request.body() == "{\"input_activity\":true}" {
                    let _ = observation_tx.send(BrowserEvent::InputActivity);
                    return;
                }
                if request.body().len() > 4096 {
                    return;
                }
                if let Ok(observation) =
                    serde_json::from_str::<crate::domain::playback::WebPlayback>(request.body())
                    && observation.valid()
                {
                    let _ = observation_tx.send(BrowserEvent::Playback(observation));
                }
            })
            .with_navigation_handler(move |url| {
                if navigation::navigation_allowed(&url) {
                    if generation::advance(&nav_generation).is_none() {
                        return false;
                    }
                    *nav_document.borrow_mut() = url.clone();
                    let _ = nav_tx.send(BrowserEvent::Navigated(url));
                    true
                } else {
                    false
                }
            })
            .with_new_window_req_handler(move |url, _| {
                if navigation::navigation_allowed(&url) {
                    let _ = popup_tx.send(BrowserEvent::OpenRequested(url));
                } else {
                    let _ = popup_tx.send(BrowserEvent::Blocked);
                }
                NewWindowResponse::Deny
            })
            .with_devtools(false)
            .with_on_page_load_handler(move |event, _| {
                if matches!(event, wry::PageLoadEvent::Finished) {
                    let _ = load_tx.send(BrowserEvent::Loaded);
                }
            })
            .with_bounds(Rect {
                position: wry::dpi::LogicalPosition::new(0., 90.).into(),
                size: wry::dpi::LogicalSize::new(1100., 600.).into(),
            })
            .build_as_child(parent)?;
        let audio_silence =
            audio_backend::WebViewAudioBackend::new(&view.webview(), generation.clone())
                .ok()
                .map(|backend| {
                    Rc::new(RefCell::new(
                        crate::application::audio_handoff::web_silence::WebAudioSilenceOwner::new(
                            backend,
                        ),
                    ))
                });
        let enabled = Rc::new(RefCell::new(true));
        requests::install_filter(&view, tx.clone(), enabled.clone())?;
        responses::observe(&view, tx.clone())?;
        location::observe(&view, tx, generation.clone(), document.clone())?;
        if start_url != "about:blank" {
            anyhow::ensure!(
                navigation::navigation_allowed(start_url),
                "Unsupported start URL"
            );
            view.load_url(start_url)?;
        }
        Ok(Self {
            audio_silence,
            view,
            enabled,
            generation,
            document,
            visible: Cell::new(start_url != "about:blank"),
        })
    }

    pub fn navigate(&self, url: &str) -> Result<()> {
        anyhow::ensure!(
            navigation::navigation_allowed(url),
            "Only the supported HTTPS player sites can be opened"
        );
        let document = url::Url::parse(url)?.to_string();
        anyhow::ensure!(
            generation::advance(&self.generation).is_some(),
            "Browser document generation exhausted; restart the owned host"
        );
        *self.document.borrow_mut() = document;
        self.view.load_url(url)?;
        Ok(())
    }

    pub fn generation(&self) -> u64 {
        self.generation.get()
    }

    pub fn resize(&self, x: f64, y: f64, width: f64, height: f64) -> Result<()> {
        self.view.set_bounds(Rect {
            position: wry::dpi::LogicalPosition::new(x, y).into(),
            size: wry::dpi::LogicalSize::new(width.max(1.), height.max(1.)).into(),
        })?;
        Ok(())
    }

    pub fn set_blocking(&self, enabled: bool) {
        *self.enabled.borrow_mut() = enabled;
    }

    pub fn engine_version() -> Option<String> {
        wry::webview_version().ok()
    }

    pub fn set_visible(&self, visible: bool) -> Result<()> {
        self.view.set_visible(visible)?;
        let was_visible = self.visible.replace(visible);
        if was_visible && !visible {
            self.view.focus_parent()?;
        }
        Ok(())
    }

    pub fn control(&self, action: &str, value: f64) -> Result<()> {
        let message =
            serde_json::json!({"type":"kinekino-player:control", "action":action, "value":value});
        self.view
            .evaluate_script(&format!("window.postMessage({message}, location.origin)"))?;
        Ok(())
    }

    pub fn open_episode_player(&self, expected_url: &str) -> Result<()> {
        let expected = serde_json::to_string(expected_url)?;
        self.view.evaluate_script(&format!("if(location.href==={expected} && location.hostname==='catalog.example.test' && location.pathname.startsWith('/e/')) document.querySelector('button.watch-now')?.click()"))?;
        Ok(())
    }

    pub fn pause_media(&self) -> Result<()> {
        self.view.evaluate_script(
            "window.postMessage({type:'kinekino-player:pause'}, location.origin)",
        )?;
        Ok(())
    }
}
