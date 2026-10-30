use super::artboard::{self, Artwork};
use crate::presentation::design_tokens::{border, color, radius, size};
use eframe::egui::{self, Response, Ui};

pub fn show(ui: &mut Ui, asset_id: Option<&str>, label: &str, selected: bool) -> Response {
    let response = artboard::show(
        ui,
        &Artwork {
            asset_id,
            title: "",
            subtitle: "",
            cover: false,
            grid: false,
        },
        16. / 9.,
        size::FRAME_STRIP_HEIGHT,
        selected,
    );
    if response.clicked() {
        response.request_focus();
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            artboard::fitted_rect(response.rect, 16. / 9.).shrink(border::FOCUS * 2.),
            radius::CONTROL,
            egui::Stroke::new(border::THIN, color::FOCUS),
            egui::StrokeKind::Inside,
        );
    }
    ui.vertical_centered(|ui| {
        ui.colored_label(
            if selected {
                color::ACCENT
            } else {
                color::SECONDARY
            },
            label,
        );
        ui.small("샘플 · PTS 연결 필요");
    });
    response
}
