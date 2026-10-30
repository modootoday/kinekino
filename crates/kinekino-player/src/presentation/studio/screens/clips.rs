use super::view;
use crate::{
    application::workspace_read::WorkspaceSnapshot,
    presentation::{
        components::{
            artboard::{self, Artwork},
            batch_source_table::{self, SourceRow},
            card, clip_candidate, media_preview,
        },
        design_tokens::{breakpoint, size, space},
        layout,
    },
};
use eframe::egui;

pub fn show(
    ui: &mut egui::Ui,
    snapshot: &WorkspaceSnapshot,
    tab: usize,
    selected: &mut Option<String>,
) {
    let items = view::items(snapshot);
    let index = view::selected_index(&items, selected);
    if tab == 2 {
        card::show(ui, "회차 선택", |ui| {
            ui.add_enabled(false, egui::Checkbox::new(&mut false, "전체 선택"));
            ui.small("디자인 예시 · 실제 회차 연결 전");
            let rows: Vec<_> = items
                .iter()
                .map(|item| SourceRow {
                    id: &item.id,
                    title: &item.title,
                    range: "전체",
                    candidates: None,
                    output: None,
                    status: "검토 전",
                })
                .collect();
            batch_source_table::show(ui, &rows, |index, ui, rect| {
                artboard::paint(
                    ui,
                    rect,
                    &Artwork {
                        asset_id: items[index].asset_id,
                        title: "",
                        subtitle: "",
                        cover: false,
                        grid: false,
                    },
                );
            });
        });
        ui.add_space(space::GROUP);
        card::show(ui, "선택된 작업 계획 요약", |ui| {
            let columns = if ui.ctx().content_rect().width() < breakpoint::STACK_EDITOR {
                1
            } else {
                2
            };
            egui_taffy::tui(ui, ui.id().with("batch-plan-summary"))
                .reserve_available_width()
                .style(layout::full(layout::grid(columns, space::PANEL)))
                .show(|tui| {
                    for value in [
                        "대상 비율 9:16",
                        "출력 크기 · 지원 확인 필요",
                        "사용할 모델 · 선택 필요",
                        "출력 저장 위치 · 선택 필요",
                    ] {
                        layout::block(tui, |ui| {
                            ui.label(value);
                        });
                    }
                });
        });
        return;
    }
    let asset = index.and_then(|index| items[index].asset_id);
    let columns = if ui.ctx().content_rect().width() < breakpoint::STACK_EDITOR || tab == 1 {
        1
    } else {
        2
    };
    egui_taffy::tui(ui, ui.id().with("clip-comparison"))
        .reserve_available_width()
        .style(layout::full(layout::grid(columns, space::PANEL)))
        .show(|tui| {
            layout::block(tui, |ui| {
                let artwork = Artwork {
                    asset_id: asset,
                    title: "",
                    subtitle: "",
                    cover: true,
                    grid: tab == 0,
                };
                if tab == 1 {
                    media_preview::unavailable(
                        ui,
                        &artwork,
                        9. / 16.,
                        "9:16",
                        size::PREVIEW_HEIGHT,
                    );
                } else {
                    artboard::show(ui, &artwork, 9. / 16., size::PREVIEW_HEIGHT, false);
                }
            });
            if tab == 0 {
                layout::block(tui, |ui| {
                    ui.strong("원본 영상 (16:9)");
                    artboard::show(
                        ui,
                        &Artwork {
                            asset_id: asset,
                            title: "",
                            subtitle: "",
                            cover: false,
                            grid: false,
                        },
                        16. / 9.,
                        size::PREVIEW_HEIGHT,
                        false,
                    );
                });
            }
        });
    ui.add_space(space::PANEL);
    if tab == 0 {
        super::clip_timeline::show(ui, snapshot);
    } else {
        ui.strong(format!("추천 후보 ({}개) · 디자인 예시", items.len()));
        let mut responses = Vec::with_capacity(items.len());
        for (item_index, item) in items.iter().enumerate() {
            let row = &snapshot.table.rows[item_index];
            let response = clip_candidate::show(
                ui,
                clip_candidate::Candidate {
                    id: &item.id,
                    title: &item.title,
                    asset_id: item.asset_id,
                    range: &view::cell(row, "range"),
                    duration: &view::cell(row, "duration"),
                    selected: index == Some(item_index),
                },
            );
            if response.clicked() {
                *selected = Some(item.id.clone());
                response.request_focus();
            }
            responses.push(response);
            ui.add_space(space::FIELD);
        }
        if let Some(index) =
            crate::presentation::components::keyboard_selection::select(ui.ctx(), &responses, false)
        {
            *selected = Some(items[index].id.clone());
        }
    }
}
