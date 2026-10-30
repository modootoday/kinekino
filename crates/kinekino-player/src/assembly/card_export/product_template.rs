//! Product-owned PNG1 proposal data. No renderer, acceptance, publication, or GUI policy executes here.
use crate::infrastructure::noto_fonts;
use kinekino_card_render::domain::{
    CardTemplate, FontWeight, FrameFit, Rect, Rgb, Size, TextAlignment, TextOrder, TextSlot,
};
use meari::domain::design::fonts::Pin;

pub const TEMPLATE_ID: &str = "card.frame-two-slot";
pub const PROPOSAL_VERSION: &str = "0.1.0-proposal.1";
pub const RENDER_VERSION: &str = "card.frame-two-slot/0.1.0-proposal.1";

// Export tokens, explicitly proposed output pixels rather than GUI preview logical units.
const CANVAS: Size = Size {
    width: 1080,
    height: 1080,
};
const BACKGROUND: Rgb = Rgb(11, 12, 14);
const TEXT_PRIMARY: Rgb = Rgb(236, 238, 242);
const SAFE_AREA: Rect = Rect {
    x: 64,
    y: 64,
    width: 952,
    height: 952,
};
const FRAME: Rect = Rect {
    x: 64,
    y: 64,
    width: 952,
    height: 536,
};
const FRONT: Rect = Rect {
    x: 96,
    y: 640,
    width: 888,
    height: 160,
};
const BACK: Rect = Rect {
    x: 96,
    y: 832,
    width: 888,
    height: 160,
};
const FONT_PX: u32 = 40;
const LINE_HEIGHT_PX: u32 = 56;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProposalStatus {
    ProposedNotUserApproved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AcceptedField {
    Front,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalAlignment {
    Top,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnknownCoveragePolicy {
    ReviewArtifactOnlyNoApproval,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnavailableOrOverflowPolicy {
    RejectCompletedImage,
}

#[derive(Clone, Copy, Debug)]
pub struct TextBinding {
    pub field: AcceptedField,
    pub slot_id: &'static str,
    pub content_id: &'static str,
    pub z_index: u8,
    pub safe_area: Rect,
    pub vertical_alignment: VerticalAlignment,
    pub minimum_font_px: u32,
    pub automatic_shrink: bool,
    pub ellipsis: bool,
    pub text_crop: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct FontPin {
    pub logical_id: &'static str,
    pub source_hint: &'static str,
    pub sha256: &'static str,
    pub release: &'static str,
    pub weight: u16,
    pub license_hint: &'static str,
}

// Pinned Noto faces (SIL OFL-1.1) from the verified download cache; regular and semibold are
// the same variable Noto Sans KR file at two weights. `release` is the Google Fonts revision.
const KOREAN: Pin = noto_fonts::KOREAN;
const JAPANESE_FACE: Pin = noto_fonts::JAPANESE;
pub const REGULAR: FontPin = FontPin {
    logical_id: "common.font.noto-sans-kr.regular.6157dfaa640c",
    source_hint: "fonts/noto/NotoSansKR[wght].ttf",
    sha256: KOREAN.sha256,
    release: "6157dfaa640c",
    weight: 400,
    license_hint: "fonts/noto/notosanskr-OFL.txt",
};
pub const SEMIBOLD: FontPin = FontPin {
    logical_id: "common.font.noto-sans-kr.semibold.6157dfaa640c",
    source_hint: "fonts/noto/NotoSansKR[wght].ttf",
    sha256: KOREAN.sha256,
    release: "6157dfaa640c",
    weight: 600,
    license_hint: "fonts/noto/notosanskr-OFL.txt",
};
pub const JAPANESE: FontPin = FontPin {
    logical_id: "common.font.noto-sans-jp.regular.6157dfaa640c",
    source_hint: "fonts/noto/NotoSansJP[wght].ttf",
    sha256: JAPANESE_FACE.sha256,
    release: "6157dfaa640c",
    weight: 400,
    license_hint: "fonts/noto/notosansjp-OFL.txt",
};

pub struct ProductTemplateProposal {
    pub id: &'static str,
    pub version: &'static str,
    pub status: ProposalStatus,
    pub layout: CardTemplate,
    pub canvas_safe_area: Rect,
    pub frame_z_index: u8,
    pub text_bindings: [TextBinding; 2],
    pub common_fonts: [FontPin; 3],
    pub content_font_fallback_order: [&'static str; 2],
    pub paragraph_gap_px: u32,
    pub letter_spacing_px: i32,
    pub layout_overflow_or_unavailable: UnavailableOrOverflowPolicy,
    pub glyph_ink_fallback_or_frame_unverified: UnknownCoveragePolicy,
    pub template_unapproved: UnknownCoveragePolicy,
}

/// Pass `layout` explicitly to Media's RenderRequest; keep this complete declaration identity in provenance.
pub fn png1_proposal() -> ProductTemplateProposal {
    let slot = |rect| TextSlot {
        rect,
        font_px: FONT_PX,
        line_height_px: LINE_HEIGHT_PX,
        color: TEXT_PRIMARY,
        weight: FontWeight::Regular,
        alignment: TextAlignment::Center,
    };
    let binding = |field, slot_id, content_id, z_index, rect| TextBinding {
        field,
        slot_id,
        content_id,
        z_index,
        safe_area: rect,
        vertical_alignment: VerticalAlignment::Top,
        minimum_font_px: FONT_PX,
        automatic_shrink: false,
        ellipsis: false,
        text_crop: false,
    };
    ProductTemplateProposal {
        id: TEMPLATE_ID,
        version: PROPOSAL_VERSION,
        status: ProposalStatus::ProposedNotUserApproved,
        layout: CardTemplate {
            version: RENDER_VERSION.into(),
            canvas: CANVAS,
            background: BACKGROUND,
            frame: FRAME,
            fit: FrameFit::Contain,
            frame_anchor_x_percent: 50,
            frame_anchor_y_percent: 50,
            text_order: TextOrder::FrontThenBack,
            front: slot(FRONT),
            back: slot(BACK),
        },
        canvas_safe_area: SAFE_AREA,
        frame_z_index: 0,
        text_bindings: [
            binding(
                AcceptedField::Front,
                "front-slot",
                "front-content",
                1,
                FRONT,
            ),
            binding(AcceptedField::Back, "back-slot", "back-content", 2, BACK),
        ],
        common_fonts: [REGULAR, SEMIBOLD, JAPANESE],
        content_font_fallback_order: [REGULAR.logical_id, JAPANESE.logical_id],
        paragraph_gap_px: 0,
        letter_spacing_px: 0,
        layout_overflow_or_unavailable: UnavailableOrOverflowPolicy::RejectCompletedImage,
        glyph_ink_fallback_or_frame_unverified: UnknownCoveragePolicy::ReviewArtifactOnlyNoApproval,
        template_unapproved: UnknownCoveragePolicy::ReviewArtifactOnlyNoApproval,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_font_pins_follow_the_noto_manifest() {
        for (pin, face) in [
            (REGULAR, KOREAN),
            (SEMIBOLD, KOREAN),
            (JAPANESE, JAPANESE_FACE),
        ] {
            assert_eq!(pin.release, noto_fonts::revision());
            assert_eq!(pin.source_hint, noto_fonts::source_path(&face));
            assert_eq!(
                pin.license_hint,
                format!("{}{}-OFL.txt", noto_fonts::SOURCE_PREFIX, face.dir)
            );
            assert!(pin.logical_id.ends_with(noto_fonts::revision()));
        }
    }
}
