use super::design_tokens::{border, color, radius};
use super::{icon_asset::Icon, player_theme};
use eframe::egui::{self, Response, Sense, Ui, vec2};

pub const WIDTH: f32 = super::design_tokens::size::CAPTION_WIDTH;
pub const HEIGHT: f32 = super::design_tokens::size::TITLE_BAR;
pub const GROUP_WIDTH: f32 = WIDTH * 3.;

pub enum Action {
    Minimize,
    ToggleMaximize,
    Close,
}

pub fn show(ui: &mut Ui, maximized: bool) -> Option<Action> {
    let mut action = None;
    ui.spacing_mut().item_spacing.x = 0.;
    ui.horizontal(|ui| {
        if button(ui, Icon::Minimize, "최소화", false).clicked() {
            action = Some(Action::Minimize);
        }
        if button(
            ui,
            if maximized {
                Icon::Restore
            } else {
                Icon::Maximize
            },
            if maximized { "복원" } else { "최대화" },
            false,
        )
        .clicked()
        {
            action = Some(Action::ToggleMaximize);
        }
        if button(ui, Icon::Close, "닫기", true).clicked() {
            action = Some(Action::Close);
        }
    });
    action
}

fn button(ui: &mut Ui, icon: Icon, label: &str, close: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(WIDTH, HEIGHT), Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label)
    });
    if response.hovered() || response.is_pointer_button_down_on() {
        let fill = match (close, response.is_pointer_button_down_on()) {
            (true, true) => color::DANGER_PRESSED,
            (true, false) => color::DANGER_HOVER,
            (false, true) => color::CAPTION_PRESSED,
            (false, false) => color::CAPTION_HOVER,
        };
        ui.painter().rect_filled(rect, radius::NONE, fill);
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(3.),
            radius::SMALL,
            egui::Stroke::new(border::EMPHASIS, player_theme::FOCUS),
            egui::StrokeKind::Inside,
        );
    }
    icon.paint(
        ui,
        egui::Rect::from_center_size(rect.center(), vec2(16., 16.)),
        if close && response.hovered() {
            color::ON_DANGER
        } else {
            player_theme::SECONDARY
        },
    );
    response.on_hover_text(label)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn caption_hit_regions_are_contiguous_and_independent_of_glyph_size() {
        let ctx = egui::Context::default();
        let _ = crate::presentation::test_support::run(&ctx, Default::default(), |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 0.;
                ui.horizontal(|ui| {
                    let a = button(ui, Icon::Minimize, "최소화", false);
                    let b = button(ui, Icon::Maximize, "최대화", false);
                    let c = button(ui, Icon::Close, "닫기", true);
                    assert_eq!(a.rect.size(), vec2(42., 32.));
                    assert_eq!(a.rect.right(), b.rect.left());
                    assert_eq!(b.rect.right(), c.rect.left());
                });
            });
        });
    }
}
