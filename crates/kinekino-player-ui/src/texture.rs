//! `VideoSurface` → Meari raster texture. The video element's laid-out box decides the
//! frame size; a frame is pulled only when the surface has a new one or the box resized,
//! and is painted by Meari's image path (honouring CSS `object-fit`).
use blitz_dom::node::{ImageData, RasterImageData, SpecialElementData};
use kinekino_core::playback::{RgbaFrame, VideoSurface};
use kinekino_core::port::error::{ErrorCode, PortError, PortResult};
use linebender_resource_handle::Blob;
use meari_native::infrastructure::document::InteractiveDocument;
use std::sync::Arc;

/// Element id the player source gives its video box.
pub const VIDEO_ID: &str = "video";

#[derive(Debug, Default)]
pub struct VideoTexture {
    frame: Option<RgbaFrame>,
    uploads: u64,
}

fn missing() -> PortError {
    PortError::new(ErrorCode::InvalidInput, "Player video element is missing")
}

impl VideoTexture {
    /// Frames uploaded so far (evidence for smoke runs and tests).
    pub fn uploads(&self) -> u64 {
        self.uploads
    }

    /// Size in pixels of the video box after the last layout; `None` before layout.
    pub fn target_size(document: &InteractiveDocument) -> PortResult<Option<(u32, u32)>> {
        let dom = &document.renderer.document;
        let id = dom.get_element_by_id(VIDEO_ID).ok_or_else(missing)?;
        let size = dom.get_node(id).ok_or_else(missing)?.final_layout().size;
        let (w, h) = (size.width.round() as u32, size.height.round() as u32);
        Ok((w > 0 && h > 0).then_some((w, h)))
    }

    /// Pulls a frame into the video element when one is due. Returns whether it uploaded.
    pub fn upload(
        &mut self,
        document: &mut InteractiveDocument,
        surface: &mut dyn VideoSurface,
    ) -> PortResult<bool> {
        let Some((w, h)) = Self::target_size(document)? else {
            return Ok(false);
        };
        let resized = self
            .frame
            .as_ref()
            .is_none_or(|f| (f.width, f.height) != (w, h));
        if !resized && !surface.frame_ready()? {
            return Ok(false);
        }
        let mut frame = match self.frame.take() {
            Some(f) if !resized => f,
            _ => RgbaFrame::new(w, h)?,
        };
        surface.render(&mut frame)?;
        let dom = &mut document.renderer.document;
        let id = dom.get_element_by_id(VIDEO_ID).ok_or_else(missing)?;
        let node = dom.get_node_mut(id).ok_or_else(missing)?;
        let element = node.element_data_mut().ok_or_else(missing)?;
        element.special_data =
            SpecialElementData::Image(Box::new(ImageData::Raster(RasterImageData {
                width: w,
                height: h,
                data: Blob::new(Arc::new(frame.pixels.clone())),
            })));
        node.clear_layout_cache();
        self.frame = Some(frame);
        self.uploads += 1;
        Ok(true)
    }
}
