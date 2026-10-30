use crate::presentation::{
    design_tokens::{border, color, radius, size, space},
    icon_asset::Icon,
};
use eframe::egui::{self, Ui};

/// Connected rows share one outer border; callers insert separators between rows.
pub fn group<R>(ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R> {
    egui::Frame::new()
        .fill(color::PANEL)
        .stroke(egui::Stroke::new(border::THIN, color::BORDER))
        .corner_radius(radius::CONTROL)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.;
            ui.spacing_mut().interact_size.y = size::ACCORDION_GROUP_ROW;
            let widgets = &mut ui.visuals_mut().widgets;
            for state in [
                &mut widgets.inactive,
                &mut widgets.hovered,
                &mut widgets.active,
                &mut widgets.open,
            ] {
                state.bg_stroke = egui::Stroke::NONE;
                state.corner_radius = egui::CornerRadius::ZERO;
                state.expansion = 0.;
            }
            body(ui)
        })
}

/// Full-width group header; the native collapsing widget owns input and state.
pub fn show<R>(
    ui: &mut Ui,
    id: &str,
    title: &str,
    icon: Icon,
    default_open: bool,
    body: impl FnOnce(&mut Ui) -> R,
) {
    ui.scope(|ui| {
        let indent = size::CONTROL + size::TOOLBAR_ICON + space::FIELD;
        ui.spacing_mut().indent = indent;
        ui.spacing_mut().interact_size.y = ui.spacing().interact_size.y.max(size::CONTROL);
        ui.visuals_mut().collapsing_header_frame = true;
        let response = egui::CollapsingHeader::new(title)
            .id_salt(id)
            .default_open(default_open)
            .icon(move |ui, openness, response| {
                let left = response.rect.center().x - indent / 2.;
                let y = response.rect.center().y;
                let arrow = egui::Rect::from_center_size(
                    egui::pos2(left + space::FIELD + size::STATUS_ICON / 2., y),
                    egui::Vec2::splat(size::STATUS_ICON),
                );
                Icon::ChevronDown
                    .image()
                    .rotate(
                        -std::f32::consts::FRAC_PI_2 * (1. - openness),
                        egui::Vec2::splat(0.5),
                    )
                    .tint(color::TEXT)
                    .paint_at(ui, arrow);
                icon.paint(
                    ui,
                    egui::Rect::from_center_size(
                        egui::pos2(left + indent - space::FIELD - size::TOOLBAR_ICON / 2., y),
                        egui::Vec2::splat(size::TOOLBAR_ICON),
                    ),
                    color::TEXT,
                );
            })
            .show(ui, body);
        if response.header_response.clicked() {
            response.header_response.request_focus();
        }
        if response.header_response.has_focus() {
            ui.painter().rect_stroke(
                response.header_response.rect.shrink(border::THIN),
                radius::CONTROL,
                egui::Stroke::new(border::FOCUS, color::FOCUS),
                egui::StrokeKind::Inside,
            );
        }
    });
}
