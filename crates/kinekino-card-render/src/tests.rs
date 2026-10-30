mod asset_contract;
mod fixtures;
#[cfg(feature = "native")]
mod native_png;
mod png_failures;
use crate::{
    application::{PngInspectionStage, RenderError, RenderFailure, ports::RenderPorts, render},
    domain::{BoxContainment, Limitation, Slot, TextOrder},
    infrastructure::{markup, png_encoding::BoundedPngEncoder},
};
use fixtures::*;
use meari::domain::build::ContentHash;

#[test]
fn page_range_bounds_overlap_and_adapter_support_are_explicit() {
    use crate::{
        application::{OptionalFrameRenderRequest, render_with_page},
        domain::*,
    };
    let old = request();
    let mut template = old.template;
    template.canvas.height = 12;
    let mut page = PageFooter {
        current: 2,
        total: 3,
        slot: template.back,
    };
    page.slot.rect.y = 8;
    assert!(page.validate(&template, false).is_ok());
    assert_eq!(page.text(), "2 / 3");
    let mut invalid = page;
    invalid.current = 0;
    assert!(invalid.validate(&template, false).is_err());
    invalid.current = 4;
    assert!(invalid.validate(&template, false).is_err());
    invalid = page;
    invalid.slot.rect.y = 4;
    assert!(invalid.validate(&template, false).is_err());
    invalid.slot.rect.x = u32::MAX;
    assert!(invalid.validate(&template, false).is_err());
    let mut raster = StubRasterizer::default();
    let mut limits = old.limits;
    limits.max_pixels = 128;
    let failure = render_with_page(
        OptionalFrameRenderRequest {
            selection: old.selection.into(),
            frame: None,
            fonts: old.fonts,
            template,
            limits,
            assets: old.assets,
        },
        page,
        RenderPorts {
            png: &FixtureInspection::default(),
            encoder: &BoundedPngEncoder,
            rasterizer: &mut raster,
        },
    )
    .err()
    .unwrap();
    assert!(matches!(
        failure,
        RenderFailure::Rendering(RenderError::Raster(_))
    ));
    assert_eq!(
        raster.calls, 0,
        "Existing adapters must not silently omit a requested page"
    );
}

#[test]
fn manual_text_core_matches_legacy_pixels_without_an_accepted_identity() {
    use crate::{
        application::{TextRenderRequest, render_text},
        domain::CardText,
    };
    let old = request();
    let mut old_raster = StubRasterizer::default();
    let accepted = render(
        old,
        RenderPorts {
            png: &FixtureInspection::default(),
            encoder: &BoundedPngEncoder,
            rasterizer: &mut old_raster,
        },
    )
    .unwrap();
    let old = request();
    let mut raster = StubRasterizer::default();
    let text = CardText {
        front: old.selection.front,
        back: old.selection.back,
    };
    let rendered = render_text(
        TextRenderRequest {
            selection: text,
            frame: old.frame,
            fonts: old.fonts,
            template: old.template,
            limits: old.limits,
            assets: old.assets,
        },
        RenderPorts {
            png: &FixtureInspection::default(),
            encoder: &BoundedPngEncoder,
            rasterizer: &mut raster,
        },
    )
    .unwrap();
    assert_eq!(rendered.selection, text);
    assert_eq!(rendered.png, accepted.png);
    assert_eq!(rendered.png_sha256, accepted.png_sha256);
    assert_eq!(rendered.template_sha256, accepted.template_sha256);
    assert_eq!(rendered.limitations, accepted.limitations);
}

#[test]
fn preserves_selected_revision_position_text_and_pins_in_png_result() {
    let request = request();
    let selection = request.selection;
    let frame_hash = request.frame.sha256.clone();
    let mut rasterizer = StubRasterizer::default();
    let result = render(
        request,
        RenderPorts {
            png: &FixtureInspection::default(),
            encoder: &BoundedPngEncoder,
            rasterizer: &mut rasterizer,
        },
    )
    .unwrap();
    assert_eq!(result.selection, selection);
    assert_eq!(result.frame_sha256, frame_hash);
    assert_eq!(result.png_sha256, ContentHash::of(&result.png));
    assert_eq!(result.size.width, 8);
    assert_eq!(result.front.containment(), BoxContainment::InsideLayoutBox);
    assert!(
        result
            .limitations
            .contains(&Limitation::InkOverflowUnmeasured)
    );
    assert!(
        result
            .limitations
            .contains(&Limitation::GlyphCoverageUnmeasured)
    );
    assert!(
        result
            .limitations
            .contains(&Limitation::TemplateApprovalNotChecked)
    );
    assert_eq!(rasterizer.calls, 1);
    let pixels = decode_rgba(&result.png);
    assert_eq!(pixels.len(), 8 * 8 * 4);
    assert!(
        pixels
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| *pixel == [31, 47, 63, 255])
    );
}

#[test]
fn frame_free_request_needs_no_image_descriptor_pin_or_delivery() {
    use crate::application::{OptionalFrameRenderRequest, render_optional_frame};
    let old = request();
    let mut manifest = old.assets.manifest().clone();
    manifest.files.remove(&old.frame.asset);
    let blobs = manifest
        .files
        .values()
        .map(|descriptor| {
            let path = descriptor.output_path().unwrap();
            let (_, bytes) = old.assets.resource(&path).unwrap();
            (descriptor.content.clone(), bytes.to_vec())
        })
        .collect();
    let assets = meari::application::assets::VerifiedAssets::new(manifest, blobs).unwrap();
    let mut raster = StubRasterizer {
        mode: StubMode::NoFrameDelivery,
        ..Default::default()
    };
    let output = render_optional_frame(
        OptionalFrameRenderRequest {
            selection: old.selection.into(),
            frame: None,
            fonts: old.fonts,
            template: old.template,
            limits: old.limits,
            assets,
        },
        RenderPorts {
            png: &FixtureInspection {
                reject_frame: true,
                ..Default::default()
            },
            encoder: &BoundedPngEncoder,
            rasterizer: &mut raster,
        },
    )
    .unwrap();
    assert!(output.frame_sha256.is_none());
    assert!(output.frame_size.is_none());
    assert_eq!(output.asset_delivery.requests, 0);
    assert_eq!(raster.calls, 1);
}

#[test]
fn mismatched_frame_pin_fails_before_rasterization() {
    let mut request = request();
    request.frame.sha256 = ContentHash::of(b"different frame");
    assert_eq!(
        failure(request, StubMode::Normal),
        RenderError::FramePinMismatch
    );
}

#[test]
fn parser_rejection_is_not_replaced_by_signature_or_manifest_success() {
    let mut rasterizer = StubRasterizer::default();
    let inspector = FixtureInspection {
        reject_frame: true,
        ..Default::default()
    };
    let error = render(
        request(),
        RenderPorts {
            png: &inspector,
            encoder: &BoundedPngEncoder,
            rasterizer: &mut rasterizer,
        },
    )
    .err()
    .unwrap();
    assert!(
        matches!(error, RenderFailure::PngInspection { stage: PngInspectionStage::Frame, cause } if cause == "fixture parser rejection")
    );
    assert_eq!(rasterizer.calls, 0);
}

#[test]
fn invalid_canvas_and_text_limits_fail_before_rasterization() {
    let mut oversized = request();
    oversized.limits.max_pixels = 63;
    assert_eq!(
        failure(oversized, StubMode::Normal),
        RenderError::ResourceLimit
    );
    let mut text = request();
    text.limits.max_text_bytes = 1;
    assert_eq!(failure(text, StubMode::Normal), RenderError::ResourceLimit);
    let mut viewport = request();
    viewport.template.canvas.width = 8193;
    assert!(matches!(
        failure(viewport, StubMode::Normal),
        RenderError::InvalidInput(_)
    ));
}

#[test]
fn template_requires_contained_slots_and_explicit_valid_font_sizes() {
    let mut template = request().template;
    template.front.rect.x = u32::MAX;
    assert!(template.validate().is_err());
    let mut template = request().template;
    template.front.line_height_px = 0;
    assert!(template.validate().is_err());
    let mut template = request().template;
    template.frame_anchor_x_percent = 101;
    assert!(template.validate().is_err());
}

#[test]
fn plain_text_escaping_preserves_both_slots_without_html_or_css_execution() {
    let request = request();
    let mut selection = request.selection;
    selection.front = "<style>body{display:none}</style>&\"'한글\n줄";
    selection.back = "</div><img src=https://example.invalid/onload>日本語";
    let html = markup::document(
        &request.template,
        selection,
        "assets/abc.png",
        &["assets/regular.ttf".into(), "assets/semibold.ttf".into()],
    );
    assert!(html.contains("&lt;style&gt;body{display:none}&lt;/style&gt;&amp;&quot;&#39;한글\n줄"));
    assert!(html.contains("&lt;/div&gt;&lt;img src=https://example.invalid/onload&gt;日本語"));
    assert!(!html.contains("<img src=https://example.invalid"));
    assert!(html.contains("white-space:pre-wrap"));
    assert!(!html.contains("text-overflow:ellipsis"));
    assert!(!html.contains("line-clamp"));
}

#[test]
fn content_box_remains_auto_height_and_explicit_text_order_is_preserved() {
    let mut request = request();
    request.template.text_order = TextOrder::BackThenFront;
    let html = markup::document(
        &request.template,
        request.selection,
        "assets/abc.png",
        &["assets/regular.ttf".into(), "assets/semibold.ttf".into()],
    );
    assert!(html.find("id=\"back-slot\"").unwrap() < html.find("id=\"front-slot\"").unwrap());
    let rule = html
        .split("#front-content{")
        .nth(1)
        .unwrap()
        .split('}')
        .next()
        .unwrap();
    assert!(
        !rule
            .split(';')
            .any(|declaration| declaration.starts_with("height:"))
    );
    assert!(rule.contains("line-height:"));
    assert!(html.contains("object-fit:contain;object-position:50% 50%"));
}

#[test]
fn observed_vertical_overflow_does_not_return_png_success() {
    assert_eq!(
        failure(request(), StubMode::Overflow),
        RenderError::LayoutOverflow(Slot::Back)
    );
}

#[test]
fn nonfinite_layout_is_unavailable_instead_of_fitting() {
    assert_eq!(
        failure(request(), StubMode::Unavailable),
        RenderError::LayoutUnavailable(Slot::Front)
    );
}

#[test]
fn renderer_failure_retains_its_cause() {
    assert_eq!(
        failure(request(), StubMode::Failed),
        RenderError::Raster("fixture paint failed".into())
    );
}

#[test]
fn rgba_size_mismatch_is_not_encoded_as_png() {
    assert_eq!(
        failure(request(), StubMode::ShortRgba),
        RenderError::RgbaLengthMismatch
    );
}

#[test]
fn missing_frame_delivery_and_rejected_assets_cannot_succeed() {
    assert!(matches!(
        failure(request(), StubMode::NoFrameDelivery),
        RenderError::Asset(_)
    ));
    assert!(matches!(
        failure(request(), StubMode::RejectedAsset),
        RenderError::Asset(_)
    ));
}

#[test]
fn png_byte_budget_is_enforced_during_encoding() {
    let mut request = request();
    request.limits.max_png_bytes = 16;
    assert_eq!(
        failure(request, StubMode::Normal),
        RenderError::ResourceLimit
    );
}

#[test]
fn output_inspection_failure_is_not_hidden_by_encoder_success() {
    let mut rasterizer = StubRasterizer::default();
    let inspector = FixtureInspection {
        reject_output: true,
        ..Default::default()
    };
    let error = render(
        request(),
        RenderPorts {
            png: &inspector,
            encoder: &BoundedPngEncoder,
            rasterizer: &mut rasterizer,
        },
    )
    .err()
    .unwrap();
    assert!(
        matches!(error, RenderFailure::PngInspection { stage: PngInspectionStage::Output, cause } if cause == "fixture output rejection")
    );
}

#[test]
fn identical_explicit_inputs_have_stable_template_and_manifest_identity() {
    let first = run(request(), StubMode::Normal).unwrap();
    let second = run(request(), StubMode::Normal).unwrap();
    assert_eq!(first.template_sha256, second.template_sha256);
    assert_eq!(first.asset_manifest_sha256, second.asset_manifest_sha256);
    let mut changed = request();
    changed.template.front.font_px = 2;
    changed.template.front.line_height_px = 2;
    let third = run(changed, StubMode::Normal).unwrap();
    assert_ne!(first.template_sha256, third.template_sha256);
    assert_eq!(first.selection, third.selection);
}

#[test]
fn font_delivery_is_not_claimed_as_glyph_or_approval_verification() {
    let output = run(request(), StubMode::Normal).unwrap();
    assert_eq!(output.font_sha256.len(), 3);
    assert_eq!(output.font_delivery_observed, vec![false, false, false]);
    assert!(
        output
            .limitations
            .contains(&Limitation::HostFontFallbackUncontrolled)
    );
    assert!(
        output
            .limitations
            .contains(&Limitation::WallDeadlineNotProvided)
    );
    assert!(
        output
            .limitations
            .contains(&Limitation::FramePlacementNotPixelVerified)
    );
}

#[test]
fn undeclared_font_is_not_resolved_from_os_or_network() {
    let mut request = request();
    request.fonts.regular = "missing.ttf".to_owned().try_into().unwrap();
    assert!(matches!(
        failure(request, StubMode::Normal),
        RenderError::Asset(_)
    ));
}

#[test]
fn unsupported_text_controls_are_rejected_without_mutating_source() {
    let mut request = request();
    request.selection.front = "before\0after";
    assert_eq!(
        failure(request, StubMode::Normal),
        RenderError::InvalidInput("text contains unsupported rendering controls")
    );
}

#[test]
fn output_dimensions_must_match_explicit_canvas() {
    let mut rasterizer = StubRasterizer::default();
    let inspector = FixtureInspection {
        wrong_output_size: true,
        ..Default::default()
    };
    let error = render(
        request(),
        RenderPorts {
            png: &inspector,
            encoder: &BoundedPngEncoder,
            rasterizer: &mut rasterizer,
        },
    )
    .err()
    .unwrap();
    assert_eq!(
        match error {
            RenderFailure::Rendering(error) => error,
            RenderFailure::PngInspection { .. } => panic!("unexpected parser failure"),
        },
        RenderError::OutputPng("encoded dimensions differ from canvas".into())
    );
}
