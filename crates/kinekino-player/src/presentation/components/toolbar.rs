use crate::presentation::{
    control_icon::{self, Icon},
    design_tokens::{border, color, radius, size, space},
    layout,
};
use eframe::egui;

pub fn unavailable_grouped(ui: &mut egui::Ui, groups: &[&[(Icon, &str)]]) {
    grouped(ui, groups, true);
}

pub fn unavailable_strip(ui: &mut egui::Ui, groups: &[&[(Icon, &str)]]) {
    egui::Frame::new()
        .fill(color::PANEL)
        .stroke(egui::Stroke::new(border::THIN, color::BORDER))
        .corner_radius(radius::CONTROL)
        .inner_margin(space::INLINE_MARGIN)
        .show(ui, |ui| grouped(ui, groups, false));
}

fn grouped(ui: &mut egui::Ui, groups: &[&[(Icon, &str)]], button_frames: bool) {
    use egui_taffy::taffy::{FlexWrap, Style};
    egui_taffy::tui(ui, ui.id().with("grouped-actions"))
        .reserve_available_width()
        .style(Style {
            flex_wrap: FlexWrap::Wrap,
            ..layout::full(layout::row(space::GROUP))
        })
        .show(|tui| {
            for (index, group) in groups.iter().enumerate() {
                layout::widget(tui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = space::FIELD;
                        if index > 0 {
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(border::THIN, size::CONTROL),
                                egui::Sense::hover(),
                            );
                            ui.painter().line_segment(
                                [rect.center_top(), rect.center_bottom()],
                                egui::Stroke::new(border::THIN, color::BORDER),
                            );
                        }
                        for (icon, label) in *group {
                            ui.add_enabled(
                                false,
                                egui::Button::image_and_text(
                                    icon.image()
                                        .fit_to_exact_size(egui::Vec2::splat(size::TOOLBAR_ICON)),
                                    *label,
                                )
                                .frame(button_frames)
                                .min_size(egui::vec2(0., size::CONTROL)),
                            )
                            .on_disabled_hover_text("편집 명령 연결이 필요합니다.");
                        }
                    });
                });
            }
        });
}

pub fn canvas_controls(ui: &mut egui::Ui, canvas: egui::Rect, actions: &[(Icon, &str)]) {
    let width = actions.len() as f32 * size::CONTROL
        + actions.len().saturating_sub(1) as f32 * space::TIGHT
        + space::INLINE * 2.;
    let rect = egui::Rect::from_min_size(
        canvas.right_top() + egui::vec2(-width - space::FIELD, space::FIELD),
        egui::vec2(width, size::TOOL_RAIL_COMPACT),
    );
    let mut overlay = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    overlay.set_clip_rect(canvas.intersect(ui.clip_rect()));
    overlay_frame().show(&mut overlay, |ui| {
        egui_taffy::tui(ui, ui.id().with("canvas-controls"))
            .style(layout::row(space::TIGHT))
            .show(|tui| {
                for (icon, label) in actions {
                    layout::widget(tui, |ui| {
                        ui.add_enabled_ui(false, |ui| {
                            control_icon::button(ui, *icon, label, false, size::CONTROL)
                        })
                        .inner
                        .on_disabled_hover_text(format!("{label} · 보기 명령 연결이 필요합니다."));
                    });
                }
            });
    });
}

pub fn overlay_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(color::CHROME)
        .stroke(egui::Stroke::new(border::THIN, color::BORDER))
        .corner_radius(radius::CONTROL)
        .inner_margin(space::INLINE_MARGIN)
}

pub fn canvas_unavailable(ui: &mut egui::Ui, canvas: egui::Rect, actions: &[(Icon, &str)]) {
    let height = actions.len() as f32 * size::CONTROL
        + actions.len().saturating_sub(1) as f32 * space::TIGHT
        + space::INLINE * 2.;
    let rect = egui::Rect::from_min_size(
        canvas.min + egui::vec2(space::FIELD, space::FIELD),
        egui::vec2(size::TOOL_RAIL_COMPACT, height),
    );
    let mut overlay = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    overlay.set_clip_rect(canvas.intersect(ui.clip_rect()));
    overlay_frame().show(&mut overlay, |ui| {
        egui_taffy::tui(ui, ui.id().with("canvas-tool-rail"))
            .style(layout::column(space::TIGHT))
            .show(|tui| {
                for (icon, label) in actions {
                    layout::widget(tui, |ui| {
                        ui.add_enabled_ui(false, |ui| {
                            control_icon::button(ui, *icon, label, false, size::CONTROL)
                        })
                        .inner
                        .on_disabled_hover_text(format!("{label} · 편집 명령 연결이 필요합니다."));
                    });
                }
            });
    });
}

pub fn unavailable(ui: &mut egui::Ui, id: &str, actions: &[(Icon, &str)]) {
    use egui_taffy::taffy::{FlexWrap, Style};
    egui_taffy::tui(ui, ui.id().with(("inline-actions", id)))
        .reserve_available_width()
        .style(Style {
            flex_wrap: FlexWrap::Wrap,
            ..layout::full(layout::row(space::FIELD))
        })
        .show(|tui| {
            for (icon, label) in actions {
                layout::widget(tui, |ui| {
                    ui.add_enabled(
                        false,
                        egui::Button::image_and_text(
                            icon.image()
                                .fit_to_exact_size(egui::Vec2::splat(size::TOOLBAR_ICON)),
                            *label,
                        )
                        .frame(false)
                        .min_size(egui::vec2(0., size::CONTROL)),
                    )
                    .on_disabled_hover_text(format!("{label} · 명령 연결이 필요합니다."));
                });
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_toolbars_settle_and_idle_in_one_pass() {
        let ctx = egui::Context::default();
        crate::presentation::layout::configure(&ctx);
        for width in [240., 760., 1440.] {
            for frame in 0..3 {
                let output = crate::presentation::test_support::run(
                    &ctx,
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 500.),
                        )),
                        ..Default::default()
                    },
                    |ui| {
                        unavailable(
                            ui,
                            "transport",
                            &[
                                (Icon::Play, "Play"),
                                (Icon::Volume, "Volume"),
                                (Icon::Fullscreen, "Fullscreen"),
                            ],
                        );
                        unavailable(
                            ui,
                            "edit",
                            &[
                                (Icon::Plus, "Add segment"),
                                (Icon::Segments, "Edit segment"),
                                (Icon::Delete, "Delete"),
                            ],
                        );
                    },
                );
                assert!(!ctx.will_discard(), "unsettled toolbars at width {width}");
                if frame == 2 {
                    assert_eq!(output.platform_output.num_completed_passes, 1);
                }
            }
        }
    }
}
