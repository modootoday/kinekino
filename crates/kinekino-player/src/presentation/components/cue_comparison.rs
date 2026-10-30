use crate::presentation::{
    design_tokens::{border, color, radius, size, space, text},
    layout,
};
use eframe::egui;

pub struct Cue {
    pub id: String,
    pub number: usize,
    pub timing: String,
    pub text: String,
    pub syntax: String,
}

pub fn show(
    ui: &mut egui::Ui,
    id: &str,
    title: &str,
    cues: &[Cue],
    selected: &mut Option<String>,
    empty: &str,
) {
    ui.push_id(id, |ui| {
        egui::Frame::new()
            .fill(color::PANEL)
            .stroke(egui::Stroke::new(border::THIN, color::BORDER))
            .corner_radius(radius::CONTROL)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 0.;
                egui::Frame::new()
                    .inner_margin(space::PANEL_MARGIN)
                    .show(ui, |ui| {
                        ui.strong(if cues.is_empty() {
                            title.to_owned()
                        } else {
                            format!("{title} ({})", cues.len())
                        });
                    });
                ui.separator();
                if cues.is_empty() {
                    egui::Frame::new()
                        .inner_margin(space::PANEL_MARGIN)
                        .show(ui, |ui| {
                            ui.weak(empty);
                            ui.set_min_height(size::SOURCE_REFERENCE_HEIGHT);
                        });
                }
                let mut responses = Vec::new();
                for cue in cues {
                    let active = selected.as_ref() == Some(&cue.id);
                    let response =
                        super::card::selectable_row(ui, &cue.id, &cue.text, active, |ui| {
                            egui_taffy::tui(ui, ui.id().with(("cue", &cue.id)))
                                .reserve_available_width()
                                .style(layout::full(layout::sidebar(size::CONTROL, space::FIELD)))
                                .show(|tui| {
                                    layout::block(tui, |ui| {
                                        ui.label(
                                            egui::RichText::new(cue.number.to_string()).color(
                                                if active { color::ACCENT } else { color::TEXT },
                                            ),
                                        );
                                    });
                                    layout::block(tui, |ui| {
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(&cue.timing)
                                                    .size(text::BODY)
                                                    .monospace(),
                                            )
                                            .wrap(),
                                        );
                                        ui.add_space(space::TIGHT);
                                        ui.add(egui::Label::new(&cue.text).wrap());
                                        ui.add_space(space::FIELD);
                                        egui::Frame::new()
                                            .fill(color::FIELD)
                                            .inner_margin(space::RAIL_MARGIN)
                                            .show(ui, |ui| {
                                                ui.set_min_width(ui.available_width());
                                                ui.add(
                                                    egui::Label::new(
                                                        egui::RichText::new(&cue.syntax)
                                                            .size(text::SMALL)
                                                            .monospace()
                                                            .color(color::SECONDARY),
                                                    )
                                                    .wrap(),
                                                );
                                            });
                                    });
                                });
                        })
                        .response;
                    if response.clicked() {
                        *selected = Some(cue.id.clone());
                        response.request_focus();
                    }
                    responses.push(response);
                }
                if let Some(index) = super::keyboard_selection::select(ui.ctx(), &responses, false)
                {
                    *selected = Some(cues[index].id.clone());
                }
            });
    });
}
