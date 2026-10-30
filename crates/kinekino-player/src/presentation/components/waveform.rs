use super::timeline_ruler;
use crate::presentation::design_tokens::{border, color, proportion, radius, size, text};
use eframe::egui;

/// Normalized amplitude projection. The caller owns provenance and sample selection.
pub fn show(
    ui: &mut egui::Ui,
    amplitudes: &[f32],
    duration: u64,
    position: u64,
    selection: Option<(u64, u64)>,
) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(
            ui.available_width(),
            size::WAVEFORM_HEIGHT + size::TIMELINE_RULER,
        ),
        egui::Sense::hover(),
    );
    let body = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), size::WAVEFORM_HEIGHT));
    let ruler = egui::Rect::from_min_max(body.left_bottom(), rect.right_bottom());
    timeline_ruler::paint_below(ui, ruler, duration);
    let painter = ui.painter().with_clip_rect(body.intersect(ui.clip_rect()));
    painter.rect_filled(body, radius::CONTROL, color::FIELD);
    painter.rect_stroke(
        body,
        radius::CONTROL,
        egui::Stroke::new(border::THIN, color::BORDER),
        egui::StrokeKind::Inside,
    );
    if amplitudes.is_empty() {
        painter.text(
            body.center(),
            egui::Align2::CENTER_CENTER,
            "오디오 파형 연결 필요",
            egui::FontId::proportional(text::SMALL),
            color::SECONDARY,
        );
        return;
    }
    painter.line_segment(
        [body.left_center(), body.right_center()],
        egui::Stroke::new(border::THIN, color::DISABLED),
    );
    let step = body.width() / amplitudes.len() as f32;
    for (index, amplitude) in amplitudes.iter().enumerate() {
        let height = if amplitude.is_finite() {
            amplitude.clamp(0., 1.)
        } else {
            0.
        };
        let x = body.left() + (index as f32 + 0.5) * step;
        let extent = height * (body.height() - border::FOCUS * 2.) / 2.;
        painter.line_segment(
            [
                egui::pos2(x, body.center().y - extent),
                egui::pos2(x, body.center().y + extent),
            ],
            egui::Stroke::new(border::THIN, color::DISABLED),
        );
    }
    if let Some((start, end)) =
        selection.filter(|(start, end)| duration > 0 && start < end && *start < duration)
    {
        let x1 = timeline_ruler::x(body, start, duration);
        let x2 = timeline_ruler::x(body, end, duration);
        let bounds =
            egui::Rect::from_min_max(egui::pos2(x1, body.top()), egui::pos2(x2, body.bottom()))
                .shrink2(egui::vec2(0., border::FOCUS));
        painter.rect_filled(
            bounds,
            radius::CONTROL,
            color::ACCENT.gamma_multiply(proportion::TRACK_FILL_OPACITY),
        );
        for x in [x1, x2] {
            painter.rect_filled(
                egui::Rect::from_center_size(
                    egui::pos2(x, body.center().y),
                    egui::vec2(
                        size::WAVEFORM_RANGE_GRIP,
                        size::TIMELINE_RANGE_GRIP_HEIGHT.min(body.height()),
                    ),
                ),
                radius::SMALL,
                color::ACCENT,
            );
        }
    }
    if duration > 0 {
        let x = timeline_ruler::x(body, position, duration);
        painter.line_segment(
            [egui::pos2(x, body.top()), egui::pos2(x, body.bottom())],
            egui::Stroke::new(border::EMPHASIS, color::ACCENT),
        );
    }
}
