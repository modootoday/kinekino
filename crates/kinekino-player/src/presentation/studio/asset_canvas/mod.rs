mod board;
mod inspector;
mod layers;
mod toolbar;
mod view;
use crate::application::workspace_read::{CanvasSnapshot, CanvasViewMode, LayerSnapshot};
use crate::presentation::design_tokens::{breakpoint, size, space};
use crate::presentation::{
    control_icon::{self, Icon},
    layout,
};
use eframe::egui;

#[derive(Default)]
pub(super) struct CanvasEditor {
    tool: usize,
    zoom: Option<u16>,
    pan: bool,
    offset: egui::Vec2,
    mode: Option<CanvasViewMode>,
    selected: Option<String>,
}

impl CanvasEditor {
    pub fn selected_layer<'a>(&self, canvas: &'a CanvasSnapshot) -> Option<&'a LayerSnapshot> {
        self.selected
            .as_ref()
            .or(canvas.selected_layer_id.as_ref())
            .and_then(|id| canvas.layers.iter().find(|layer| &layer.layer_id == id))
            .or_else(|| canvas.layers.last())
    }

    pub fn show(&mut self, ui: &mut egui::Ui, canvas: Option<&CanvasSnapshot>) {
        let mode = self
            .mode
            .unwrap_or(canvas.map_or(CanvasViewMode::Composite, |canvas| canvas.view_mode));
        let mut selected_mode = canvas.map(|_| match mode {
            CanvasViewMode::Original => 0,
            CanvasViewMode::Alpha => 1,
            CanvasViewMode::Composite => 2,
        });
        let source_size = canvas.map_or(egui::vec2(1., 1.), |canvas| {
            egui::vec2(canvas.width_px as f32, canvas.height_px as f32)
        });
        let height = (ui.available_height() - 220.).clamp(160., 420.);
        let tool_width = if ui.ctx().content_rect().width() < breakpoint::STACK_EDITOR {
            size::TOOL_RAIL_COMPACT
        } else {
            size::TOOL_RAIL
        };
        let fit_percent = (((ui.available_width() - tool_width - space::PANEL) / source_size.x)
            .min(height / source_size.y)
            * 100.)
            .round()
            .clamp(1., 400.) as u16;
        egui_taffy::tui(ui, ui.id().with("canvas-toolbar"))
            .reserve_available_width()
            .style(layout::full(layout::row(space::FIELD)))
            .show(|tui| {
                layout::block(tui, |ui| {
                    ui.set_max_width(size::TAB_MIN * 3. + space::FIELD * 2.);
                    ui.add_enabled_ui(canvas.is_some(), |ui| {
                        if crate::presentation::components::segmented::show(
                            ui,
                            "canvas-view-mode",
                            &["원본", "알파", "합성"],
                            &mut selected_mode,
                        ) {
                            self.mode = Some(match selected_mode {
                                Some(0) => CanvasViewMode::Original,
                                Some(1) => CanvasViewMode::Alpha,
                                _ => CanvasViewMode::Composite,
                            });
                        }
                    });
                });
                layout::widget(tui, |ui| {
                    ui.add_enabled_ui(canvas.is_some(), |ui| {
                        toolbar::zoom_controls(
                            ui,
                            &mut self.zoom,
                            fit_percent,
                            &mut self.pan,
                            &mut self.offset,
                        )
                    });
                });
            });
        ui.add_space(space::FIELD);
        egui_taffy::tui(ui, ui.id().with("canvas-body"))
            .reserve_available_width()
            .style(layout::full(layout::sidebar(tool_width, 12.)))
            .show(|tui| {
                layout::block(tui, |ui| {
                    self.tools(ui, tool_width < 100., canvas.is_some())
                });
                layout::block(tui, |ui| self.board(ui, canvas, height));
            });
        ui.add_space(space::PANEL);
        self.layers(ui, canvas);
    }

    fn tools(&mut self, ui: &mut egui::Ui, compact: bool, enabled: bool) {
        for (index, (icon, label)) in [
            (Icon::BrushAdd, "포함"),
            (Icon::MaskExclude, "제외"),
            (Icon::Eraser, "지우기"),
        ]
        .into_iter()
        .enumerate()
        {
            let response = ui
                .add_enabled_ui(enabled, |ui| {
                    control_icon::navigation(ui, icon, label, self.tool == index, compact)
                })
                .inner;
            if response.clicked() {
                self.tool = index;
                response.request_focus();
            }
            response.on_hover_text("디자인 데모 도구 선택 · 실제 마스크는 변경하지 않습니다.");
        }
        ui.separator();
        if !compact {
            ui.add_space(space::FIELD);
            crate::presentation::components::brush_settings::unavailable(ui);
        }
    }
}
