use crate::presentation::{
    design_tokens::{color, size, space, text},
    icon_asset::Icon,
};
use eframe::egui::{self, Rect, Ui};

#[derive(Clone, Copy)]
pub enum TrackKind {
    Video,
    Subtitle,
    Audio,
}

impl TrackKind {
    fn appearance(self) -> (Icon, &'static str) {
        match self {
            Self::Video => (Icon::TrackVideo, "영상"),
            Self::Subtitle => (Icon::TrackSubtitle, "자막"),
            Self::Audio => (Icon::TrackAudio, "오디오"),
        }
    }
}

pub fn paint(ui: &mut Ui, rail: Rect, kind: TrackKind) {
    paint_variant(ui, rail, kind, None);
}

pub fn paint_media(ui: &mut Ui, rail: Rect, kind: TrackKind) {
    paint_variant(
        ui,
        rail,
        kind,
        if matches!(kind, TrackKind::Subtitle) {
            Some(Icon::TrackSubtitleMedia)
        } else {
            None
        },
    );
}

fn paint_variant(ui: &mut Ui, rail: Rect, kind: TrackKind, override_icon: Option<Icon>) {
    let (icon, label) = kind.appearance();
    let icon = override_icon.unwrap_or(icon);
    let mut clipped = ui.new_child(egui::UiBuilder::new().max_rect(rail));
    clipped.set_clip_rect(rail.intersect(ui.clip_rect()));
    let glyph = Rect::from_center_size(
        egui::pos2(
            rail.left() + space::FIELD + size::TIMELINE_TRACK_ICON / 2.,
            rail.center().y,
        ),
        egui::vec2(size::TIMELINE_TRACK_ICON, size::TIMELINE_TRACK_ICON),
    );
    icon.paint(&clipped, glyph, color::TEXT);
    clipped.painter().text(
        egui::pos2(glyph.right() + space::INLINE, rail.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(text::BODY),
        color::TEXT,
    );
}
