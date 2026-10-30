use super::{artboard::Artwork, card, media_preview};
use crate::presentation::{
    control_icon::Icon,
    design_tokens::{color, size, space},
    layout,
};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, title: &str, artwork: &Artwork<'_>, height: f32) {
    card::show(ui, "", |ui| {
        egui_taffy::tui(ui, ui.id().with("source-media-header"))
            .reserve_available_width()
            .style(layout::full(layout::row(space::FIELD)))
            .show(|tui| {
                layout::widget(tui, |ui| {
                    ui.add(
                        Icon::Folder
                            .image()
                            .tint(color::TEXT)
                            .fit_to_exact_size(egui::Vec2::splat(size::TOOLBAR_ICON)),
                    );
                });
                layout::block(tui, |ui| {
                    ui.add(egui::Label::new(egui::RichText::new(title).strong()).truncate())
                        .on_hover_text(title);
                });
            });
        ui.add_space(space::FIELD);
        media_preview::unavailable(ui, artwork, 16. / 9., "16:9", height);
    });
}
