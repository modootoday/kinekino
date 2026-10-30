use super::view;
use crate::{
    application::workspace_read::{
        CellKind, CellValue, TableCell, TableColumn, TableSnapshot, WorkspaceSnapshot,
    },
    presentation::{
        components::{data_table, input, inspector, notice, tabs},
        design_tokens::space,
    },
};
use eframe::egui;

pub(super) fn results(
    ui: &mut egui::Ui,
    snapshot: &WorkspaceSnapshot,
    selected: &mut Option<String>,
) {
    ui.horizontal_wrapped(|ui| {
        let _ = tabs::tab(ui, "감지된 텍스트", true);
        ui.add_enabled_ui(false, |ui| tabs::tab(ui, "낮은 신뢰도", false))
            .inner
            .on_disabled_hover_text("신뢰도 데이터 연결이 필요합니다.");
    });
    let mut table = TableSnapshot {
        columns: Vec::new(),
        rows: snapshot.table.rows.clone(),
    };
    for (id, label) in [
        ("start", "시작 시간"),
        ("end", "종료 시간"),
        ("original", "감지된 텍스트"),
        ("correction", "예시 후보"),
    ] {
        if let Some(column) = snapshot.table.columns.iter().find(|c| c.column_id == id) {
            table.columns.push(TableColumn {
                label: label.into(),
                ..column.clone()
            });
        }
    }
    table.columns.push(TableColumn {
        column_id: "ocr-review".into(),
        label: "검토 상태".into(),
        kind: CellKind::Text,
    });
    for row in &mut table.rows {
        row.cells.push(TableCell {
            column_id: "ocr-review".into(),
            value: CellValue::Missing,
        });
    }
    let table = data_table::with_row_numbers(&table);
    data_table::show_with_widths(
        ui,
        &table,
        selected,
        &[
            (
                "display-number",
                crate::presentation::design_tokens::size::TABLE_INDEX,
            ),
            (
                "start",
                crate::presentation::design_tokens::size::TABLE_TIME,
            ),
            ("end", crate::presentation::design_tokens::size::TABLE_TIME),
            (
                "ocr-review",
                crate::presentation::design_tokens::size::TABLE_STATUS,
            ),
        ],
    );
    ui.add_space(space::FIELD);
    ui.weak(if table.rows.is_empty() {
        "감지 결과 연결 필요"
    } else {
        "검토 상태·신뢰도 데이터 연결 필요"
    });
}

pub(super) fn text_inspector(
    ui: &mut egui::Ui,
    snapshot: &WorkspaceSnapshot,
    selected: &Option<String>,
) {
    notice::show(
        ui,
        notice::Kind::Info,
        "읽기 기능",
        "영상의 글자를 제거하지 않습니다.",
    );
    inspector::section(ui, "텍스트 검토");
    let row = snapshot
        .table
        .rows
        .iter()
        .find(|row| selected.as_ref() == Some(&row.row_id));
    ui.label("원본 텍스트 (감지 결과)");
    input::readonly(
        ui,
        &row.map_or_else(
            || "감지 결과 선택 필요".into(),
            |row| view::cell(row, "original"),
        ),
    );
    ui.add_space(space::FIELD);
    ui.label("인식 후보 (읽기 전용)");
    input::readonly(
        ui,
        &row.map_or_else(
            || "감지 결과 선택 필요".into(),
            |row| view::cell(row, "correction"),
        ),
    );
    ui.weak("텍스트 편집 명령 연결 필요");
}
