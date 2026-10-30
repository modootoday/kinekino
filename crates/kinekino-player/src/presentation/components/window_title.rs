//! Shared title content; the surrounding shell owns drag and caption actions.
use crate::presentation::design_tokens::{border, size, space, text};
use crate::presentation::player_theme;
use eframe::egui::{self, Rect, Ui};

pub fn paint(ui: &Ui, rect: Rect, product: &str, media: &str) {
    let painter = ui.painter().with_clip_rect(rect);
    let font = egui::FontId::proportional(text::BODY);
    let brand = painter.layout_no_wrap(product.into(), font.clone(), player_theme::TEXT);
    let brand_width = brand.size().x.min(rect.width());
    painter.galley(
        rect.left_center() - egui::vec2(0., brand.size().y / 2.),
        brand,
        player_theme::TEXT,
    );
    if media.is_empty() || rect.width() < brand_width + size::CONTROL + space::GROUP {
        return;
    }
    let separator = rect.left() + brand_width + space::PANEL;
    painter.line_segment(
        [
            egui::pos2(separator, rect.center().y - space::FIELD),
            egui::pos2(separator, rect.center().y + space::FIELD),
        ],
        egui::Stroke::new(border::THIN, player_theme::BORDER),
    );
    let left = separator + space::PANEL;
    let title = egui::WidgetText::from(egui::RichText::new(media).font(font)).into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        (rect.right() - left).max(0.),
        egui::TextStyle::Body,
    );
    painter.galley(
        egui::pos2(left, rect.center().y - title.size().y / 2.),
        title,
        player_theme::SECONDARY,
    );
}
