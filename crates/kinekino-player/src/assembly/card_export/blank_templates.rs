//! Explicit text-only proposed templates; no placeholder image or platform approval.
use kinekino_card_render::domain::{CardTemplate, Rect};
pub const VERSION: &str = "0.1.0-proposal.1";
pub const IDS: [&str; 3] = [
    "card.text-square",
    "card.text-portrait",
    "card.text-landscape",
];
pub fn layout(id: &str) -> Option<CardTemplate> {
    let base = match id {
        "card.text-square" => "card.frame-square",
        "card.text-portrait" => "card.frame-portrait",
        "card.text-landscape" => "card.frame-landscape",
        _ => return None,
    };
    let mut layout = super::canvas_templates::layout(base)?;
    let width = layout.canvas.width - 192;
    layout.front.rect = Rect {
        x: 96,
        y: 96,
        width,
        height: 224,
    };
    layout.back.rect = Rect {
        x: 96,
        y: 368,
        width,
        height: layout.canvas.height - 464,
    };
    layout.version = format!("{id}/{VERSION}");
    Some(layout)
}
