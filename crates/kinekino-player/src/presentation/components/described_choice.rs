//! Read-only choice anatomy; callers own option meaning and availability.
use crate::presentation::{
    design_tokens::{color, size, space, text},
    layout,
};
use eframe::egui;

pub struct Choice<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub description: &'a str,
}

pub fn unavailable(ui: &mut egui::Ui, choices: &[Choice<'_>], reason: &str) {
    egui_taffy::tui(ui, ui.id().with("described-choices"))
        .reserve_available_width()
        .style(layout::full(layout::column(space::FIELD)))
        .show(|tui| {
            for choice in choices {
                layout::block(tui, |ui| {
                    ui.push_id(choice.id, |ui| {
                        ui.spacing_mut().indent =
                            ui.spacing().icon_width + ui.spacing().icon_spacing;
                        ui.visuals_mut().indent_has_left_vline = false;
                        ui.add_enabled_ui(false, |ui| {
                            ui.spacing_mut().interact_size.y = size::CONTROL;
                            ui.add(egui::RadioButton::new(
                                false,
                                egui::RichText::new(choice.title).strong(),
                            ))
                            .on_disabled_hover_text(reason);
                        });
                        ui.indent("description", |ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(choice.description)
                                        .size(text::SMALL)
                                        .color(color::SECONDARY),
                                )
                                .wrap(),
                            );
                        });
                    });
                });
            }
        });
}
