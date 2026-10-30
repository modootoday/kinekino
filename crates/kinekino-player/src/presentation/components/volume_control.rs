use super::media_rail::Rail;
use crate::presentation::{
    control_icon::{self, Icon},
    design_tokens::{breakpoint, size},
};
use eframe::egui;

fn popup_id() -> egui::Id {
    egui::Id::new("player-volume-popup")
}

pub fn is_open(ctx: &egui::Context) -> bool {
    egui::Popup::is_id_open(ctx, popup_id())
}

pub fn show(ui: &mut egui::Ui, value: &mut f64) -> bool {
    let button = control_icon::button(
        ui,
        if *value <= 0. {
            Icon::VolumeZero
        } else {
            Icon::Volume
        },
        &format!("볼륨 {:.0}% · 설정", *value),
        false,
        size::CONTROL,
    );
    let opening = button.clicked() && !is_open(ui.ctx());
    let mut changed = false;
    egui::Popup::menu(&button)
        .id(popup_id())
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.set_min_width(size::VOLUME_POPOVER);
            ui.label(format!("볼륨 {:.0}%", *value));
            let response = rail(ui, value, size::VOLUME_POPOVER);
            if opening {
                response.request_focus();
            }
            changed = response.changed();
        });
    if ui.ctx().content_rect().width() >= breakpoint::STACK_EDITOR {
        changed |= rail(ui, value, size::VOLUME_INLINE).changed();
    }
    changed
}

fn rail(ui: &mut egui::Ui, value: &mut f64, width: f32) -> egui::Response {
    let label = format!("볼륨 {:.0}%", *value);
    ui.add(Rail {
        value,
        maximum: 100.,
        step: 1.,
        width,
        compact: false,
        label: "볼륨",
    })
    .on_hover_text(label)
}
