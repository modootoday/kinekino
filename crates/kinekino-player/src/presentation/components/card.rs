use crate::presentation::design_tokens::{border, color, radius, space};
use eframe::egui::{self, InnerResponse, Ui};

pub fn show<R>(ui: &mut Ui, title: &str, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    frame(color::PANEL).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        if !title.is_empty() {
            ui.strong(title);
            ui.add_space(space::FIELD);
        }
        body(ui)
    })
}

fn frame(fill: egui::Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .stroke(egui::Stroke::new(border::THIN, color::BORDER))
        .corner_radius(radius::CONTROL)
        .inner_margin(space::PANEL_MARGIN)
}

pub fn selectable<R>(
    ui: &mut Ui,
    id: &str,
    label: &str,
    selected: bool,
    body: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let style = frame(if selected {
        color::SELECTED
    } else {
        color::PANEL
    });
    selectable_frame(ui, id, label, selected, style, body)
}

pub fn selectable_row<R>(
    ui: &mut Ui,
    id: &str,
    label: &str,
    selected: bool,
    body: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let style = egui::Frame::new()
        .fill(if selected {
            color::SELECTED
        } else {
            color::PANEL
        })
        .inner_margin(space::PANEL_MARGIN);
    let result = selectable_frame(ui, id, label, selected, style, body);
    ui.painter().line_segment(
        [
            result.response.rect.left_bottom(),
            result.response.rect.right_bottom(),
        ],
        egui::Stroke::new(border::THIN, color::BORDER),
    );
    result
}

fn selectable_frame<R>(
    ui: &mut Ui,
    id: &str,
    label: &str,
    selected: bool,
    style: egui::Frame,
    body: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let inner = style.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        body(ui)
    });
    let response = ui.interact(
        inner.response.rect,
        ui.id().with(("selectable-card", id)),
        egui::Sense::click(),
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            ui.is_enabled(),
            selected,
            label,
        )
    });
    if selected {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                response.rect.min,
                egui::vec2(border::FOCUS, response.rect.height()),
            ),
            radius::SMALL,
            color::ACCENT,
        );
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect.shrink(border::THIN),
            radius::CONTROL,
            egui::Stroke::new(border::FOCUS, color::FOCUS),
            egui::StrokeKind::Inside,
        );
    }
    InnerResponse {
        inner: inner.inner,
        response,
    }
}
