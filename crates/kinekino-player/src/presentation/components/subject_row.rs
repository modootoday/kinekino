use crate::presentation::{
    control_icon::{self, Icon},
    design_tokens::{border, color, radius, size, space},
    layout,
};
use eframe::egui;

pub struct SubjectRow<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub detail: &'a str,
    pub current: bool,
}

/// A passive track summary; selection and editing require an explicit caller contract.
pub fn show(
    ui: &mut egui::Ui,
    row: SubjectRow<'_>,
    thumbnail: impl FnOnce(&mut egui::Ui, egui::Rect),
) {
    egui::Frame::new()
        .fill(color::PANEL)
        .stroke(egui::Stroke::new(
            if row.current {
                border::FOCUS
            } else {
                border::THIN
            },
            if row.current {
                color::ACCENT
            } else {
                color::BORDER
            },
        ))
        .corner_radius(radius::CONTROL)
        .inner_margin(space::INLINE_MARGIN)
        .show(ui, |ui| {
            egui_taffy::tui(ui, ui.id().with(("subject-row", row.id)))
                .reserve_available_width()
                .style(layout::full(layout::row(space::FIELD)))
                .show(|tui| {
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
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(row.detail)
                                    .small()
                                    .color(color::SECONDARY),
                            )
                            .truncate(),
                        )
                        .on_hover_text(row.detail);
                    });
                    layout::widget(tui, |ui| {
                        ui.add_enabled_ui(false, |ui| {
                            control_icon::button(
                                ui,
                                Icon::MoreVertical,
                                "피사체 메뉴",
                                false,
                                size::CONTROL,
                            )
                        })
                        .inner
                        .on_disabled_hover_text("피사체 메뉴 연결 필요");
                    });
                });
        });
}
