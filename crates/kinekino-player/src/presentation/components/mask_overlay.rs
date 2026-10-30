use super::{canvas_transform::SourceTransform, selection_frame};
use crate::{
    application::workspace_read::{MaskSnapshot, ReadOrigin},
    presentation::{
        design_assets,
        design_tokens::{border, color, proportion},
    },
};
use eframe::egui;

/// Alpha-only view uses the same source canvas and embedded asset boundary.
pub fn paint_mask(ui: &mut egui::Ui, viewport: egui::Rect, mask: &MaskSnapshot) {
    if mask.origin != ReadOrigin::DesignSample {
        return;
    }
    let Some(transform) = SourceTransform::fit(
        egui::vec2(mask.width_px as f32, mask.height_px as f32),
        viewport,
        1.,
    ) else {
        return;
    };
    let mut overlay = ui.new_child(egui::UiBuilder::new().max_rect(transform.image_rect));
    overlay.set_clip_rect(transform.image_rect.intersect(ui.clip_rect()));
    overlay
        .painter()
        .rect_filled(transform.image_rect, 0., color::VIDEO_SURFACE);
    for subject in &mask.subjects {
        if let Some(asset) = subject
            .alpha_asset
            .as_ref()
            .and_then(|a| design_assets::find(&a.asset_id))
        {
            egui::Image::from_bytes(asset.uri, asset.bytes)
                .tint(color::IMAGE_TINT)
                .paint_at(&overlay, transform.image_rect);
        }
    }
}

/// Passive sample mask; asset paths never bypass the embedded allowlist.
pub fn paint(ui: &mut egui::Ui, viewport: egui::Rect, mask: &MaskSnapshot) {
    if mask.origin != ReadOrigin::DesignSample {
        return;
    }
    let Some(transform) = SourceTransform::fit(
        egui::vec2(mask.width_px as f32, mask.height_px as f32),
        viewport,
        1.,
    ) else {
        return;
    };
    let painter = ui
        .painter()
        .with_clip_rect(transform.image_rect.intersect(ui.clip_rect()));
    let mut overlay = ui.new_child(egui::UiBuilder::new().max_rect(transform.image_rect));
    overlay.set_clip_rect(painter.clip_rect());
    for subject in &mask.subjects {
        if let Some(asset) = subject
            .alpha_asset
            .as_ref()
            .and_then(|a| design_assets::find(&a.asset_id))
        {
            egui::Image::from_bytes(asset.uri, asset.bytes)
                .tint(color::SUBJECT_SELECTION.gamma_multiply(proportion::SUBJECT_MASK_OPACITY))
                .paint_at(&overlay, transform.image_rect);
        }
        if let Some(points) = &subject.polygon_px {
            if points.len() < 3 {
                continue;
            }
            let screen: Vec<_> = points
                .iter()
                .map(|p| transform.point(egui::pos2(p.x as f32, p.y as f32)))
                .collect();
            painter.add(egui::Shape::closed_line(
                screen.clone(),
                egui::Stroke::new(border::EMPHASIS, color::SUBJECT_SELECTION),
            ));
            let bounds = egui::Rect::from_points(&screen);
            selection_frame::paint_subject_bounds(&overlay, bounds.shrink(border::EMPHASIS));
        }
    }
}
