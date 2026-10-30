use super::super::control_icon::{self, Icon};
use super::*;
use crate::presentation::design_tokens::size;
impl PlayerApp {
    pub(super) fn choose_episode(&mut self, url: String) {
        let native =
            self.route.native_preferred() || self.mpv.is_some() || self.transition.native_intent();
        let mut paused = if let Some(player) = &mut self.mpv {
            self.audio_intents
                .pending_requested(crate::domain::api_values::DecimalU64(
                    player.control_handle().instance_id(),
                ))
                .ok()
                .flatten()
                .map_or_else(
                    || self.native_pause.intent(player.snapshot().paused),
                    |requested| requested.paused,
                )
        } else {
            self.session.playback.as_ref().is_none_or(|p| p.paused)
        };
        // Selecting the first episode from a catalogue starts playback when the browser permits it.
        if self.session.playback.is_none() && self.mpv.is_none() {
            paused = false;
        }
        self.transition
            .select(url.clone(), native, paused, self.ui_started.elapsed());
        diagnostics::record(
            "episode-selection",
            json!({"index":self.episodes.entries.iter().position(|e|e.url==url),"native":native,"paused":self.transition.paused_intent()}),
        );
        self.navigate(url);
    }
    pub(super) fn episode_button(&mut self, ui: &mut egui::Ui, delta: isize) {
        let (label, icon) = if delta < 0 {
            ("이전화", Icon::Previous)
        } else {
            ("다음화", Icon::Next)
        };
        let target = self.episodes.adjacent(delta);
        if ui
            .add_enabled_ui(target.is_some(), |ui| {
                control_icon::button(ui, icon, label, false, 32.)
            })
            .inner
            .clicked()
            && let Some(url) = target
        {
            self.choose_episode(url);
        }
    }
    pub(super) fn episode_sidebar(&mut self, ui: &mut egui::Ui) {
        if !self.episode_panel || !self.chrome_visible {
            return;
        }
        let mut selected = None;
        egui::Panel::right("episodes")
            .default_size(size::EPISODE_PANEL)
            .min_size(size::EPISODE_PANEL_MIN)
            .max_size(size::EPISODE_PANEL_MAX)
            .show(ui, |ui| {
                use crate::presentation::components::episode_list::{self, Episode};
                let episodes: Vec<_> = self
                    .episodes
                    .entries
                    .iter()
                    .map(|episode| Episode {
                        id: &episode.url,
                        title: &episode.title,
                    })
                    .collect();
                let action =
                    episode_list::show(ui, &self.episodes.title, &episodes, self.episodes.current);
                if action.close {
                    self.episode_panel = false;
                }
                if let Some(index) = action.activate {
                    selected = Some(self.episodes.entries[index].url.clone());
                }
            });
        if let Some(url) = selected {
            self.choose_episode(url);
        }
    }
}
