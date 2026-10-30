use crate::presentation::{
    components::{
        artboard::{self, Artwork},
        card, toolbar,
    },
    control_icon::Icon,
    design_tokens::{breakpoint, size, space},
    layout,
};
use eframe::egui;

pub struct Candidate<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub asset_id: Option<&'a str>,
    pub range: &'a str,
    pub duration: &'a str,
    pub selected: bool,
}

pub fn show(ui: &mut egui::Ui, candidate: Candidate<'_>) -> egui::Response {
    use egui_taffy::taffy::{
        prelude::{fr, length},
        style_helpers::minmax,
    };
    let compact = ui.ctx().content_rect().width() < breakpoint::STACK_EDITOR;
    card::selectable(ui, candidate.id, candidate.title, candidate.selected, |ui| {
        egui_taffy::tui(ui, ui.id().with(("candidate-body", candidate.id)))
            .reserve_available_width()
            .style(layout::full(if compact { layout::column(space::FIELD) } else { layout::detail_sidebar(size::CANDIDATE_ACTIONS, space::PANEL) }))
            .show(|tui| {
                layout::block(tui, |ui| {
                    let mut style = layout::column(space::PANEL);
                    style.grid_template_columns = vec![length(size::CONTROL), length(size::CARD_RAIL), minmax(length(0.0_f32), fr(1.0_f32))];
                    egui_taffy::tui(ui, ui.id().with("candidate-details"))
                        .reserve_available_width().style(layout::full(style)).show(|tui| {
                            layout::widget(tui, |ui| {
                                ui.add_enabled(false, egui::Checkbox::new(&mut false, ""))
                                    .on_disabled_hover_text("작업 포함 선택 연결이 필요합니다. 미리보기 선택과 별개입니다.");
                            });
                            layout::block(tui, |ui| {
                                artboard::show(ui, &Artwork { asset_id: candidate.asset_id, title: "", subtitle: "", cover: false, grid: false }, 16. / 9., size::CARD_RAIL * 9. / 16., false);
                            });
                            layout::block(tui, |ui| {
                                ui.strong(candidate.title);
                                ui.small(format!("{} · {}", candidate.range, candidate.duration));
                                ui.small("선정 근거 · 디자인 샘플");
                                ui.add_enabled(false, egui::Link::new("원본 구간 연결 필요"));
                            });
                        });
                });
                layout::block(tui, |ui| toolbar::unavailable_grouped(ui, &[&[
                    (Icon::Play, "미리보기"), (Icon::Clip, "편집"), (Icon::Delete, "제외"),
                ]]));
            });
    }).response
}
