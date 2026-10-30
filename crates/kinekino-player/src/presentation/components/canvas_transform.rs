//! Coordinate conversion for rendering source-pixel overlays in a canvas viewport.
use eframe::egui;

pub struct SourceTransform {
    pub image_rect: egui::Rect,
    scale: f32,
}

impl SourceTransform {
    pub fn fit(source_size: egui::Vec2, viewport: egui::Rect, zoom: f32) -> Option<Self> {
        if source_size.x <= 0.
            || source_size.y <= 0.
            || !source_size.is_finite()
            || !zoom.is_finite()
            || zoom <= 0.
        {
            return None;
        }
        let ratios = viewport.size() / source_size;
        let scale = ratios.x.min(ratios.y) * zoom;
        Self::at_scale(source_size, viewport, scale)
    }

    pub fn at_scale(source_size: egui::Vec2, viewport: egui::Rect, scale: f32) -> Option<Self> {
        if source_size.x <= 0.
            || source_size.y <= 0.
            || !source_size.is_finite()
            || !scale.is_finite()
            || scale <= 0.
        {
            return None;
        }
        Some(Self {
            image_rect: egui::Rect::from_center_size(viewport.center(), source_size * scale),
            scale,
        })
    }

    pub fn translate(&mut self, offset: egui::Vec2) {
        self.image_rect = self.image_rect.translate(offset);
    }

    pub fn point(&self, source: egui::Pos2) -> egui::Pos2 {
        self.image_rect.min + source.to_vec2() * self.scale
    }

    pub fn bounds(&self, source: egui::Rect) -> egui::Rect {
        egui::Rect::from_min_max(self.point(source.min), self.point(source.max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portrait_source_bounds_and_pivot_share_the_same_transform() {
        let viewport = egui::Rect::from_min_size(egui::pos2(100., 50.), egui::vec2(800., 450.));
        let transform = SourceTransform::fit(egui::vec2(1000., 1500.), viewport, 1.).unwrap();
        assert_eq!(transform.image_rect.size(), egui::vec2(300., 450.));
        assert_eq!(transform.point(egui::pos2(500., 750.)), viewport.center());
        let bounds = transform.bounds(egui::Rect::from_min_size(
            egui::pos2(100., 200.),
            egui::vec2(800., 1200.),
        ));
        assert_eq!(bounds.size(), egui::vec2(240., 360.));
        let zoomed = SourceTransform::fit(egui::vec2(1000., 1500.), viewport, 2.).unwrap();
        assert_eq!(zoomed.point(egui::pos2(500., 750.)), viewport.center());
        assert_eq!(
            zoomed
                .bounds(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800., 1200.)
                ))
                .size(),
            egui::vec2(480., 720.)
        );
        assert!(SourceTransform::fit(egui::Vec2::ZERO, viewport, 1.).is_none());
    }
}
