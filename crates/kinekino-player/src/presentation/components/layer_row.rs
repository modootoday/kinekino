use crate::presentation::{
    design_tokens::{border, radius, size, space},
    icon_asset::Icon,
    layout, player_theme,
};
use eframe::egui::{self, Response, Ui};

pub struct LayerRow<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub selected: bool,
    pub visible: bool,
    pub locked: bool,
}

/// Row selection is interactive; eye/lock/grip are read-only status slots.
pub fn show(
    ui: &mut Ui,
    row: LayerRow<'_>,
    thumbnail: impl FnOnce(&mut Ui, egui::Rect),
) -> Response {
    let response = egui::Frame::new()
        .fill(if row.selected {
            player_theme::SELECTED
        } else {
            player_theme::PANEL
        })
        .corner_radius(radius::CONTROL)
        .inner_margin(space::INLINE_MARGIN)
        .show(ui, |ui| {
            egui_taffy::tui(ui, ui.id().with(row.id))
                .reserve_available_width()
                .style(layout::full(layout::row(space::INLINE)))
                .show(|tui| {
                    layout::widget(tui, |ui| {
                        status(
                            ui,
                            if row.visible { Icon::Eye } else { Icon::EyeOff },
                            if row.visible { "표시됨" } else { "숨김" },
                        )
                    });
                    layout::widget(tui, |ui| {
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(size::LAYER_THUMBNAIL_WIDTH, size::LAYER_THUMBNAIL_HEIGHT),
                            egui::Sense::hover(),
                        );
                        thumbnail(ui, rect);
                    });
                    layout::block(tui, |ui| {
                        ui.add(egui::Label::new(row.label).truncate())
                            .on_hover_text(row.label);
                    });
                    layout::widget(tui, |ui| {
                        status(
                            ui,
                            if row.locked { Icon::Lock } else { Icon::Unlock },
                            if row.locked {
                                "잠김"
                            } else {
                                "잠금 해제"
                            },
                        )
                    });
                    layout::widget(tui, |ui| status(ui, Icon::Grip, "레이어 순서 · 읽기 전용"));
                });
        })
        .response;
    let response = ui.interact(
        response.rect,
        ui.id().with(("layer-row", row.id)),
        egui::Sense::click(),
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            ui.is_enabled(),
            row.selected,
            row.label,
        )
    });
    if row.selected {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                response.rect.min,
                egui::vec2(border::FOCUS, response.rect.height()),
            ),
            radius::SMALL,
            player_theme::ACCENT,
        );
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect,
            radius::CONTROL,
            egui::Stroke::new(border::FOCUS, player_theme::FOCUS),
            egui::StrokeKind::Inside,
        );
    }
    response
}

fn status(ui: &mut Ui, icon: Icon, label: &str) {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(size::CONTROL, size::CONTROL),
        egui::Sense::hover(),
    );
    icon.paint(
        ui,
        egui::Rect::from_center_size(
            rect.center(),
            egui::vec2(size::STATUS_ICON, size::STATUS_ICON),
        ),
        player_theme::SECONDARY,
    );
    response.on_hover_text(format!("{label} · 읽기 전용"));
}
