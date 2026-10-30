use eframe::egui;
use std::num::NonZeroUsize;

/// Nested Taffy leaves need bounded measurement passes before presenting a frame.
pub fn configure(ctx: &egui::Context) {
    ctx.options_mut(|options| options.max_passes = NonZeroUsize::new(8).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multipass_consumes_a_click_once_and_idle_uses_one_pass() {
        let ctx = egui::Context::default();
        configure(&ctx);
        let mut clicks = 0;
        let mut rect = egui::Rect::NOTHING;
        let mut frame = |events, measuring| {
            let output = crate::presentation::test_support::run(
                &ctx,
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400., 300.),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| {
                    let response = ui.button("검토 동작");
                    rect = response.rect;
                    if response.clicked() {
                        clicks += 1;
                    }
                    if measuring && ctx.current_pass_index() < 6 {
                        ctx.request_discard("layout regression fixture");
                    }
                },
            );
            (output, rect)
        };
        let (_, first_rect) = frame(vec![], false);
        // The button's position is fixed by the first frame, not guessed by the test.
        let pos = first_rect.center();
        frame(
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            false,
        );
        let (output, _) = frame(
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
            true,
        );
        assert_eq!(output.platform_output.num_completed_passes, 7);
        let (idle, _) = frame(vec![], false);
        assert_eq!(idle.platform_output.num_completed_passes, 1);
        assert_eq!(clicks, 1);
    }
}
