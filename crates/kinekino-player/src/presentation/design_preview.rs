//! Offline design review host using the production studio and components.
mod episodes;
use super::{caption_controls, layout, player_app::studio::StudioShell, player_theme, typography};
use crate::application::workspace_read::preview::{PreviewScenario, PreviewWorkspaceProvider};
use eframe::egui;
use std::sync::Arc;

pub fn run(position: Option<[f32; 2]>) -> anyhow::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1440., 900.])
        .with_min_inner_size([760., 480.])
        .with_decorations(false);
    if let Some(position) = position {
        viewport = viewport.with_position(position);
    }
    eframe::run_native(
        crate::domain::product::PRODUCT.display_name,
        eframe::NativeOptions {
            viewport,
            renderer: eframe::Renderer::Glow,
            ..Default::default()
        },
        Box::new(|cc| {
            layout::configure(&cc.egui_ctx);
            player_theme::apply(&cc.egui_ctx);
            typography::install(&cc.egui_ctx)?;
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let mut studio = StudioShell::default();
            studio.open = true;
            studio.set_reader(Arc::new(PreviewWorkspaceProvider::new(
                PreviewScenario::Populated,
            )));
            Ok(Box::new(PreviewApp {
                studio,
                scenario: PreviewScenario::Populated,
                review_size: [1440, 900],
                layout_repaint: false,
                episode_review: false,
                episode_current: Some(2),
            }))
        }),
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))
}

struct PreviewApp {
    studio: StudioShell,
    scenario: PreviewScenario,
    review_size: [u16; 2],
    layout_repaint: bool,
    episode_review: bool,
    episode_current: Option<usize>,
}

impl PreviewApp {
    fn size_control(&mut self, ui: &mut egui::Ui) {
        let maximized = ui
            .ctx()
            .input(|input| input.viewport().maximized.unwrap_or(false));
        let actual = ui.ctx().content_rect().size();
        let mut chosen = false;
        let response = egui::ComboBox::from_id_salt("review-window-size")
            .width(100.)
            .selected_text(if maximized {
                "최대화".into()
            } else {
                format!("{:.0}×{:.0}", actual.x, actual.y)
            })
            .show_ui(ui, |ui| {
                ui.small("검토 창 크기 프리셋");
                for size in [[760, 480], [980, 760], [1180, 760], [1440, 900]] {
                    chosen |= ui
                        .selectable_value(
                            &mut self.review_size,
                            size,
                            format!("{}×{}", size[0], size[1]),
                        )
                        .clicked();
                }
            });
        response.response.on_hover_text(format!(
            "현재 창: {:.0}×{:.0} 논리 단위\n마지막 프리셋: {}×{}\n프리셋을 선택하면 최대화를 해제하고 해당 크기로 변경합니다.",
            actual.x, actual.y, self.review_size[0], self.review_size[1],
        ));
        if chosen {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Maximized(false));
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                    f32::from(self.review_size[0]),
                    f32::from(self.review_size[1]),
                )));
        }
    }

    fn scenario_control(&mut self, ui: &mut egui::Ui) {
        let previous = self.scenario;
        let label = match self.scenario {
            PreviewScenario::Empty => "빈 상태",
            PreviewScenario::Populated => "데이터 채움",
            PreviewScenario::LongLabels => "긴 문구",
            PreviewScenario::NeedsInput => "입력 필요",
        };
        egui::ComboBox::from_id_salt("design-scenario")
            .selected_text(label)
            .show_ui(ui, |ui| {
                for (scenario, label) in [
                    (PreviewScenario::Empty, "빈 상태"),
                    (PreviewScenario::Populated, "데이터 채움"),
                    (PreviewScenario::LongLabels, "긴 문구"),
                    (PreviewScenario::NeedsInput, "입력 필요"),
                ] {
                    ui.selectable_value(&mut self.scenario, scenario, label);
                }
            });
        if previous != self.scenario {
            self.studio
                .set_reader(Arc::new(PreviewWorkspaceProvider::new(self.scenario)));
        }
    }

    fn caption(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        egui_taffy::tui(ui, ui.id().with("caption"))
            .reserve_available_width()
            .style(layout::full(layout::row(0.)))
            .show(|tui| {
                layout::block(tui, |ui| {
                    let title = ui.add(
                        egui::Label::new(
                            egui::RichText::new("디자인 데모 · 샘플 데이터")
                                .color(player_theme::ACCENT),
                        )
                        .sense(egui::Sense::click_and_drag()),
                    );
                    if title.drag_started() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                });
                layout::widget(tui, |ui| {
                    ui.toggle_value(&mut self.episode_review, "회차 예시");
                });
                layout::widget(tui, |ui| self.scenario_control(ui));
                layout::widget(tui, |ui| self.size_control(ui));
                layout::widget(tui, |ui| {
                    let maximized = ctx.input(|input| input.viewport().maximized.unwrap_or(false));
                    if let Some(action) = caption_controls::show(ui, maximized) {
                        ctx.send_viewport_cmd(match action {
                            caption_controls::Action::Minimize => {
                                egui::ViewportCommand::Minimized(true)
                            }
                            caption_controls::Action::ToggleMaximize => {
                                egui::ViewportCommand::Maximized(!maximized)
                            }
                            caption_controls::Action::Close => egui::ViewportCommand::Close,
                        });
                    }
                });
            });
    }
}

impl eframe::App for PreviewApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.episode_review {
            if ui
                .ctx()
                .input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
            {
                self.episode_review = false;
            }
        } else {
            self.studio.handle_escape(ui.ctx());
        }
        egui::Panel::top("preview-title")
            .frame(
                egui::Frame::new()
                    .fill(player_theme::CHROME)
                    .inner_margin(0),
            )
            .show(ui, |ui| self.caption(ui));
        if self.episode_review {
            self.show_episode_review(ui);
        } else {
            self.studio.show(ui, "");
        }
        super::window_frame::show(ui.ctx());
        if ui
            .ctx()
            .output(|output| !output.request_discard_reasons.is_empty())
        {
            self.layout_repaint = true;
            ui.ctx().request_repaint();
        } else if std::mem::take(&mut self.layout_repaint) {
            // Finish the settled frame instead of leaving the previous diagnostic painted.
            ui.ctx().request_repaint();
        }
        if !self.studio.open {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
