use super::*;
use crate::application::{OptionalFrameRenderRequest, render_with_page};

#[test]
#[ignore = "Central CPU allocation, pinned existing common fonts and new page observation PNG"]
fn real_page_slot_is_measured_and_painted_without_changing_source_text() {
    let root = std::env::var_os("MEDIA_CARD_RENDER_FONT_ROOT").expect("Existing fonts");
    let target = std::path::PathBuf::from(
        std::env::var_os("MEDIA_CARD_RENDER_PAGE_OUTPUT").expect("New observation PNG"),
    );
    assert!(target.is_absolute());
    let template = fixture_template();
    let page = PageFooter {
        current: 2,
        total: 3,
        slot: TextSlot {
            rect: Rect {
                x: 16,
                y: 296,
                width: 288,
                height: 20,
            },
            font_px: 12,
            line_height_px: 16,
            color: Rgb(233, 235, 237),
            weight: FontWeight::Regular,
            alignment: TextAlignment::Center,
        },
    };
    let text = CardText {
        front: "검토할 카드 제목",
        back: "원래 문구는 그대로",
    };
    let rendered = render_with_page(
        OptionalFrameRenderRequest {
            selection: text,
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
            assets: asset_set(Path::new(&root), None),
        },
        page,
        RenderPorts {
            png: &FixtureInspection::default(),
            encoder: &BoundedPngEncoder,
            rasterizer: &mut MeariCpuRasterizer,
        },
    )
    .unwrap();
    assert_eq!(rendered.card.selection, text);
    assert!(rendered.card.frame_sha256.is_none());
    assert_eq!(rendered.page.containment(), BoxContainment::InsideLayoutBox);
    assert!(has_ink(&decode_rgba(&rendered.card.png), page.slot.rect));
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)
        .unwrap();
    std::io::Write::write_all(&mut output, &rendered.card.png).unwrap();
    output.sync_all().unwrap();
}
