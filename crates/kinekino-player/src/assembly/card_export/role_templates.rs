//! Product role layouts with an independently measured page slot. Proposed only.
use crate::domain::card_document::CardRole;
use kinekino_card_render::domain::{
    CardTemplate, FontWeight, PageFooter, Rect, Rgb, TextAlignment, TextSlot,
};
pub const VERSION: &str = "0.1.0-proposal.1";
pub const IDS: [&str; 6] = [
    "card.editor-frame-square",
    "card.editor-frame-portrait",
    "card.editor-frame-landscape",
    "card.editor-text-square",
    "card.editor-text-portrait",
    "card.editor-text-landscape",
];
pub fn frame_required(id: &str) -> bool {
    id.starts_with("card.editor-frame-")
}
pub fn layout(id: &str, role: CardRole) -> Option<CardTemplate> {
    let mut layout = match id {
        "card.editor-frame-square" => super::canvas_templates::layout("card.frame-square"),
        "card.editor-frame-portrait" => super::canvas_templates::layout("card.frame-portrait"),
        "card.editor-frame-landscape" => super::canvas_templates::layout("card.frame-landscape"),
        "card.editor-text-square" => super::blank_templates::layout("card.text-square"),
        "card.editor-text-portrait" => super::blank_templates::layout("card.text-portrait"),
        "card.editor-text-landscape" => super::blank_templates::layout("card.text-landscape"),
        _ => None,
    }?;
    layout.back.rect.height = layout
        .back
        .rect
        .height
        .min(layout.canvas.height - 96 - layout.back.rect.y);
    layout.front.weight = FontWeight::Semibold;
    match role {
        CardRole::Cover => {
            layout.front.font_px = 48;
            layout.front.line_height_px = 64;
            layout.front.alignment = TextAlignment::Center;
            layout.back.alignment = TextAlignment::Center;
        }
        CardRole::Body => {
            layout.front.alignment = TextAlignment::Start;
            layout.back.alignment = TextAlignment::Start;
            layout.back.font_px = 36;
            layout.back.line_height_px = 52;
        }
        CardRole::Closing => {
            layout.front.font_px = 44;
            layout.front.line_height_px = 64;
            layout.front.alignment = TextAlignment::Center;
            layout.back.alignment = TextAlignment::Center;
        }
    }
    let role_name = match role {
        CardRole::Cover => "cover",
        CardRole::Body => "body",
        CardRole::Closing => "closing",
    };
    layout.version = format!("{id}/{VERSION}/{role_name}");
    Some(layout)
}
pub fn page(layout: &CardTemplate, current: u32, total: u32) -> PageFooter {
    PageFooter {
        current,
        total,
        slot: TextSlot {
            rect: Rect {
                x: 64,
                y: layout.canvas.height - 64,
                width: layout.canvas.width - 128,
                height: 40,
            },
            font_px: 28,
            line_height_px: 40,
            color: Rgb(236, 238, 242),
            weight: FontWeight::Regular,
            alignment: TextAlignment::End,
        },
    }
}
