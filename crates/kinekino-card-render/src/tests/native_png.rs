//! Explicitly allocated native buffer test; not accepted-store/CLI/publication/visual approval.
mod frame_free;
mod page;
use super::fixtures::{FixtureInspection, decode_rgba};
use crate::{
    application::{
        FontSelection, FramePin, RenderRequest,
        ports::{PngEncoding, RenderPorts},
        render,
    },
    domain::*,
    infrastructure::{native::MeariCpuRasterizer, png_encoding::BoundedPngEncoder},
};
use meari::{
    application::assets::VerifiedAssets,
    domain::{
        assets::{AssetDescriptor, AssetManifest},
        build::{BuildPath, ContentHash, InputRole},
        design::fonts::{MANIFEST, Pin, REVISION},
    },
};
use std::{collections::BTreeMap, path::Path};

// Pinned Noto faces (SIL OFL-1.1) from `meari::domain::design::fonts`; regular and semibold
// are the same variable file at two `font-weight`s.
const KOREAN: Pin = pin("Noto Sans KR");
const JAPANESE: Pin = pin("Noto Sans JP");
const REGULAR: &str = KOREAN.sha256;
const SEMIBOLD: &str = KOREAN.sha256;
const KOREAN_FILE: &str = "fonts/noto/NotoSansKR[wght].ttf";
const JAPANESE_FILE: &str = "fonts/noto/NotoSansJP[wght].ttf";

const fn pin(family: &str) -> Pin {
    const fn same(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let mut i = 0;
        while i < a.len() {
            if a[i] != b[i] {
                return false;
            }
            i += 1;
        }
        true
    }
    let mut i = 0;
    while i < MANIFEST.len() {
        let p = MANIFEST[i];
        if same(p.family.as_bytes(), family.as_bytes()) && same(p.style.as_bytes(), b"normal") {
            return p;
        }
        i += 1;
    }
    panic!("family absent from the pinned Noto manifest")
}

fn path(text: &str) -> BuildPath {
    text.to_owned().try_into().unwrap()
}

fn asset_set(root: &Path, frame: Option<Vec<u8>>) -> VerifiedAssets {
    assert!(
        root.is_absolute(),
        "explicit absolute common font root required"
    );
    // Root is the Noto cache root (`runtime/fonts`, `kinekino fonts fetch`).
    let dir = root.join("noto").join(&REVISION[..12]);
    for face in [KOREAN, JAPANESE] {
        let bytes = std::fs::read(dir.join(format!("{}-OFL.txt", face.dir)))
            .expect("pinned Noto OFL notice required");
        assert_eq!(ContentHash::of(&bytes).as_str(), face.ofl_sha256);
    }
    let mut manifest = AssetManifest {
        schema_version: 1,
        files: BTreeMap::new(),
    };
    let mut blobs = BTreeMap::new();
    for (file, face) in [(KOREAN_FILE, KOREAN), (JAPANESE_FILE, JAPANESE)] {
        let sha = face.sha256;
        let bytes = std::fs::read(dir.join(face.file)).expect("pinned Noto TTF required");
        let hash = ContentHash::of(&bytes);
        assert_eq!(hash.as_str(), sha);
        manifest.files.insert(
            path(file),
            AssetDescriptor {
                role: InputRole::Font,
                content: hash.clone(),
                length: bytes.len() as u64,
                media_type: "font/ttf".into(),
                license: Some(format!("OFL-1.1; {}-OFL.txt", face.dir)),
            },
        );
        blobs.insert(hash, bytes);
    }
    if let Some(frame) = frame {
        let hash = ContentHash::of(&frame);
        manifest.files.insert(
            path("frame.png"),
            AssetDescriptor {
                role: InputRole::Image,
                content: hash.clone(),
                length: frame.len() as u64,
                media_type: "image/png".into(),
                license: None,
            },
        );
        blobs.insert(hash, frame);
    }
    VerifiedAssets::new(manifest, blobs).unwrap()
}

fn fixture_template() -> CardTemplate {
    let slot = TextSlot {
        rect: Rect {
            x: 16,
            y: 180,
            width: 288,
            height: 48,
        },
        font_px: 16,
        line_height_px: 24,
        color: Rgb(233, 235, 237),
        weight: FontWeight::Regular,
        alignment: TextAlignment::Start,
    };
    CardTemplate {
        version: "unapproved.native-buffer-test.1".into(),
        canvas: Size {
            width: 320,
            height: 320,
        },
        background: Rgb(3, 5, 7),
        frame: Rect {
            x: 0,
            y: 0,
            width: 320,
            height: 160,
        },
        fit: FrameFit::Contain,
        frame_anchor_x_percent: 50,
        frame_anchor_y_percent: 50,
        text_order: TextOrder::FrontThenBack,
        front: slot,
        back: TextSlot {
            rect: Rect {
                y: 244,
                ..slot.rect
            },
            weight: FontWeight::Semibold,
            ..slot
        },
    }
}

fn pixel(bytes: &[u8], x: usize, y: usize) -> &[u8] {
    let offset = (y * 320 + x) * 4;
    &bytes[offset..offset + 4]
}

fn has_ink(bytes: &[u8], rect: Rect) -> bool {
    (rect.y..rect.y + rect.height).any(|y| {
        (rect.x..rect.x + rect.width)
            .any(|x| pixel(bytes, x as usize, y as usize) != [3, 5, 7, 255])
    })
}

#[test]
#[ignore = "requires central native CPU allocation and MEDIA_CARD_RENDER_FONT_ROOT; buffer only, no publication"]
fn real_frame_and_pinned_fonts_render_decodable_png_with_explicit_limitations() {
    let root = std::env::var_os("MEDIA_CARD_RENDER_FONT_ROOT")
        .expect("allocated existing common font root required; no fallback/download");
    // Keep the bilinear neighbours at both sample points inside a solid quadrant.
    // A 2x2 image has no such interior when enlarged with the default image filter.
    let colors = [
        [220, 20, 30, 255],
        [30, 200, 40, 255],
        [30, 40, 210, 255],
        [210, 180, 20, 255],
    ];
    let mut source_pixels = Vec::with_capacity(8 * 8 * 4);
    for y in 0..8 {
        for x in 0..8 {
            source_pixels.extend_from_slice(&colors[(y / 4) * 2 + x / 4]);
        }
    }
    let frame = BoundedPngEncoder
        .encode_rgba(
            Size {
                width: 8,
                height: 8,
            },
            &source_pixels,
            65536,
        )
        .unwrap();
    let frame_sha = ContentHash::of(&frame);
    let assets = asset_set(Path::new(&root), Some(frame));
    let manifest_sha = assets.manifest().fingerprint().unwrap();
    let template = fixture_template();
    let template_sha = ContentHash::of(&serde_json::to_vec(&template).unwrap());
    let front_rect = template.front.rect;
    let back_rect = template.back.rect;
    let selection = SelectedCard {
        proposal_id: "native-buffer-fixture-only",
        accepted_output_revision: 1,
        accepted_output_sha256: "host-selection-fixture-not-a-store-acceptance",
        position: 0,
        front: "앞문구 <b>& 한글",
        back: "かな 日本語 뒷문구",
    };
    let input = RenderRequest {
        selection,
        frame: FramePin {
            asset: path("frame.png"),
            sha256: frame_sha.clone(),
        },
        fonts: FontSelection {
            regular: path(KOREAN_FILE),
            semibold: path(KOREAN_FILE),
            fallback: Some(path(JAPANESE_FILE)),
        },
        template,
        limits: RenderLimits {
            max_pixels: 320 * 320,
            max_png_bytes: 1024 * 1024,
            max_text_bytes: 8192,
        },
        assets,
    };
    // Known-good generated frame/output decoding only. Root strict parser remains SDK-owned.
    let output = render(
        input,
        RenderPorts {
            png: &FixtureInspection::default(),
            encoder: &BoundedPngEncoder,
            rasterizer: &mut MeariCpuRasterizer,
        },
    )
    .unwrap();
    assert_eq!(
        output.size,
        Size {
            width: 320,
            height: 320
        }
    );
    assert_eq!(
        output.frame_size,
        Size {
            width: 8,
            height: 8
        }
    );
    assert_eq!(output.selection, selection);
    assert_eq!(output.frame_sha256, frame_sha);
    assert_eq!(output.template_sha256, template_sha);
    assert_eq!(output.asset_manifest_sha256, manifest_sha);
    assert_eq!(
        output
            .font_sha256
            .iter()
            .map(ContentHash::as_str)
            .collect::<Vec<_>>(),
        [REGULAR, SEMIBOLD, JAPANESE.sha256]
    );
    assert_eq!(output.font_delivery_observed.len(), 3);
    assert!(
        output.font_delivery_observed[0],
        "regular TTF handler delivery required, not a glyph-choice claim"
    );
    assert!(
        output.font_delivery_observed[1],
        "semibold TTF handler delivery required, not a glyph-choice claim"
    );
    assert!(
        output.font_delivery_observed[2],
        "declared Japanese fallback TTF handler delivery required, not a coverage claim"
    );
    assert_eq!(output.front.containment(), BoxContainment::InsideLayoutBox);
    assert_eq!(output.back.containment(), BoxContainment::InsideLayoutBox);
    assert_eq!(output.png_sha256, ContentHash::of(&output.png));
    let pixels = decode_rgba(&output.png);
    assert_eq!(pixels.len(), 320 * 320 * 4);
    assert_eq!(
        pixel(&pixels, 16, 80),
        [3, 5, 7, 255],
        "contain side margin must preserve background"
    );
    assert_eq!(pixel(&pixels, 120, 40), [220, 20, 30, 255]);
    assert_eq!(pixel(&pixels, 200, 120), [210, 180, 20, 255]);
    assert!(
        has_ink(&pixels, front_rect),
        "front slot must paint some ink, not prove readable spelling"
    );
    assert!(
        has_ink(&pixels, back_rect),
        "back slot must paint some ink, not prove readable spelling"
    );
    assert_eq!(output.limitations, RENDER_LIMITATIONS);
    eprintln!(
        "native_card_png_buffer={{\"sha256\":\"{}\",\"bytes\":{},\"width\":320,\"height\":320,\"font_delivery\":{:?},\"visual_approval\":false,\"published\":false}}",
        output.png_sha256.as_str(),
        output.png.len(),
        output.font_delivery_observed
    );
}
