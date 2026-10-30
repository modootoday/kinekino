use crate::{
    application::workspace_read::{CellValue, ReadOrigin, WorkspaceSnapshot},
    presentation::{components::text_comparison, design_tokens::space},
};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, snapshot: &WorkspaceSnapshot, selected: &Option<String>) {
    ui.add_space(space::FIELD);
    ui.strong("원문 / 교정안 비교");
    let row = snapshot
        .table
        .rows
        .iter()
        .find(|row| Some(&row.row_id) == selected.as_ref());
    let text = |id: &str| {
        row.and_then(|row| row.cells.iter().find(|cell| cell.column_id == id))
            .and_then(|cell| match &cell.value {
                CellValue::Text(value) => Some(value.as_str()),
                _ => None,
            })
    };
    let sample = snapshot.origin == ReadOrigin::DesignSample;
    text_comparison::show(
        ui,
        "correction-copy",
        text("original"),
        sample.then(|| text("correction")).flatten(),
    );
    ui.weak(if sample {
        "디자인 샘플 · 예시 후보이며 실제 교정 결과 아님"
    } else {
        "영상 자막 행 교정 미연결 · 저장된 문서 제안은 제안 검토에서 확인합니다."
    });
    if row.is_none() {
        ui.weak("비교할 자막 행을 선택해 주세요.");
    }
    ui.add_space(space::GROUP);
}
