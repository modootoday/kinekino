use super::*;
use crate::presentation::{
    components::episode_list::{self, Episode},
    design_tokens::{color, size, space},
};

impl PreviewApp {
    pub(super) fn show_episode_review(&mut self, ui: &mut egui::Ui) {
        let titles: Vec<_> = (1..=18).map(|index| {
            if self.scenario == PreviewScenario::LongLabels {
                format!("{index:02} 아주 긴 회차 제목 · 한국어·日本語·English 줄임 확인 · 디자인 예시")
            } else { format!("{index:02} 회차 제목 · 디자인 예시") }
        }).collect();
        let ids: Vec<_> = (1..=18)
            .map(|index| format!("design-episode-{index}"))
            .collect();
        let episodes: Vec<_> = if self.scenario == PreviewScenario::Empty {
            Vec::new()
        } else {
            titles
                .iter()
                .zip(&ids)
                .map(|(title, id)| Episode { id, title })
                .collect()
        };
        egui::Panel::right("design-episode-panel")
            .exact_size(size::EPISODE_PANEL)
            .frame(
                egui::Frame::new()
                    .fill(color::PANEL)
                    .inner_margin(space::INLINE_MARGIN),
            )
            .show(ui, |ui| {
                let action = episode_list::show(
                    ui,
                    "회차 패널 디자인 샘플",
                    &episodes,
                    self.episode_current.filter(|_| !episodes.is_empty()),
                );
                if action.close {
                    self.episode_review = false;
                }
                if let Some(index) = action.activate {
                    self.episode_current = Some(index);
                }
            });
        egui::CentralPanel::default().frame(egui::Frame::new().fill(color::VIDEO_SURFACE)).show(ui, |ui| {
            ui.strong("회차 패널 · 디자인 샘플");
            ui.label("실제 회차·영상·연결 상태가 아닙니다. 활성화는 샘플의 현재 행만 변경합니다.");
            ui.label("방향키/Home/End: 포커스 이동 · Enter/Space: 샘플 활성화 · Escape: 제작 데모 복귀");
        });
    }
}
