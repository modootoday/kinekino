use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    fn inside(self, canvas: Size) -> bool {
        self.width > 0
            && self.height > 0
            && self
                .x
                .checked_add(self.width)
                .is_some_and(|n| n <= canvas.width)
            && self
                .y
                .checked_add(self.height)
                .is_some_and(|n| n <= canvas.height)
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Rgb(pub u8, pub u8, pub u8);

#[derive(Clone, Copy, Debug, Serialize)]
pub enum FrameFit {
    Contain,
    Cover,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub enum FontWeight {
    Regular,
    Semibold,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub enum TextAlignment {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub enum TextOrder {
    FrontThenBack,
    BackThenFront,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct TextSlot {
    pub rect: Rect,
    pub font_px: u32,
    pub line_height_px: u32,
    pub color: Rgb,
    pub weight: FontWeight,
    pub alignment: TextAlignment,
}

/// Explicit product template declaration; there are no platform/layout defaults.
#[derive(Clone, Debug, Serialize)]
pub struct CardTemplate {
    pub version: String,
    pub canvas: Size,
    pub background: Rgb,
    pub frame: Rect,
    pub fit: FrameFit,
    pub frame_anchor_x_percent: u8,
    pub frame_anchor_y_percent: u8,
    pub text_order: TextOrder,
    pub front: TextSlot,
    pub back: TextSlot,
}

impl CardTemplate {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.version.trim().is_empty() || self.version.len() > 256 {
            return Err("template version is required within 256 bytes");
        }
        if self.frame_anchor_x_percent > 100 || self.frame_anchor_y_percent > 100 {
            return Err("explicit frame anchor must be in 0..=100 percent");
        }
        if !self.frame.inside(self.canvas)
            || !self.front.rect.inside(self.canvas)
            || !self.back.rect.inside(self.canvas)
        {
            return Err("frame and text slots must be positive and within the canvas");
        }
        for slot in [self.front, self.back] {
            if slot.font_px == 0
                || slot.font_px > self.canvas.height
                || slot.line_height_px < slot.font_px
                || slot.line_height_px > self.canvas.height
            {
                return Err("explicit positive font and line-height sizes required");
            }
        }
        Ok(())
    }
}
