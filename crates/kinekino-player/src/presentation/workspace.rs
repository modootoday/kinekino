use super::*;

impl PlayerApp {
    pub(super) fn open_category(&mut self, path: &str) {
        self.route.select_native();
        self.local_media = false;
        self.browser_open = true;
        self.episode_panel = true;
        self.navigate(format!("https://catalog.example.test/{path}"));
    }

    pub(super) fn open_local(&mut self) {
        let selection = match self.file_picker.pick() {
            Ok(Some(video)) => video,
            Ok(None) => return,
            Err(error) => {
                self.session.status = error.to_string();
                return;
            }
        };
        self.clear_native_status();
        if !self.stop_native() {
            return;
        }
        let prepared = crate::domain::native_audio::AudioIntent::new(self.volume, false)
            .map_err(|message| crate::application::api::ApiError::new(crate::application::api::ErrorCode::InvalidArgument, message))
            .and_then(|audio| crate::application::audio_handoff::coordinator::PreparedAudioHandoff::local_with_web_view(
                crate::domain::api_values::DecimalU64(self.browser.generation()), self.browser.audio_handoff_available(), audio, false, Instant::now()));
        let prepared = match prepared {
            Ok(prepared) => prepared,
            Err(error) => {
                self.session.status = error.message;
                return;
            }
        };
        match Mpv::launch_local(&selection, self.surface.id(), self.volume) {
            Ok(player) => {
                self.transition.cancel();
                self.session.reset_page();
                self.route.select_native();
                self.local_media = true;
                self.browser_open = false;
                self.address.clear();
                self.episodes = Default::default();
                self.episodes.title = selection
                    .path()
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                self.episode_panel = false;
                self.native_pause.reset(false);
                if !self.install_native(player, prepared) {
                    return;
                }
                self.session.status = "로컬 영상 연결 중".into();
                let _ = self.browser.set_visible(false);
                diagnostics::record("local-open", json!({"selected_by_user":true}));
            }
            Err(error) => self.session.status = error.to_string(),
        }
    }

    pub(super) fn workspace_toolbar(&mut self, ui: &mut egui::Ui) {
        self.title_bar(ui);
        if self.browser_open {
            self.toolbar(ui);
        } else {
            self.address_focused = false;
        }
    }

    pub(super) fn idle_workspace(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space((ui.available_height() * 0.46 - 80.).max(12.));
            let (rect, _) = ui.allocate_exact_size(egui::vec2(100., 100.), egui::Sense::hover());
            let color = super::super::player_theme::IDLE_MARK;
            super::super::icon_asset::Icon::StartMark.paint(ui, rect, color);
            ui.add_space(12.);
            ui.heading(crate::domain::product::PRODUCT.display_name);
            ui.add_space(24.);
            let width = 312.;
            ui.allocate_ui_with_layout(
                egui::vec2(width, 44.),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    if super::super::control_icon::action(
                        ui,
                        super::super::control_icon::Icon::Folder,
                        "파일 열기",
                        150.,
                        true,
                    )
                    .clicked()
                    {
                        self.open_local();
                    }
                    if super::super::control_icon::action(
                        ui,
                        super::super::control_icon::Icon::Search,
                        "작품 탐색",
                        150.,
                        false,
                    )
                    .clicked()
                    {
                        self.open_category("방영중");
                    }
                },
            );
            ui.add_space(12.);
            ui.weak("Ctrl+O로 파일 열기");
            if self.session.media.is_some() && ui.button("네이티브 재시도").clicked() {
                self.start_native();
            }
            if self.session.playback.is_some() && ui.button("웹으로 재생").clicked() {
                self.return_web();
            }
            ui.add_space(12.);
            ui.label(&self.session.status);
        });
    }
}
