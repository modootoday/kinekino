use super::*;
use crate::presentation::{
    components::layer_row::{self, LayerRow},
    design_assets,
    design_tokens::{breakpoint, size, space},
};

impl CanvasEditor {
    pub(super) fn layers(&mut self, ui: &mut egui::Ui, canvas: Option<&CanvasSnapshot>) {
        let mut responses = vec![None; canvas.map_or(0, |canvas| canvas.layers.len())];
        let columns = if ui.ctx().content_rect().width() < breakpoint::STACK_EDITOR {
            1
        } else {
            2
        };
        egui_taffy::tui(ui, ui.id().with("canvas-details"))
            .reserve_available_width()
            .style(layout::full(layout::grid(columns, space::PANEL)))
            .show(|tui| {
                layout::block(tui, |ui| {
                    ui.strong("레이어");
                    ui.add_space(space::FIELD);
                    let Some(canvas) = canvas else {
                        ui.weak("표시할 레이어 없음");
                        return;
                    };
                    for (index, layer) in canvas.layers.iter().rev().enumerate() {
                        let selected = self
                            .selected_layer(canvas)
                            .is_some_and(|selected| selected.layer_id == layer.layer_id);
                        let response = layer_row::show(
                            ui,
                            LayerRow {
                                id: &layer.layer_id,
                                label: &layer.label,
                                selected,
                                visible: layer.visible,
                                locked: layer.locked,
                            },
                            |ui, rect| {
                                if let Some(asset) = layer
                                    .asset
                                    .as_ref()
                                    .and_then(|asset| design_assets::find(&asset.asset_id))
                                {
                                    asset.paint(ui, rect);
                                }
                            },
                        );
                        if response.clicked() {
                            self.selected = Some(layer.layer_id.clone());
                            response.request_focus();
                        }
                        responses[index] = Some(response);
                        ui.add_space(space::INLINE);
                    }
                });
                layout::block(tui, |ui| {
                    ui.strong("원본 프레임 (참고)");
                    ui.add_space(space::FIELD);
                    if let Some(asset) = canvas
                        .and_then(|canvas| canvas.source_asset.as_ref())
                        .and_then(|asset| design_assets::find(&asset.asset_id))
                    {
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), size::SOURCE_REFERENCE_HEIGHT),
                            egui::Sense::hover(),
                        );
                        asset.paint(ui, rect);
                    } else {
                        ui.weak("원본 프레임 연결 필요");
                    }
                });
            });
        let responses: Vec<_> = responses.into_iter().flatten().collect();
        if let Some(index) =
            crate::presentation::components::keyboard_selection::select(ui.ctx(), &responses, false)
            && let Some(layer) = canvas.and_then(|canvas| canvas.layers.iter().rev().nth(index))
        {
            self.selected = Some(layer.layer_id.clone());
        }
        ui.weak("디자인 데모 · 보기·선택만 변경하며 마스크 편집과 저장은 실행하지 않습니다.");
    }
}
