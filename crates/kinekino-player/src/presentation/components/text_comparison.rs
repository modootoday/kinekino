use crate::presentation::{
    design_tokens::{border, color, radius, size, space, text},
    layout,
};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, id: &str, original: Option<&str>, proposed: Option<&str>) {
    ui.push_id(id, |ui| {
        egui::Frame::new()
            .fill(color::FIELD)
            .stroke(egui::Stroke::new(border::THIN, color::BORDER))
            .corner_radius(radius::CONTROL)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 0.;
                row(ui, "original", "원문", original, None);
                ui.separator();
                let changed = original.zip(proposed).map(|(a, b)| changed_range(a, b));
                row(ui, "proposed", "교정안", proposed, changed);
            });
        if let Some((original, proposed)) = original.zip(proposed) {
            if original == proposed {
                ui.weak("변경 없음");
            } else if changed_range(original, proposed).is_empty() {
                ui.weak("원문에서 삭제된 문구가 있습니다.");
            }
        }
    });
}

fn row(
    ui: &mut egui::Ui,
    id: &str,
    label: &str,
    value: Option<&str>,
    changed: Option<std::ops::Range<usize>>,
) {
    let background = ui.painter().add(egui::Shape::Noop);
    let response = ui
        .scope(|ui| {
            egui_taffy::tui(ui, ui.id().with(id))
                .reserve_available_width()
                .style(layout::full(layout::sidebar(size::FIELD_LABEL_MIN, 0.)))
                .show(|tui| {
                    layout::block(tui, |ui| {
                        egui::Frame::new()
                            .inner_margin(space::PANEL_MARGIN)
                            .show(ui, |ui| {
                                ui.label(label);
                            });
                    });
                    layout::block(tui, |ui| {
                        egui::Frame::new()
                            .inner_margin(space::PANEL_MARGIN)
                            .show(ui, |ui| {
                                if let Some(value) = value {
                                    let mut job = egui::text::LayoutJob::default();
                                    let range = changed.unwrap_or(0..0);
                                    for (part, tint) in [
                                        (&value[..range.start], color::TEXT),
                                        (&value[range.clone()], color::ACCENT),
                                        (&value[range.end..], color::TEXT),
                                    ] {
                                        job.append(
                                            part,
                                            0.,
                                            egui::TextFormat {
                                                font_id: egui::FontId::proportional(text::BODY),
                                                color: tint,
                                                ..Default::default()
                                            },
                                        );
                                    }
                                    ui.add(egui::Label::new(job).wrap()).on_hover_text(value);
                                } else {
                                    ui.weak("연결 필요");
                                }
                            });
                    });
                });
        })
        .response;
    let label_rect = egui::Rect::from_min_size(
        response.rect.min,
        egui::vec2(
            size::FIELD_LABEL_MIN.min(response.rect.width()),
            response.rect.height(),
        ),
    );
    let corners = egui::CornerRadius {
        nw: if id == "original" { radius::CONTROL } else { 0 },
        sw: if id == "proposed" { radius::CONTROL } else { 0 },
        ..Default::default()
    };
    ui.painter().set(
        background,
        egui::Shape::rect_filled(label_rect, corners, color::SELECTED),
    );
    ui.painter().line_segment(
        [label_rect.right_top(), label_rect.right_bottom()],
        egui::Stroke::new(border::THIN, color::BORDER),
    );
}

/// One changed span, bounded by common Unicode scalar prefix/suffix; never edits copy.
fn changed_range(original: &str, proposed: &str) -> std::ops::Range<usize> {
    let prefix = original
        .chars()
        .zip(proposed.chars())
        .take_while(|(a, b)| a == b)
        .map(|(_, c)| c.len_utf8())
        .sum::<usize>();
    let suffix = original[prefix..]
        .chars()
        .rev()
        .zip(proposed[prefix..].chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(_, c)| c.len_utf8())
        .sum::<usize>();
    prefix..proposed.len() - suffix
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn highlight_preserves_unicode_and_handles_insert_replace_delete_and_equal() {
        for (a, b, expected) in [
            ("오늘 밤", "오늘 밤.", "."),
            ("日本語の文", "日本語の言葉", "言葉"),
            ("앞🙂뒤", "앞🌙뒤", "🌙"),
            ("삭제문구", "삭제", ""),
            ("동일", "동일", ""),
            ("", "새 문장", "새 문장"),
        ] {
            let range = changed_range(a, b);
            assert_eq!(&b[range.clone()], expected);
            assert_eq!(
                format!(
                    "{}{}{}",
                    &b[..range.start],
                    &b[range.clone()],
                    &b[range.end..]
                ),
                b
            );
        }
    }
}
