#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Front,
    Back,
    Page,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutBounds {
    fn valid(self) -> bool {
        [self.x, self.y, self.width, self.height]
            .into_iter()
            .all(f32::is_finite)
            && self.width >= 0.0
            && self.height >= 0.0
            && (self.x + self.width).is_finite()
            && (self.y + self.height).is_finite()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlotObservation {
    pub slot: LayoutBounds,
    pub content: LayoutBounds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxContainment {
    InsideLayoutBox,
    OutsideLayoutBox,
    Unavailable,
}

impl SlotObservation {
    pub fn containment(self) -> BoxContainment {
        let (slot, content) = (self.slot, self.content);
        if !slot.valid() || !content.valid() || slot.width == 0.0 || slot.height == 0.0 {
            return BoxContainment::Unavailable;
        }
        if content.x < slot.x
            || content.y < slot.y
            || content.x + content.width > slot.x + slot.width
            || content.y + content.height > slot.y + slot.height
        {
            BoxContainment::OutsideLayoutBox
        } else {
            BoxContainment::InsideLayoutBox
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limitation {
    GlyphCoverageUnmeasured,
    InkOverflowUnmeasured,
    HostFontFallbackUncontrolled,
    WallDeadlineNotProvided,
    FramePlacementNotPixelVerified,
    TemplateApprovalNotChecked,
    CancellationNotProvided,
}

pub const RENDER_LIMITATIONS: [Limitation; 7] = [
    Limitation::GlyphCoverageUnmeasured,
    Limitation::InkOverflowUnmeasured,
    Limitation::HostFontFallbackUncontrolled,
    Limitation::WallDeadlineNotProvided,
    Limitation::FramePlacementNotPixelVerified,
    Limitation::TemplateApprovalNotChecked,
    Limitation::CancellationNotProvided,
];
