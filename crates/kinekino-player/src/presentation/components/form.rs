use crate::presentation::design_tokens::{size, space};
use crate::presentation::layout;
use eframe::egui;
use egui_taffy::taffy::prelude::{auto, length, percent};
use egui_taffy::taffy::{Size, Style};
use egui_taffy::{Tui, TuiBuilderLogic};

/// Consistent label/control proportions across every inspector.
pub fn field<T>(tui: &mut Tui, label: &str, body: impl FnOnce(&mut egui::Ui) -> T) -> T {
    tui.style(layout::row(space::PANEL)).add(|tui| {
        tui.style(Style {
            flex_basis: percent(0.36_f32),
            flex_shrink: 0.,
            min_size: Size {
                width: length(size::FIELD_LABEL_MIN),
                height: auto(),
            },
            ..Default::default()
        })
        .ui(|ui| {
            ui.add(egui::Label::new(label).wrap());
        });
        layout::block(tui, body)
    })
}
