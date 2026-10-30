//! Explicit recording-mode controls; never enabled by ordinary product startup.
use eframe::egui;

pub fn window_size(ui: &mut egui::Ui) {
    let current = ui.ctx().content_rect().size();
    let mut requested = None;
    egui::ComboBox::from_id_salt("validation-window-size")
        .selected_text(format!("검토 {:.0}×{:.0}", current.x, current.y))
        .width(super::design_tokens::size::CONTROL * 4.)
        .show_ui(ui, |ui| {
            for [width, height] in [[760, 760], [980, 760], [1180, 760], [1440, 900]] {
                if ui
                    .selectable_label(
                        current == egui::vec2(width as f32, height as f32),
                        format!("{width}×{height}"),
                    )
                    .clicked()
                {
                    requested = Some(egui::vec2(width as f32, height as f32));
                }
            }
        });
    if let Some(size) = requested {
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
    }
}
