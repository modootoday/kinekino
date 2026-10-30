use crate::presentation::design_tokens::radius;
use crate::presentation::player_theme;
use eframe::egui;

pub fn paint(ui: &egui::Ui, rect: egui::Rect) {
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    for y in 0..(rect.height() / 16.).ceil() as u32 {
        for x in 0..(rect.width() / 16.).ceil() as u32 {
            let tile = egui::Rect::from_min_size(
                rect.min + egui::vec2(x as f32 * 16., y as f32 * 16.),
                egui::vec2(16., 16.),
            )
            .intersect(rect);
            painter.rect_filled(
                tile,
                radius::NONE,
                if (x + y) % 2 == 0 {
                    player_theme::SELECTED
                } else {
                    player_theme::PANEL
                },
            );
        }
    }
}
