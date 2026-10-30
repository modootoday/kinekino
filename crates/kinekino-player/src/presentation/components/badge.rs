use crate::presentation::design_tokens::{border, color, radius, size, space, text};
use eframe::egui;

/// Neutral metadata; the caller retains the exact value and status meaning.
pub fn show(ui: &mut egui::Ui, label: &str) -> egui::Response {
    egui::Frame::new()
        .fill(color::FIELD)
        .stroke(egui::Stroke::new(border::THIN, color::BORDER))
        .corner_radius(radius::PILL)
        .inner_margin(egui::Margin::symmetric(
            space::INLINE_MARGIN,
            space::TIGHT as i8,
        ))
        .show(ui, |ui| {
            ui.set_max_width(size::BADGE_MAX_WIDTH);
            ui.set_min_height(size::BADGE - 2. * (space::TIGHT + border::THIN));
            ui.add(
                egui::Label::new(
                    egui::RichText::new(label)
                        .size(text::SMALL)
                        .color(color::SECONDARY),
                )
                .truncate(),
            );
        })
        .response
        .on_hover_text(label)
}

pub fn list(ui: &mut egui::Ui, id: &str, labels: &[String]) {
    use crate::presentation::layout;
    use egui_taffy::taffy::{FlexWrap, Style};
    egui_taffy::tui(ui, ui.id().with(id))
        .reserve_available_width()
        .style(layout::full(Style {
            flex_wrap: FlexWrap::Wrap,
            ..layout::row(space::FIELD)
        }))
        .show(|tui| {
            for label in labels {
                layout::widget(tui, |ui| {
                    show(ui, label);
                });
            }
        });
}
