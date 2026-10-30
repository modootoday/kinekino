use crate::presentation::design_tokens::space;
use crate::presentation::{
    control_icon::{self, Icon},
    layout,
};
use eframe::egui;

pub(super) fn zoom_controls(
    ui: &mut egui::Ui,
    zoom: &mut Option<u16>,
    fit_percent: u16,
    pan: &mut bool,
    offset: &mut egui::Vec2,
) {
    egui_taffy::tui(ui, ui.id().with("canvas-zoom"))
        .style(layout::row(space::TIGHT))
        .show(|tui| {
            layout::widget(tui, |ui| {
                if control_icon::button(ui, Icon::Minus, "축소", false, 32.).clicked() {
                    *zoom = Some(zoom.unwrap_or(fit_percent).saturating_sub(25).max(10));
                }
            });
            layout::widget(tui, |ui| {
                ui.monospace(zoom.map_or_else(|| "맞춤".into(), |value| format!("{value}%")));
            });
            layout::widget(tui, |ui| {
                if control_icon::button(ui, Icon::Plus, "확대", false, 32.).clicked() {
                    *zoom = Some(zoom.unwrap_or(fit_percent).saturating_add(25).min(400));
                }
            });
            layout::widget(tui, |ui| {
                if control_icon::button(ui, Icon::Fullscreen, "화면에 맞춤", false, 32.).clicked()
                {
                    *zoom = None;
                    *offset = egui::Vec2::ZERO;
                }
            });
            layout::widget(tui, |ui| {
                if control_icon::button(ui, Icon::Hand, "이동 보기", *pan, 32.).clicked() {
                    *pan = !*pan;
                }
            });
        });
}
