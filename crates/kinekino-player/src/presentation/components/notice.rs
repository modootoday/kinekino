use crate::presentation::{
    design_tokens::{border, color, radius, size, space},
    icon_asset::Icon,
    layout,
};
use eframe::egui;

#[derive(Clone, Copy)]
pub enum Kind {
    Info,
    Warning,
}

pub fn show(ui: &mut egui::Ui, kind: Kind, title: &str, detail: &str) {
    let (surface, border_color, foreground) = match kind {
        Kind::Info => (
            color::INFO_SURFACE,
            color::INFO_BORDER,
            color::INFO_FOREGROUND,
        ),
        Kind::Warning => (color::NOTICE_SURFACE, color::NOTICE_BORDER, color::ACCENT),
    };
    egui::Frame::new()
        .fill(surface)
        .stroke(egui::Stroke::new(border::THIN, border_color))
        .corner_radius(radius::CONTROL)
        .inner_margin(space::PANEL_MARGIN)
        .show(ui, |ui| {
            egui_taffy::tui(ui, ui.id().with(title))
                .reserve_available_width()
                .style(layout::full(layout::sidebar(
                    size::STATUS_ICON,
                    space::FIELD,
                )))
                .show(|tui| {
                    layout::block(tui, |ui| {
                        let (rect, _) = ui.allocate_exact_size(
                            egui::Vec2::splat(size::STATUS_ICON),
                            egui::Sense::hover(),
                        );
                        match kind {
                            Kind::Info => Icon::Info,
                            Kind::Warning => Icon::Warning,
                        }
                        .paint(ui, rect, foreground);
                    });
                    layout::block(tui, |ui| {
                        ui.label(egui::RichText::new(title).strong().color(foreground));
                        ui.add_space(space::TIGHT);
                        ui.label(egui::RichText::new(detail).color(color::SECONDARY));
                    });
                });
        });
}
