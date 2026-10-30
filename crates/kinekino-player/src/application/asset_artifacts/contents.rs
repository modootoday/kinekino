use super::*;
use crate::{
    application::{
        asset_bundle::*,
        asset_projects::{AssetPublication, publication_sha256},
        review_source::MutationControl,
    },
    domain::asset_generation::*,
};

impl AssetArtifactPublication {
    /// Cross-checks metadata against pinned bytes without granting source/Jobs authority.
    pub fn validate_contents(
        &self,
        project_bytes: &[u8],
        manifest_bytes: &[u8],
        control: &dyn MutationControl,
    ) -> Result<AssetBundleManifest, ApiError> {
        let project: AssetPublication =
            serde_json::from_slice(project_bytes).map_err(|_| corrupt())?;
        let manifest: AssetBundleManifest =
            serde_json::from_slice(manifest_bytes).map_err(|_| corrupt())?;
        let project_file = self.files.first().ok_or_else(corrupt)?;
        if project.schema_version != "asset-project-publication/1"
            || project.publication_revision.0 == 0
            || project_file.bytes.0 != project_bytes.len() as u64
            || publication_sha256(&project)? != project_file.sha256
            || manifest.schema_version != "asset-bundle/1"
            || manifest.state != "review_artifact"
            || manifest.quality_approved
            || manifest.colorimetry_verified
            || manifest.project_file != *project_file
            || manifest.plan.project.document_id != project.document.document_id
            || manifest.plan.project.publication_revision != project.publication_revision
            || manifest.plan.project.publication_sha256 != project_file.sha256
        {
            return Err(corrupt());
        }
        let request = AssetBundleRequest {
            project: manifest.plan.project.clone(),
            missing_frames: manifest.plan.missing_frame_policy,
            review_artifact_only: true,
        };
        let expected = plan_publication(&project, &request, control)?;
        if manifest.plan != expected
            || manifest.layers.len() != expected.units.len()
            || self.files.len() != manifest.layers.len() * 3 + 2
        {
            return Err(corrupt());
        }
        for (index, layer) in manifest.layers.iter().enumerate() {
            control.checkpoint()?;
            let unit = &expected.units[index];
            let request = project
                .document
                .frame_request(&unit.frame_id, &unit.layer_id)
                .map_err(|_| corrupt())?;
            let geometry = geometry(
                request.transform,
                request.image.width,
                request.image.height,
                layer.geometry.source_bounds,
            )
            .map_err(|_| corrupt())?;
            let conversion = match request.transform.alpha_mode {
                AlphaMode::Straight => AssetPngConversion::None,
                AlphaMode::Premultiplied => AssetPngConversion::UnassociateSrgb8WithQuantization,
            };
            let files = &self.files[1 + index * 3..1 + (index + 1) * 3];
            let stem = format!("frame-{:06}-unit-{index:06}", unit.frame_position.0);
            if layer.unit != *unit
                || layer.request != request
                || layer.geometry != geometry
                || layer.raw_alpha_mode != request.transform.alpha_mode
                || layer.png_alpha_mode != AlphaMode::Straight
                || layer.png_conversion != conversion
                || layer.color != request.transform.color
                || layer.unreviewed_mask_warning
                    != matches!(
                        request.review,
                        AssetMaskReview::UnreviewedAcknowledged { .. }
                    )
                || layer.rgba8 != files[0]
                || layer.rgba_png != files[1]
                || layer.mask_png != files[2]
                || layer.rgba8.name != format!("{stem}.rgba8")
                || layer.rgba_png.name != format!("{stem}.png")
                || layer.mask_png.name != format!("{stem}.mask.png")
                || layer.rgba8.bytes.0
                    != u64::from(geometry.output_width) * u64::from(geometry.output_height) * 4
            {
                return Err(corrupt());
            }
        }
        Ok(manifest)
    }
}
