use eframe::egui;
use egui_taffy::taffy::prelude::{auto, length};
use egui_taffy::taffy::{Size, Style};
use egui_taffy::{Tui, TuiBuilderLogic, TuiContainerResponse};

/// A shrinkable layout leaf. Its contents determine height, never the parent width.
pub fn block<T>(tui: &mut Tui, body: impl FnOnce(&mut egui::Ui) -> T) -> T {
    tui.style(Style {
        min_size: Size {
            width: length(0.0_f32),
            height: auto(),
        },
        flex_grow: 1.,
        flex_shrink: 1.,
        ..Default::default()
    })
    .ui_manual(|ui, _| {
        let inner = body(ui);
        let used = ui.min_size();
        TuiContainerResponse {
            inner,
            min_size: egui::vec2(0., used.y),
            intrinsic_size: None,
            max_size: used,
            infinite: egui::Vec2b::FALSE,
        }
    })
}

/// A content-sized control that must not collapse when a neighbouring label grows.
pub fn widget<T>(tui: &mut Tui, body: impl FnOnce(&mut egui::Ui) -> T) -> T {
    tui.style(Style {
        flex_shrink: 0.,
        ..Default::default()
    })
    .wrap_mode(egui::TextWrapMode::Extend)
    .ui(body)
}
