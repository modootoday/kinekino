use super::{assets, frame, inspection::StrictPng};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        proposals::card_export::{
            ExportCardReviewPng, LIMITATIONS, PreparedReviewPng, SelectedAcceptedCard,
            TemplateSelection,
        },
    },
    assembly::card_export::{
        canvas_templates,
        product_template::{self, png1_proposal},
    },
    domain::frame_capture::{MAX_FRAME_PIXELS, MAX_FRAME_PNG_BYTES},
};
use kinekino_card_render::{
    application::{
        self, FontSelection, FramePin, OptionalFrameRenderRequest, PngInspectionStage, RenderError,
        RenderFailure, ports::RenderPorts,
    },
    domain::{CardText, RenderLimits},
    infrastructure::{native::MeariCpuRasterizer, png_encoding::BoundedPngEncoder},
};
use meari::domain::build::{BuildPath, ContentHash};
use serde_json::json;
use sha2::{Digest, Sha256};

pub(super) fn render(
    selection: SelectedAcceptedCard<'_>,
    input: &ExportCardReviewPng,
) -> Result<PreparedReviewPng, ApiError> {
    render_content(
        CardText {
            front: selection.front,
            back: selection.back,
        },
        selection.source_refs,
        &input.frame_path,
        &input.frame_sha256,
        &input.template,
        Some(&input.output_path),
    )
}

pub(super) fn canvas_declaration_sha256() -> String {
    let mut hash = Sha256::new();
    hash.update(include_bytes!(
        "../../assembly/card_export/product_template.rs"
    ));
    hash.update(include_bytes!(
        "../../assembly/card_export/canvas_templates.rs"
    ));
    format!("{:x}", hash.finalize())
}

pub(super) fn resolve_template(
    template: &TemplateSelection,
) -> Result<kinekino_card_render::domain::CardTemplate, ApiError> {
    if role_pages(template)
        && template.version == crate::assembly::card_export::role_templates::VERSION
        && template.declaration_sha256 == role_declaration_sha256()
        && let Some(layout) = crate::assembly::card_export::role_templates::layout(
            &template.id,
            crate::domain::card_document::CardRole::Body,
        )
    {
        return Ok(layout);
    }
    if template.version == crate::assembly::card_export::blank_templates::VERSION
        && template.declaration_sha256 == blank_declaration_sha256()
        && let Some(layout) = crate::assembly::card_export::blank_templates::layout(&template.id)
    {
        return Ok(layout);
    }
    if template.version == canvas_templates::VERSION
        && template.declaration_sha256 == canvas_declaration_sha256()
        && let Some(layout) = canvas_templates::layout(&template.id)
    {
        return Ok(layout);
    }
    let declaration = declaration_sha256();
    if template.id != product_template::TEMPLATE_ID
        || template.version != product_template::PROPOSAL_VERSION
        || template.declaration_sha256 != declaration
    {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Explicit template selection does not match the product proposal declaration",
        ));
    }
    Ok(png1_proposal().layout)
}

pub(super) fn blank_declaration_sha256() -> String {
    let mut hash = Sha256::new();
    hash.update(include_bytes!(
        "../../assembly/card_export/product_template.rs"
    ));
    hash.update(include_bytes!(
        "../../assembly/card_export/canvas_templates.rs"
    ));
    hash.update(include_bytes!(
        "../../assembly/card_export/blank_templates.rs"
    ));
    format!("{:x}", hash.finalize())
}

pub(super) fn frame_required(template: &TemplateSelection) -> bool {
    if role_pages(template) {
        return crate::assembly::card_export::role_templates::frame_required(&template.id);
    }
    !crate::assembly::card_export::blank_templates::IDS.contains(&template.id.as_str())
}

pub(super) fn role_pages(template: &TemplateSelection) -> bool {
    crate::assembly::card_export::role_templates::IDS.contains(&template.id.as_str())
}
pub(super) fn role_declaration_sha256() -> String {
    let mut hash = Sha256::new();
    hash.update(include_bytes!(
        "../../assembly/card_export/product_template.rs"
    ));
    hash.update(include_bytes!(
        "../../assembly/card_export/canvas_templates.rs"
    ));
    hash.update(include_bytes!(
        "../../assembly/card_export/blank_templates.rs"
    ));
    hash.update(include_bytes!(
        "../../assembly/card_export/role_templates.rs"
    ));
    format!("{:x}", hash.finalize())
}
pub(super) struct CardPageContext {
    pub role: crate::domain::card_document::CardRole,
    pub position: usize,
    pub total: usize,
}

pub(super) fn declaration_sha256() -> String {
    format!(
        "{:x}",
        Sha256::digest(include_bytes!(
            "../../assembly/card_export/product_template.rs"
        ))
    )
}

pub(super) fn render_content(
    text: CardText<'_>,
    source_refs: &[String],
    frame_path: &str,
    frame_sha256: &str,
    template: &TemplateSelection,
    output_path: Option<&str>,
) -> Result<PreparedReviewPng, ApiError> {
    render_optional_content(
        text,
        source_refs,
        Some((frame_path, frame_sha256)),
        template,
        output_path,
    )
}

pub(super) fn render_optional_content(
    text: CardText<'_>,
    source_refs: &[String],
    frame_input: Option<(&str, &str)>,
    template: &TemplateSelection,
    output_path: Option<&str>,
) -> Result<PreparedReviewPng, ApiError> {
    render_project_content(text, source_refs, frame_input, template, output_path, None)
}

pub(super) fn render_project_content(
    text: CardText<'_>,
    source_refs: &[String],
    frame_input: Option<(&str, &str)>,
    template: &TemplateSelection,
    output_path: Option<&str>,
    context: Option<CardPageContext>,
) -> Result<PreparedReviewPng, ApiError> {
    let mut layout = resolve_template(template)?;
    let page = if role_pages(template) {
        let context = context.as_ref().ok_or_else(|| {
            ApiError::new(
                ErrorCode::NeedsInput,
                "Role template requires the immutable project page context",
            )
        })?;
        if context.total == 0
            || context.total > crate::domain::card_document::MAX_CARDS
            || context.position >= context.total
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid project page context",
            ));
        }
        layout = crate::assembly::card_export::role_templates::layout(&template.id, context.role)
            .ok_or_else(|| {
            ApiError::new(ErrorCode::RevisionConflict, "Role template absent")
        })?;
        Some(crate::assembly::card_export::role_templates::page(
            &layout,
            (context.position + 1) as u32,
            context.total as u32,
        ))
    } else {
        None
    };
    if frame_required(template) != frame_input.is_some() {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Selected template frame input requirement differs",
        ));
    }
    if let Some(path) = output_path {
        frame::validate_output(path)?;
    }
    let bytes = frame_input
        .map(|(path, sha)| frame::read(path, sha))
        .transpose()?;
    let frame_hash = bytes.as_ref().map(|bytes| ContentHash::of(bytes));
    let (assets, licenses) = assets::prepare(bytes)?;
    let fonts = assets::font_metadata(&assets)?;
    let font = |pin: &meari::domain::design::fonts::Pin| {
        BuildPath::try_from(crate::infrastructure::noto_fonts::source_path(pin))
            .map_err(assets::error)
    };
    let request = OptionalFrameRenderRequest {
        selection: text,
        frame: if let Some(sha256) = frame_hash {
            Some(FramePin {
                asset: BuildPath::try_from(assets::FRAME.to_owned()).map_err(assets::error)?,
                sha256,
            })
        } else {
            None
        },
        fonts: FontSelection {
            regular: font(&crate::infrastructure::noto_fonts::KOREAN)?,
            semibold: font(&crate::infrastructure::noto_fonts::KOREAN)?,
            fallback: Some(font(&crate::infrastructure::noto_fonts::JAPANESE)?),
        },
        template: layout,
        limits: RenderLimits {
            max_pixels: MAX_FRAME_PIXELS,
            max_png_bytes: usize::try_from(MAX_FRAME_PNG_BYTES)
                .map_err(|_| assets::error("Frame budget exceeds platform size"))?,
            max_text_bytes: 131072,
        },
        assets,
    };
    let mut raster = MeariCpuRasterizer;
    let ports = RenderPorts {
        png: &StrictPng,
        encoder: &BoundedPngEncoder,
        rasterizer: &mut raster,
    };
    let (rendered, page_observation) = if let Some(page) = page {
        let painted = application::render_with_page(request, page, ports).map_err(map_failure)?;
        (painted.card, Some(painted.page))
    } else {
        (
            application::render_optional_frame(request, ports).map_err(map_failure)?,
            None,
        )
    };
    let mut provenance = json!({"frame_path":frame_input.map(|(path,_)|path),"frame_sha256":frame_input.map(|(_,sha)|sha),"frame_size":rendered.frame_size,"canvas":{"width":rendered.size.width,"height":rendered.size.height},"template":template,"template_render_version":rendered.template_version,"template_dto_sha256":rendered.template_sha256.as_str(),"template_status":"proposed_not_user_approved","asset_manifest_sha256":rendered.asset_manifest_sha256.as_str(),"fonts":fonts,"font_licenses":licenses,"font_delivery_observed":rendered.font_delivery_observed,"front_layout":format!("{:?}",rendered.front),"back_layout":format!("{:?}",rendered.back),"limitations":LIMITATIONS,"renderer_limitations":rendered.limitations.iter().map(|x|format!("{x:?}")).collect::<Vec<_>>(),"cancellation_supported":false,"timeout_supported":false,"source_pts_authenticated":false,"frame_rights_verified":false,"source_refs":source_refs});
    if let Some(page) = page {
        provenance["page"] = json!(page);
        provenance["page_layout"] =
            json!(format!("{:?}", page_observation.expect("Measured page")));
        provenance["card_role"] =
            serde_json::to_value(context.expect("Role context").role).map_err(assets::error)?;
    }
    Ok(PreparedReviewPng {
        bytes: rendered.png,
        provenance,
    })
}
fn map_failure(failure: RenderFailure<ApiError>) -> ApiError {
    match failure {
        RenderFailure::PngInspection { stage, cause } => {
            // Stage is secondary diagnostics; preserve every original cause field/allocation.
            let stage = match stage {
                PngInspectionStage::Frame => "frame",
                PngInspectionStage::Output => "output",
            };
            let _diagnostic = std::io::Write::write_fmt(
                &mut std::io::stderr().lock(),
                format_args!("card_review_png_inspection_stage={stage}\n"),
            );
            cause
        }
        RenderFailure::Rendering(error) => {
            let code = match &error {
                RenderError::ResourceLimit => ErrorCode::ResourceExhausted,
                RenderError::FramePinMismatch => ErrorCode::RevisionConflict,
                RenderError::LayoutOverflow(_) | RenderError::LayoutUnavailable(_) => {
                    ErrorCode::NeedsInput
                }
                RenderError::InvalidInput(_) => ErrorCode::InvalidArgument,
                _ => ErrorCode::EngineFailed,
            };
            let mut cause = ApiError::new(code, format!("Card review render failed: {error:?}"));
            cause.details = json!({"state":"no_review_artifact","quality_approved":false,"limitations":LIMITATIONS});
            cause
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parser_error_fields_and_heap_identity_survive_root_mapping() {
        for stage in [PngInspectionStage::Frame, PngInspectionStage::Output] {
            for code in [
                ErrorCode::InvalidArgument,
                ErrorCode::Unsupported,
                ErrorCode::ResourceExhausted,
            ] {
                let mut error = ApiError::new(code, "original parser allocation");
                error.retryable = true;
                error.details = json!({"first_cause":"original"});
                error.remediation = "original remedy".into();
                error.request_id = "original request".into();
                let ptr = error.message.as_ptr();
                let result = map_failure(RenderFailure::PngInspection {
                    stage,
                    cause: error,
                });
                assert_eq!(result.code, code);
                assert_eq!(result.message.as_ptr(), ptr);
                assert!(result.retryable);
                assert_eq!(result.details, json!({"first_cause":"original"}));
                assert_eq!(result.remediation, "original remedy");
                assert_eq!(result.request_id, "original request");
            }
        }
    }
}
