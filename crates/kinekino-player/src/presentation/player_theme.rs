pub use super::design_tokens::color::{
    ACCENT, BORDER, CHROME, FOCUS, IDLE_MARK, PANEL, SECONDARY, SELECTED, TEXT,
};
use super::design_tokens::{border, color, radius, size, space, text};
use eframe::egui::{self, FontId, TextStyle};
pub const CONTROL_SIZE: f32 = size::CONTROL;

pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = CHROME;
    style.visuals.window_fill = PANEL;
    style.visuals.extreme_bg_color = color::INPUT_SURFACE;
    style.visuals.override_text_color = Some(TEXT);
    style.visuals.weak_text_color = Some(SECONDARY);
    style.visuals.window_corner_radius = egui::CornerRadius::same(radius::WINDOW);
    style.visuals.window_stroke = egui::Stroke::new(border::THIN, BORDER);
    style.visuals.selection.bg_fill = SELECTED;
    style.visuals.selection.stroke = egui::Stroke::new(border::THIN, ACCENT);
    style.visuals.slider_trailing_fill = true;
    let widgets = &mut style.visuals.widgets;
    for (state, fill, edge) in [
        (&mut widgets.noninteractive, PANEL, BORDER),
        (&mut widgets.inactive, color::FIELD, BORDER),
        (&mut widgets.hovered, color::HOVER, SECONDARY),
        (&mut widgets.active, SELECTED, ACCENT),
        (&mut widgets.open, color::HOVER, ACCENT),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = egui::Stroke::new(border::THIN, edge);
        state.fg_stroke = egui::Stroke::new(border::THIN, TEXT);
        state.corner_radius = egui::CornerRadius::same(radius::CONTROL);
        state.expansion = 0.;
    }
    style.spacing.scroll = egui::style::ScrollStyle {
        bar_width: 6.,
        ..egui::style::ScrollStyle::solid()
    };
    style.spacing.item_spacing = egui::vec2(space::INLINE, space::TIGHT);
    style.spacing.button_padding = egui::vec2(9., 5.);
    style.spacing.interact_size = egui::vec2(size::CONTROL, size::CONTROL);
    style.spacing.window_margin = egui::Margin::same(space::PANEL_MARGIN);
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(text::HEADING));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(text::BODY));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(text::BODY));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(text::SMALL));
    ctx.set_style_of(egui::Theme::Dark, style);
}
