use crate::{
    application::ports::{
        AssetDelivery, CardRasterizer, CardScene, PagedRasterizedCard, RasterizedCard,
    },
    domain::{LayoutBounds, Rect, Size, SlotObservation},
};
use meari::application::assets::VerifiedAssets;
use meari_native::{
    application::Renderer,
    domain::{LayoutBox, Viewport},
    infrastructure::engine::NativeRenderer,
};

/// Uses the current public headless Renderer API; no window or screenshot automation.
pub struct MeariCpuRasterizer;

impl CardRasterizer for MeariCpuRasterizer {
    fn rasterize(
        &mut self,
        scene: CardScene<'_>,
        assets: VerifiedAssets,
        size: Size,
    ) -> Result<RasterizedCard, String> {
        self.rasterize_with_page(scene, assets, size, None)
            .map(|painted| painted.card)
    }

    fn rasterize_with_page(
        &mut self,
        scene: CardScene<'_>,
        assets: VerifiedAssets,
        size: Size,
        page: Option<&crate::domain::PageFooter>,
    ) -> Result<PagedRasterizedCard, String> {
        let viewport = Viewport {
            width: size.width,
            height: size.height,
            dark: false,
        };
        let html = if page.is_some() {
            super::markup::document_with_page(
                scene.template,
                scene.selection,
                scene.frame_url,
                scene.font_urls,
                page,
            )
        } else {
            super::markup::document(
                scene.template,
                scene.selection,
                scene.frame_url,
                scene.font_urls,
            )
        };
        let mut renderer = NativeRenderer::from_html_with_assets(&html, assets)?;
        renderer.layout(&viewport, 0.0)?;
        let rgba = renderer.rasterize(&viewport)?;
        let front = observation(&renderer, "front-slot", "front-content")?;
        let back = observation(&renderer, "back-slot", "back-content")?;
        require_slot(front.slot, scene.template.front.rect)?;
        require_slot(back.slot, scene.template.back.rect)?;
        let page_observation = if let Some(page) = page {
            let measured = observation(&renderer, "page-slot", "page-content")?;
            require_slot(measured.slot, page.slot.rect)?;
            Some(measured)
        } else {
            None
        };
        let report = renderer
            .asset_report()?
            .ok_or("declared-memory asset report unavailable")?;
        Ok(PagedRasterizedCard {
            card: RasterizedCard {
                rgba,
                front,
                back,
                assets: AssetDelivery {
                    requests: report.requests,
                    delivered: report.delivered,
                    rejected: report.rejected,
                    failures: report
                        .failures
                        .into_iter()
                        .map(|f| format!("{}: {}", f.path, f.reason))
                        .collect(),
                },
            },
            page: page_observation,
        })
    }
}

fn require_slot(actual: LayoutBounds, expected: Rect) -> Result<(), String> {
    let values = [
        (actual.x, expected.x),
        (actual.y, expected.y),
        (actual.width, expected.width),
        (actual.height, expected.height),
    ];
    if values.into_iter().any(|(observed, declared)| {
        !observed.is_finite() || (observed - declared as f32).abs() > 0.25
    }) {
        return Err("renderer slot box differs from the explicit template".into());
    }
    Ok(())
}

fn observation(
    renderer: &NativeRenderer,
    slot: &str,
    content: &str,
) -> Result<SlotObservation, String> {
    Ok(SlotObservation {
        slot: bounds(renderer.bounds(slot)?),
        content: bounds(renderer.bounds(content)?),
    })
}

fn bounds(value: LayoutBox) -> LayoutBounds {
    LayoutBounds {
        x: value.x,
        y: value.y,
        width: value.width,
        height: value.height,
    }
}
