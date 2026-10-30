use crate::presentation::design_tokens::{color, proportion, size, space, text};
use eframe::egui::{self, Galley, Rect, Ui};
use std::sync::Arc;

fn layout(ui: &Ui, value: &str, font_size: f32, width: f32) -> Arc<Galley> {
    let mut job = egui::text::LayoutJob::simple(
        value.into(),
        egui::FontId::proportional(font_size),
        color::TEXT,
        width,
    );
    job.halign = egui::Align::Center;
    ui.painter().layout_job(job)
}

fn text_bounds(rect: Rect) -> Rect {
    Rect::from_center_size(
        rect.center(),
        rect.size()
            * egui::vec2(
                proportion::ARTWORK_TEXT_WIDTH,
                proportion::ARTWORK_TEXT_HEIGHT,
            ),
    )
}

fn fit(
    ui: &Ui,
    rect: Rect,
    bounds: Rect,
    title: &str,
    subtitle: &str,
) -> (Arc<Galley>, Arc<Galley>, f32) {
    let scale = (rect.width() / size::PREVIEW_HEIGHT).min(1.);
    let width = bounds.width();
    let height = bounds.height();
    let gap = space::FIELD * scale;
    let mut factor = 1.;
    loop {
        let title = layout(ui, title, text::ARTWORK_TITLE * scale * factor, width);
        let subtitle = layout(ui, subtitle, text::ARTWORK_SUBTITLE * scale * factor, width);
        let used = title.size().y + subtitle.size().y + gap;
        if used <= height || factor <= text::ARTWORK_MIN_SCALE {
            return (title, subtitle, gap);
        }
        factor = (factor * 0.9).max(text::ARTWORK_MIN_SCALE);
    }
}

pub(super) fn paint(ui: &Ui, rect: Rect, title: &str, subtitle: &str) {
    paint_in_bounds(ui, rect, text_bounds(rect), title, subtitle);
}

pub(super) fn paint_in_bounds(ui: &Ui, rect: Rect, bounds: Rect, title: &str, subtitle: &str) {
    let (title, subtitle, gap) = fit(ui, rect, bounds, title, subtitle);
    let height = title.size().y + subtitle.size().y + gap;
    let top = bounds.center().y - height / 2.;
    let painter = ui
        .painter()
        .with_clip_rect(bounds.intersect(ui.clip_rect()));
    painter.galley(
        egui::pos2(origin_x(bounds, &title), top),
        title.clone(),
        color::TEXT,
    );
    painter.galley(
        egui::pos2(origin_x(bounds, &subtitle), top + title.size().y + gap),
        subtitle,
        color::TEXT,
    );
    if height > bounds.height() {
        painter.text(
            bounds.center_bottom() - egui::vec2(0., space::FIELD),
            egui::Align2::CENTER_BOTTOM,
            "텍스트 넘침 · 검토 필요",
            egui::FontId::proportional(text::SMALL),
            color::ACCENT,
        );
    }
}

fn origin_x(rect: Rect, galley: &Galley) -> f32 {
    rect.center().x - galley.rect.center().x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_copy_stays_between_number_badge_and_page_counter() {
        let ctx = egui::Context::default();
        crate::presentation::typography::install(&ctx).unwrap();
        let value = "장면 예시 1 — 아주 긴 한국어 · 日本語 · English 예시 문구로 말줄임과 줄바꿈을 검토합니다";
        crate::presentation::test_support::run(&ctx, Default::default(), |ui| {
            for edge in [132., 223., 645.] {
                let image = Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(edge, edge));
                let bounds = super::super::page_marker::text_bounds(image);
                let (heading, caption, gap) =
                    fit(ui, image, bounds, value, "한 장면에서 시작되는 이야기");
                let height = heading.size().y + caption.size().y + gap;
                let top = bounds.center().y - height / 2.;
                assert_eq!(heading.job.text, value);
                assert!(!heading.elided);
                assert!(height <= bounds.height());
                assert!(top >= image.top() + size::PAGE_NUMBER_BADGE + space::TIGHT * 2.);
                assert!(top + height <= image.bottom() - space::GROUP - text::BODY);
            }
        });
    }

    #[test]
    fn multilingual_title_fits_canvas_and_thumbnail_without_ellipsis() {
        let ctx = egui::Context::default();
        crate::presentation::typography::install(&ctx).unwrap();
        let title = "장면 예시 1 — 아주 긴 한국어 · 日本語 · English 예시 문구로 말줄임과 줄바꿈을 검토합니다";
        let _ = crate::presentation::test_support::run(&ctx, Default::default(), |ui| {
            for edge in [132., 645.] {
                let rect = Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(edge, edge));
                let (heading, subtitle, gap) = fit(
                    ui,
                    rect,
                    text_bounds(rect),
                    title,
                    "한 장면에서 시작되는 이야기",
                );
                assert_eq!(heading.job.text, title);
                assert!(!heading.elided);
                assert!(heading.size().x <= edge * proportion::ARTWORK_TEXT_WIDTH + 1.);
                let bounds = heading
                    .rect
                    .translate(egui::vec2(origin_x(rect, &heading), 0.));
                assert!(bounds.left() >= rect.left());
                assert!(bounds.right() <= rect.right());
                assert!((bounds.center().x - rect.center().x).abs() < 1.);
                assert!(
                    heading.size().y + subtitle.size().y + gap
                        <= edge * proportion::ARTWORK_TEXT_HEIGHT
                );
            }
        });
    }
}
