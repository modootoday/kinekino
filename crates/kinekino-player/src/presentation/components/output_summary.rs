use super::artboard::{self, Artwork};
use crate::presentation::{
    design_tokens::{size, space},
    layout,
};
use eframe::egui;

pub struct OutputSummary<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub format: &'a str,
    pub source_artwork: &'a Artwork<'a>,
}

pub fn show(ui: &mut egui::Ui, output: &OutputSummary<'_>) {
    egui_taffy::tui(ui, ui.id().with(("output-summary-row", output.id)))
        .reserve_available_width()
        .style(layout::full(layout::sidebar(
            size::OUTPUT_THUMBNAIL_WIDTH,
            space::GROUP,
        )))
        .show(|tui| {
            layout::block(tui, |ui| {
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(size::OUTPUT_THUMBNAIL_WIDTH, size::OUTPUT_THUMBNAIL_HEIGHT),
                    egui::Sense::hover(),
                );
                artboard::thumbnail(ui, rect, output.source_artwork);
                response.on_hover_text("소스 미리보기 · 출력 결과 이미지가 아닙니다.");
            });
            layout::block(tui, |ui| {
                ui.add(egui::Label::new(egui::RichText::new(output.name).strong()).wrap())
                    .on_hover_text(output.name);
                ui.add_space(space::FIELD);
                ui.add(egui::Label::new(output.format).wrap());
                ui.small("소스 미리보기 · 출력 결과 아님");
            });
        });
}
