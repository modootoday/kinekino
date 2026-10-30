use super::design_tokens::{border, color, radius};
pub use super::icon_asset::Icon;
use super::player_theme;
use eframe::egui::{self, Response, Sense, Stroke, Ui, vec2};

pub fn button(ui: &mut Ui, icon: Icon, label: &str, selected: bool, size: f32) -> Response {
    render(ui, icon, label, selected, size, false, false)
}

pub fn navigation(ui: &mut Ui, icon: Icon, label: &str, selected: bool, compact: bool) -> Response {
    render(
        ui,
        icon,
        label,
        selected,
        if compact { 40. } else { 44. },
        !compact,
        false,
    )
}

pub fn action(ui: &mut Ui, icon: Icon, label: &str, width: f32, primary: bool) -> Response {
    let response = ui
        .allocate_ui_with_layout(
            vec2(width, 44.),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.set_width(width);
                render(ui, icon, label, false, 44., true, primary)
            },
        )
        .inner;
    ui.painter().rect_stroke(
        response.rect.shrink(1.),
        radius::CONTROL,
        Stroke::new(
            border::THIN,
            if primary {
                player_theme::ACCENT
            } else {
                player_theme::BORDER
            },
        ),
        egui::StrokeKind::Inside,
    );
    response
}

fn render(
    ui: &mut Ui,
    icon: Icon,
    label: &str,
    selected: bool,
    size: f32,
    text: bool,
    accent_icon: bool,
) -> Response {
    let size = size.max(player_theme::CONTROL_SIZE);
    let width = if text {
        ui.available_width().max(size)
    } else {
        size
    };
    let (rect, response) = ui.allocate_exact_size(vec2(width, size), Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), selected, label)
    });
    if selected || response.hovered() || response.is_pointer_button_down_on() {
        let fill = if response.is_pointer_button_down_on() {
            color::PRESSED
        } else if selected {
            player_theme::SELECTED
        } else {
            color::HOVER
        };
        ui.painter()
            .rect_filled(rect.shrink(border::THIN), radius::CONTROL, fill);
    }
    if selected && text {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(rect.min + vec2(0., 5.), vec2(3., rect.height() - 10.)),
            radius::SMALL,
            player_theme::ACCENT,
        );
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(1.),
            radius::CONTROL,
            Stroke::new(border::FOCUS, player_theme::FOCUS),
            egui::StrokeKind::Inside,
        );
    }
    let color = if !ui.is_enabled() {
        color::DISABLED
    } else if selected || matches!(icon, Icon::Brand) {
        player_theme::ACCENT
    } else {
        player_theme::TEXT
    };
    if text {
        ui.painter().with_clip_rect(rect).text(
            rect.left_center() + vec2(size + 4., 0.),
            egui::Align2::LEFT_CENTER,
            label,
            egui::TextStyle::Button.resolve(ui.style()),
            color,
        );
    }
    icon.paint(
        ui,
        egui::Rect::from_center_size(rect.min + vec2(size / 2., size / 2.), vec2(20., 20.)),
        if accent_icon && ui.is_enabled() {
            player_theme::ACCENT
        } else {
            color
        },
    );
    response.on_hover_text(label)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keyboard_activation_preserves_accessible_label_and_minimum_target() {
        let ctx = egui::Context::default();
        egui_extras::install_image_loaders(&ctx);
        let mut id = egui::Id::NULL;
        let _ = crate::presentation::test_support::run(&ctx, Default::default(), |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                let response = button(ui, Icon::Play, "재생", false, 20.);
                assert!(response.rect.width() >= 32. && response.rect.height() >= 32.);
                id = response.id;
            });
        });
        ctx.memory_mut(|memory| memory.request_focus(id));
        let input = egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::Space,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        };
        let mut clicked = false;
        let output = crate::presentation::test_support::run(&ctx, input, |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                clicked = button(ui, Icon::Play, "재생", false, 20.).clicked();
            });
        });
        assert!(clicked);
        assert!(output.platform_output.events.iter().any(|event| matches!(event, egui::output::OutputEvent::Clicked(info) if info.label.as_deref() == Some("재생"))));
    }
}
