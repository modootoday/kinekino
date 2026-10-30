//! Composable inspector primitives. No workspace types or application policies.
use crate::presentation::{
    design_tokens::{size, space},
    layout,
};
use eframe::egui;

/// Responsive inspector surface, bounded by the product caption and queue.
pub fn drawer(ctx: &egui::Context, open: &mut bool, body: impl FnOnce(&mut egui::Ui)) {
    let viewport = ctx.content_rect();
    let right = viewport.right() - space::PANEL;
    let rect = egui::Rect::from_min_max(
        egui::pos2(
            (right - size::INSPECTOR).max(viewport.left() + space::PANEL),
            viewport.top() + size::TITLE_BAR + space::PANEL,
        ),
        egui::pos2(right, viewport.bottom() - size::QUEUE - space::PANEL),
    );
    egui::Window::new("설정")
        .id(egui::Id::new("studio-settings-drawer"))
        .open(open)
        .collapsible(false)
        .resizable(false)
        .fixed_rect(rect)
        .show(ctx, body);
}

pub fn panel(
    ui: &mut egui::Ui,
    id: &str,
    body: impl FnOnce(&mut egui::Ui),
    footer: impl FnOnce(&mut egui::Ui),
) {
    egui::Panel::bottom(egui::Id::new(id).with("footer"))
        .exact_size(size::INSPECTOR_FOOTER)
        .resizable(false)
        .frame(egui::Frame::NONE)
        .show(ui, footer);
    egui::ScrollArea::vertical().id_salt(id).show(ui, body);
}

pub fn panel_with_state<S>(
    ui: &mut egui::Ui,
    id: &str,
    state: &mut S,
    body: impl FnOnce(&mut egui::Ui, &mut S),
    footer: impl FnOnce(&mut egui::Ui, &mut S),
) {
    egui::Panel::bottom(egui::Id::new(id).with("footer"))
        .exact_size(size::INSPECTOR_FOOTER)
        .resizable(false)
        .frame(egui::Frame::NONE)
        .show(ui, |ui| footer(ui, state));
    egui::ScrollArea::vertical()
        .id_salt(id)
        .show(ui, |ui| body(ui, state));
}

pub fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(space::GROUP);
    ui.separator();
    ui.add_space(space::FIELD);
    heading(ui, title);
}

pub fn heading(ui: &mut egui::Ui, title: &str) {
    ui.strong(title);
    ui.add_space(space::FIELD);
}

pub fn scalar(
    ui: &mut egui::Ui,
    label: &str,
    value: Option<f32>,
    range: std::ops::RangeInclusive<f32>,
) {
    egui_taffy::tui(ui, ui.id().with(label))
        .reserve_available_width()
        .style(layout::full(layout::column(space::FIELD)))
        .show(|tui| {
            super::field(tui, label, |ui| {
                let Some(mut value) = value else {
                    ui.weak("—");
                    return;
                };
                ui.add_enabled_ui(false, |ui| {
                    let width = (ui.available_width() - size::NUMERIC_FIELD - space::FIELD)
                        .max(size::CONTROL);
                    ui.horizontal(|ui| {
                        let minimum = f64::from(*range.start());
                        let mut relative = f64::from(value) - minimum;
                        ui.add(super::media_rail::Rail {
                            value: &mut relative,
                            maximum: f64::from(*range.end()) - minimum,
                            step: 0.01,
                            width,
                            compact: false,
                            label,
                        });
                        ui.add_sized(
                            [size::NUMERIC_FIELD, size::CONTROL],
                            egui::DragValue::new(&mut value),
                        );
                    });
                });
            });
        });
}

pub fn coordinates(ui: &mut egui::Ui, id: &str, values: &[(&str, Option<i64>)]) {
    egui_taffy::tui(ui, ui.id().with(id))
        .reserve_available_width()
        .style(layout::full(layout::grid(2, space::FIELD)))
        .show(|tui| {
            for (label, value) in values {
                layout::block(tui, |ui| {
                    ui.horizontal(|ui| {
                        let caption = ui.label(*label);
                        if let Some(mut value) = *value {
                            ui.add_enabled(false, egui::DragValue::new(&mut value).min_decimals(0))
                                .labelled_by(caption.id);
                        } else {
                            ui.weak("—");
                        }
                    });
                });
            }
        });
}

pub fn readonly(ui: &mut egui::Ui, label: &str, value: &str) {
    egui_taffy::tui(ui, ui.id().with(label))
        .reserve_available_width()
        .style(layout::full(layout::column(space::FIELD)))
        .show(|tui| {
            super::field(tui, label, |ui| {
                super::input::readonly(ui, value).on_hover_text(value);
            });
        });
}

pub fn unavailable_actions(ui: &mut egui::Ui, secondary: &str, primary: &str) {
    use super::button::{Button, Variant};
    ui.add_space(space::GROUP);
    egui_taffy::tui(ui, ui.id().with(primary))
        .reserve_available_width()
        .style(layout::full(layout::grid(2, space::FIELD)))
        .show(|tui| {
            for (label, variant) in [(secondary, Variant::Secondary), (primary, Variant::Primary)] {
                layout::block(tui, |ui| {
                    ui.add_enabled(false, Button::new(label, variant))
                        .on_disabled_hover_text("실제 명령 연결 필요");
                });
            }
        });
}
