use crate::presentation::{
    components::artboard::{self, Artwork},
    design_tokens::{border, color, size, text},
};
use eframe::egui::{self, Ui};

pub struct Region<'a> {
    pub label: &'a str,
    pub start: u64,
    pub end: u64,
}

pub fn show(
    ui: &mut Ui,
    asset_id: Option<&str>,
    duration: u64,
    position: u64,
    regions: &[Region<'_>],
) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(
            ui.available_width(),
            size::FRAME_STRIP_HEIGHT + size::TIMELINE_RULER,
        ),
        egui::Sense::hover(),
    );
    let ruler = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), size::TIMELINE_RULER));
    super::timeline_ruler::paint(ui, ruler, duration);
    let body = egui::Rect::from_min_max(ruler.left_bottom(), rect.right_bottom());
    paint(ui, body, asset_id, duration, regions);
    super::timeline_ruler::playhead(ui, rect, duration, position);
}

pub fn paint(
    ui: &mut Ui,
    rect: egui::Rect,
    asset_id: Option<&str>,
    duration: u64,
    regions: &[Region<'_>],
) {
    let body = rect.shrink2(egui::vec2(0., border::FOCUS));
    if asset_id.is_none() {
        artboard::paint(
            ui,
            body,
            &Artwork {
                asset_id: None,
                title: "",
                subtitle: "",
                cover: false,
                grid: false,
            },
        );
        return;
    }
    for n in 0..12 {
        let image = egui::Rect::from_min_max(
            egui::pos2(egui::lerp(rect.x_range(), n as f32 / 12.), body.top()),
            egui::pos2(
                egui::lerp(rect.x_range(), (n + 1) as f32 / 12.),
                body.bottom(),
            ),
        );
        artboard::paint(
            ui,
            image,
            &Artwork {
                asset_id,
                title: "",
                subtitle: "",
                cover: true,
                grid: false,
            },
        );
    }
    let total = duration.max(1) as f64;
    for region in regions {
        let x1 = egui::lerp(
            rect.x_range(),
            (region.start as f64 / total).clamp(0., 1.) as f32,
        );
        let x2 = egui::lerp(
            rect.x_range(),
            (region.end as f64 / total).clamp(0., 1.) as f32,
        );
        let bounds =
            egui::Rect::from_min_max(egui::pos2(x1, body.top()), egui::pos2(x2, body.bottom()));
        let painter = ui.painter().with_clip_rect(body.intersect(ui.clip_rect()));
        painter.rect_stroke(
            bounds,
            crate::presentation::design_tokens::radius::SMALL,
            egui::Stroke::new(border::FOCUS, color::ACCENT),
            egui::StrokeKind::Inside,
        );
        let grip_width = size::TIMELINE_RANGE_GRIP_WIDTH.min(bounds.width().max(0.));
        for x in [x1, x2] {
            let grip = egui::Rect::from_center_size(
                egui::pos2(x, body.center().y),
                egui::vec2(
                    grip_width,
                    size::TIMELINE_RANGE_GRIP_HEIGHT.min(body.height()),
                ),
            );
            painter.rect_filled(
                grip,
                crate::presentation::design_tokens::radius::SMALL,
                color::ACCENT,
            );
            painter.line_segment(
                [
                    egui::pos2(x, grip.top() + grip.height() / 4.),
                    egui::pos2(x, grip.bottom() - grip.height() / 4.),
                ],
                egui::Stroke::new(border::THIN, color::TEXT),
            );
        }
        let label = ui.painter().layout_no_wrap(
            region.label.to_owned(),
            egui::FontId::proportional(text::SMALL),
            color::TEXT,
        );
        if label.size().x <= bounds.width() - grip_width {
            painter
                .with_clip_rect(bounds.intersect(painter.clip_rect()))
                .galley(bounds.center() - label.size() / 2., label, color::TEXT);
        }
        ui.interact(
            bounds,
            ui.id().with((region.start, region.end, region.label)),
            egui::Sense::hover(),
        )
        .on_hover_text(region.label);
    }
}
