//! Explicit proposed pixel layouts; these are not platform requirements or approvals.
use kinekino_card_render::domain::{CardTemplate, Rect, Size};

pub const VERSION: &str = "0.1.0-proposal.1";
pub const IDS: [&str; 3] = [
    "card.frame-square",
    "card.frame-portrait",
    "card.frame-landscape",
];

pub fn layout(id: &str) -> Option<CardTemplate> {
    let mut layout = super::product_template::png1_proposal().layout;
    let rect = |x, y, width, height| Rect {
        x,
        y,
        width,
        height,
    };
    match id {
        "card.frame-square" => {}
        "card.frame-portrait" => {
            layout.canvas = Size {
                width: 1080,
                height: 1350,
            };
            layout.frame = rect(64, 64, 952, 714);
            layout.front.rect = rect(96, 826, 888, 168);
            layout.back.rect = rect(96, 1030, 888, 224);
        }
        "card.frame-landscape" => {
            layout.canvas = Size {
                width: 1920,
                height: 1080,
            };
            layout.frame = rect(64, 64, 1040, 952);
            layout.front.rect = rect(1168, 128, 656, 280);
            layout.back.rect = rect(1168, 472, 656, 480);
        }
        _ => return None,
    }
    layout.version = format!("{id}/{VERSION}");
    Some(layout)
}
