//! Independent decoding of the real saved fixture PNG; not a second production strict parser.
use kinekino_player::{
    application::proposals::card_export::ExportCardReviewPng,
    assembly::card_export::product_template,
};
use serde_json::{Value, json};
use std::{fs, io::Cursor, path::Path};

pub(super) fn inspect(path: &Path, data: &Value, input: &ExportCardReviewPng) -> Value {
    let bytes = fs::read(path).unwrap();
    assert_eq!(data["output_sha256"], super::hash(&bytes));
    assert_eq!(data["output_bytes"], bytes.len().to_string());
    let mut reader = png::Decoder::new(Cursor::new(&bytes)).read_info().unwrap();
    assert!(reader.info().animation_control.is_none());
    let size = reader.output_buffer_size().unwrap();
    assert!(size <= 1080 * 1080 * 4);
    let mut pixels = vec![0; size];
    let info = reader.next_frame(&mut pixels).unwrap();
    reader.finish().unwrap();
    assert_eq!((info.width, info.height), (1080, 1080));
    assert_eq!(info.color_type, png::ColorType::Rgba);
    assert_eq!(info.bit_depth, png::BitDepth::Eight);
    let pixel = |x: usize, y: usize| &pixels[(y * 1080 + x) * 4..(y * 1080 + x) * 4 + 4];
    assert_eq!(pixel(100, 200), [11, 12, 14, 255], "contain side margin");
    assert_eq!(
        pixel(400, 200),
        [220, 20, 30, 255],
        "source top-left interior"
    );
    assert_eq!(
        pixel(650, 450),
        [210, 180, 20, 255],
        "source bottom-right interior"
    );
    let ink = |y0, y1| (y0..y1).any(|y| (96..984).any(|x| pixel(x, y) != [11, 12, 14, 255]));
    assert!(ink(640, 800));
    assert!(ink(832, 992));
    let p = &data["provenance"];
    assert_eq!(p["canvas"], json!({"width":1080,"height":1080}));
    assert_eq!(p["frame_size"], json!({"width":8,"height":8}));
    assert_eq!(p["frame_sha256"], input.frame_sha256);
    assert_eq!(
        p["template"],
        serde_json::to_value(&input.template).unwrap()
    );
    assert_eq!(
        p["template_render_version"],
        product_template::RENDER_VERSION
    );
    assert_eq!(p["source_refs"], json!(["cue-1"]));
    let fonts = p["fonts"].as_array().unwrap();
    assert_eq!(fonts.len(), 3);
    for (font, pin) in fonts
        .iter()
        .zip(product_template::png1_proposal().common_fonts)
    {
        assert_eq!(font["sha256"], pin.sha256);
        assert_eq!(font["logical_id"], pin.logical_id);
        assert_eq!(font["release"], pin.release);
        assert!(font["manifest_license"].as_str().unwrap().contains("OFL"));
    }
    assert_eq!(p["font_delivery_observed"], json!([true, true, true]));
    let licenses = p["font_licenses"].as_array().unwrap();
    // Noto Sans, Noto Sans KR and Noto Sans JP OFL notices (pinned in the Meari font manifest).
    assert_eq!(licenses.len(), 3);
    for (license, expected) in licenses.iter().zip([
        "cee9892f9f0cc8fe882c9e9537ee6a89621d86ee7ceaf70b02e2b2b1c25c061a",
        "1c05c68c34f9708415aada51f17e1b0092d2cea709bf4a94cd38114f9e73d7d9",
        "1c05c68c34f9708415aada51f17e1b0092d2cea709bf4a94cd38114f9e73d7d9",
    ]) {
        assert_eq!(
            super::hash(license[1].as_str().unwrap().as_bytes()),
            expected
        );
    }
    json!({"width":1080,"height":1080,"sha256":super::hash(&bytes),"frame_interior_samples":true,"contain_margin_sample":true,"front_and_back_have_ink":true,"font_delivery":p["font_delivery_observed"],"glyph_coverage_proven":false,"readability_proven":false,"visual_approval":false,"template_approved":false})
}
