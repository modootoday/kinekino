use super::{CanvasEditor, view::CanvasInspectorView};
use crate::presentation::components::{
    button::{Button, Variant},
    inspector::{coordinates, panel, readonly, scalar, section},
};
use crate::{
    application::workspace_read::WorkspaceSnapshot,
    presentation::{
        design_assets,
        design_tokens::{breakpoint, size, space},
        layout,
    },
};
use eframe::egui;

impl CanvasEditor {
    pub fn inspector(&self, ui: &mut egui::Ui, snapshot: &WorkspaceSnapshot) {
        let layer = snapshot
            .canvas
            .as_ref()
            .and_then(|canvas| self.selected_layer(canvas));
        let view = CanvasInspectorView::new(snapshot, layer);
        panel(
            ui,
            "asset-inspector-body",
            |ui| {
                ui.strong("프레임 정보");
                ui.add_space(space::FIELD);
                egui_taffy::tui(ui, ui.id().with("frame-info"))
                    .reserve_available_width()
                    .style(layout::full(layout::grid(2, space::FIELD)))
                    .show(|tui| {
                        layout::block(tui, |ui| {
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), size::SOURCE_THUMBNAIL),
                                egui::Sense::hover(),
                            );
                            if let Some(asset) = view
                                .source_frame
                                .and_then(|asset| design_assets::find(&asset.asset_id))
                            {
                                asset.paint(ui, rect);
                            }
                        });
                        layout::block(tui, |ui| {
                            ui.label(view.position_us.map_or_else(
                                || "PTS —".into(),
                                |us| {
                                    format!(
                                        "PTS {:02}:{:02}.{:03}",
                                        us / 60_000_000,
                                        us / 1_000_000 % 60,
                                        us / 1000 % 1000
                                    )
                                },
                            ));
                            ui.small("원본 좌표와 PTS 보존");
                        });
                    });
                if ui.ctx().content_rect().width() < breakpoint::STACK_EDITOR {
                    section(ui, "브러시 설정");
                    crate::presentation::components::brush_settings::unavailable(ui);
                }
                section(ui, "알파 추출");
                scalar(ui, "임계값", view.alpha.map(|v| v.threshold), 0.0..=1.0);
                scalar(
                    ui,
                    "페더 (px)",
                    view.alpha.map(|v| v.feather_px),
                    0.0..=64.0,
                );
                scalar(
                    ui,
                    "경계 정제",
                    view.alpha.map(|v| v.edge_refinement),
                    0.0..=1.0,
                );
                if view.alpha.is_some() {
                    ui.small("디자인 예시 · 편집/추론에 적용되지 않습니다.");
                } else {
                    ui.small("알파 설정 정보 연결 필요");
                }
                section(ui, "피벗 포인트");
                ui.add_enabled_ui(false, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let _ = ui.radio(false, "중앙");
                        let _ = ui.radio(false, "하단");
                        let _ = ui.radio(view.layer.is_some(), "지정 좌표");
                    });
                });
                coordinates(
                    ui,
                    "pivot",
                    &[
                        ("X", view.layer.map(|l| i64::from(l.pivot_px.x))),
                        ("Y", view.layer.map(|l| i64::from(l.pivot_px.y))),
                    ],
                );
                ui.small("원본 캔버스 절대 좌표 · px");
                section(ui, "경계 영역 (바운딩 박스)");
                coordinates(
                    ui,
                    "bounds",
                    &[
                        ("X", view.layer.map(|l| i64::from(l.bounds_px.x))),
                        ("Y", view.layer.map(|l| i64::from(l.bounds_px.y))),
                        ("W", view.layer.map(|l| i64::from(l.bounds_px.width))),
                        ("H", view.layer.map(|l| i64::from(l.bounds_px.height))),
                    ],
                );
                if let Some(layer) = view.layer {
                    ui.add(egui::Label::new(&layer.label).truncate())
                        .on_hover_text(&layer.label);
                }
                if let Some(canvas) = view.canvas {
                    ui.small(format!(
                        "캔버스 {} × {} px",
                        canvas.width_px, canvas.height_px
                    ));
                }
                section(ui, "변환 스케일");
                readonly(ui, "출력 배율", "— · 지원 정보 필요");
                readonly(
                    ui,
                    "색상 공간",
                    if view.sample {
                        "sRGB · 디자인 예시"
                    } else {
                        "— · 확인 필요"
                    },
                );
                section(ui, "내보내기 옵션");
                for label in [
                    "RGBA PNG (투명 배경)",
                    "마스크 PNG (알파 마스크)",
                    "매니페스트 JSON",
                ] {
                    let mut selected = view.sample;
                    ui.add_enabled(false, egui::Checkbox::new(&mut selected, label))
                        .on_disabled_hover_text("출력 계약/저장 유스케이스 연결이 필요합니다.");
                }
            },
            |ui| {
                ui.separator();
                ui.add_space(space::FIELD);
                egui_taffy::tui(ui, ui.id().with("asset-inspector-actions"))
                    .reserve_available_width()
                    .style(layout::full(layout::grid(2, space::FIELD)))
                    .show(|tui| {
                        for (label, variant) in [
                            ("합성 검증", Variant::Secondary),
                            ("패키지 저장", Variant::Primary),
                        ] {
                            layout::block(tui, |ui| {
                                ui.add_enabled(false, Button::new(label, variant))
                                    .on_disabled_hover_text(
                                        "검증·저장 유스케이스 연결이 필요합니다.",
                                    );
                            });
                        }
                    });
                ui.small(if view.sample {
                    "디자인 데모 · 실제 편집과 저장은 실행하지 않습니다."
                } else {
                    "선택 레이어 정보 · 읽기 전용"
                });
            },
        );
    }
}
