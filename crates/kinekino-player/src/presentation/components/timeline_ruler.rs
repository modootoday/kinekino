use crate::presentation::design_tokens::{border, color, size, text};
use eframe::egui;

pub fn x(rect: egui::Rect, time: u64, duration: u64) -> f32 {
    egui::lerp(
        rect.x_range(),
        (time as f64 / duration.max(1) as f64).clamp(0., 1.) as f32,
    )
}

fn label(time: u64) -> String {
    let seconds = time / 1_000_000;
    if seconds >= 3600 {
        format!(
            "{:02}:{:02}:{:02}",
            seconds / 3600,
            seconds / 60 % 60,
            seconds % 60
        )
    } else {
        format!("{:02}:{:02}", seconds / 60, seconds % 60)
    }
}

pub fn paint(ui: &egui::Ui, rect: egui::Rect, duration: u64) {
    paint_labels(ui, rect, duration, false);
}

pub fn paint_below(ui: &egui::Ui, rect: egui::Rect, duration: u64) {
    paint_labels(ui, rect, duration, true);
}

fn paint_labels(ui: &egui::Ui, rect: egui::Rect, duration: u64, below: bool) {
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    if duration == 0 {
        painter.text(
            rect.left_top(),
            egui::Align2::LEFT_TOP,
            "시간 범위 연결 필요",
            egui::FontId::proportional(text::SMALL),
            color::SECONDARY,
        );
        return;
    }
    let count = (rect.width() / size::TIMELINE_TICK_WIDTH)
        .floor()
        .clamp(1., 20.) as u64;
    let minimum = duration.div_ceil(count);
    let step = [
        1_000_000,
        2_000_000,
        5_000_000,
        10_000_000,
        15_000_000,
        30_000_000,
        60_000_000,
        120_000_000,
        300_000_000,
        600_000_000,
        1_800_000_000,
        3_600_000_000,
    ]
    .into_iter()
    .find(|v| *v >= minimum)
    .unwrap_or(minimum);
    for index in 0..=duration / step {
        let time = index * step;
        let pos = x(rect, time, duration);
        painter.line_segment(
            [
                egui::pos2(pos, if below { rect.top() } else { rect.center().y }),
                egui::pos2(
                    pos,
                    if below {
                        rect.center().y
                    } else {
                        rect.bottom()
                    },
                ),
            ],
            egui::Stroke::new(border::THIN, color::BORDER),
        );
        let galley = painter.layout_no_wrap(
            label(time),
            egui::FontId::monospace(text::SMALL),
            color::SECONDARY,
        );
        let left = (pos - galley.size().x / 2.).clamp(
            rect.left(),
            (rect.right() - galley.size().x).max(rect.left()),
        );
        let top = if below {
            rect.bottom() - galley.size().y
        } else {
            rect.top()
        };
        painter.galley(egui::pos2(left, top), galley, color::SECONDARY);
    }
}

pub fn playhead(ui: &egui::Ui, rect: egui::Rect, duration: u64, position: u64) {
    if duration == 0 {
        return;
    }
    let pos = x(rect, position, duration);
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    painter.line_segment(
        [egui::pos2(pos, rect.top()), egui::pos2(pos, rect.bottom())],
        egui::Stroke::new(border::EMPHASIS, color::ACCENT),
    );
    let s = size::TIMELINE_MARKER;
    painter.add(egui::Shape::convex_polygon(
        vec![
            egui::pos2(pos - s, rect.top()),
            egui::pos2(pos + s, rect.top()),
            egui::pos2(pos, rect.top() + s),
        ],
        color::ACCENT,
        egui::Stroke::NONE,
    ));
}
