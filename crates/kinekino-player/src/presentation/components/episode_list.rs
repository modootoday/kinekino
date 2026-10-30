use crate::presentation::{
    control_icon::{self, Icon},
    design_tokens::{border, color, radius, size, space},
};
use eframe::egui;

pub struct Episode<'a> {
    pub id: &'a str,
    pub title: &'a str,
}

pub struct Action {
    pub close: bool,
    pub activate: Option<usize>,
}

pub fn show(
    ui: &mut egui::Ui,
    title: &str,
    episodes: &[Episode<'_>],
    current: Option<usize>,
) -> Action {
    let mut action = Action {
        close: false,
        activate: None,
    };
    ui.horizontal(|ui| {
        ui.strong("회차 목록");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            action.close = control_icon::button(
                ui,
                Icon::Close,
                "회차 목록 닫기 · Esc",
                false,
                size::CONTROL,
            )
            .clicked();
        });
    });
    if !title.is_empty() {
        ui.label(title);
    }
    ui.separator();
    if episodes.is_empty() {
        ui.label("작품이나 회차 페이지를 열면 목록을 가져옵니다.");
    }
    egui::ScrollArea::vertical()
        .id_salt("episode-list")
        .show(ui, |ui| {
            let rows: Vec<_> = episodes
                .iter()
                .enumerate()
                .map(|(index, episode)| {
                    let response = row(ui, episode, current == Some(index));
                    if response.clicked() {
                        response.request_focus();
                        action.activate = Some(index);
                    }
                    response
                })
                .collect();
            let _ = super::keyboard_selection::select(ui.ctx(), &rows, false);
        });
    action
}

fn row(ui: &mut egui::Ui, episode: &Episode<'_>, current: bool) -> egui::Response {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), size::PRIMARY),
        egui::Sense::hover(),
    );
    let response = ui.interact(
        rect,
        ui.id().with(("episode-row", episode.id)),
        egui::Sense::click(),
    );
    let fill = if response.is_pointer_button_down_on() {
        color::PRESSED
    } else if response.hovered() {
        color::HOVER
    } else if current {
        color::SELECTED
    } else {
        color::PANEL
    };
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    painter.rect_filled(rect, radius::SMALL, fill);
    let foreground = if current { color::ACCENT } else { color::TEXT };
    if current {
        painter.rect_filled(
            egui::Rect::from_min_size(rect.min, egui::vec2(border::FOCUS, rect.height())),
            radius::NONE,
            color::ACCENT,
        );
        Icon::Play.paint(
            ui,
            egui::Rect::from_center_size(
                egui::pos2(rect.left() + size::CONTROL / 2., rect.center().y),
                egui::Vec2::splat(size::TOOLBAR_ICON),
            ),
            foreground,
        );
    }
    let galley = egui::WidgetText::from(egui::RichText::new(episode.title).color(foreground))
        .into_galley(
            ui,
            Some(egui::TextWrapMode::Truncate),
            (rect.width() - size::CONTROL - space::FIELD).max(0.),
            egui::TextStyle::Button,
        );
    painter.galley(
        egui::pos2(
            rect.left() + size::CONTROL,
            rect.center().y - galley.size().y / 2.,
        ),
        galley,
        foreground,
    );
    if response.has_focus() {
        painter.rect_stroke(
            rect.shrink(border::THIN),
            radius::CONTROL,
            egui::Stroke::new(border::FOCUS, color::FOCUS),
            egui::StrokeKind::Inside,
        );
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Button,
            ui.is_enabled(),
            current,
            episode.title,
        )
    });
    response.on_hover_text(episode.title)
}
