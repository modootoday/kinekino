use crate::presentation::design_tokens::{border, color, size, space, text};
use eframe::egui::{self, Ui};

pub fn show(ui: &mut Ui, labels: &[&str], current: Option<usize>) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), size::SOURCE_THUMBNAIL),
        egui::Sense::hover(),
    );
    let step = rect.width() / labels.len().max(1) as f32;
    let y = rect.top() + size::CONTROL / 2.;
    for (index, label) in labels.iter().enumerate() {
        let point = egui::pos2(rect.left() + step * (index as f32 + 0.5), y);
        let selected = current == Some(index);
        if index + 1 < labels.len() {
            ui.painter().line_segment(
                [point, point + egui::vec2(step, 0.)],
                egui::Stroke::new(border::THIN, color::BORDER),
            );
        }
        ui.painter()
            .circle_filled(point, size::CONTROL / 2. - space::TIGHT, color::PANEL);
        ui.painter().circle_stroke(
            point,
            size::CONTROL / 2. - space::TIGHT,
            egui::Stroke::new(
                border::FOCUS,
                if selected {
                    color::ACCENT
                } else {
                    color::BORDER
                },
            ),
        );
        ui.painter().text(
            point,
            egui::Align2::CENTER_CENTER,
            format!("{}", index + 1),
            egui::FontId::proportional(text::BODY),
            if selected {
                color::ACCENT
            } else {
                color::SECONDARY
            },
        );
        ui.painter().text(
            point + egui::vec2(0., size::CONTROL),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(text::SMALL),
            if selected {
                color::ACCENT
            } else {
                color::SECONDARY
            },
        );
    }
}
