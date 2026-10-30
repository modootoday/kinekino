//! Headless egui frames deliberately have no renderer to consume texture uploads.
use eframe::egui;

pub fn run(
    ctx: &egui::Context,
    input: egui::RawInput,
    body: impl FnMut(&mut egui::Ui),
) -> egui::FullOutput {
    let mut output = ctx.run_ui(input, body);
    output.textures_delta.clear();
    output
}
