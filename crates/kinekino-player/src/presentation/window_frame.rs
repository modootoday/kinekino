//! Frameless window resize targets. The backend owns the platform resize operation.
use super::{caption_controls, design_tokens::size};
use eframe::egui::{self, CursorIcon, Rect, ResizeDirection};

fn targets(frame: Rect) -> Vec<(ResizeDirection, Rect, CursorIcon)> {
    let e = size::WINDOW_RESIZE_EDGE;
    let c = size::WINDOW_RESIZE_CORNER;
    let (l, t, r, b) = (frame.left(), frame.top(), frame.right(), frame.bottom());
    let rect = |x1, y1, x2, y2| Rect::from_min_max(egui::pos2(x1, y1), egui::pos2(x2, y2));
    vec![
        (
            ResizeDirection::NorthWest,
            rect(l, t, l + c, t + c),
            CursorIcon::ResizeNwSe,
        ),
        (
            ResizeDirection::North,
            rect(l + c, t, r - caption_controls::GROUP_WIDTH, t + e),
            CursorIcon::ResizeVertical,
        ),
        (
            ResizeDirection::West,
            rect(l, t + c, l + e, b - c),
            CursorIcon::ResizeHorizontal,
        ),
        (
            ResizeDirection::East,
            rect(r - e, t + caption_controls::HEIGHT, r, b - c),
            CursorIcon::ResizeHorizontal,
        ),
        (
            ResizeDirection::SouthWest,
            rect(l, b - c, l + c, b),
            CursorIcon::ResizeNeSw,
        ),
        (
            ResizeDirection::South,
            rect(l + c, b - e, r - c, b),
            CursorIcon::ResizeVertical,
        ),
        (
            ResizeDirection::SouthEast,
            rect(r - c, b - c, r, b),
            CursorIcon::ResizeNwSe,
        ),
    ]
}

pub fn show(ctx: &egui::Context) {
    if ctx.input(|i| {
        i.viewport().maximized.unwrap_or(false) || i.viewport().fullscreen.unwrap_or(false)
    }) || egui::Popup::is_any_open(ctx)
    {
        return;
    }
    for (direction, rect, cursor) in targets(ctx.content_rect()) {
        egui::Area::new(egui::Id::new(("window-resize", format!("{direction:?}"))))
            .order(egui::Order::Foreground)
            .movable(false)
            .constrain(false)
            .fixed_pos(rect.min)
            .show(ctx, |ui| {
                let (_, response) = ui.allocate_exact_size(rect.size(), egui::Sense::drag());
                let response = response.on_hover_cursor(cursor);
                if response.is_pointer_button_down_on()
                    && ctx.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary))
                {
                    crate::infrastructure::diagnostics::record(
                        "window-resize-request",
                        serde_json::json!({"direction":format!("{direction:?}"),"frame":[ctx.content_rect().width(),ctx.content_rect().height()]}),
                    );
                    ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(direction));
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pressing_bottom_corner_requests_native_resize() {
        let ctx = egui::Context::default();
        let frame = Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1180., 760.));
        let input = |events| egui::RawInput {
            screen_rect: Some(frame),
            events,
            ..Default::default()
        };
        for _ in 0..3 {
            super::super::test_support::run(&ctx, input(vec![]), |ui| show(ui.ctx()));
        }
        let point = egui::pos2(1178., 758.);
        let output = super::super::test_support::run(
            &ctx,
            input(vec![
                egui::Event::PointerMoved(point),
                egui::Event::PointerButton {
                    pos: point,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ]),
            |ui| show(ui.ctx()),
        );
        assert!(
            output
                .viewport_output
                .values()
                .any(|viewport| viewport.commands.iter().any(|command| matches!(
                    command,
                    egui::ViewportCommand::BeginResize(ResizeDirection::SouthEast)
                )))
        );
    }
    #[test]
    fn resize_targets_preserve_caption_buttons_and_have_unique_bottom_corner() {
        for dimensions in [egui::vec2(760., 480.), egui::vec2(1180., 760.)] {
            let frame = Rect::from_min_size(egui::Pos2::ZERO, dimensions);
            let caption = Rect::from_min_max(
                egui::pos2(frame.right() - caption_controls::GROUP_WIDTH, 0.),
                egui::pos2(frame.right(), caption_controls::HEIGHT),
            );
            let regions = targets(frame);
            assert!(
                regions
                    .iter()
                    .all(|(_, r, _)| r.is_positive() && !r.intersect(caption).is_positive())
            );
            let corner = frame.max - egui::vec2(1., 1.);
            let matching: Vec<_> = regions
                .iter()
                .filter(|(_, rect, _)| rect.contains(corner))
                .collect();
            assert_eq!(matching.len(), 1);
            assert_eq!(matching[0].0, ResizeDirection::SouthEast);
        }
    }
}
