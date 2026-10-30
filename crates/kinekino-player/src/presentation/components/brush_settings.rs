use crate::presentation::design_tokens::{size, space};
use eframe::egui;

pub fn unavailable(ui: &mut egui::Ui) {
    for (id, label) in [
        ("brush-size", "브러시 크기 (px)"),
        ("brush-softness", "부드러움"),
    ] {
        ui.push_id(id, |ui| {
            ui.small(label);
            super::input::unavailable_scalar(ui, size::CONTROL, "브러시 값 연결 필요");
        });
        ui.add_space(space::FIELD);
    }
    ui.small("브러시 값 연결 필요");
}
