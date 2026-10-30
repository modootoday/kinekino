use super::*;
use crate::application::{OptionalFrameRenderRequest, render_optional_frame};

#[test]
#[ignore = "Central native CPU allocation, existing fonts and explicit new observation PNG"]
fn real_frame_free_text_uses_only_common_fonts_and_has_no_image_identity() {
    let root = std::env::var_os("MEDIA_CARD_RENDER_FONT_ROOT").expect("Pinned existing font root");
    let target =
        std::env::var_os("MEDIA_CARD_RENDER_FRAME_FREE_OUTPUT").expect("New observation PNG");
    let target = Path::new(&target);
    assert!(target.is_absolute());
    assert!(!target.exists());
    let assets = asset_set(Path::new(&root), None);
    assert!(
        assets
            .manifest()
            .files
            .values()
            .all(|entry| entry.role == InputRole::Font)
    );
    let template = fixture_template();
    let front = template.front.rect;
    let back = template.back.rect;
    let output = render_optional_frame(
        OptionalFrameRenderRequest {
            selection: CardText {
                front: "프레임 없는 카드",
                back: "검토할 문구 초안",
            },
            frame: None,
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
        },
        RenderPorts {
            png: &FixtureInspection::default(),
            encoder: &BoundedPngEncoder,
            rasterizer: &mut MeariCpuRasterizer,
        },
    )
    .unwrap();
    assert!(output.frame_sha256.is_none());
    assert!(output.frame_size.is_none());
    assert!(
        output
            .asset_delivery
            .delivered
            .keys()
            .all(|key| !key.ends_with(".png"))
    );
    assert_eq!(output.front.containment(), BoxContainment::InsideLayoutBox);
    assert_eq!(output.back.containment(), BoxContainment::InsideLayoutBox);
    let pixels = decode_rgba(&output.png);
    assert_eq!(
        output.size,
        Size {
            width: 320,
            height: 320
        }
    );
    assert_eq!(pixel(&pixels, 80, 60), [3, 5, 7, 255]);
    assert!(has_ink(&pixels, front));
    assert!(has_ink(&pixels, back));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)
        .unwrap();
    std::io::Write::write_all(&mut file, &output.png).unwrap();
    file.sync_all().unwrap();
}
