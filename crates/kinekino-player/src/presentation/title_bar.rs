use super::super::{
    caption_controls,
    control_icon::{self, Icon},
};
use super::*;

impl PlayerApp {
    pub(super) fn title_bar(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let (bar, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), caption_controls::HEIGHT),
            egui::Sense::hover(),
        );
        let brand_rect =
            egui::Rect::from_min_size(bar.min + egui::vec2(6., 0.), egui::vec2(32., 32.));
        let mut brand_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("window-brand")
                .max_rect(brand_rect),
        );
        let menu = control_icon::button(&mut brand_ui, Icon::Brand, "플레이어 메뉴", false, 32.);
        egui::Popup::menu(&menu).show(|ui| {
            ui.set_min_width(190.);
            if control_icon::navigation(ui, Icon::Folder, "파일 열기 · Ctrl+O", false, false)
                .clicked()
            {
                self.open_local();
            }
            if control_icon::navigation(ui, Icon::Search, "작품 탐색 · Ctrl+L", false, false)
                .clicked()
            {
                self.open_category("방영중");
            }
        });
        let caption_rect = egui::Rect::from_min_size(
            egui::pos2(bar.right() - caption_controls::GROUP_WIDTH, bar.top()),
            egui::vec2(caption_controls::GROUP_WIDTH, caption_controls::HEIGHT),
        );
        let maximized = ctx.input(|input| input.viewport().maximized.unwrap_or(false));
        let mut caption_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("window-caption")
                .max_rect(caption_rect),
        );
        if let Some(action) = caption_controls::show(&mut caption_ui, maximized) {
            ctx.send_viewport_cmd(match action {
                caption_controls::Action::Minimize => egui::ViewportCommand::Minimized(true),
                caption_controls::Action::ToggleMaximize => {
                    egui::ViewportCommand::Maximized(!maximized)
                }
                caption_controls::Action::Close => egui::ViewportCommand::Close,
            });
        }
        let title_right = if self.studio.open {
            caption_rect.left() - 8.
        } else {
            let studio_rect = egui::Rect::from_min_size(
                egui::pos2(caption_rect.left() - 68., bar.top()),
                egui::vec2(60., 32.),
            );
            if ui
                .put(studio_rect, egui::Button::new("제작").frame(false))
                .on_hover_text("제작 작업 공간 열기")
                .clicked()
            {
                self.studio.open = true;
                self.studio_window_size = Some(ctx.content_rect().size());
                if let Some(monitor) = ctx.input(|input| input.viewport().monitor_size) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                        1440_f32.min(monitor.x - 64.),
                        900_f32.min(monitor.y - 64.),
                    )));
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
            studio_rect.left() - 8.
        };
        let title_rect = egui::Rect::from_min_max(
            egui::pos2(44., bar.top()),
            egui::pos2(title_right.max(60.), bar.bottom()),
        );
        let product = crate::domain::product::PRODUCT.display_name;
        let context = if self.studio.open {
            if self.episodes.title.is_empty() {
                "스튜디오".to_owned()
            } else {
                format!("스튜디오 · {}", self.episodes.title)
            }
        } else {
            self.episodes.title.clone()
        };
        let title = if context.is_empty() {
            product.to_owned()
        } else {
            format!("{product} · {context}")
        };
        crate::presentation::components::window_title::paint(ui, title_rect, product, &context);
        let response = ui
            .interact(
                title_rect,
                ui.make_persistent_id("window-title-drag"),
                egui::Sense::click_and_drag(),
            )
            .on_hover_text(title);
        if response.drag_started() {
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        if response.double_clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
        }
    }
}
