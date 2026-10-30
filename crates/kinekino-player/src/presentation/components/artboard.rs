use crate::presentation::{
    design_assets,
    design_tokens::{border, color, proportion, radius, text},
};
use eframe::egui::{self, Rect, Response, Ui};

pub struct Artwork<'a> {
    pub asset_id: Option<&'a str>,
    pub title: &'a str,
    pub subtitle: &'a str,
    pub cover: bool,
    pub grid: bool,
}

/// A contain preview leaves room for the timeline and results below it.
pub fn media_height(width: f32, viewport_height: f32, aspect: f32, minimum: f32) -> f32 {
    let budget = (viewport_height * proportion::MEDIA_PREVIEW_VIEWPORT).max(minimum);
    (width / aspect).min(budget).max(1.)
}

pub fn fitted_rect(slot: Rect, aspect: f32) -> Rect {
    let width = slot.width().min(slot.height() * aspect).max(1.);
    Rect::from_center_size(
        slot.center(),
        egui::vec2(width, (width / aspect).min(slot.height())),
    )
}

pub fn show(
    ui: &mut Ui,
    artwork: &Artwork<'_>,
    aspect: f32,
    height: f32,
    selected: bool,
) -> Response {
    show_with_text_bounds(ui, artwork, aspect, height, selected, None)
}

pub fn show_with_text_bounds(
    ui: &mut Ui,
    artwork: &Artwork<'_>,
    aspect: f32,
    height: f32,
    selected: bool,
    text_bounds: Option<fn(Rect) -> Rect>,
) -> Response {
    let (slot, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::click(),
    );
    let rect = fitted_rect(slot, aspect);
    paint_with_text_bounds(ui, rect, artwork, text_bounds);
    if selected {
        ui.painter().rect_stroke(
            rect,
            radius::CONTROL,
            egui::Stroke::new(border::FOCUS, color::ACCENT),
            egui::StrokeKind::Inside,
        );
    }
    if artwork.title.is_empty() {
        response
    } else {
        response.on_hover_text(format!("{}\n{}", artwork.title, artwork.subtitle))
    }
}

pub fn paint(ui: &mut Ui, rect: Rect, artwork: &Artwork<'_>) {
    paint_with_text_bounds(ui, rect, artwork, None);
}

/// Static source thumbnail with the shared rounded image boundary.
pub fn thumbnail(ui: &mut Ui, rect: Rect, artwork: &Artwork<'_>) {
    if let Some(asset) = artwork.asset_id.and_then(design_assets::find) {
        if asset.transparent {
            super::checkerboard::paint(ui, rect);
        }
        egui::Image::from_bytes(asset.uri, asset.bytes)
            .corner_radius(radius::CONTROL)
            .paint_at(ui, fitted_rect(rect, asset.size.x / asset.size.y));
    } else {
        paint(ui, rect, artwork);
    }
}

pub(super) fn paint_with_text_bounds(
    ui: &mut Ui,
    rect: Rect,
    artwork: &Artwork<'_>,
    text_bounds: Option<fn(Rect) -> Rect>,
) {
    ui.painter()
        .rect_filled(rect, radius::CONTROL, color::VIDEO_SURFACE);
    if let Some(asset) = artwork.asset_id.and_then(design_assets::find) {
        if artwork.cover {
            let ratio = rect.size() / asset.size;
            let image = Rect::from_center_size(rect.center(), asset.size * ratio.x.max(ratio.y));
            asset.paint_in_rect(ui, image, rect);
        } else {
            asset.paint(ui, rect);
        }
    } else {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "소스 이미지 연결 필요",
            egui::FontId::proportional(text::BODY),
            color::SECONDARY,
        );
    }
    let painter = ui.painter().with_clip_rect(rect);
    if artwork.grid {
        for n in 1..3 {
            let x = egui::lerp(rect.x_range(), n as f32 / 3.);
            let y = egui::lerp(rect.y_range(), n as f32 / 3.);
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(border::THIN, color::SECONDARY),
            );
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(border::THIN, color::SECONDARY),
            );
        }
        painter.rect_stroke(
            rect,
            radius::NONE,
            egui::Stroke::new(border::EMPHASIS, color::TEXT),
            egui::StrokeKind::Inside,
        );
    }
    if !artwork.title.is_empty() {
        if let Some(bounds) = text_bounds {
            super::artboard_text::paint_in_bounds(
                ui,
                rect,
                bounds(rect),
                artwork.title,
                artwork.subtitle,
            );
        } else {
            super::artboard_text::paint(ui, rect, artwork.title, artwork.subtitle);
        }
    }
}
