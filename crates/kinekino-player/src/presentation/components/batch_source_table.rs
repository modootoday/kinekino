use crate::presentation::{
    design_tokens::{border, color, proportion, radius, size, space},
    layout,
};
use eframe::egui;

pub struct SourceRow<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub range: &'a str,
    pub candidates: Option<usize>,
    pub output: Option<&'a str>,
    pub status: &'a str,
}

pub fn show(
    ui: &mut egui::Ui,
    rows: &[SourceRow<'_>],
    mut thumbnail: impl FnMut(usize, &mut egui::Ui, egui::Rect),
) {
    egui::Frame::new()
        .stroke(egui::Stroke::new(border::THIN, color::BORDER))
        .corner_radius(radius::CONTROL)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.;
            let mut boundaries = Vec::new();
            let header = egui::Frame::new()
                .fill(color::CHROME)
                .inner_margin(space::RAIL_MARGIN)
                .show(ui, |ui| {
                    let width = ui.available_width();
                    egui_taffy::tui(ui, ui.id().with("batch-header"))
                        .reserve_available_width()
                        .style(columns(width))
                        .show(|tui| {
                            for label in ["선택", "회차", "범위", "후보", "출력", "상태"]
                            {
                                cell(tui, &mut boundaries, |ui| {
                                    ui.strong(label);
                                });
                            }
                        });
                });
            dividers(ui, header.response.rect, &boundaries);
            for (index, row) in rows.iter().enumerate() {
                ui.separator();
                let mut boundaries = Vec::new();
                let response = egui::Frame::new()
                    .fill(if index % 2 == 0 {
                        color::PANEL
                    } else {
                        color::FIELD
                    })
                    .inner_margin(space::RAIL_MARGIN)
                    .show(ui, |ui| {
                        let width = ui.available_width();
                        egui_taffy::tui(ui, ui.id().with(row.id))
                            .reserve_available_width()
                            .style(columns(width))
                            .show(|tui| {
                                cell(tui, &mut boundaries, |ui| {
                                    ui.add_enabled(false, egui::Checkbox::new(&mut false, ""))
                                        .on_disabled_hover_text("회차 포함 선택 연결 필요");
                                });
                                cell(tui, &mut boundaries, |ui| {
                                    egui_taffy::tui(ui, ui.id().with("source"))
                                        .reserve_available_width()
                                        .style(source_columns())
                                        .show(|tui| {
                                            layout::block(tui, |ui| {
                                                let (rect, _) = ui.allocate_exact_size(
                                                    egui::vec2(
                                                        size::SOURCE_THUMBNAIL,
                                                        size::SOURCE_THUMBNAIL * 9. / 16.,
                                                    ),
                                                    egui::Sense::hover(),
                                                );
                                                thumbnail(index, ui, rect);
                                            });
                                            layout::block(tui, |ui| {
                                                ui.add(egui::Label::new(row.title).truncate())
                                                    .on_hover_text(row.title);
                                            });
                                        });
                                });
                                for value in [
                                    row.range.to_owned(),
                                    row.candidates.map_or_else(|| "—".into(), |n| n.to_string()),
                                    row.output.unwrap_or("—").to_owned(),
                                ] {
                                    cell(tui, &mut boundaries, |ui| {
                                        ui.add(egui::Label::new(&value).truncate())
                                            .on_hover_text(&value);
                                    });
                                }
                                cell(tui, &mut boundaries, |ui| {
                                    super::badge::show(ui, row.status);
                                });
                            });
                    });
                dividers(ui, response.response.rect, &boundaries);
            }
            if rows.is_empty() {
                egui::Frame::new()
                    .inner_margin(space::PANEL_MARGIN)
                    .show(ui, |ui| {
                        ui.weak("표시할 회차가 없습니다. 소스 연결 상태를 확인해 주세요.");
                    });
            }
        });
}

fn cell(tui: &mut egui_taffy::Tui, boundaries: &mut Vec<f32>, body: impl FnOnce(&mut egui::Ui)) {
    layout::block(tui, |ui| {
        boundaries.push(ui.max_rect().left());
        body(ui);
    });
}

fn dividers(ui: &egui::Ui, rect: egui::Rect, boundaries: &[f32]) {
    for left in boundaries.iter().skip(1) {
        let x = left - space::FIELD / 2.;
        if x > rect.left() && x < rect.right() {
            ui.painter().line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(border::THIN, color::BORDER),
            );
        }
    }
}

fn source_columns() -> egui_taffy::taffy::Style {
    let mut style = layout::full(layout::sidebar(size::SOURCE_THUMBNAIL, space::FIELD));
    style.align_items = Some(egui_taffy::taffy::AlignItems::Center);
    style
}

fn columns(width: f32) -> egui_taffy::taffy::Style {
    use egui_taffy::taffy::{
        AlignItems,
        prelude::{fr, length},
        style_helpers::minmax,
    };
    let fixed = size::CONTROL + size::TABLE_TIME + size::TABLE_INDEX * 2. + size::TABLE_STATUS;
    let scale =
        (((width - space::FIELD * 5.).max(0.) * proportion::TABLE_FIXED_MAX) / fixed).min(1.);
    let mut style = layout::full(layout::column(space::FIELD));
    style.align_items = Some(AlignItems::Center);
    style.grid_template_columns = vec![
        length(size::CONTROL * scale),
        minmax(length(0.0_f32), fr(1.0_f32)),
        length(size::TABLE_TIME * scale),
        length(size::TABLE_INDEX * scale),
        length(size::TABLE_INDEX * scale),
        length(size::TABLE_STATUS * scale),
    ];
    style
}
