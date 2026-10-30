use crate::presentation::{
    control_icon::Icon,
    design_tokens::{color, size},
};
use eframe::egui::{self, Response, Ui, Widget};

#[derive(Clone, Copy)]
pub enum Variant {
    Primary,
    Secondary,
}

/// Source-owned primitive: callers own actions, this component owns appearance.
pub struct Button<'a> {
    label: &'a str,
    variant: Variant,
    full_width: bool,
    icon: Option<Icon>,
}
impl<'a> Button<'a> {
    pub fn new(label: &'a str, variant: Variant) -> Self {
        Self {
            label,
            variant,
            full_width: true,
            icon: None,
        }
    }

    pub fn compact(mut self) -> Self {
        self.full_width = false;
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
}
impl Widget for Button<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let width = if self.full_width {
            ui.available_width()
        } else {
            size::CONTROL
        };
        let primary = matches!(self.variant, Variant::Primary) && ui.is_enabled();
        let tint = if primary { color::CHROME } else { color::TEXT };
        let label = if primary {
            egui::RichText::new(self.label).color(tint)
        } else {
            egui::RichText::new(self.label)
        };
        let mut button = match self.icon {
            Some(icon) => egui::Button::image_and_text(
                icon.image()
                    .tint(tint)
                    .fit_to_exact_size(egui::Vec2::splat(size::TOOLBAR_ICON)),
                label,
            ),
            None => egui::Button::new(label),
        }
        .min_size(egui::vec2(width, size::PRIMARY));
        if primary {
            button = button.fill(color::ACCENT);
        }
        if self.full_width {
            ui.add_sized([width, size::PRIMARY], button)
        } else {
            ui.add(button)
        }
    }
}
