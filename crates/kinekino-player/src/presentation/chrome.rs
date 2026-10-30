use super::*;
#[path = "chrome/interaction.rs"]
mod interaction;
pub(super) use interaction::{ChromeInteraction, ControlRegion};
impl PlayerApp {
    pub(super) fn update_chrome(&mut self, ctx: &egui::Context) {
        let pointer = self.activity.pointer_moved();
        let input = ctx.input(|i| !i.events.is_empty());
        let interaction = ctx.input(|input| {
            self.chrome_interaction
                .observe(input.focused, input.pointer.hover_pos())
        });
        let paused = if let Some(player) = &mut self.mpv {
            player.snapshot().paused || !self.native_started
        } else {
            self.session.playback.as_ref().is_none_or(|p| p.paused)
        };
        let pinned = interaction.hovering_controls
            || self.studio.open
            || self.episode_panel
            || self.native_control_error.is_some()
            || paused
            || self.address_focused
            || self.scrub.is_some()
            || egui::Popup::is_any_open(ctx)
            || ctx.egui_wants_keyboard_input()
            || ctx.egui_is_using_pointer();
        let web_input = std::mem::take(&mut self.web_input);
        let visible = self.visibility.visible(
            self.ui_started.elapsed(),
            self.fullscreen,
            pointer || input || web_input || interaction.resumed,
            pinned,
        );
        if visible != self.chrome_visible {
            diagnostics::record(
                "controls-visibility",
                json!({"visible":visible,"fullscreen":self.fullscreen}),
            );
        }
        self.chrome_visible = visible;
    }
}
