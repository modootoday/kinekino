mod diagnostics;
mod page;
mod template;
pub use diagnostics::*;
pub use page::*;
pub use template::*;

/// Pixel content only. Carries no proposal acceptance or project authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardText<'a> {
    pub front: &'a str,
    pub back: &'a str,
}

impl<'a> From<SelectedCard<'a>> for CardText<'a> {
    fn from(card: SelectedCard<'a>) -> Self {
        Self {
            front: card.front,
            back: card.back,
        }
    }
}

/// Borrowed from one immutable accepted output by the product SDK.
/// This renderer cannot authenticate a proposal store or create an acceptance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectedCard<'a> {
    pub proposal_id: &'a str,
    pub accepted_output_revision: u64,
    pub accepted_output_sha256: &'a str,
    pub position: u64,
    pub front: &'a str,
    pub back: &'a str,
}

/// The caller supplies its existing shared image/text resource limits.
#[derive(Clone, Copy, Debug)]
pub struct RenderLimits {
    pub max_pixels: u64,
    pub max_png_bytes: usize,
    pub max_text_bytes: usize,
}

impl RenderLimits {
    pub fn rgba_bytes(self, size: Size) -> Option<usize> {
        let pixels = u64::from(size.width).checked_mul(u64::from(size.height))?;
        if size.width == 0 || size.height == 0 || pixels > self.max_pixels {
            return None;
        }
        usize::try_from(pixels.checked_mul(4)?).ok()
    }
}
