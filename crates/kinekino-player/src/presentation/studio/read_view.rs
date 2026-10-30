//! Presentation of the shared workspace read model. Asset IDs are explicitly registered.
use crate::application::workspace_read::{ReadOrigin, WorkspaceSnapshot};
use crate::presentation::design_tokens::{border, radius};
use crate::presentation::{components::data_table, layout, player_theme};
use eframe::egui;

pub(super) fn preview(ui: &mut egui::Ui, source: &str, snapshot: &WorkspaceSnapshot) {
    let asset = snapshot
        .preview
        .frame
        .as_ref()
        .filter(|_| snapshot.origin == ReadOrigin::DesignSample)
        .and_then(|frame| crate::presentation::design_assets::find(&frame.asset_id));
    let Some(asset) = asset else {
        super::editor::preview(ui, source);
        return;
    };
    egui::Frame::new()
        .fill(crate::presentation::design_tokens::color::VIDEO_SURFACE)
        .stroke(egui::Stroke::new(border::THIN, player_theme::BORDER))
        .corner_radius(radius::CONTROL)
        .show(ui, |ui| {
            let height = (ui.available_height() - 240.).clamp(140., 340.);
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), height),
                egui::Sense::hover(),
            );
            asset.paint(ui, rect);
            let position =
                data_table::value(&crate::application::workspace_read::CellValue::TimeUs(
                    snapshot.preview.position_us,
                ));
            let duration = snapshot
                .preview
                .duration_us
                .map(|time| {
                    data_table::value(&crate::application::workspace_read::CellValue::TimeUs(time))
                })
                .unwrap_or_else(|| "--:--".into());
            egui_taffy::tui(ui, ui.id().with("preview-status"))
                .reserve_available_width()
                .style(layout::full(layout::row(12.)))
                .show(|tui| {
                    layout::block(tui, |ui| {
                        ui.monospace(format!("{position} / {duration}"));
                    });
                    layout::widget(tui, |ui| {
                        ui.weak("디자인용 예시 프레임");
                    });
                });
        });
}

pub(super) fn timeline(ui: &mut egui::Ui, snapshot: &WorkspaceSnapshot) {
    let end = snapshot
        .timeline
        .iter()
        .flat_map(|track| &track.segments)
        .map(|segment| segment.out_us.0)
        .max()
        .unwrap_or(1)
        .saturating_add(5_000_000);
    egui::Frame::group(ui.style())
        .fill(player_theme::PANEL)
        .inner_margin(8)
        .show(ui, |ui| {
            for track in &snapshot.timeline {
                ui.small(&track.label);
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 42.),
                    egui::Sense::hover(),
                );
                for step in 0..=10 {
                    let x = egui::lerp(rect.x_range(), step as f32 / 10.);
                    ui.painter().line_segment(
                        [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                        egui::Stroke::new(border::THIN, player_theme::BORDER),
                    );
                }
                for segment in &track.segments {
                    let left = egui::lerp(rect.x_range(), segment.in_us.0 as f32 / end as f32);
                    let right = egui::lerp(rect.x_range(), segment.out_us.0 as f32 / end as f32);
                    let bounds = egui::Rect::from_min_max(
                        egui::pos2(left, rect.top() + 6.),
                        egui::pos2(right, rect.bottom() - 6.),
                    );
                    ui.painter()
                        .rect_filled(bounds, radius::CONTROL, player_theme::SELECTED);
                    ui.painter().rect_stroke(
                        bounds,
                        radius::CONTROL,
                        egui::Stroke::new(border::THIN, player_theme::ACCENT),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().with_clip_rect(bounds.shrink(4.)).text(
                        bounds.center(),
                        egui::Align2::CENTER_CENTER,
                        &segment.label,
                        egui::FontId::proportional(12.),
                        player_theme::TEXT,
                    );
                    ui.interact(
                        bounds,
                        ui.id().with(&segment.segment_id),
                        egui::Sense::hover(),
                    )
                    .on_hover_text(&segment.label);
                }
            }
        });
}
