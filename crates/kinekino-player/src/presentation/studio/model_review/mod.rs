mod state;
mod view;
mod worker;

use crate::application::api::ApplicationClientHandle;
use eframe::egui;

pub(super) struct ModelReview {
    state: state::ReviewState,
}

impl ModelReview {
    pub fn new(client: ApplicationClientHandle, available: bool) -> Self {
        Self {
            state: state::ReviewState::new(client, available),
        }
    }

    pub fn poll(&mut self) {
        self.state.poll();
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        view::show(ui, &mut self.state);
    }
}
