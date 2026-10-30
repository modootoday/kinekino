use super::{CardTemplate, Rect, TextSlot};

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct PageFooter {
    pub current: u32,
    pub total: u32,
    pub slot: TextSlot,
}
impl PageFooter {
    pub fn text(self) -> String {
        format!("{} / {}", self.current, self.total)
    }
    pub fn validate(self, template: &CardTemplate, has_frame: bool) -> Result<(), &'static str> {
        if self.current == 0 || self.current > self.total {
            return Err("page must be within 1..=total");
        }
        let rect = self.slot.rect;
        if rect.width == 0
            || rect.height == 0
            || rect
                .x
                .checked_add(rect.width)
                .is_none_or(|right| right > template.canvas.width)
            || rect
                .y
                .checked_add(rect.height)
                .is_none_or(|bottom| bottom > template.canvas.height)
            || self.slot.font_px == 0
            || self.slot.line_height_px < self.slot.font_px
            || self.slot.line_height_px > rect.height
        {
            return Err("page slot must have explicit valid bounds and font sizes");
        }
        if overlaps(rect, template.front.rect)
            || overlaps(rect, template.back.rect)
            || (has_frame && overlaps(rect, template.frame))
        {
            return Err("page slot overlaps declared content");
        }
        Ok(())
    }
}
fn overlaps(a: Rect, b: Rect) -> bool {
    u64::from(a.x) < u64::from(b.x) + u64::from(b.width)
        && u64::from(b.x) < u64::from(a.x) + u64::from(a.width)
        && u64::from(a.y) < u64::from(b.y) + u64::from(b.height)
        && u64::from(b.y) < u64::from(a.y) + u64::from(a.height)
}
