use crate::application::workspace_read::{CellValue, ReadOrigin, TableRow, WorkspaceSnapshot};

pub fn cell(row: &TableRow, column: &str) -> String {
    row.cells
        .iter()
        .find(|cell| cell.column_id == column)
        .map(|cell| crate::presentation::components::data_table::value(&cell.value))
        .unwrap_or_else(|| "—".into())
}

pub struct VisualItem<'a> {
    pub id: String,
    pub title: String,
    pub subtitle: &'static str,
    pub asset_id: Option<&'a str>,
}

/// UI-only design fixture; live results are never manufactured from sample copy.
pub fn items(snapshot: &WorkspaceSnapshot) -> Vec<VisualItem<'_>> {
    if snapshot.origin != ReadOrigin::DesignSample || snapshot.table.rows.is_empty() {
        return Vec::new();
    }
    let labels = [
        ("새로운 여정의 시작", "한 장면에서 시작되는 이야기"),
        ("별빛 여행", "한 장면에 담긴 이야기"),
        ("기억은 계속 빛난다", "다음에, 또 다른 곳에서"),
    ];
    snapshot
        .table
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| VisualItem {
            id: row.row_id.clone(),
            title: row
                .cells
                .iter()
                .find(|cell| matches!(cell.column_id.as_str(), "name" | "title"))
                .and_then(|cell| {
                    if let CellValue::Text(text) = &cell.value {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| labels[index % labels.len()].0.into()),
            subtitle: labels[index % labels.len()].1,
            asset_id: snapshot
                .preview
                .frame
                .as_ref()
                .map(|asset| asset.asset_id.as_str()),
        })
        .collect()
}

pub fn selected_index(items: &[VisualItem<'_>], selected: &Option<String>) -> Option<usize> {
    if items.is_empty() {
        None
    } else {
        Some(
            items
                .iter()
                .position(|item| Some(&item.id) == selected.as_ref())
                .unwrap_or(0),
        )
    }
}

pub fn waveform(
    ui: &mut eframe::egui::Ui,
    snapshot: &WorkspaceSnapshot,
    selected: &Option<String>,
) {
    let sample = snapshot.origin == ReadOrigin::DesignSample && snapshot.source.is_some();
    // Explicit graphical fixture, never represented as decoded source audio.
    let amplitudes = if sample {
        ui.small("디자인 샘플 파형 · 실제 오디오 분석 결과 아님");
        sample_amplitudes()
    } else {
        Vec::new()
    };
    let selection = snapshot
        .table
        .rows
        .iter()
        .find(|row| Some(&row.row_id) == selected.as_ref())
        .and_then(|row| {
            let time = |id: &str| {
                row.cells
                    .iter()
                    .find(|cell| cell.column_id == id)
                    .and_then(|cell| match cell.value {
                        CellValue::TimeUs(time) => Some(time.0),
                        _ => None,
                    })
            };
            Some((time("start")?, time("end")?))
        });
    crate::presentation::components::waveform::show(
        ui,
        &amplitudes,
        snapshot.preview.duration_us.map_or(0, |d| d.0),
        snapshot.preview.position_us.0,
        selection,
    );
}

pub(super) fn visible_end(snapshot: &WorkspaceSnapshot) -> u64 {
    let Some(source_end) = snapshot.preview.duration_us.map(|v| v.0) else {
        return 0;
    };
    let last = snapshot
        .timeline
        .iter()
        .flat_map(|track| &track.segments)
        .map(|s| s.out_us.0)
        .max()
        .unwrap_or(source_end)
        .max(snapshot.preview.position_us.0);
    last.saturating_add(last / 5).min(source_end)
}

pub(super) fn sample_amplitudes() -> Vec<f32> {
    [
        0.08, 0.14, 0.22, 0.43, 0.64, 0.35, 0.91, 0.66, 0.32, 0.18, 0.07, 0.12, 0.32, 0.73, 0.87,
        0.56, 0.34, 0.62, 0.28, 0.15,
    ]
    .repeat(12)
}
