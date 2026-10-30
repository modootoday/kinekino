use crate::presentation::{
    design_tokens::{border, color, radius, size, space},
    icon_asset::Icon,
};
use eframe::egui::{self, Response, Ui};

/// A source-owned file-list leaf. The caller owns identity and selection policy.
pub fn show(ui: &mut Ui, id: &str, label: &str, icon: Icon, selected: bool) -> Response {
    ui.push_id(id, |ui| {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width().max(0.), size::CONTROL),
            egui::Sense::click(),
        );
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::SelectableLabel,
                ui.is_enabled(),
                selected,
                label,
            )
        });
        let fill = if response.is_pointer_button_down_on() {
            color::PRESSED
        } else if selected {
            color::SELECTED
        } else if response.hovered() {
            color::HOVER
        } else {
            color::PANEL
        };
        let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
        painter.rect_filled(rect, radius::CONTROL, fill);
        let icon_rect = egui::Rect::from_center_size(
            rect.left_center() + egui::vec2(space::FIELD + size::TOOLBAR_ICON / 2., 0.),
            egui::Vec2::splat(size::TOOLBAR_ICON),
        );
        icon.paint(ui, icon_rect, color::TEXT);
        let text_start = icon_rect.right() + space::FIELD;
        let mut job = egui::text::LayoutJob::simple_singleline(
            label.into(),
            egui::TextStyle::Body.resolve(ui.style()),
            color::TEXT,
        );
        job.wrap.max_width = (rect.right() - text_start - space::FIELD).max(0.);
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        let pos = egui::pos2(text_start, rect.center().y - galley.size().y / 2.);
        painter.galley(pos, galley, color::TEXT);
        if selected {
            painter.rect_filled(
                egui::Rect::from_min_size(rect.min, egui::vec2(border::FOCUS, rect.height())),
                radius::SMALL,
                color::ACCENT,
            );
        }
        if response.has_focus() {
            painter.rect_stroke(
                rect.shrink(border::THIN),
                radius::CONTROL,
                egui::Stroke::new(border::FOCUS, color::FOCUS),
                egui::StrokeKind::Inside,
            );
        }
        response.on_hover_text(label)
    })
    .inner
}
