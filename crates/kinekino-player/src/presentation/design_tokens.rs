//! Shared logical units. These are design decisions, not raster measurements.
mod palette {
    use eframe::egui::Color32;
    pub const BLACK: Color32 = Color32::from_rgb(0, 0, 0);
    pub const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
    pub const NEUTRAL_50: Color32 = Color32::from_rgb(236, 238, 242);
    pub const NEUTRAL_300: Color32 = Color32::from_rgb(180, 187, 199);
    pub const NEUTRAL_400: Color32 = Color32::from_rgb(128, 133, 140);
    pub const NEUTRAL_500: Color32 = Color32::from_rgb(104, 110, 121);
    pub const NEUTRAL_600: Color32 = Color32::from_rgb(65, 70, 80);
    pub const NEUTRAL_650: Color32 = Color32::from_rgb(62, 67, 76);
    pub const NEUTRAL_700: Color32 = Color32::from_rgb(48, 52, 60);
    pub const NEUTRAL_750: Color32 = Color32::from_rgb(43, 47, 55);
    pub const NEUTRAL_800: Color32 = Color32::from_rgb(42, 45, 52);
    pub const NEUTRAL_850: Color32 = Color32::from_rgb(32, 35, 41);
    pub const NEUTRAL_875: Color32 = Color32::from_rgb(28, 31, 37);
    pub const NEUTRAL_900: Color32 = Color32::from_rgb(25, 27, 31);
    pub const NEUTRAL_950: Color32 = Color32::from_rgb(11, 12, 14);
    pub const NEUTRAL_975: Color32 = Color32::from_rgb(6, 7, 9);
    pub const AMBER_200: Color32 = Color32::from_rgb(241, 209, 138);
    pub const AMBER_400: Color32 = Color32::from_rgb(232, 188, 90);
    pub const AMBER_950: Color32 = Color32::from_rgb(80, 65, 35);
    pub const AMBER_975: Color32 = Color32::from_rgb(41, 37, 29);
    pub const BLUE_400: Color32 = Color32::from_rgb(104, 169, 255);
    pub const BLUE_950: Color32 = Color32::from_rgb(43, 70, 106);
    pub const BLUE_975: Color32 = Color32::from_rgb(25, 33, 45);
    pub const CYAN_400: Color32 = Color32::from_rgb(77, 208, 225);
    pub const PINK_300: Color32 = Color32::from_rgb(244, 159, 183);
    pub const PURPLE_300: Color32 = Color32::from_rgb(188, 160, 235);
    pub const RED_600: Color32 = Color32::from_rgb(196, 43, 28);
    pub const RED_800: Color32 = Color32::from_rgb(158, 30, 22);
    pub const GREEN_400: Color32 = Color32::from_rgb(80, 190, 100);
    pub const WHITE_40: Color32 = Color32::from_rgba_premultiplied(102, 102, 102, 102);
}
pub mod color {
    pub const SUBTITLE_TRACK: eframe::egui::Color32 = super::palette::PURPLE_300;
    pub const AUDIO_WAVEFORM: eframe::egui::Color32 = super::palette::GREEN_400;
    use super::palette;
    use eframe::egui::Color32;
    pub const ACCENT: Color32 = palette::AMBER_400;
    pub const MEDIA_RAIL_FILL: Color32 = palette::AMBER_400;
    pub const MEDIA_RAIL_THUMB: Color32 = palette::AMBER_400;
    pub const MEDIA_RAIL_TRACK: Color32 = palette::NEUTRAL_600;
    pub const MEDIA_RAIL_DISABLED: Color32 = palette::NEUTRAL_300;
    pub const FOCUS: Color32 = palette::AMBER_200;
    pub const TEXT: Color32 = palette::NEUTRAL_50;
    pub const TEXT_SWATCHES: [(&str, Color32); 7] = [
        ("흰색", palette::WHITE),
        ("회색", palette::NEUTRAL_400),
        ("검정", palette::BLACK),
        ("진회색", palette::NEUTRAL_700),
        ("분홍", palette::PINK_300),
        ("파랑", palette::BLUE_400),
        ("보라", palette::PURPLE_300),
    ];
    pub const SECONDARY: Color32 = palette::NEUTRAL_300;
    pub const DISABLED: Color32 = palette::NEUTRAL_500;
    pub const IDLE_MARK: Color32 = palette::NEUTRAL_400;
    pub const CANVAS_SELECTION: Color32 = palette::BLUE_400;
    pub const SUBJECT_SELECTION: Color32 = palette::CYAN_400;
    pub const OCR_REGION: Color32 = palette::AMBER_400;
    pub const REGION_HANDLE: Color32 = palette::WHITE;
    pub const PANEL: Color32 = palette::NEUTRAL_850;
    pub const CHROME: Color32 = palette::NEUTRAL_900;
    pub const BORDER: Color32 = palette::NEUTRAL_600;
    pub const SELECTED: Color32 = palette::NEUTRAL_800;
    pub const TABLE_SELECTED: Color32 = palette::AMBER_975;
    pub const FIELD: Color32 = palette::NEUTRAL_875;
    pub const HOVER: Color32 = palette::NEUTRAL_750;
    pub const PRESSED: Color32 = palette::AMBER_950;
    pub const NOTICE_SURFACE: Color32 = palette::AMBER_975;
    pub const NOTICE_BORDER: Color32 = palette::AMBER_950;
    pub const INFO_SURFACE: Color32 = palette::BLUE_975;
    pub const INFO_BORDER: Color32 = palette::BLUE_950;
    pub const INFO_FOREGROUND: Color32 = palette::BLUE_400;
    pub const CAPTION_HOVER: Color32 = palette::NEUTRAL_650;
    pub const CAPTION_PRESSED: Color32 = palette::NEUTRAL_700;
    pub const DANGER_HOVER: Color32 = palette::RED_600;
    pub const DANGER_PRESSED: Color32 = palette::RED_800;
    pub const ON_DANGER: Color32 = palette::WHITE;
    pub const INPUT_SURFACE: Color32 = palette::NEUTRAL_950;
    pub const PREVIEW_SURFACE: Color32 = palette::NEUTRAL_975;
    pub const VIDEO_SURFACE: Color32 = palette::BLACK;
    pub const IMAGE_TINT: Color32 = palette::WHITE;
    pub const SCENE_BACK: Color32 = palette::WHITE_40;
    pub const AXIS_X: Color32 = palette::RED_600;
    pub const AXIS_Y: Color32 = palette::GREEN_400;
    pub const AXIS_Z: Color32 = palette::BLUE_400;
}
pub mod space {
    pub const SCENE_AXIS_LEFT: f32 = 64.;
    pub const SCENE_AXIS_BOTTOM: f32 = 44.;
    pub const TIGHT: f32 = 4.;
    pub const INLINE: f32 = 6.;
    pub const FIELD: f32 = 8.;
    pub const PANEL: f32 = 12.;
    pub const GROUP: f32 = 16.;
    pub const PANEL_MARGIN: i8 = 12;
    pub const RAIL_MARGIN: i8 = 8;
    pub const INLINE_MARGIN: i8 = 6;
    pub const HEADER_TO_DESCRIPTION: f32 = TIGHT;
    pub const HEADER_TO_TABS: f32 = GROUP;
    pub const TABS_TO_BODY: f32 = FIELD;
}
pub mod size {
    pub const MEDIA_RAIL_COMPACT: f32 = 18.;
    pub const MEDIA_RAIL_TRACK: f32 = 3.;
    pub const MEDIA_RAIL_THUMB_RADIUS: f32 = 5.;
    pub const MEDIA_RAIL_ACTIVE_RADIUS: f32 = 6.;
    pub const ACCORDION_GROUP_ROW: f32 = 48.;
    pub const QUEUE_ROW: f32 = 48.;
    pub const QUEUE_STATUS: f32 = 112.;
    pub const QUEUE_PROGRESS: f32 = 80.;
    pub const WINDOW_RESIZE_EDGE: f32 = 6.;
    pub const WINDOW_RESIZE_CORNER: f32 = 12.;
    pub const VOLUME_INLINE: f32 = 72.;
    pub const VOLUME_POPOVER: f32 = 180.;
    pub const VOLUME_OVERLAY_CLEARANCE: f32 = 64.;
    pub const TIMELINE_RULER: f32 = 32.;
    pub const TIMELINE_LABEL: f32 = 80.;
    pub const TIMELINE_TRACK_ICON: f32 = 20.;
    pub const TRACK_REFRESH: f32 = 160.;
    pub const TOOLBAR_ICON: f32 = 20.;
    pub const VIDEO_TRACK: f32 = 64.;
    pub const AUDIO_TRACK: f32 = 44.;
    pub const TIMELINE_MARKER: f32 = 6.;
    pub const TIMELINE_RANGE_GRIP_WIDTH: f32 = 12.;
    pub const TIMELINE_RANGE_GRIP_HEIGHT: f32 = 32.;
    pub const TIMELINE_TICK_WIDTH: f32 = 90.;
    pub const TABLE_INDEX: f32 = 48.;
    pub const TABLE_TIME: f32 = 132.;
    pub const TABLE_STATUS: f32 = 88.;
    pub const SELECTION_HANDLE: f32 = 7.;
    pub const COLOR_SWATCH: f32 = 20.;
    pub const REGION_HANDLE_RADIUS: f32 = 4.;
    pub const CONTROL: f32 = 32.;
    pub const PRIMARY: f32 = 40.;
    pub const TITLE_BAR: f32 = 32.;
    pub const CAPTION_WIDTH: f32 = 42.;
    pub const NAVIGATION: f32 = 180.;
    pub const NAVIGATION_RAIL: f32 = 48.;
    pub const INSPECTOR: f32 = 300.;
    pub const EPISODE_PANEL: f32 = 260.;
    pub const EPISODE_PANEL_MIN: f32 = 180.;
    pub const EPISODE_PANEL_MAX: f32 = 360.;
    pub const QUEUE: f32 = 40.;
    pub const FIELD_LABEL_MIN: f32 = 72.;
    pub const TAB_MIN: f32 = 72.;
    pub const INSPECTOR_FOOTER: f32 = 112.;
    pub const SOURCE_THUMBNAIL: f32 = 64.;
    pub const OUTPUT_THUMBNAIL_WIDTH: f32 = 144.;
    pub const OUTPUT_THUMBNAIL_HEIGHT: f32 = 81.;
    pub const METADATA_LABEL_WIDTH: f32 = 144.;
    pub const NUMERIC_FIELD: f32 = 48.;
    pub const LAYER_THUMBNAIL_WIDTH: f32 = 44.;
    pub const LAYER_THUMBNAIL_HEIGHT: f32 = 40.;
    pub const STATUS_ICON: f32 = 18.;
    pub const SOURCE_REFERENCE_HEIGHT: f32 = 140.;
    pub const TOOL_RAIL: f32 = 112.;
    pub const TOOL_RAIL_COMPACT: f32 = 44.;
    pub const CARD_RAIL: f32 = 132.;
    pub const PAGE_NUMBER_BADGE: f32 = 24.;
    pub const PACKAGE_TREE: f32 = 240.;
    pub const CANDIDATE_ACTIONS: f32 = 240.;
    pub const MODEL_FILTER: f32 = 160.;
    pub const BADGE: f32 = 24.;
    pub const BADGE_MAX_WIDTH: f32 = 240.;
    pub const ARTBOARD_MIN: f32 = 180.;
    pub const PREVIEW_HEIGHT: f32 = 320.;
    pub const REVIEW_TEXT_HEIGHT: f32 = 256.;
    pub const FRAME_STRIP_HEIGHT: f32 = 84.;
    pub const WAVEFORM_HEIGHT: f32 = 56.;
    pub const WAVEFORM_RANGE_GRIP: f32 = 4.;
}
pub mod text {
    pub const ARTWORK_MIN_SCALE: f32 = 0.5;
    pub const ARTWORK_TITLE: f32 = 32.;
    pub const ARTWORK_SUBTITLE: f32 = 18.;
    pub const LATIN_FACE: &str = "Noto Sans Regular";
    pub const REGULAR_FACE: &str = "Noto Sans KR Regular";
    pub const SEMIBOLD_FACE: &str = "Noto Sans KR SemiBold";
    pub const JAPANESE_FACE: &str = "Noto Sans JP Regular";
    pub const HEADING_FAMILY: &str = "ui-heading";
    pub const HEADING: f32 = 23.;
    pub const BODY: f32 = 13.;
    pub const SMALL: f32 = 11.;
}
pub mod proportion {
    pub const MEDIA_PREVIEW_VIEWPORT: f32 = 0.4;
    pub const TRACK_FILL_OPACITY: f32 = 0.3;
    pub const TABLE_FIXED_MAX: f32 = 0.65;
    pub const SUBJECT_MASK_OPACITY: f32 = 0.28;
    pub const ARTWORK_TEXT_WIDTH: f32 = 0.84;
    pub const ARTWORK_TEXT_HEIGHT: f32 = 0.7;
}
pub mod radius {
    pub const NONE: u8 = 0;
    pub const SMALL: u8 = 2;
    pub const CONTROL: u8 = 5;
    pub const PILL: u8 = 12;
    pub const WINDOW: u8 = 8;
}
pub mod border {
    pub const THIN: f32 = 1.;
    pub const EMPHASIS: f32 = 1.5;
    pub const FOCUS: f32 = 2.;
}
pub mod breakpoint {
    pub const STACK_EDITOR: f32 = 980.;
    pub const FULL_NAVIGATION: f32 = 1180.;
    pub const FIXED_INSPECTOR: f32 = 1280.;
}
