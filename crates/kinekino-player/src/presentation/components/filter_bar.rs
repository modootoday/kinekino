use crate::presentation::{
    design_tokens::{border, color, radius, size, space},
    layout,
};
use eframe::egui;
use egui_taffy::taffy::{FlexWrap, Style};

pub struct Filter<'a> {
    pub label: &'a str,
    pub available: bool,
}

/// Availability is supplied by the caller; this component never infers result states.
pub fn show(
    ui: &mut egui::Ui,
    id: &str,
    filters: &[Filter<'_>],
    selected: &mut Option<usize>,
    reason: &str,
) -> Vec<egui::Response> {
    if selected.is_some_and(|index| filters.get(index).is_none_or(|filter| !filter.available)) {
        *selected = None;
    }
    let mut responses = Vec::new();
    egui_taffy::tui(ui, ui.id().with(id))
        .reserve_available_width()
        .style(layout::full(Style {
            flex_wrap: FlexWrap::Wrap,
            ..layout::row(space::FIELD)
        }))
        .show(|tui| {
            for (index, filter) in filters.iter().enumerate() {
                let active = *selected == Some(index);
                let response = layout::widget(tui, |ui| {
                    let response = ui
                        .push_id((id, index), |ui| {
                            ui.add_enabled(
                                filter.available,
                                egui::Button::new(
                                    egui::RichText::new(filter.label).color(if active {
                                        color::ACCENT
                                    } else {
                                        color::TEXT
                                    }),
                                )
                                .min_size(egui::vec2(size::TAB_MIN, size::CONTROL))
                                .fill(if active { color::CHROME } else { color::FIELD })
                                .stroke(egui::Stroke::new(
                                    border::THIN,
                                    if active { color::ACCENT } else { color::BORDER },
                                ))
                                .corner_radius(radius::CONTROL),
                            )
                        })
                        .inner
                        .on_disabled_hover_text(reason);
                    if response.enabled() && response.has_focus() {
                        ui.painter().rect_stroke(
                            response.rect.shrink(border::FOCUS * 2.),
                            radius::SMALL,
                            egui::Stroke::new(border::THIN, color::FOCUS),
                            egui::StrokeKind::Inside,
                        );
                    }
                    response
                });
                if response.clicked() {
                    *selected = Some(index);
                    response.request_focus();
                }
                responses.push(response);
            }
        });
    let available: Vec<_> = responses
        .iter()
        .enumerate()
        .filter(|(_, response)| response.enabled())
        .collect();
    let enabled: Vec<_> = available
        .iter()
        .map(|(_, response)| (*response).clone())
        .collect();
    if let Some(index) = super::keyboard_selection::select(ui.ctx(), &enabled, true) {
        *selected = Some(available[index].0);
    }
    responses
}

#[cfg(test)]
mod tests;
