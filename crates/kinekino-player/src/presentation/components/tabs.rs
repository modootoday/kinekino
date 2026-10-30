use crate::presentation::{
    design_tokens::{border, size},
    player_theme,
};
use eframe::egui::{self, Response};

pub fn tab(ui: &mut egui::Ui, label: &str, selected: bool) -> Response {
    let text = egui::RichText::new(label).color(if selected {
        player_theme::ACCENT
    } else {
        player_theme::TEXT
    });
    let response = ui.add(
        egui::Button::new(text)
            .frame(false)
            .min_size(egui::vec2(size::TAB_MIN, size::CONTROL)),
    );
    if selected {
        ui.painter().line_segment(
            [response.rect.left_bottom(), response.rect.right_bottom()],
            egui::Stroke::new(border::FOCUS, player_theme::ACCENT),
        );
    }
    response
}
