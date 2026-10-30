use crate::presentation::design_tokens::{border, color, size, space};
use eframe::egui;

/// Read-only palette preview. An absent value never selects a default color.
pub fn unavailable(ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = space::INLINE;
        for (label, value) in color::TEXT_SWATCHES {
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(size::COLOR_SWATCH, size::COLOR_SWATCH),
                egui::Sense::hover(),
            );
            ui.painter()
                .circle_filled(rect.center(), size::COLOR_SWATCH / 2., value);
            ui.painter().circle_stroke(
                rect.center(),
                size::COLOR_SWATCH / 2.,
                egui::Stroke::new(border::THIN, color::BORDER),
            );
            response
                .on_hover_text(format!("{label} · 색상표 예시 · 실제 선택값 연결 필요"))
                .widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::ColorButton, false, label)
                });
        }
    });
    ui.weak("선택값 연결 필요");
}
