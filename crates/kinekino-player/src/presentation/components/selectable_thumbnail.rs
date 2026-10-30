use super::artboard::{self, Artwork};
use crate::presentation::design_tokens::{border, color, radius, space};
use eframe::egui::{self, Rect, Response, Ui};

pub struct ThumbnailResponse {
    pub response: Response,
    pub image: Rect,
}

/// Stable footprint separates selection, image, and keyboard focus furniture.
pub fn show(
    ui: &mut Ui,
    artwork: &Artwork<'_>,
    aspect: f32,
    height: f32,
    selected: bool,
    text_bounds: Option<fn(Rect) -> Rect>,
) -> ThumbnailResponse {
    let (slot, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::click(),
    );
    let frame = artboard::fitted_rect(slot, aspect);
    let image = if frame.width().min(frame.height()) > space::TIGHT * 2. {
        artboard::fitted_rect(frame.shrink(space::TIGHT), aspect)
    } else {
        frame
    };
    if response.hovered() || response.is_pointer_button_down_on() {
        ui.painter().rect_filled(
            frame,
            radius::CONTROL,
            if response.is_pointer_button_down_on() {
                color::PRESSED
            } else {
                color::HOVER
            },
        );
    }
    artboard::paint_with_text_bounds(ui, image, artwork, text_bounds);
    ui.painter().rect_stroke(
        frame,
        radius::CONTROL,
        egui::Stroke::new(
            if selected {
                border::FOCUS
            } else {
                border::THIN
            },
            if selected {
                color::ACCENT
            } else {
                color::BORDER
            },
        ),
        egui::StrokeKind::Inside,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            image.shrink(border::FOCUS * 2.),
            radius::CONTROL,
            egui::Stroke::new(border::THIN, color::FOCUS),
            egui::StrokeKind::Inside,
        );
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            ui.is_enabled(),
            selected,
            artwork.title,
        )
    });
    ThumbnailResponse {
        response: response.on_hover_text(format!("{}\n{}", artwork.title, artwork.subtitle)),
        image,
    }
}
