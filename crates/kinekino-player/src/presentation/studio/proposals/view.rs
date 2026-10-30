use super::State;
use crate::{application::api::ApplicationClientHandle, presentation::components::inspector};
use eframe::egui;

pub(super) fn show(
    ui: &mut egui::Ui,
    state: &mut State,
    available: bool,
    client: &ApplicationClientHandle,
) {
    inspector::panel_with_state(
        ui,
        "proposal-review",
        state,
        |ui, state| super::body::show(ui, state, available, client),
        |ui, state| super::actions::show(ui, state, available, client),
    );
}
