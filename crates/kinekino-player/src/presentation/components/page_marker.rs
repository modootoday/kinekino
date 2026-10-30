use crate::presentation::design_tokens::{color, proportion, radius, size, space, text};
use eframe::egui::{self, Rect, Ui};

/// Page furniture has the same reserved proportion in the canvas and its thumbnail.
pub fn text_bounds(image: Rect) -> Rect {
    let band = (size::PAGE_NUMBER_BADGE + space::TIGHT * 2.) / size::CARD_RAIL * image.width();
    Rect::from_center_size(
        image.center(),
        egui::vec2(
            image.width() * proportion::ARTWORK_TEXT_WIDTH,
            (image.height() - band * 2.).max(1.),
        ),
    )
}

pub fn badge(ui: &Ui, image: Rect, number: usize) {
    let painter = ui.painter().with_clip_rect(image.intersect(ui.clip_rect()));
    let rect = Rect::from_min_size(
        image.min + egui::vec2(space::TIGHT, space::TIGHT),
        egui::vec2(size::PAGE_NUMBER_BADGE, size::PAGE_NUMBER_BADGE),
    );
    painter.rect_filled(rect, radius::CONTROL, color::CHROME);
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        number.to_string(),
        egui::FontId::proportional(text::BODY),
        color::TEXT,
    );
}

pub fn counter(ui: &Ui, image: Rect, number: usize, total: usize) {
    ui.painter()
        .with_clip_rect(image.intersect(ui.clip_rect()))
        .text(
            image.center_bottom() - egui::vec2(0., space::GROUP),
            egui::Align2::CENTER_BOTTOM,
            format!("{number} / {total}"),
            egui::FontId::proportional(text::BODY),
            color::TEXT,
        );
}
