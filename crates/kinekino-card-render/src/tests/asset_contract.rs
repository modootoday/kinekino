//! Port-delivery evidence only: synthetic fonts/frame and stub paint are not native PNG proof.
use super::fixtures::{FixtureInspection, StubRasterizer, request};
use crate::{
    application::{
        RenderError,
        ports::{
            CardRasterizer, CardScene, PngEncoding, PngInspection, RasterizedCard, RenderPorts,
        },
        render,
    },
    domain::{RenderLimits, SelectedCard, Size},
    infrastructure::png_encoding::BoundedPngEncoder,
};
use meari::{
    application::assets::VerifiedAssets,
    domain::build::{BuildPath, ContentHash},
};
use std::{cell::RefCell, rc::Rc};

type Events = Rc<RefCell<Vec<&'static str>>>;

struct Inspection(Events);
impl PngInspection for Inspection {
    type Error = String;

    fn inspect_static_png(&self, bytes: &[u8], limits: RenderLimits) -> Result<Size, String> {
        self.0.borrow_mut().push(if bytes == b"frame" {
            "frame-inspect"
        } else {
            "output-inspect"
        });
        FixtureInspection::default().inspect_static_png(bytes, limits)
    }
}

struct Encoder(Events);
impl PngEncoding for Encoder {
    fn encode_rgba(&self, size: Size, rgba: &[u8], limit: usize) -> Result<Vec<u8>, RenderError> {
        self.0.borrow_mut().push("encode");
        BoundedPngEncoder.encode_rgba(size, rgba, limit)
    }
}

struct SceneCheck {
    events: Events,
    selection: SelectedCard<'static>,
    template: serde_json::Value,
    manifest: ContentHash,
}
impl CardRasterizer for SceneCheck {
    fn rasterize(
        &mut self,
        scene: CardScene<'_>,
        assets: VerifiedAssets,
        size: Size,
    ) -> Result<RasterizedCard, String> {
        assert_eq!(*self.events.borrow(), ["frame-inspect"]);
        assert_eq!(scene.selection, self.selection.into());
        assert_eq!(
            scene.selection.front.as_ptr(),
            self.selection.front.as_ptr()
        );
        assert_eq!(scene.selection.back.as_ptr(), self.selection.back.as_ptr());
        assert_eq!(serde_json::to_value(scene.template).unwrap(), self.template);
        assert_eq!(scene.template.canvas, size);
        assert_eq!(assets.manifest().fingerprint().unwrap(), self.manifest);
        let frame_path = BuildPath::try_from(scene.frame_url.to_owned()).unwrap();
        let (mime, frame) = assets.resource(&frame_path).unwrap();
        assert_eq!(mime, "image/png");
        assert_eq!(frame, b"frame");
        assert_eq!(scene.font_urls.len(), 3);
        for (url, expected) in scene.font_urls.iter().zip([
            b"font-regular".as_slice(),
            b"font-semibold".as_slice(),
            b"font-fallback".as_slice(),
        ]) {
            let path = BuildPath::try_from(url.to_owned()).unwrap();
            let (mime, bytes) = assets.resource(&path).unwrap();
            assert_eq!(mime, "font/ttf");
            assert_eq!(bytes, expected);
            assert!(url.contains(ContentHash::of(expected).as_str()));
        }
        self.events.borrow_mut().push("raster");
        StubRasterizer::default().rasterize(scene, assets, size)
    }
}

#[test]
fn exact_template_selection_and_asset_bytes_reach_port_in_inspect_render_encode_order() {
    let input = request();
    let events = Events::default();
    let expected_template = ContentHash::of(&serde_json::to_vec(&input.template).unwrap());
    let mut raster = SceneCheck {
        events: events.clone(),
        selection: input.selection,
        template: serde_json::to_value(&input.template).unwrap(),
        manifest: input.assets.manifest().fingerprint().unwrap(),
    };
    let output = render(
        input,
        RenderPorts {
            png: &Inspection(events.clone()),
            encoder: &Encoder(events.clone()),
            rasterizer: &mut raster,
        },
    )
    .unwrap();
    assert_eq!(
        *events.borrow(),
        ["frame-inspect", "raster", "encode", "output-inspect"]
    );
    assert_eq!(output.template_sha256, expected_template);
    assert_eq!(output.asset_manifest_sha256, raster.manifest);
    assert_eq!(output.selection, raster.selection);
}
