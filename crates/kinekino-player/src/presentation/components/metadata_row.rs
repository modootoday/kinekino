use crate::presentation::{
    control_icon::Icon,
    design_tokens::{color, size, space},
    layout,
};
use eframe::egui;
use egui_taffy::taffy::{
    AlignItems, Size, Style,
    prelude::{auto, length},
};

pub fn show(ui: &mut egui::Ui, id: &str, icon: Icon, label: &str, value: &str) {
    egui_taffy::tui(ui, ui.id().with(("metadata-row", id)))
        .reserve_available_width()
        .style(Style {
            min_size: Size {
                width: auto(),
                height: length(size::CONTROL),
            },
            align_items: Some(AlignItems::Center),
            ..layout::full(layout::sidebar(size::METADATA_LABEL_WIDTH, space::FIELD))
        })
        .show(|tui| {
            layout::block(tui, |ui| {
                egui_taffy::tui(ui, ui.id().with("icon-label"))
                    .reserve_available_width()
                    .style(layout::full(layout::row(space::FIELD)))
                    .show(|tui| {
                        layout::widget(tui, |ui| {
                            ui.add(
                                icon.image()
                                    .tint(color::TEXT)
                                    .fit_to_exact_size(egui::Vec2::splat(size::TOOLBAR_ICON)),
                            )
                        });
                        layout::block(tui, |ui| {
                            ui.add(egui::Label::new(label).wrap()).on_hover_text(label);
                        });
                    });
            });
            layout::block(tui, |ui| {
                ui.add(egui::Label::new(value).wrap()).on_hover_text(value);
            });
        });
}
