use super::{
    button::{Button, Variant},
    card,
};
use crate::presentation::design_tokens::{size, space};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, id: &str, text: &str) {
    card::show(ui, "", |ui| {
        if ui
            .add(Button::new("원문 복사", Variant::Secondary).compact())
            .clicked()
        {
            ui.ctx().copy_text(text.to_owned());
        }
        ui.add_space(space::FIELD);
        egui::ScrollArea::vertical()
            .id_salt(id)
            .max_height(size::REVIEW_TEXT_HEIGHT)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.add(egui::Label::new(text).wrap().selectable(true));
            });
    });
}
