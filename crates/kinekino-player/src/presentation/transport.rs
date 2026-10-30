use super::super::control_icon::{self, Icon};
use super::*;
use crate::presentation::components::media_rail::Rail;
use crate::presentation::layout;

fn clock(seconds: f64) -> String {
    if !seconds.is_finite() || seconds < 0. {
        return "--:--".into();
    }
    let total = seconds.max(0.) as u64;
    if total < 3600 {
        return format!("{:02}:{:02}", total / 60, total % 60);
    }
    format!(
        "{:02}:{:02}:{:02}",
        total / 3600,
        total / 60 % 60,
        total % 60
    )
}

impl PlayerApp {
    fn playback(&mut self) -> (f64, f64, bool) {
        if let Some(player) = &mut self.mpv {
            let state = player.snapshot();
            return (
                state.time.unwrap_or(0.),
                state.duration.unwrap_or(0.),
                state.paused,
            );
        }
        self.session
            .playback
            .as_ref()
            .map(|s| (s.time, s.duration, s.paused))
            .unwrap_or((0., 0., true))
    }

    pub(super) fn start_native(&mut self) {
        if self.mpv.is_some() {
            return;
        }
        if self.native_audio.is_some() && !self.stop_native() {
            return;
        }
        self.clear_native_status();
        self.clear_native_control();
        let (position, _, paused) = self.playback();
        let prepared = crate::domain::native_audio::AudioIntent::new(self.volume, false)
            .map_err(|message| {
                crate::application::api::ApiError::new(
                    crate::application::api::ErrorCode::InvalidArgument,
                    message,
                )
            })
            .and_then(|audio| {
                crate::application::audio_handoff::coordinator::PreparedAudioHandoff::web(
                    self.session.audio_handoff_origin(),
                    crate::domain::api_values::DecimalU64(self.browser.generation()),
                    self.browser.audio_handoff_available(),
                    audio,
                    paused,
                    Instant::now(),
                )
            });
        let prepared = match prepared {
            Ok(prepared) => prepared,
            Err(error) => {
                self.route.attempted(self.browser.generation());
                self.session.status = error.message;
                return;
            }
        };
        if let Some(source) = &self.session.media {
            self.route.select_native();
            self.route.attempted(self.browser.generation());
            match Mpv::launch(source, self.surface.id(), position, self.volume) {
                Ok(player) => {
                    if !self.install_native(player, prepared) {
                        return;
                    }
                    self.native_pause.reset(paused);
                    self.native_started = false;
                    self.native_connected = false;
                    self.session.status =
                        "네이티브 영상 연결 중 · 웹 재생은 준비될 때까지 유지됩니다".into();
                }
                Err(error) => {
                    self.browser_open = false;
                    let _ = self.browser.pause_media();
                    let _ = self.browser.set_visible(false);
                    self.session.status = error.to_string();
                }
            }
        }
    }

    pub(super) fn return_web(&mut self) {
        if self.local_media {
            return;
        }
        let (position, _, paused) = self.playback();
        if !self.stop_native() {
            return;
        }
        if let Err(error) = self.browser.set_visible(true) {
            self.session.status = format!("웹 화면 표시 실패: {error}");
            return;
        }
        self.route.select_web();
        self.browser_open = true;
        let result = self
            .browser
            .control("seek", position)
            .and_then(|_| self.browser.control("volume", self.volume))
            .and_then(|_| {
                self.browser
                    .control(if paused { "pause" } else { "play" }, 0.)
            });
        self.session.status = result
            .err()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "웹 모드 · 재생이 막히면 영상의 재생 버튼을 누르세요".into());
    }

    fn seek(&mut self, position: f64) {
        if self.mpv.is_some() {
            self.seek_native(position.max(0.));
        } else if let Err(e) = self.browser.control("seek", position) {
            self.session.status = e.to_string();
        }
    }

    fn toggle(&mut self) {
        if let Some(player) = &mut self.mpv {
            let state = player.snapshot();
            if !state.pause_observed {
                self.native_control_error =
                    Some("재생 상태를 확인한 뒤 다시 조작해 주세요.".into());
                return;
            }
            let control = if state.paused {
                crate::domain::playback_control::PlaybackControl::Play
            } else {
                crate::domain::playback_control::PlaybackControl::Pause
            };
            self.submit_native(control);
        } else if let Err(e) = self.browser.control("toggle", 0.) {
            self.session.status = e.to_string();
        }
    }

    pub(super) fn transport(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if let Some(error) = &self.native_control_error {
            crate::presentation::components::notice::show(
                ui,
                crate::presentation::components::notice::Kind::Warning,
                "재생 제어 확인 필요",
                error,
            );
        }
        if crate::presentation::components::volume_control::is_open(ctx) {
            ui.add_space(crate::presentation::design_tokens::size::VOLUME_OVERLAY_CLEARANCE);
        }
        let (mut position, duration, paused) = self.playback();
        position = self.scrub.unwrap_or(position);
        ui.spacing_mut().item_spacing.y = 8.;
        let seek = ui.add_enabled(
            duration > 0.,
            Rail {
                value: &mut position,
                maximum: duration.max(1.),
                step: 5.,
                width: ui.available_width(),
                compact: true,
                label: "재생 위치",
            },
        );
        if seek.dragged() {
            self.scrub = Some(position);
        }
        if seek.drag_stopped() || (seek.changed() && !seek.dragged()) {
            self.seek(position);
            self.scrub = None;
        }
        ui.spacing_mut().slider_width = 72.;
        egui_taffy::tui(ui, ui.id().with("transport-row"))
            .reserve_available_width()
            .style(layout::full(layout::row(8.)))
            .show(|tui| {
                layout::block(tui, |ui| {
                    ui.horizontal(|ui| {
                        let ready = self.native_started
                            || (self.mpv.is_none() && self.session.playback.is_some());
                        self.episode_button(ui, -1);
                        if ui
                            .add_enabled_ui(ready, |ui| {
                                control_icon::button(
                                    ui,
                                    if paused { Icon::Play } else { Icon::Pause },
                                    if paused {
                                        "재생 · Space"
                                    } else {
                                        "일시정지 · Space"
                                    },
                                    false,
                                    40.,
                                )
                            })
                            .inner
                            .clicked()
                        {
                            self.toggle();
                        }
                        self.episode_button(ui, 1);
                        ui.monospace(format!(
                            "{} / {}",
                            if ready {
                                clock(position)
                            } else {
                                "--:--".into()
                            },
                            if duration > 0. {
                                clock(duration)
                            } else {
                                "--:--".into()
                            }
                        ));
                    });
                });
                layout::widget(tui, |ui| {
                    ui.horizontal(|ui| {
                        let ready = self.native_started
                            || (self.mpv.is_none() && self.session.playback.is_some());
                        if self.mpv.is_some()
                            || self.browser_open
                            || self.session.playback.is_some()
                        {
                            ui.menu_button(
                                if self.mpv.is_some() {
                                    "네이티브"
                                } else if self.browser_open {
                                    "웹 재생"
                                } else {
                                    "재생 방식"
                                },
                                |ui| {
                                    if ui
                                        .add_enabled_ui(ready && duration > 0., |ui| {
                                            control_icon::navigation(
                                                ui,
                                                Icon::Back,
                                                "10초 뒤로",
                                                false,
                                                false,
                                            )
                                        })
                                        .inner
                                        .clicked()
                                    {
                                        self.seek(position - 10.);
                                    }
                                    if ui
                                        .add_enabled_ui(ready && duration > 0., |ui| {
                                            control_icon::navigation(
                                                ui,
                                                Icon::Forward,
                                                "30초 앞으로",
                                                false,
                                                false,
                                            )
                                        })
                                        .inner
                                        .clicked()
                                    {
                                        self.seek((position + 30.).min(duration));
                                    }
                                    if ui
                                        .add_enabled(
                                            !self.local_media && self.session.playback.is_some(),
                                            egui::Button::new("웹으로 재생"),
                                        )
                                        .clicked()
                                    {
                                        self.return_web();
                                    }
                                    if ui
                                        .add_enabled(
                                            self.session.media.is_some(),
                                            egui::Button::new("네이티브")
                                                .selected(self.mpv.is_some()),
                                        )
                                        .clicked()
                                    {
                                        self.start_native();
                                    }
                                    ui.separator();
                                    ui.label(&self.session.status);
                                },
                            );
                        }
                        let volume_changed = crate::presentation::components::volume_control::show(
                            ui,
                            &mut self.volume,
                        );
                        if volume_changed {
                            if self.mpv.is_some() {
                                self.submit_native(
                                    crate::domain::playback_control::PlaybackControl::SetVolume {
                                        volume_percent: self.volume,
                                    },
                                );
                            } else {
                                let _ = self.browser.control("volume", self.volume);
                            }
                        }
                        ui.separator();
                        if control_icon::button(
                            ui,
                            Icon::List,
                            "회차 목록 · P",
                            self.episode_panel,
                            32.,
                        )
                        .clicked()
                        {
                            self.episode_panel = !self.episode_panel;
                        }
                        if control_icon::button(
                            ui,
                            Icon::Fullscreen,
                            "전체화면 · Esc로 복귀",
                            self.fullscreen,
                            32.,
                        )
                        .clicked()
                        {
                            self.fullscreen = !self.fullscreen;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                                self.fullscreen,
                            ));
                        }
                    });
                });
            });
    }

    pub(super) fn shortcuts(&mut self, ctx: &egui::Context) {
        if self.studio.open {
            self.studio.handle_escape(ctx);
            if !self.studio.open {
                let _ = self.browser.set_visible(self.browser_open);
            }
            return;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            if egui::Popup::is_any_open(ctx) {
                egui::Popup::close_all(ctx);
            } else if self.episode_panel {
                self.episode_panel = false;
            } else {
                self.fullscreen = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
            return;
        }
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::O)) {
            self.open_local();
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::L)) {
            self.open_category("방영중");
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::P)) {
            self.episode_panel = !self.episode_panel;
        }
        if (self.native_started || self.session.playback.is_some())
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Space))
        {
            self.toggle();
        }
        for (key, delta) in [(egui::Key::ArrowLeft, -5.), (egui::Key::ArrowRight, 5.)] {
            if (self.native_started || self.session.playback.is_some())
                && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key))
            {
                let (time, duration, _) = self.playback();
                self.seek((time + delta).clamp(0., duration.max(0.)));
            }
        }
    }
}
