use super::*;

impl DemoAsset {
    /// Opaque grayscale of the immutable embedded PNG's stored alpha channel.
    pub fn paint_alpha(&self, ui: &mut egui::Ui, image_rect: egui::Rect, clip: egui::Rect) -> bool {
        let id = egui::Id::new(("embedded-alpha-preview", self.uri));
        let cached = ui
            .ctx()
            .data_mut(|data| data.get_temp::<egui::TextureHandle>(id));
        let texture = if let Some(texture) = cached {
            texture
        } else {
            let Ok(decoded) = image::load_from_memory(self.bytes) else {
                return false;
            };
            let rgba = decoded.to_rgba8();
            let pixels = rgba
                .pixels()
                .map(|pixel| egui::Color32::from_gray(pixel[3]))
                .collect();
            let texture = ui.ctx().load_texture(
                format!("alpha:{}", self.uri),
                egui::ColorImage::new([rgba.width() as usize, rgba.height() as usize], pixels),
                egui::TextureOptions::LINEAR,
            );
            ui.ctx()
                .data_mut(|data| data.insert_temp(id, texture.clone()));
            texture
        };
        let mut overlay = ui.new_child(egui::UiBuilder::new().max_rect(clip));
        overlay.set_clip_rect(clip.intersect(ui.clip_rect()));
        egui::Image::new((texture.id(), texture.size_vec2())).paint_at(&overlay, image_rect);
        true
    }
}
