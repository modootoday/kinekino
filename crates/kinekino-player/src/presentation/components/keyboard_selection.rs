use eframe::egui::{self, Response};

pub fn select(ctx: &egui::Context, rows: &[Response], horizontal: bool) -> Option<usize> {
    let current = rows.iter().position(Response::has_focus)?;
    ctx.memory_mut(|memory| {
        memory.set_focus_lock_filter(
            rows[current].id,
            egui::EventFilter {
                horizontal_arrows: horizontal,
                vertical_arrows: !horizontal,
                ..Default::default()
            },
        )
    });
    let previous = if horizontal {
        egui::Key::ArrowLeft
    } else {
        egui::Key::ArrowUp
    };
    let next = if horizontal {
        egui::Key::ArrowRight
    } else {
        egui::Key::ArrowDown
    };
    let target = ctx.input_mut(|input| {
        if input.consume_key(egui::Modifiers::NONE, egui::Key::Home) {
            Some(0)
        } else if input.consume_key(egui::Modifiers::NONE, egui::Key::End) {
            Some(rows.len() - 1)
        } else if input.consume_key(egui::Modifiers::NONE, previous) {
            Some(current.saturating_sub(1))
        } else if input.consume_key(egui::Modifiers::NONE, next) {
            Some((current + 1).min(rows.len() - 1))
        } else {
            None
        }
    })?;
    rows[target].request_focus();
    rows[target].scroll_to_me(Some(egui::Align::Center));
    Some(target)
}
