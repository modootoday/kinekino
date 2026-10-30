use crate::application::workspace_read::{
    CellKind, CellValue, TableCell, TableColumn, TableSnapshot,
};
use crate::presentation::{
    control_icon::Icon,
    design_tokens::{border, radius, size, space},
    layout, player_theme,
};
use eframe::egui;

pub struct Appearance {
    pub row_height: f32,
    pub leading_icon: Option<Icon>,
    pub whole_row_focus: bool,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            row_height: size::CONTROL,
            leading_icon: None,
            whole_row_focus: false,
        }
    }
}

pub fn value(value: &CellValue) -> String {
    match value {
        CellValue::Text(text) | CellValue::Integer(text) => text.clone(),
        CellValue::TimeUs(time) => {
            let millis = time.0 / 1_000;
            format!(
                "{:02}:{:02}:{:02}.{:03}",
                millis / 3_600_000,
                millis / 60_000 % 60,
                millis / 1_000 % 60,
                millis % 1_000
            )
        }
        CellValue::Percent(percent) => format!("{percent}%"),
        CellValue::Boolean(enabled) => if *enabled { "켬" } else { "끔" }.into(),
        CellValue::Missing => "—".into(),
    }
}

pub fn show(ui: &mut egui::Ui, table: &TableSnapshot, selected: &mut Option<String>) {
    show_with_widths(ui, table, selected, &[]);
}

pub fn with_row_numbers(table: &TableSnapshot) -> TableSnapshot {
    let mut table = table.clone();
    table.columns.insert(
        0,
        TableColumn {
            column_id: "display-number".into(),
            label: "번호".into(),
            kind: CellKind::Integer,
        },
    );
    for (index, row) in table.rows.iter_mut().enumerate() {
        row.cells.push(TableCell {
            column_id: "display-number".into(),
            value: CellValue::Integer((index + 1).to_string()),
        });
    }
    table
}

pub fn show_with_widths(
    ui: &mut egui::Ui,
    table: &TableSnapshot,
    selected: &mut Option<String>,
    fixed: &[(&str, f32)],
) {
    show_styled(ui, table, selected, fixed, &Appearance::default());
}

pub fn show_styled(
    ui: &mut egui::Ui,
    table: &TableSnapshot,
    selected: &mut Option<String>,
    fixed: &[(&str, f32)],
    appearance: &Appearance,
) -> Vec<egui::Response> {
    show_content(ui, table, selected, fixed, appearance, false)
}

pub fn show_framed(
    ui: &mut egui::Ui,
    table: &TableSnapshot,
    selected: &mut Option<String>,
    fixed: &[(&str, f32)],
    appearance: &Appearance,
) -> Vec<egui::Response> {
    egui::Frame::new()
        .stroke(egui::Stroke::new(border::THIN, player_theme::BORDER))
        .corner_radius(radius::CONTROL)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            show_content(ui, table, selected, fixed, appearance, true)
        })
        .inner
}

fn show_content(
    ui: &mut egui::Ui,
    table: &TableSnapshot,
    selected: &mut Option<String>,
    fixed: &[(&str, f32)],
    appearance: &Appearance,
    framed: bool,
) -> Vec<egui::Response> {
    use egui_taffy::taffy::style_helpers::{fr, length, minmax};
    let total: f32 = table
        .columns
        .iter()
        .filter_map(|column| {
            fixed
                .iter()
                .find(|(id, _)| *id == column.column_id)
                .map(|(_, width)| *width)
        })
        .sum();
    let scale = (ui.available_width()
        * crate::presentation::design_tokens::proportion::TABLE_FIXED_MAX
        / total.max(1.))
    .min(1.);
    let mut style = layout::full(layout::grid(table.columns.len(), 0.));
    style.grid_template_columns = table
        .columns
        .iter()
        .map(|column| {
            fixed
                .iter()
                .find(|(id, _)| *id == column.column_id)
                .map_or_else(
                    || minmax(length(0_f32), fr(1_f32)),
                    |(_, width)| length(*width * scale),
                )
        })
        .collect();
    let mut row_responses = Vec::with_capacity(table.rows.len());
    let mut row_rects = vec![egui::Rect::NOTHING; table.rows.len()];
    egui_taffy::tui(ui, ui.id().with("data-table"))
        .reserve_available_width()
        .style(style)
        .show(|tui| {
            for (column_index, column) in table.columns.iter().enumerate() {
                layout::block(tui, |ui| {
                    let response = egui::Frame::new()
                        .fill(player_theme::CHROME)
                        .corner_radius(cell_corners(
                            framed,
                            column_index,
                            table.columns.len(),
                            true,
                            table.rows.is_empty(),
                        ))
                        .inner_margin(space::RAIL_MARGIN)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&column.label)
                                        .color(player_theme::SECONDARY),
                                )
                                .truncate(),
                            );
                        });
                    if framed && column_index > 0 {
                        let rect = response.response.rect.shrink2(egui::vec2(0., space::FIELD));
                        ui.painter().line_segment(
                            [rect.left_top(), rect.left_bottom()],
                            egui::Stroke::new(border::THIN, player_theme::BORDER),
                        );
                    }
                });
            }
            for (index, row) in table.rows.iter().enumerate() {
                let active = selected.as_ref() == Some(&row.row_id);
                for (column_index, column) in table.columns.iter().enumerate() {
                    let text = row
                        .cells
                        .iter()
                        .find(|cell| cell.column_id == column.column_id)
                        .map(|cell| value(&cell.value))
                        .unwrap_or_else(|| "—".into());
                    layout::block(tui, |ui| {
                        let response = egui::Frame::new()
                            .corner_radius(cell_corners(
                                framed,
                                column_index,
                                table.columns.len(),
                                false,
                                index + 1 == table.rows.len(),
                            ))
                            .fill(if active && framed {
                                crate::presentation::design_tokens::color::TABLE_SELECTED
                            } else if active {
                                player_theme::SELECTED
                            } else if index % 2 == 0 {
                                player_theme::PANEL
                            } else {
                                player_theme::CHROME
                            })
                            .inner_margin(space::RAIL_MARGIN)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                let height = appearance.row_height - space::FIELD * 2.;
                                ui.set_min_height(height);
                                let rich = egui::RichText::new(&text);
                                let rich = if matches!(column.kind, CellKind::Time) {
                                    rich.monospace()
                                } else {
                                    rich
                                };
                                if !appearance.whole_row_focus && appearance.leading_icon.is_none()
                                {
                                    ui.add(egui::Label::new(rich).truncate())
                                        .on_hover_text(&text);
                                    return;
                                }
                                ui.allocate_ui_with_layout(
                                    egui::vec2(ui.available_width(), height),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        if column_index == 0
                                            && let Some(icon) = appearance.leading_icon
                                        {
                                            ui.add(
                                                icon.image()
                                                    .tint(player_theme::TEXT)
                                                    .fit_to_exact_size(egui::Vec2::splat(
                                                        size::TOOLBAR_ICON,
                                                    )),
                                            );
                                        }
                                        ui.add(egui::Label::new(rich).truncate())
                                            .on_hover_text(&text);
                                    },
                                );
                            })
                            .response;
                        if appearance.whole_row_focus {
                            row_rects[index] = row_rects[index].union(response.rect);
                            return;
                        }
                        let response = ui.interact(
                            response.rect,
                            ui.id().with((&row.row_id, &column.column_id)),
                            egui::Sense::click(),
                        );
                        if column_index == 0 {
                            row_responses.push(response.clone());
                        }
                        if response.clicked() {
                            *selected = Some(row.row_id.clone());
                            if let Some(first) = row_responses.last() {
                                first.request_focus();
                            }
                        }
                        if column_index == 0 && active {
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(
                                    response.rect.min,
                                    egui::vec2(border::FOCUS, response.rect.height()),
                                ),
                                radius::NONE,
                                player_theme::ACCENT,
                            );
                        }
                        if response.has_focus() {
                            ui.painter().rect_stroke(
                                response.rect.shrink(border::THIN),
                                radius::NONE,
                                egui::Stroke::new(border::FOCUS, player_theme::FOCUS),
                                egui::StrokeKind::Inside,
                            );
                        }
                    });
                }
            }
        });
    if appearance.whole_row_focus {
        for (index, rect) in row_rects.into_iter().enumerate() {
            if !rect.is_positive() {
                continue;
            }
            let row = &table.rows[index];
            if framed {
                ui.painter().line_segment(
                    [rect.left_top(), rect.right_top()],
                    egui::Stroke::new(border::THIN, player_theme::BORDER),
                );
            }
            let response = ui.interact(
                rect,
                ui.id().with(("table-row", &row.row_id)),
                egui::Sense::click(),
            );
            let active = selected.as_ref() == Some(&row.row_id);
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::WidgetType::SelectableLabel,
                    ui.is_enabled(),
                    active,
                    row.cells
                        .iter()
                        .map(|cell| value(&cell.value))
                        .collect::<Vec<_>>()
                        .join(" · "),
                )
            });
            if response.clicked() {
                *selected = Some(row.row_id.clone());
                response.request_focus();
            }
            if active {
                if framed {
                    ui.painter().rect_stroke(
                        rect,
                        radius::CONTROL,
                        egui::Stroke::new(border::THIN, player_theme::ACCENT),
                        egui::StrokeKind::Inside,
                    );
                } else {
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(
                            rect.min,
                            egui::vec2(border::FOCUS, rect.height()),
                        ),
                        radius::NONE,
                        player_theme::ACCENT,
                    );
                }
            }
            if response.has_focus() {
                ui.painter().rect_stroke(
                    rect.shrink(if framed {
                        border::FOCUS + border::THIN
                    } else {
                        border::THIN
                    }),
                    radius::SMALL,
                    egui::Stroke::new(border::FOCUS, player_theme::FOCUS),
                    egui::StrokeKind::Inside,
                );
            }
            row_responses.push(response);
        }
    }
    if let Some(index) = super::keyboard_selection::select(ui.ctx(), &row_responses, false) {
        *selected = Some(table.rows[index].row_id.clone());
    }
    row_responses
}

fn cell_corners(
    framed: bool,
    column: usize,
    columns: usize,
    first_row: bool,
    last_row: bool,
) -> egui::CornerRadius {
    let corner = if framed {
        radius::CONTROL
    } else {
        radius::NONE
    };
    egui::CornerRadius {
        nw: if first_row && column == 0 { corner } else { 0 },
        ne: if first_row && column + 1 == columns {
            corner
        } else {
            0
        },
        sw: if last_row && column == 0 { corner } else { 0 },
        se: if last_row && column + 1 == columns {
            corner
        } else {
            0
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::workspace_read::{
        WorkspaceId, WorkspaceReadPort,
        preview::{PreviewScenario, PreviewWorkspaceProvider},
    };

    #[test]
    fn decorated_rows_keep_full_width_hit_targets_and_keyboard_selection() {
        let ctx = egui::Context::default();
        crate::presentation::typography::install(&ctx).unwrap();
        let snapshot =
            PreviewWorkspaceProvider::new(PreviewScenario::LongLabels).snapshot(WorkspaceId::Job);
        let appearance = Appearance {
            row_height: size::QUEUE_ROW,
            leading_icon: Some(Icon::List),
            whole_row_focus: true,
        };
        let mut selected = None;
        let mut rows = Vec::new();
        let mut frame = |events| {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(688., 480.),
                )),
                events,
                ..Default::default()
            };
            let _ = crate::presentation::test_support::run(&ctx, input, |ui| {
                rows = show_styled(ui, &snapshot.table, &mut selected, &[], &appearance);
                assert!(ui.min_rect().width() <= 688.);
            });
            rows.iter().map(|row| row.rect).collect::<Vec<_>>()
        };
        for _ in 0..3 {
            frame(vec![]);
        }
        let rects = frame(vec![]);
        assert_eq!(rects.len(), snapshot.table.rows.len());
        assert!(rects[0].height() >= size::QUEUE_ROW);
        assert!(rects[0].width() > 600.);
        let pos = rects[0].right_center() - egui::vec2(space::FIELD, 0.);
        frame(vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
        frame(vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        frame(vec![egui::Event::Key {
            key: egui::Key::ArrowDown,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        assert_eq!(selected.as_ref(), Some(&snapshot.table.rows[1].row_id));
    }

    #[test]
    fn localized_sample_tables_do_not_expand_their_available_width() {
        for width in [240., 480., 960.] {
            let ctx = egui::Context::default();
            crate::presentation::typography::install(&ctx).unwrap();
            for scenario in [
                PreviewScenario::Populated,
                PreviewScenario::LongLabels,
                PreviewScenario::NeedsInput,
            ] {
                let provider = PreviewWorkspaceProvider::new(scenario);
                for id in WorkspaceId::ALL {
                    let snapshot = provider.snapshot(id);
                    let mut selected = None;
                    for _ in 0..3 {
                        let input = egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(width, 600.),
                            )),
                            ..Default::default()
                        };
                        let _ = crate::presentation::test_support::run(&ctx, input, |ui| {
                            let available = ui.available_width();
                            show(ui, &snapshot.table, &mut selected);
                            assert!(
                                ui.min_rect().width() <= available + 1.,
                                "{id:?} table expanded beyond {available} px"
                            );
                        });
                    }
                }
            }
        }
    }
}
