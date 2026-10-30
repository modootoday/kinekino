use super::{
    artboard::{self, Artwork},
    media_rail::Rail,
};
use crate::presentation::{
    control_icon::{self, Icon},
    design_tokens::{border, color, radius, size, space},
    layout,
};
use eframe::egui;

/// Display-only media surface; callers must supply playback capabilities before enabling controls.
pub fn unavailable(
    ui: &mut egui::Ui,
    artwork: &Artwork<'_>,
    aspect: f32,
    aspect_label: &str,
    height: f32,
) {
    unavailable_with_overlay(ui, artwork, aspect, aspect_label, height, |_, _| {});
}

/// The overlay receives the contained source area, excluding the transport strip.
pub fn unavailable_with_overlay(
    ui: &mut egui::Ui,
    artwork: &Artwork<'_>,
    aspect: f32,
    aspect_label: &str,
    height: f32,
    overlay: impl FnOnce(&mut egui::Ui, egui::Rect),
) {
    egui::Frame::new()
        .fill(color::VIDEO_SURFACE)
        .stroke(egui::Stroke::new(border::THIN, color::BORDER))
        .corner_radius(radius::CONTROL)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.;
            let response = artboard::show(ui, artwork, aspect, height, false);
            overlay(ui, artboard::fitted_rect(response.rect, aspect));
            egui::Frame::new()
                .fill(color::CHROME)
                .inner_margin(space::INLINE_MARGIN)
                .show(ui, |ui| {
                    egui_taffy::tui(ui, ui.id().with("preview-transport"))
                        .reserve_available_width()
                        .style(layout::full(layout::row(space::FIELD)))
                        .show(|tui| {
                            layout::widget(tui, |ui| disabled_icon(ui, Icon::Play, "프리뷰 재생"));
                            layout::widget(tui, |ui| {
                                ui.monospace("--:-- / --:--");
                            });
                            layout::block(tui, |ui| {
                                ui.add_enabled(
                                    false,
                                    Rail {
                                        value: &mut 0.,
                                        maximum: 0.,
                                        step: 1.,
                                        width: ui.available_width(),
                                        compact: false,
                                        label: "프리뷰 재생 위치",
                                    },
                                )
                                .on_disabled_hover_text("프리뷰 재생 위치 연결 필요");
                            });
                            layout::widget(tui, |ui| {
                                disabled_icon(ui, Icon::Volume, "프리뷰 볼륨")
                            });
                            layout::widget(tui, |ui| {
                                ui.small(aspect_label);
                            });
                            layout::widget(tui, |ui| {
                                disabled_icon(ui, Icon::Fullscreen, "프리뷰 전체화면")
                            });
                        });
                });
        });
}

fn disabled_icon(ui: &mut egui::Ui, icon: Icon, label: &str) {
    ui.add_enabled_ui(false, |ui| {
        control_icon::button(ui, icon, label, false, size::CONTROL)
    })
    .inner
    .on_disabled_hover_text(format!("{label} 연결 필요"));
}
