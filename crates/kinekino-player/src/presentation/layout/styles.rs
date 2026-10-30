use egui_taffy::taffy::prelude::{auto, fr, length, percent};
use egui_taffy::taffy::{self, AlignItems, Display, FlexDirection, Size, Style};

pub fn full(style: Style) -> Style {
    Style {
        size: Size {
            width: percent(1.0_f32),
            height: auto(),
        },
        ..style
    }
}

pub fn column(gap: f32) -> Style {
    Style {
        display: Display::Grid,
        grid_template_columns: vec![taffy::style_helpers::minmax(length(0.0_f32), fr(1.0_f32))],
        align_items: Some(AlignItems::Start),
        justify_items: Some(AlignItems::Stretch),
        gap: Size {
            width: length(gap),
            height: length(gap),
        },
        ..Default::default()
    }
}

pub fn row(gap: f32) -> Style {
    Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: Some(AlignItems::Center),
        gap: Size {
            width: length(gap),
            height: length(gap),
        },
        ..Default::default()
    }
}

pub fn grid(columns: usize, gap: f32) -> Style {
    Style {
        grid_template_columns: vec![
            taffy::style_helpers::minmax(length(0.0_f32), fr(1.0_f32));
            columns.max(1)
        ],
        ..column(gap)
    }
}

pub fn sidebar(width: f32, gap: f32) -> Style {
    Style {
        grid_template_columns: vec![
            length(width),
            taffy::style_helpers::minmax(length(0.0_f32), fr(1.0_f32)),
        ],
        ..column(gap)
    }
}

pub fn detail_sidebar(width: f32, gap: f32) -> Style {
    Style {
        grid_template_columns: vec![
            taffy::style_helpers::minmax(length(0.0_f32), fr(1.0_f32)),
            length(width),
        ],
        ..column(gap)
    }
}
