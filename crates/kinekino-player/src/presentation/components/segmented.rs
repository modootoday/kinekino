use crate::presentation::{
    design_tokens::{border, color, radius, size, space},
    layout,
};
use eframe::egui;

/// A shared choice group. Ownership and persistence of selection stay with the caller.
pub fn show(ui: &mut egui::Ui, id: &str, labels: &[&str], selected: &mut Option<usize>) -> bool {
    let mut changed = false;
    let mut responses = vec![None; labels.len()];
    egui_taffy::tui(ui, ui.id().with(id))
        .reserve_available_width()
        .style(layout::full(layout::grid(labels.len(), space::FIELD)))
        .show(|tui| {
            for (index, label) in labels.iter().enumerate() {
                layout::block(tui, |ui| {
                    let active = *selected == Some(index);
                    let response = ui
                        .push_id((id, index), |ui| {
                            ui.add_sized(
                                [ui.available_width(), size::CONTROL],
                                egui::Button::new(egui::RichText::new(*label).color(if active {
                                    color::ACCENT
                                } else {
                                    color::TEXT
                                }))
                                .fill(if active { color::PRESSED } else { color::FIELD })
                                .corner_radius(radius::CONTROL)
                                .stroke(egui::Stroke::new(
                                    if active { border::FOCUS } else { border::THIN },
                                    if active { color::ACCENT } else { color::BORDER },
                                )),
                            )
                        })
                        .inner;
                    if response.clicked() {
                        *selected = Some(index);
                        changed = true;
                        response.request_focus();
                    }
                    if response.enabled() && response.has_focus() {
                        ui.painter().rect_stroke(
                            response.rect.shrink(border::FOCUS * 2.),
                            radius::CONTROL,
                            egui::Stroke::new(border::THIN, color::FOCUS),
                            egui::StrokeKind::Inside,
                        );
                    }
                    responses[index] = Some(response);
                });
            }
        });
    let responses: Vec<_> = responses
        .into_iter()
        .flatten()
        .filter(|r| r.enabled())
        .collect();
    if let Some(index) = super::keyboard_selection::select(ui.ctx(), &responses, true) {
        *selected = Some(index);
        changed = true;
    }
    changed
}
