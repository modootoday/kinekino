pub(super) use crate::presentation::components::keyboard_selection::select as keyboard_selection;
pub(super) use crate::presentation::components::tabs::tab;
#[cfg(test)]
use eframe::egui;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focused_navigation_handles_arrows_and_boundaries() {
        let ctx = egui::Context::default();
        let mut ids = Vec::new();
        let _ = crate::presentation::test_support::run(&ctx, Default::default(), |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                for label in ["구간", "자막", "피사체"] {
                    ids.push(ui.button(label).id);
                }
            });
        });
        ctx.memory_mut(|memory| memory.request_focus(ids[0]));
        for (key, expected) in [
            (egui::Key::ArrowDown, 1),
            (egui::Key::End, 2),
            (egui::Key::ArrowDown, 2),
            (egui::Key::Home, 0),
            (egui::Key::ArrowUp, 0),
        ] {
            for _ in 0..2 {
                let _ = crate::presentation::test_support::run(&ctx, Default::default(), |ui| {
                    let ctx = ui.ctx().clone();
                    egui::CentralPanel::default().show(ui, |ui| {
                        let rows = ["구간", "자막", "피사체"].map(|label| ui.button(label));
                        assert!(keyboard_selection(&ctx, &rows, false).is_none());
                    });
                });
            }
            let input = egui::RawInput {
                events: vec![egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
                ..Default::default()
            };
            let _ = crate::presentation::test_support::run(&ctx, input, |ui| {
                let ctx = ui.ctx().clone();
                egui::CentralPanel::default().show(ui, |ui| {
                    let rows = ["구간", "자막", "피사체"].map(|label| ui.button(label));
                    assert_eq!(keyboard_selection(&ctx, &rows, false), Some(expected));
                });
            });
            assert_eq!(ctx.memory(|memory| memory.focused()), Some(ids[expected]));
        }
    }
}
