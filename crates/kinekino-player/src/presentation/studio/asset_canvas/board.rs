use super::*;
use crate::presentation::design_tokens::{border, radius, text};
use crate::presentation::{
    components::{canvas_transform::SourceTransform, checkerboard, selection_frame},
    design_assets, player_theme,
};

impl CanvasEditor {
    pub(super) fn board(
        &mut self,
        ui: &mut egui::Ui,
        canvas: Option<&CanvasSnapshot>,
        height: f32,
    ) {
        egui::Frame::new()
            .fill(player_theme::CHROME)
            .stroke(egui::Stroke::new(border::THIN, player_theme::BORDER))
            .corner_radius(radius::CONTROL)
            .show(ui, |ui| {
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), height),
                    if self.pan {
                        egui::Sense::drag()
                    } else {
                        egui::Sense::hover()
                    },
                );
                let Some(canvas) = canvas else {
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "프레임 또는 2D 에셋을 선택해 주세요",
                        egui::FontId::proportional(text::BODY),
                        player_theme::SECONDARY,
                    );
                    return;
                };
                if self.pan && response.dragged() {
                    self.offset += ui.input(|input| input.pointer.delta());
                }
                response.on_hover_cursor(if self.pan {
                    egui::CursorIcon::Grab
                } else {
                    egui::CursorIcon::Default
                });
                let source_size = egui::vec2(canvas.width_px as f32, canvas.height_px as f32);
                let transform = if let Some(zoom) = self.zoom {
                    SourceTransform::at_scale(source_size, rect, f32::from(zoom) / 100.)
                } else {
                    SourceTransform::fit(source_size, rect, 1.)
                };
                let Some(mut transform) = transform else {
                    return;
                };
                transform.translate(self.offset);
                let clip = rect.intersect(ui.clip_rect());
                let mode = self.mode.unwrap_or(canvas.view_mode);
                if mode == CanvasViewMode::Alpha {
                    ui.painter().with_clip_rect(clip).rect_filled(
                        transform.image_rect,
                        radius::NONE,
                        crate::presentation::design_tokens::color::VIDEO_SURFACE,
                    );
                } else {
                    checkerboard::paint(ui, transform.image_rect.intersect(clip));
                }
                if mode == CanvasViewMode::Original {
                    if let Some(asset) = canvas
                        .source_asset
                        .as_ref()
                        .and_then(|asset| design_assets::find(&asset.asset_id))
                    {
                        asset.paint_in_rect(ui, transform.image_rect, clip);
                    }
                } else {
                    let selected = self.selected_layer(canvas);
                    for layer in &canvas.layers {
                        if !layer.visible
                            || (mode == CanvasViewMode::Alpha
                                && selected
                                    .is_none_or(|selected| selected.layer_id != layer.layer_id))
                        {
                            continue;
                        }
                        if let Some(asset) = layer
                            .asset
                            .as_ref()
                            .and_then(|asset| design_assets::find(&asset.asset_id))
                        {
                            let bounds = transform.bounds(source_bounds(layer));
                            if mode == CanvasViewMode::Alpha {
                                if !asset.paint_alpha(ui, bounds, clip) {
                                    ui.painter().with_clip_rect(clip).text(
                                        bounds.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "알파 미리보기 연결 필요",
                                        egui::FontId::proportional(text::BODY),
                                        player_theme::SECONDARY,
                                    );
                                }
                            } else {
                                asset.paint_in_rect(ui, bounds, clip);
                            }
                        }
                    }
                    if let Some(layer) = selected {
                        let bounds = transform.bounds(source_bounds(layer));
                        let mut overlay = ui.new_child(egui::UiBuilder::new().max_rect(rect));
                        overlay.set_clip_rect(clip);
                        if layer.asset.is_some() {
                            selection_frame::paint(
                                &overlay,
                                bounds,
                                transform.point(egui::pos2(
                                    layer.pivot_px.x as f32,
                                    layer.pivot_px.y as f32,
                                )),
                            );
                        } else {
                            overlay.painter().text(
                                bounds.center(),
                                egui::Align2::CENTER_CENTER,
                                "레이어 에셋 연결 필요",
                                egui::FontId::proportional(text::BODY),
                                player_theme::ACCENT,
                            );
                        }
                    }
                }
            });
    }
}

fn source_bounds(layer: &LayerSnapshot) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(layer.bounds_px.x as f32, layer.bounds_px.y as f32),
        egui::vec2(layer.bounds_px.width as f32, layer.bounds_px.height as f32),
    )
}
