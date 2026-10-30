use crate::presentation::{
    control_icon::{self, Icon},
    design_tokens::{border, color, radius, size, space},
    layout,
};
use eframe::egui::{self, Response, Ui};

pub fn unavailable_search(ui: &mut Ui, hint: &str) {
    egui::Frame::new()
        .fill(color::INPUT_SURFACE)
        .stroke(egui::Stroke::new(border::THIN, color::BORDER))
        .corner_radius(radius::CONTROL)
        .inner_margin(egui::Margin::symmetric(space::INLINE_MARGIN, 0))
        .show(ui, |ui| {
            ui.add_enabled_ui(false, |ui| {
                ui.spacing_mut().interact_size.y = size::CONTROL - border::THIN * 2.;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = space::FIELD;
                    let (rect, _) = ui.allocate_exact_size(
                        egui::Vec2::splat(size::TOOLBAR_ICON),
                        egui::Sense::hover(),
                    );
                    Icon::Search.paint(ui, rect, color::DISABLED);
                    ui.add_sized(
                        [ui.available_width(), size::CONTROL - border::THIN * 2.],
                        egui::TextEdit::singleline(&mut String::new())
                            .frame(egui::Frame::new())
                            .vertical_align(egui::Align::Center)
                            .hint_text(hint),
                    );
                });
            });
        })
        .response
        .on_hover_text("검색 기능 연결이 필요합니다.");
}

pub fn unavailable_scalar(ui: &mut Ui, value_width: f32, reason: &str) {
    ui.horizontal(|ui| {
        let width = (ui.available_width() - value_width - space::FIELD).max(size::CONTROL);
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(width, size::CONTROL), egui::Sense::hover());
        ui.painter().line_segment(
            [rect.left_center(), rect.right_center()],
            egui::Stroke::new(border::FOCUS, color::BORDER),
        );
        ui.add_sized([value_width, size::CONTROL], egui::Label::new("—"));
    })
    .response
    .on_hover_text(reason);
}

pub fn unavailable_select(ui: &mut Ui, label: &str, reason: &str) -> Response {
    ui.add_enabled_ui(false, |ui| {
        ui.spacing_mut().interact_size.y = size::CONTROL;
        egui::ComboBox::from_id_salt("select")
            .width(ui.available_width())
            .selected_text(label)
            .icon(|ui, rect, visuals, _| Icon::ChevronDown.paint(ui, rect, visuals.fg_stroke.color))
            .show_ui(ui, |_| {})
            .response
            .on_disabled_hover_text(reason)
    })
    .inner
}

/// Shared enabled selector; the caller supplies choices and selection semantics.
pub fn select(
    ui: &mut Ui,
    id: &str,
    label: &str,
    selected: impl Into<egui::WidgetText>,
    choices: impl FnOnce(&mut Ui),
) -> Response {
    let label = ui.label(label);
    egui::ComboBox::from_id_salt(id)
        .width(ui.available_width())
        .selected_text(selected)
        .icon(|ui, rect, visuals, _| Icon::ChevronDown.paint(ui, rect, visuals.fg_stroke.color))
        .show_ui(ui, choices)
        .response
        .labelled_by(label.id)
}

pub fn readonly(ui: &mut Ui, value: &str) -> Response {
    let mut value = value.to_owned();
    ui.add_sized(
        [ui.available_width(), size::CONTROL],
        egui::TextEdit::singleline(&mut value).interactive(false),
    )
    .on_hover_text(value)
}

/// Caller-owned editing buffer, with shared field styling and accessible labelling.
pub fn multiline(ui: &mut Ui, id: &str, label: &str, value: &mut String) -> Response {
    let label = ui.label(label);
    egui::ScrollArea::vertical()
        .id_salt((id, "scroll"))
        .max_height(size::REVIEW_TEXT_HEIGHT)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(value)
                    .id(ui.id().with(id))
                    .desired_width(ui.available_width())
                    .desired_rows(3),
            )
            .labelled_by(label.id)
        })
        .inner
}

pub fn unavailable_path(ui: &mut Ui) {
    egui_taffy::tui(ui, ui.id().with("path-field"))
        .reserve_available_width()
        .style(layout::full(layout::detail_sidebar(
            size::CONTROL,
            space::FIELD,
        )))
        .show(|tui| {
            layout::block(tui, |ui| readonly(ui, "경로 선택 필요"));
            layout::widget(tui, |ui| {
                ui.add_enabled_ui(false, |ui| {
                    control_icon::button(ui, Icon::Folder, "저장 폴더 선택", false, size::CONTROL)
                })
                .inner
                .on_disabled_hover_text("실제 경로 선택 명령 연결 필요");
            });
        });
}
