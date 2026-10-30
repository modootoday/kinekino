//! Thin media rails with a separate, keyboard-accessible input target.
use crate::presentation::design_tokens::{border, color, radius, size, space};
use crate::presentation::player_theme;
use eframe::egui::{self, Response, Sense, Ui};

pub struct Rail<'a> {
    pub value: &'a mut f64,
    pub maximum: f64,
    pub step: f64,
    pub width: f32,
    pub compact: bool,
    pub label: &'a str,
}

impl egui::Widget for Rail<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let height = if self.compact {
            size::MEDIA_RAIL_COMPACT
        } else {
            size::CONTROL
        };
        let (visual, _) = ui.allocate_exact_size(
            egui::vec2(self.width.max(size::CONTROL), height),
            Sense::hover(),
        );
        let target = egui::Rect::from_center_size(
            visual.center(),
            egui::vec2(visual.width(), size::CONTROL),
        );
        let mut response =
            ui.interact(target, ui.auto_id_with(self.label), Sense::click_and_drag());
        let left = visual.left() + space::INLINE;
        let right = visual.right() - space::INLINE;
        let old = *self.value;
        let maximum = self.maximum.max(0.);
        if ui.is_enabled() {
            if (response.clicked() || response.dragged())
                && let Some(pointer) = response.interact_pointer_pos()
            {
                *self.value =
                    f64::from(((pointer.x - left) / (right - left)).clamp(0., 1.)) * maximum;
                response.request_focus();
            }
            if response.has_focus() {
                ui.memory_mut(|memory| {
                    memory.set_focus_lock_filter(
                        response.id,
                        egui::EventFilter {
                            horizontal_arrows: true,
                            ..Default::default()
                        },
                    )
                });
                ui.input_mut(|input| {
                    if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft) {
                        *self.value -= self.step;
                    }
                    if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight) {
                        *self.value += self.step;
                    }
                    if input.consume_key(egui::Modifiers::NONE, egui::Key::Home) {
                        *self.value = 0.;
                    }
                    if input.consume_key(egui::Modifiers::NONE, egui::Key::End) {
                        *self.value = maximum;
                    }
                });
            }
            *self.value = self.value.clamp(0., maximum);
            if old != *self.value {
                response.mark_changed();
            }
        }
        response.widget_info(|| egui::WidgetInfo::slider(ui.is_enabled(), *self.value, self.label));
        let fraction = if maximum > 0. {
            (*self.value / maximum).clamp(0., 1.) as f32
        } else {
            0.
        };
        let center = egui::pos2(egui::lerp(left..=right, fraction), visual.center().y);
        let track = egui::Rect::from_min_max(
            egui::pos2(left, center.y - size::MEDIA_RAIL_TRACK / 2.),
            egui::pos2(right, center.y + size::MEDIA_RAIL_TRACK / 2.),
        );
        ui.painter()
            .rect_filled(track, radius::SMALL, color::MEDIA_RAIL_TRACK);
        if fraction > 0. {
            ui.painter().rect_filled(
                egui::Rect::from_min_max(track.min, egui::pos2(center.x, track.bottom())),
                radius::SMALL,
                if ui.is_enabled() {
                    color::MEDIA_RAIL_FILL
                } else {
                    color::MEDIA_RAIL_DISABLED
                },
            );
        }
        ui.painter().circle_filled(
            center,
            if response.hovered() || response.dragged() {
                size::MEDIA_RAIL_ACTIVE_RADIUS
            } else {
                size::MEDIA_RAIL_THUMB_RADIUS
            },
            if ui.is_enabled() {
                color::MEDIA_RAIL_THUMB
            } else {
                color::MEDIA_RAIL_DISABLED
            },
        );
        if response.has_focus() {
            ui.painter().rect_stroke(
                target.shrink(border::THIN),
                radius::CONTROL,
                egui::Stroke::new(border::FOCUS, player_theme::FOCUS),
                egui::StrokeKind::Inside,
            );
        }
        response.on_hover_text(self.label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rail_keeps_large_target_and_keyboard_changes_within_bounds() {
        let ctx = egui::Context::default();
        let mut value = 10.;
        let mut id = egui::Id::NULL;
        let _ = crate::presentation::test_support::run(&ctx, Default::default(), |ui| {
            let response = ui.add(Rail {
                value: &mut value,
                maximum: 100.,
                step: 5.,
                width: 200.,
                compact: true,
                label: "재생 위치",
            });
            assert_eq!(response.rect.height(), 32.);
            assert_eq!(response.rect.width(), 200.);
            id = response.id;
        });
        ctx.memory_mut(|memory| memory.request_focus(id));
        for (key, expected) in [
            (egui::Key::ArrowRight, 15.),
            (egui::Key::Home, 0.),
            (egui::Key::End, 100.),
            (egui::Key::ArrowRight, 100.),
            (egui::Key::ArrowLeft, 95.),
        ] {
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
                ui.add(Rail {
                    value: &mut value,
                    maximum: 100.,
                    step: 5.,
                    width: 200.,
                    compact: true,
                    label: "재생 위치",
                });
            });
            assert_eq!(value, expected);
        }
        let _ = crate::presentation::test_support::run(&ctx, Default::default(), |ui| {
            let response = ui.add_enabled(
                false,
                Rail {
                    value: &mut value,
                    maximum: 100.,
                    step: 5.,
                    width: 200.,
                    compact: true,
                    label: "재생 위치",
                },
            );
            assert!(!response.changed());
        });
        assert_eq!(value, 95.);
    }
}
