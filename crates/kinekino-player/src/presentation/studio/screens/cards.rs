use super::view;
use crate::{
    application::workspace_read::WorkspaceSnapshot,
    presentation::{
        components::{
            artboard::{self, Artwork},
            keyboard_selection, page_marker, selectable_thumbnail, toolbar,
        },
        control_icon::Icon,
        design_tokens::{breakpoint, size, space},
        layout,
    },
};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, snapshot: &WorkspaceSnapshot, selected: &mut Option<String>) {
    toolbar::unavailable_strip(
        ui,
        &[
            &[
                (Icon::Plus, "카드 추가"),
                (Icon::Copy, "복제"),
                (Icon::Delete, "삭제"),
            ],
            &[(Icon::Undo, "되돌리기"), (Icon::Redo, "다시 실행")],
        ],
    );
    ui.add_space(space::PANEL);
    let items = view::items(snapshot);
    let index = view::selected_index(&items, selected);
    let mut responses = vec![None; items.len()];
    let stacked = ui.ctx().content_rect().width() < breakpoint::STACK_EDITOR;
    let height = (ui.available_height() - size::CONTROL).max(size::ARTBOARD_MIN);
    let style = if stacked {
        layout::column(space::PANEL)
    } else {
        layout::sidebar(size::CARD_RAIL, space::PANEL)
    };
    egui_taffy::tui(ui, ui.id().with("card-editor"))
        .reserve_available_width()
        .style(layout::full(style))
        .show(|tui| {
            layout::block(tui, |ui| {
                let list = |ui: &mut egui::Ui| {
                    for (n, item) in items.iter().enumerate() {
                        ui.push_id(&item.id, |ui| {
                            ui.vertical(|ui| {
                                ui.set_width(size::CARD_RAIL);
                                let artwork = Artwork {
                                    asset_id: item.asset_id,
                                    title: &item.title,
                                    subtitle: item.subtitle,
                                    cover: true,
                                    grid: false,
                                };
                                let thumbnail = selectable_thumbnail::show(
                                    ui,
                                    &artwork,
                                    1.,
                                    size::CARD_RAIL,
                                    index == Some(n),
                                    Some(page_marker::text_bounds),
                                );
                                page_marker::badge(ui, thumbnail.image, n + 1);
                                let response = thumbnail.response;
                                if response.clicked() {
                                    response.request_focus();
                                    *selected = Some(item.id.clone());
                                }
                                responses[n] = Some(response);
                                ui.add_space(space::FIELD);
                            });
                        });
                    }
                };
                if stacked {
                    egui::ScrollArea::horizontal()
                        .min_scrolled_height(size::CARD_RAIL + space::GROUP + space::FIELD)
                        .show(ui, |ui| {
                            ui.horizontal(list);
                        });
                } else {
                    egui::ScrollArea::vertical()
                        .min_scrolled_height(height)
                        .max_height(height)
                        .show(ui, list);
                }
            });
            layout::block(tui, |ui| {
                if let Some(index) = index {
                    let item = &items[index];
                    let response = artboard::show_with_text_bounds(
                        ui,
                        &Artwork {
                            asset_id: item.asset_id,
                            title: &item.title,
                            subtitle: item.subtitle,
                            cover: true,
                            grid: false,
                        },
                        1.,
                        height,
                        false,
                        Some(page_marker::text_bounds),
                    );
                    page_marker::counter(
                        ui,
                        artboard::fitted_rect(response.rect, 1.),
                        index + 1,
                        items.len(),
                    );
                    ui.weak("디자인 샘플 · 실제 카드 문서 아님");
                } else {
                    artboard::show(
                        ui,
                        &Artwork {
                            asset_id: None,
                            title: "",
                            subtitle: "",
                            cover: false,
                            grid: false,
                        },
                        1.,
                        height,
                        false,
                    );
                }
            });
        });
    let responses: Vec<_> = responses.into_iter().flatten().collect();
    if let Some(index) = keyboard_selection::select(ui.ctx(), &responses, stacked) {
        *selected = Some(items[index].id.clone());
    }
}
