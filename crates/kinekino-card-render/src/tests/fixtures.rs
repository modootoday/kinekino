//! Definition-based synchronous test doubles, not a PNG validator or actual renderer.
use crate::{
    application::{
        FontSelection, FramePin, RenderError, RenderFailure, RenderRequest, RenderedCard,
        ports::{
            AssetDelivery, CardRasterizer, CardScene, PngInspection, RasterizedCard, RenderPorts,
        },
        render,
    },
    domain::*,
    infrastructure::png_encoding::BoundedPngEncoder,
};
use meari::{
    application::assets::VerifiedAssets,
    domain::{
        assets::{AssetDescriptor, AssetManifest},
        build::{BuildPath, ContentHash, InputRole},
    },
};
use std::{collections::BTreeMap, io::Cursor};

pub(super) fn request() -> RenderRequest<'static> {
    let mut files = BTreeMap::new();
    let mut blobs = BTreeMap::new();
    for (name, bytes, role, mime) in [
        (
            "frame.png",
            b"frame".as_slice(),
            InputRole::Image,
            "image/png",
        ),
        (
            "regular.ttf",
            b"font-regular".as_slice(),
            InputRole::Font,
            "font/ttf",
        ),
        (
            "semibold.ttf",
            b"font-semibold".as_slice(),
            InputRole::Font,
            "font/ttf",
        ),
        (
            "fallback.ttf",
            b"font-fallback".as_slice(),
            InputRole::Font,
            "font/ttf",
        ),
    ] {
        let hash = ContentHash::of(bytes);
        files.insert(
            path(name),
            AssetDescriptor {
                role,
                content: hash.clone(),
                length: bytes.len() as u64,
                media_type: mime.into(),
                license: Some("fixture metadata only; not a real font".into()),
            },
        );
        blobs.insert(hash, bytes.to_vec());
    }
    let slot = TextSlot {
        rect: Rect {
            x: 0,
            y: 0,
            width: 8,
            height: 4,
        },
        font_px: 1,
        line_height_px: 1,
        color: Rgb(0, 0, 0),
        weight: FontWeight::Regular,
        alignment: TextAlignment::Start,
    };
    RenderRequest {
        selection: SelectedCard {
            proposal_id: "fixture-proposal",
            accepted_output_revision: 7,
            accepted_output_sha256: "opaque host-verified accepted output identity",
            position: 2,
            front: "앞면 원문",
            back: "뒷면 日本語",
        },
        frame: FramePin {
            asset: path("frame.png"),
            sha256: ContentHash::of(b"frame"),
        },
        fonts: FontSelection {
            regular: path("regular.ttf"),
            semibold: path("semibold.ttf"),
            fallback: Some(path("fallback.ttf")),
        },
        template: CardTemplate {
            version: "unapproved-test-template-only".into(),
            canvas: Size {
                width: 8,
                height: 8,
            },
            background: Rgb(255, 255, 255),
            frame: Rect {
                x: 0,
                y: 0,
                width: 8,
                height: 8,
            },
            fit: FrameFit::Contain,
            frame_anchor_x_percent: 50,
            frame_anchor_y_percent: 50,
            text_order: TextOrder::FrontThenBack,
            front: slot,
            back: TextSlot {
                rect: Rect { y: 4, ..slot.rect },
                ..slot
            },
        },
        limits: RenderLimits {
            max_pixels: 64,
            max_png_bytes: 4096,
            max_text_bytes: 4096,
        },
        assets: VerifiedAssets::new(
            AssetManifest {
                schema_version: 1,
                files,
            },
            blobs,
        )
        .unwrap(),
    }
}

fn path(text: &str) -> BuildPath {
    text.to_owned().try_into().unwrap()
}

#[derive(Default)]
pub(super) struct FixtureInspection {
    pub reject_frame: bool,
    pub reject_output: bool,
    pub wrong_output_size: bool,
}

impl PngInspection for FixtureInspection {
    type Error = String;

    fn inspect_static_png(&self, bytes: &[u8], _limits: RenderLimits) -> Result<Size, String> {
        if bytes == b"frame" {
            if self.reject_frame {
                return Err("fixture parser rejection".into());
            }
            return Ok(Size {
                width: 8,
                height: 8,
            });
        }
        if self.reject_output {
            return Err("fixture output rejection".into());
        }
        let decoder = png::Decoder::new(Cursor::new(bytes));
        let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
        let size = Size {
            width: reader.info().width,
            height: reader.info().height,
        };
        let mut rgba = vec![
            0;
            reader
                .output_buffer_size()
                .ok_or("fixture output size missing")?
        ];
        reader.next_frame(&mut rgba).map_err(|e| e.to_string())?;
        reader.finish().map_err(|e| e.to_string())?;
        if self.wrong_output_size {
            return Ok(Size {
                width: 1,
                height: 1,
            });
        }
        Ok(size)
    }
}

#[derive(Default, Clone, Copy)]
pub(super) enum StubMode {
    #[default]
    Normal,
    Overflow,
    Unavailable,
    Failed,
    ShortRgba,
    NoFrameDelivery,
    RejectedAsset,
}

#[derive(Default)]
pub(super) struct StubRasterizer {
    pub calls: usize,
    pub mode: StubMode,
}

impl CardRasterizer for StubRasterizer {
    fn rasterize(
        &mut self,
        scene: CardScene<'_>,
        _assets: VerifiedAssets,
        size: Size,
    ) -> Result<RasterizedCard, String> {
        self.calls += 1;
        if matches!(self.mode, StubMode::Failed) {
            return Err("fixture paint failed".into());
        }
        let mut front = observation(scene.template.front.rect);
        let mut back = observation(scene.template.back.rect);
        if matches!(self.mode, StubMode::Overflow) {
            back.content.height += 1.0;
        }
        if matches!(self.mode, StubMode::Unavailable) {
            front.content.width = f32::NAN;
        }
        let mut rgba = [31, 47, 63, 255].repeat(size.width as usize * size.height as usize);
        if matches!(self.mode, StubMode::ShortRgba) {
            rgba.pop();
        }
        let mut delivery = AssetDelivery::default();
        if !matches!(self.mode, StubMode::NoFrameDelivery) {
            delivery.delivered.insert(scene.frame_url.to_owned(), 1);
            delivery.requests = 1;
        }
        if matches!(self.mode, StubMode::RejectedAsset) {
            delivery.rejected = 1;
        }
        Ok(RasterizedCard {
            rgba,
            front,
            back,
            assets: delivery,
        })
    }
}

fn observation(rect: Rect) -> SlotObservation {
    let slot = LayoutBounds {
        x: rect.x as f32,
        y: rect.y as f32,
        width: rect.width as f32,
        height: rect.height as f32,
    };
    SlotObservation {
        slot,
        content: slot,
    }
}

pub(super) fn run(
    request: RenderRequest<'static>,
    mode: StubMode,
) -> Result<RenderedCard<'static>, RenderFailure<String>> {
    let mut rasterizer = StubRasterizer { calls: 0, mode };
    render(
        request,
        RenderPorts {
            png: &FixtureInspection::default(),
            encoder: &BoundedPngEncoder,
            rasterizer: &mut rasterizer,
        },
    )
}

pub(super) fn failure(request: RenderRequest<'static>, mode: StubMode) -> RenderError {
    match run(request, mode).err().expect("fixture must fail") {
        RenderFailure::Rendering(error) => error,
        RenderFailure::PngInspection { .. } => {
            panic!("unexpected inspection failure in rendering fixture")
        }
    }
}

pub(super) fn decode_rgba(bytes: &[u8]) -> Vec<u8> {
    let decoder = png::Decoder::new(Cursor::new(bytes));
    let mut reader = decoder.read_info().unwrap();
    assert_eq!(reader.info().color_type, png::ColorType::Rgba);
    assert_eq!(reader.info().bit_depth, png::BitDepth::Eight);
    let mut rgba = vec![0; reader.output_buffer_size().unwrap()];
    reader.next_frame(&mut rgba).unwrap();
    reader.finish().unwrap();
    rgba
}
