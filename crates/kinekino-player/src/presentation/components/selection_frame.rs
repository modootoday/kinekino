//! Source-space selection decoration, shared by canvas-based editors.
use crate::presentation::design_tokens::{border, color, radius, size};
use crate::presentation::player_theme;
use eframe::egui;

pub fn paint(ui: &egui::Ui, bounds: egui::Rect, pivot: egui::Pos2) {
    paint_bounds(ui, bounds);
    paint_pivot(ui, pivot);
}

pub fn paint_bounds(ui: &egui::Ui, bounds: egui::Rect) {
    paint_square_bounds(ui, bounds, color::CANVAS_SELECTION);
}

/// Passive subject bounds, distinct from an asset transform or OCR region.
pub fn paint_subject_bounds(ui: &egui::Ui, bounds: egui::Rect) {
    paint_square_bounds(ui, bounds, color::SUBJECT_SELECTION);
}

fn paint_square_bounds(ui: &egui::Ui, bounds: egui::Rect, accent: egui::Color32) {
    ui.painter().rect_stroke(
        bounds,
        radius::NONE,
        egui::Stroke::new(border::EMPHASIS, accent),
        egui::StrokeKind::Inside,
    );
    for center in [
        bounds.left_top(),
        bounds.center_top(),
        bounds.right_top(),
        bounds.left_center(),
        bounds.right_center(),
        bounds.left_bottom(),
        bounds.center_bottom(),
        bounds.right_bottom(),
    ] {
        let handle = egui::Rect::from_center_size(
            center,
            egui::vec2(size::SELECTION_HANDLE, size::SELECTION_HANDLE),
        );
        ui.painter()
            .rect_filled(handle, radius::NONE, player_theme::TEXT);
        ui.painter().rect_stroke(
            handle,
            radius::NONE,
            egui::Stroke::new(border::EMPHASIS, accent),
            egui::StrokeKind::Inside,
        );
    }
}

/// Passive OCR region preview; does not imply that resize commands are available.
pub fn paint_ocr_region(ui: &egui::Ui, bounds: egui::Rect) {
    let stroke = egui::Stroke::new(border::EMPHASIS, color::OCR_REGION);
    ui.painter()
        .rect_stroke(bounds, radius::NONE, stroke, egui::StrokeKind::Inside);
    for center in [
        bounds.left_top(),
        bounds.right_top(),
        bounds.left_bottom(),
        bounds.right_bottom(),
    ] {
        ui.painter()
            .circle_filled(center, size::REGION_HANDLE_RADIUS, color::REGION_HANDLE);
        ui.painter()
            .circle_stroke(center, size::REGION_HANDLE_RADIUS, stroke);
    }
}

fn paint_pivot(ui: &egui::Ui, pivot: egui::Pos2) {
    let accent = egui::Stroke::new(border::EMPHASIS, player_theme::ACCENT);
    ui.painter().circle_stroke(pivot, 7., accent);
    ui.painter().line_segment(
        [pivot - egui::vec2(12., 0.), pivot + egui::vec2(12., 0.)],
        accent,
    );
    ui.painter().line_segment(
        [pivot - egui::vec2(0., 12.), pivot + egui::vec2(0., 12.)],
        accent,
    );
}
