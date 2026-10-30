#[cfg(test)]
mod tests;
use super::{assets, frame, render};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        card_bundle::{
            BundleFrameObservation, BundleTemplatePin, CardBundleAssets, CardBundleRasterizer,
            PreparedCardBundle, RenderedBundleCard, SelectedCardFrame, card_output_name,
        },
        card_projects::publication_sha256,
        proposals::card_export::{LIMITATIONS, TemplateSelection},
    },
    assembly::card_export::product_template,
    domain::{api_values::DecimalU64, card_document::CardCanvas, frame_capture::MAX_FRAME_PIXELS},
    infrastructure::{immutable_storage_paths, meari::product_styles::prepare_product_styles},
};
use std::{
    fs,
    path::{Component, Path, Prefix},
};

pub struct CardBundleFiles;
impl crate::application::card_bundle::CardTemplateCatalogPort for CardBundleFiles {
    fn inspect_templates(
        &self,
    ) -> Result<crate::application::card_bundle::CardTemplateCatalog, ApiError> {
        let common = prepare_product_styles().map_err(assets::error)?;
        let fonts = assets::font_metadata(&common.assets)?;
        let templates = Self::proposed_canvases()
            .iter()
            .map(|canvas| verified_template(canvas, &fonts))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(crate::application::card_bundle::CardTemplateCatalog {
            schema_version: "1".into(),
            backend_available: true,
            templates,
            review_artifact_only: true,
            quality_approved: false,
            platform_requirements_verified: false,
        })
    }
}
impl CardBundleFiles {
    /// Explicit proposed layouts. This catalog confers no platform or user approval.
    pub fn proposed_canvases() -> Vec<CardCanvas> {
        let mut canvases = vec![Self::current_proposed_canvas()];
        for id in crate::assembly::card_export::canvas_templates::IDS {
            let layout = crate::assembly::card_export::canvas_templates::layout(id)
                .expect("Static catalog layout");
            canvases.push(CardCanvas {
                width: layout.canvas.width,
                height: layout.canvas.height,
                template_id: id.into(),
                template_version: crate::assembly::card_export::canvas_templates::VERSION.into(),
                template_sha256: render::canvas_declaration_sha256(),
            });
        }
        for id in crate::assembly::card_export::blank_templates::IDS {
            let layout = crate::assembly::card_export::blank_templates::layout(id)
                .expect("Static text template");
            canvases.push(CardCanvas {
                width: layout.canvas.width,
                height: layout.canvas.height,
                template_id: id.into(),
                template_version: crate::assembly::card_export::blank_templates::VERSION.into(),
                template_sha256: render::blank_declaration_sha256(),
            });
        }
        for id in crate::assembly::card_export::role_templates::IDS {
            let layout = crate::assembly::card_export::role_templates::layout(
                id,
                crate::domain::card_document::CardRole::Body,
            )
            .expect("Static role layout");
            canvases.push(CardCanvas {
                width: layout.canvas.width,
                height: layout.canvas.height,
                template_id: id.into(),
                template_version: crate::assembly::card_export::role_templates::VERSION.into(),
                template_sha256: render::role_declaration_sha256(),
            });
        }
        canvases
    }
    /// Proposed template identity, never a user quality/template approval.
    pub fn current_proposed_canvas() -> CardCanvas {
        let proposal = product_template::png1_proposal();
        CardCanvas {
            width: proposal.layout.canvas.width,
            height: proposal.layout.canvas.height,
            template_id: proposal.id.into(),
            template_version: proposal.version.into(),
            template_sha256: render::declaration_sha256(),
        }
    }
}
fn error(code: ErrorCode, message: &str) -> ApiError {
    ApiError::new(code, message)
}
fn selection(canvas: &CardCanvas) -> TemplateSelection {
    TemplateSelection {
        id: canvas.template_id.clone(),
        version: canvas.template_version.clone(),
        declaration_sha256: canvas.template_sha256.clone(),
    }
}
fn verified_template(
    canvas: &CardCanvas,
    fonts: &[assets::FontMetadata],
) -> Result<BundleTemplatePin, ApiError> {
    let layout = render::resolve_template(&selection(canvas))?;
    if (canvas.width, canvas.height) != (layout.canvas.width, layout.canvas.height) {
        return Err(error(
            ErrorCode::Unsupported,
            "Selected canvas is not supported by this explicitly pinned template",
        ));
    }
    Ok(BundleTemplatePin {
        role_layouts_and_pages: render::role_pages(&selection(canvas)),
        frame_required: render::frame_required(&selection(canvas)),
        canvas: canvas.clone(),
        common_font_sha256: fonts.iter().map(|p| p.sha256.into()).collect(),
        license_refs: fonts.iter().map(|p| p.license_hint.into()).collect(),
        limitations: LIMITATIONS.iter().map(|s| (*s).into()).collect(),
    })
}
impl CardBundleAssets for CardBundleFiles {
    fn verify_template(&self, canvas: &CardCanvas) -> Result<BundleTemplatePin, ApiError> {
        let common = prepare_product_styles().map_err(assets::error)?;
        let fonts = assets::font_metadata(&common.assets)?;
        verified_template(canvas, &fonts)
    }
    fn inspect_frame(
        &self,
        selected: &SelectedCardFrame,
    ) -> Result<BundleFrameObservation, ApiError> {
        let bytes = frame::read(&selected.path, &selected.png_sha256)?;
        let (width, height) = crate::infrastructure::snapshot_export::inspect_static_png(&bytes)?;
        if u64::from(width) * u64::from(height) > MAX_FRAME_PIXELS {
            return Err(error(
                ErrorCode::ResourceExhausted,
                "Card frame pixel budget exceeded",
            ));
        }
        Ok(BundleFrameObservation {
            sha256: selected.png_sha256.clone(),
            bytes: DecimalU64(bytes.len() as u64),
            width,
            height,
        })
    }
    fn verify_new_output_directory(&self, path: &str) -> Result<(), ApiError> {
        let path = Path::new(path);
        if !path.is_absolute()
            || !matches!(path.components().next(), Some(Component::Prefix(p)) if matches!(p.kind(), Prefix::Disk(_)))
            || path.components().any(|c| matches!(c, Component::ParentDir))
            || path
                .to_str()
                .is_none_or(|s| s.chars().any(char::is_control))
        {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Explicit ordinary absolute local output directory required",
            ));
        }
        let name = path.file_name().and_then(|s| s.to_str()).ok_or_else(|| {
            error(
                ErrorCode::InvalidArgument,
                "Named output directory required",
            )
        })?;
        if name.contains(':') || name.ends_with([' ', '.']) {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Ambiguous output directory name",
            ));
        }
        let parent = path
            .parent()
            .ok_or_else(|| error(ErrorCode::InvalidArgument, "Output parent required"))?;
        let (_canonical, _guards) = immutable_storage_paths::root(parent)?;
        match fs::symlink_metadata(path) {
            Ok(_) => Err(error(
                ErrorCode::RevisionConflict,
                "Card bundle output already exists; no overwrite",
            )),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(kinekino_core::file_publication::io_error(e)),
        }
    }
}
impl CardBundleRasterizer for CardBundleFiles {
    fn read_frame_png(&self, selected: &SelectedCardFrame) -> Result<Vec<u8>, ApiError> {
        let bytes = frame::read(&selected.path, &selected.png_sha256)?;
        crate::infrastructure::snapshot_export::inspect_static_png(&bytes)?;
        Ok(bytes)
    }
    fn render_card(
        &self,
        bundle: &PreparedCardBundle,
        position: DecimalU64,
    ) -> Result<RenderedBundleCard, ApiError> {
        if bundle.schema_version != "1"
            || !bundle.review_artifact_only
            || bundle.publication.schema_version != "1"
            || bundle.publication.document.document_id != bundle.project.document_id
            || bundle.publication.publication_revision != bundle.project.publication_revision
            || publication_sha256(&bundle.publication)? != bundle.project.publication_sha256
        {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Prepared bundle differs from its immutable project pin",
            ));
        }
        let document = &bundle.publication.document;
        document
            .validate()
            .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid prepared card document"))?;
        if bundle.cards.len() != document.layout.cards.len() {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Prepared card count differs from saved project",
            ));
        }
        let actual_template = self.verify_template(&document.layout.canvas)?;
        if serde_json::to_vec(&actual_template).map_err(assets::error)?
            != serde_json::to_vec(&bundle.template).map_err(assets::error)?
        {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Prepared template or common font pins changed",
            ));
        }
        let index = usize::try_from(position.0)
            .map_err(|_| error(ErrorCode::InvalidArgument, "Invalid card position"))?;
        let card = document.layout.cards.get(index).ok_or_else(|| {
            error(
                ErrorCode::InvalidArgument,
                "Card position is outside the saved project",
            )
        })?;
        let prepared = bundle
            .cards
            .get(index)
            .ok_or_else(|| error(ErrorCode::InvalidArgument, "Prepared card position absent"))?;
        if prepared.position != position
            || prepared.card_id != card.card_id
            || prepared.output_name != card_output_name(index, card.role)?
        {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Prepared card selection differs from saved content",
            ));
        }
        let frame_input = match (&card.frame, &prepared.frame, &prepared.observation) {
            (None, None, None) if !actual_template.frame_required => None,
            (Some(pin), Some(frame), Some(observation)) if actual_template.frame_required => {
                if frame.artifact_id != pin.artifact_id || frame.png_sha256 != pin.png_sha256 {
                    return Err(error(
                        ErrorCode::RevisionConflict,
                        "Prepared frame differs from saved pin",
                    ));
                }
                let observed = self.inspect_frame(frame).map_err(|mut failure| {
                    if failure.code == ErrorCode::RevisionConflict {
                        failure.code = ErrorCode::SourceExpired;
                    }
                    failure
                })?;
                if serde_json::to_vec(&observed).map_err(assets::error)?
                    != serde_json::to_vec(observation).map_err(assets::error)?
                {
                    return Err(error(
                        ErrorCode::SourceExpired,
                        "PNG observation changed after admission",
                    ));
                }
                Some((frame.path.as_str(), frame.png_sha256.as_str()))
            }
            _ => {
                return Err(error(
                    ErrorCode::RevisionConflict,
                    "Prepared frame requirement differs from saved template",
                ));
            }
        };
        let output = render::render_project_content(
            kinekino_card_render::domain::CardText {
                front: &card.content.front,
                back: &card.content.back,
            },
            &card.content.source_refs,
            frame_input,
            &selection(&document.layout.canvas),
            None,
            Some(render::CardPageContext {
                role: card.role,
                position: index,
                total: document.layout.cards.len(),
            }),
        )?;
        let mut provenance = output.provenance;
        provenance["card_project"] =
            serde_json::to_value(&bundle.project).map_err(assets::error)?;
        provenance["document_edit_revision"] =
            serde_json::to_value(document.revision).map_err(assets::error)?;
        provenance["card_id"] = serde_json::json!(card.card_id);
        provenance["position"] = serde_json::to_value(position).map_err(assets::error)?;
        provenance["quality_approved"] = serde_json::json!(false);
        Ok(RenderedBundleCard {
            png: output.bytes,
            provenance,
        })
    }
}
