#[path = "chrome.rs"]
mod chrome;
#[path = "episode_panel.rs"]
mod episode_panel;
#[path = "native_audio.rs"]
mod native_audio;
#[path = "native_controls.rs"]
mod native_controls;
#[path = "studio/mod.rs"]
pub(super) mod studio;
#[path = "title_bar.rs"]
mod title_bar;
#[path = "transport.rs"]
mod transport;
#[path = "workspace.rs"]
mod workspace;
use crate::{
    application::session::{BrowserEvent, Session},
    infrastructure::{browser::Browser, diagnostics, mpv::Mpv},
};
use eframe::egui;
use serde_json::json;
use std::{
    cell::Cell,
    rc::Rc,
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};

pub struct PlayerApp {
    mpv: Option<Mpv>,
    native_audio: Option<crate::application::audio_handoff::coordinator::AudioHandoffCoordinator>,
    audio_intents: crate::application::audio_handoff::intents::AudioHandoffIntents,
    native_audio_logged: Option<serde_json::Value>,
    surface: crate::infrastructure::video_surface::VideoSurface,
    browser: Browser,
    events: Receiver<crate::application::session::BrowserMessage>,
    session: Session,
    address: String,
    blocking: bool,
    volume: f64,
    smoke_started: Option<Instant>,
    smoke_result: Option<Rc<Cell<bool>>>,
    native_started: bool,
    native_connected: bool,
    native_reported: Instant,
    fullscreen: bool,
    scrub: Option<f64>,
    address_focused: bool,
    native_pause: crate::application::native_playback::NativePauseIntent,
    episodes: crate::application::episode_queue::EpisodeQueue,
    episode_panel: bool,
    transition: crate::application::episode_transition::EpisodeTransition,
    web_input: bool,
    visibility: crate::application::control_visibility::ControlVisibility,
    activity: crate::infrastructure::window_activity::WindowActivity,
    ui_started: Instant,
    chrome_visible: bool,
    chrome_interaction: chrome::ChromeInteraction,
    window_metrics: crate::infrastructure::window_metrics::WindowMetrics,
    metrics_reported: Instant,
    route: crate::application::playback_route::PlaybackRoute,
    browser_open: bool,
    local_media: bool,
    file_picker: crate::infrastructure::local_file_picker::LocalFilePicker,
    studio: studio::StudioShell,
    studio_window_size: Option<egui::Vec2>,
    native_status: Option<crate::application::playback_status::NativeStatusPublisher>,
    client: crate::application::api::ApplicationClientHandle,
    native_control: Option<crate::infrastructure::native_playback_control::NativeControlBinder>,
    native_control_error: Option<String>,
}

impl PlayerApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        smoke_result: Option<Rc<Cell<bool>>>,
        start_url: String,
        client: crate::application::api::ApplicationClientHandle,
        paths: crate::application::subtitles::path_picker::SubtitlePathPickerHandle,
    ) -> anyhow::Result<Self> {
        super::layout::configure(&cc.egui_ctx);
        super::player_theme::apply(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        super::typography::install(&cc.egui_ctx).map_err(anyhow::Error::msg)?;
        let (tx, events) = mpsc::channel();
        let surface = crate::infrastructure::video_surface::VideoSurface::new(cc)?;
        let browser = Browser::new(cc, tx, &start_url)?;
        Ok(Self {
            surface,
            browser,
            events,
            session: Session::default(),
            mpv: None,
            address: if start_url == "about:blank" {
                String::new()
            } else {
                start_url.clone()
            },
            blocking: true,
            volume: 80.,
            native_audio: None,
            audio_intents: Default::default(),
            native_audio_logged: None,
            smoke_started: smoke_result.as_ref().map(|_| Instant::now()),
            smoke_result,
            native_started: false,
            native_connected: false,
            native_reported: Instant::now(),
            fullscreen: false,
            scrub: None,
            address_focused: false,
            native_pause: Default::default(),
            episodes: Default::default(),
            episode_panel: start_url != "about:blank",
            transition: Default::default(),
            web_input: false,
            visibility: Default::default(),
            activity: crate::infrastructure::window_activity::WindowActivity::new(cc)?,
            ui_started: Instant::now(),
            chrome_visible: true,
            chrome_interaction: Default::default(),
            window_metrics: crate::infrastructure::window_metrics::WindowMetrics::new(cc)?,
            metrics_reported: Instant::now(),
            route: Default::default(),
            browser_open: start_url != "about:blank",
            local_media: false,
            file_picker: crate::infrastructure::local_file_picker::LocalFilePicker::new(cc)?,
            studio: studio::StudioShell::with_client(client.clone(), paths)
                .with_review_tools(diagnostics::enabled()),
            studio_window_size: None,
            native_status: None,
            client,
            native_control: None,
            native_control_error: None,
        })
    }

    pub fn with_native_status(
        mut self,
        publisher: Option<crate::application::playback_status::NativeStatusPublisher>,
    ) -> Self {
        self.native_status = publisher;
        self.clear_native_status();
        self
    }

    fn clear_native_status(&self) {
        if let Some(publisher) = &self.native_status {
            let _ = publisher.clear();
        }
    }

    fn navigate(&mut self, url: String) {
        if !self.stop_native() {
            return;
        }
        self.episodes.navigate(&url);
        if self.episodes.entries.is_empty() {
            self.transition.cancel();
        }
        self.session.reset_page();
        self.address = url;
        if let Err(e) = self.browser.navigate(&self.address) {
            self.transition.cancel();
            self.session.status = e.to_string();
        }
    }

    fn pump_events(&mut self) {
        while let Ok(message) = self.events.try_recv() {
            if self.local_media {
                continue;
            }
            let original_generation = message.generation;
            let Some(event) = message.into_current(self.browser.generation()) else {
                diagnostics::record("obsolete-browser-event", json!({"discarded":true}));
                continue;
            };
            if matches!(event, BrowserEvent::InputActivity) {
                self.web_input = true;
            }
            if let BrowserEvent::Episodes(page) = &event
                && self.episodes.observe(page.clone(), &self.address)
            {
                diagnostics::record(
                    "episode-list",
                    json!({"count":self.episodes.entries.len(),"current":self.episodes.current}),
                );
                if self.transition.page_ready(&page.page)
                    && let Err(error) = self.browser.open_episode_player(&page.page)
                {
                    self.transition.cancel();
                    self.session.status = error.to_string();
                }
            }
            if let BrowserEvent::Playback(observation) = &event {
                diagnostics::record(
                    "web-playback",
                    json!({"blocked":self.session.blocked,"observation":observation}),
                );
            }
            if let BrowserEvent::Media(source) = &event {
                let (url, referer) = (&source.url, &source.referer);
                diagnostics::record(
                    "media-candidate",
                    json!({"host":url::Url::parse(url).ok().and_then(|u| u.host_str().map(str::to_owned)), "referer_host":url::Url::parse(referer).ok().and_then(|u| u.host_str().map(str::to_owned))}),
                );
            }
            if let BrowserEvent::Navigated(url) = &event {
                self.episodes.navigate(url);
                if self.episodes.entries.is_empty() {
                    self.transition.cancel();
                }
                self.stop_native();
                self.address = url.clone();
            }
            if let Some(url) = self.session.observe_message(
                crate::application::session::BrowserMessage {
                    generation: original_generation,
                    event,
                },
                self.browser.generation(),
            ) {
                self.navigate(url);
            }
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for (title, path) in [("방영중", "방영중"), ("종영", "종영"), ("극장판", "극장판")]
            {
                if ui.button(title).clicked() {
                    self.open_category(path);
                }
            }
            if ui.checkbox(&mut self.blocking, "광고 요청 차단").changed() {
                self.browser.set_blocking(self.blocking);
            }
            ui.label(format!("차단 {}", self.session.blocked));
        });
        ui.horizontal(|ui| {
            self.address_focused = ui
                .add(
                    egui::TextEdit::singleline(&mut self.address)
                        .desired_width((ui.available_width() - 70.).max(100.)),
                )
                .has_focus();
            if ui.button("열기").clicked() {
                self.navigate(self.address.clone());
            }
        });
    }

    fn update_native(&mut self) {
        let Some(player) = &mut self.mpv else {
            return;
        };
        let state = player.snapshot().clone();
        let pending = self
            .audio_intents
            .pending_requested(crate::domain::api_values::DecimalU64(
                player.control_handle().instance_id(),
            ))
            .ok()
            .flatten();
        if let Some(volume) = state.volume {
            if volume.is_finite() && (0.0..=100.0).contains(&volume) {
                self.volume = volume;
            } else {
                self.native_control_error =
                    Some("관측 음량이 GUI 지원 범위를 벗어났습니다.".into());
            }
        }
        if let Some(publisher) = &self.native_status {
            let _ = publisher.publish(&state);
        }
        if let Some(requested) = pending {
            self.volume = requested.audio.volume_percent();
        }
        self.native_connected = state.connected;
        self.advance_native_audio();
        self.native_pause.observe(self.native_started, state.paused);
        if self.native_audio.is_none()
            && let Some(error) = &state.error
        {
            let _ = self.browser.pause_media();
            self.browser_open = false;
            self.stop_native();
            self.session.status = format!("{error} · 다시 열거나 웹으로 재생을 선택하세요");
        }
        if self.native_reported.elapsed() >= Duration::from_secs(1) {
            diagnostics::record("native-playback", &state);
            self.native_reported = Instant::now();
        }
    }
}

impl Drop for PlayerApp {
    fn drop(&mut self) {
        self.clear_native_control();
        self.clear_native_status();
        self.mpv.take();
    }
}

impl eframe::App for PlayerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = &ui.ctx().clone();
        self.pump_events();
        if self.transition.expire(self.ui_started.elapsed()) {
            self.session.status =
                "회차 연결 시간이 초과되었습니다 · 회차를 다시 선택해 주세요".into();
            diagnostics::record("episode-timeout", json!({"cancelled":true}));
        }
        if self.transition.ready()
            && self.mpv.is_none()
            && self.session.media.is_some()
            && self
                .session
                .playback
                .as_ref()
                .is_some_and(|p| p.duration > 0.)
            && let Some(paused) = self.transition.take_native()
        {
            self.start_native();
            self.native_pause.reset(paused);
        }
        if self.transition.ready()
            && self.session.playback.is_some()
            && let Some(true) = self.transition.take_autoplay()
        {
            let _ = self.browser.control("play", 0.);
        }
        self.update_native();
        if !self.local_media
            && self.mpv.is_none()
            && self.route.can_start(self.browser.generation())
            && self.session.media.is_some()
            && self
                .session
                .playback
                .as_ref()
                .is_some_and(|p| p.duration > 0.)
        {
            self.start_native();
        }
        if self
            .smoke_started
            .is_some_and(|start| start.elapsed() >= Duration::from_secs(20))
        {
            println!(
                "{}",
                json!({"smoke":"webview-startup", "page_finished":self.session.loaded, "requests":self.session.requested, "blocked_requests":self.session.blocked, "engine": Browser::engine_version()})
            );
            self.smoke_started = None;
            if let Some(result) = &self.smoke_result {
                result.set(self.session.loaded && self.session.requested > 0);
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.shortcuts(ctx);
        if !self.studio.open
            && let Some(size) = self.studio_window_size.take()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
        }
        self.update_chrome(ctx);
        if self.studio.open {
            self.address_focused = false;
            let _ = self.browser.set_visible(false);
            let _ = self
                .surface
                .layout([0., 0., 0., 0.], ctx.pixels_per_point() as f64, false);
            egui::Panel::top("studio-title")
                .frame(
                    egui::Frame::new()
                        .fill(super::player_theme::CHROME)
                        .inner_margin(egui::Margin::ZERO),
                )
                .show(ui, |ui| self.title_bar(ui));
            self.studio.show(ui, &self.episodes.title);
            if !self.studio.open {
                let _ = self.browser.set_visible(self.browser_open);
                if let Some(size) = self.studio_window_size.take() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                }
            }
            ctx.request_repaint_after(Duration::from_millis(100));
            crate::presentation::window_frame::show(ctx);
            return;
        }
        if self.chrome_visible {
            if !self.fullscreen {
                let toolbar = egui::Panel::top("toolbar")
                    .frame(
                        egui::Frame::new()
                            .fill(super::player_theme::CHROME)
                            .inner_margin(egui::Margin::ZERO),
                    )
                    .show(ui, |ui| self.workspace_toolbar(ui));
                self.chrome_interaction
                    .remember(chrome::ControlRegion::Toolbar, toolbar.response.rect);
            } else {
                self.address_focused = false;
            }
            let controls = egui::Panel::bottom("controls").show(ui, |ui| self.transport(ui, ctx));
            self.chrome_interaction
                .remember(chrome::ControlRegion::Transport, controls.response.rect);
        } else {
            self.address_focused = false;
        }
        self.episode_sidebar(ui);
        if diagnostics::enabled() && self.metrics_reported.elapsed() >= Duration::from_secs(1) {
            if let Ok(window) = self.window_metrics.snapshot() {
                diagnostics::record(
                    "window-metrics",
                    json!({"window":window,"pixels_per_point":ctx.pixels_per_point(),"fullscreen":self.fullscreen,"controls_visible":self.chrome_visible,"episode_panel":self.episode_panel,"episode_count":self.episodes.entries.len()}),
                );
            }
            self.metrics_reported = Instant::now();
        }
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new().fill(crate::presentation::design_tokens::color::VIDEO_SURFACE),
            )
            .show(ui, |ui| {
                let rect = ui.max_rect();
                if let Err(error) = self.surface.layout(
                    [
                        rect.min.x as f64,
                        rect.min.y as f64,
                        rect.width() as f64,
                        rect.height() as f64,
                    ],
                    ctx.pixels_per_point() as f64,
                    self.mpv.is_some() && !self.browser_open,
                ) {
                    self.session.status = error.to_string();
                }
                if let Err(e) = self.browser.resize(
                    rect.min.x as f64,
                    rect.min.y as f64,
                    rect.width() as f64,
                    rect.height() as f64,
                ) {
                    self.session.status = e.to_string();
                }
                if !self.browser_open && self.mpv.is_none() {
                    self.idle_workspace(ui);
                }
                if !self.browser_open
                    && ui
                        .interact(
                            rect,
                            egui::Id::new("video-interaction"),
                            egui::Sense::click(),
                        )
                        .double_clicked()
                {
                    self.fullscreen = !self.fullscreen;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
                }
            });
        crate::presentation::window_frame::show(ctx);
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}
