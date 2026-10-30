//! Original publication lease and deterministic sequence plan; values do not admit pixels.
mod output;
mod sources;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        asset_generation::{self, AssetInputAdmission, AssetRasterPort, RenderedAssetFrame},
        asset_projects::AssetPublication,
        review_source::MutationControl,
    },
    domain::{
        api_values::DecimalU64,
        asset_generation::*,
        identifiers::{valid_id, valid_sha256},
    },
};
pub use output::*;
use serde::{Deserialize, Serialize};
pub use sources::*;

pub const MAX_ASSET_RENDER_UNITS: usize = 4096;
pub const BUILD_OPERATION: &str = "production.build_assets";
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedAssetProjectPin {
    pub document_id: String,
    pub publication_revision: DecimalU64,
    pub publication_sha256: String,
}
impl SavedAssetProjectPin {
    pub fn validate(&self) -> Result<(), ApiError> {
        if !valid_id(&self.document_id)
            || self.publication_revision.0 == 0
            || !valid_sha256(&self.publication_sha256)
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Exact saved asset publication identity required",
            ));
        }
        Ok(())
    }
}
/// Host admission owns the original publication/ancestor handles. JSON cannot mint it.
pub trait AssetProjectLease: Send + Sync {
    fn publication(&self) -> &AssetPublication;
    fn publication_sha256(&self) -> &str;
}
pub trait AssetProjectAdmission {
    type Project: AssetProjectLease;
    fn open_project(
        &self,
        pin: &SavedAssetProjectPin,
        control: &dyn MutationControl,
    ) -> Result<Self::Project, ApiError>;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingAssetFramePolicy {
    Reject,
    PreserveGap,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetBundleRequest {
    pub project: SavedAssetProjectPin,
    pub missing_frames: MissingAssetFramePolicy,
    pub review_artifact_only: bool,
}
impl AssetBundleRequest {
    pub fn schema() -> serde_json::Value {
        serde_json::json!({"type":"object","additionalProperties":false,"required":["project","missing_frames","review_artifact_only"],"properties":{"project":{"type":"object","additionalProperties":false,"required":["document_id","publication_revision","publication_sha256"],"properties":{"document_id":{"type":"string","pattern":"^[A-Za-z0-9_.-]{1,80}$"},"publication_revision":{"type":"string","pattern":"^[1-9][0-9]*$"},"publication_sha256":{"type":"string","pattern":"^[a-f0-9]{64}$"}}},"missing_frames":{"enum":["reject","preserve_gap"]},"review_artifact_only":{"const":true}}})
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetPlannedFrame {
    pub position: DecimalU64,
    pub frame_id: String,
    pub source: AssetFramePosition,
    pub missing_reason: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetRenderUnit {
    pub frame_position: DecimalU64,
    pub frame_id: String,
    pub layer_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetBundlePlan {
    pub schema_version: String,
    pub project: SavedAssetProjectPin,
    pub order_basis: AssetOrderBasis,
    pub missing_frame_policy: MissingAssetFramePolicy,
    pub frames: Vec<AssetPlannedFrame>,
    pub units: Vec<AssetRenderUnit>,
}
/// Clones share one original lease identity; JSON cannot create or replace it.
pub struct CapturedAssetProject<L: AssetProjectLease> {
    inner: std::sync::Arc<CapturedAssetProjectInner<L>>,
}
struct CapturedAssetProjectInner<L: AssetProjectLease> {
    lease: std::sync::Arc<L>,
    plan: AssetBundlePlan,
}
impl<L: AssetProjectLease> Clone for CapturedAssetProject<L> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
pub fn capture<P: AssetProjectAdmission>(
    ports: &P,
    request: &AssetBundleRequest,
    control: &dyn MutationControl,
) -> Result<CapturedAssetProject<P::Project>, ApiError> {
    request.project.validate()?;
    if !request.review_artifact_only {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Asset outputs require explicit unapproved review-artifact mode",
        ));
    }
    control.checkpoint()?;
    let lease = ports.open_project(&request.project, control)?;
    let publication = lease.publication();
    if publication.schema_version != "asset-project-publication/1"
        || publication.document.document_id != request.project.document_id
        || publication.publication_revision != request.project.publication_revision
        || lease.publication_sha256() != request.project.publication_sha256
        || crate::application::asset_projects::publication_sha256(publication)?
            != request.project.publication_sha256
    {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Original saved asset publication differs",
        ));
    }
    let plan = plan_publication(publication, request, control)?;
    Ok(CapturedAssetProject {
        inner: std::sync::Arc::new(CapturedAssetProjectInner {
            lease: std::sync::Arc::new(lease),
            plan,
        }),
    })
}
/// Deterministic metadata planning only; never creates an admitted source lease.
pub(crate) fn plan_publication(
    publication: &AssetPublication,
    request: &AssetBundleRequest,
    control: &dyn MutationControl,
) -> Result<AssetBundlePlan, ApiError> {
    request.project.validate()?;
    let document = &publication.document;
    document
        .validate()
        .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
    let mut frames = Vec::new();
    let mut units = Vec::new();
    for (index, frame) in document.frames.iter().enumerate() {
        control.checkpoint()?;
        let position = DecimalU64(index as u64);
        let missing_reason = match frame {
            AssetFrameEntry::Missing { reason, .. } => {
                if request.missing_frames == MissingAssetFramePolicy::Reject {
                    return Err(ApiError::new(
                        ErrorCode::NeedsInput,
                        "Asset sequence has missing frames; select explicit gap preservation or repair",
                    ));
                }
                Some(reason.clone())
            }
            AssetFrameEntry::Available { .. } => None,
        };
        frames.push(AssetPlannedFrame {
            position,
            frame_id: frame.frame_id().into(),
            source: document.sources[frame.frame_id()].position.clone(),
            missing_reason: missing_reason.clone(),
        });
        if missing_reason.is_none() {
            for layer in document.layers.iter().filter(|layer| layer.visible) {
                if units.len() >= MAX_ASSET_RENDER_UNITS {
                    return Err(ApiError::new(
                        ErrorCode::ResourceExhausted,
                        "Asset frame/layer render unit budget exceeded",
                    ));
                }
                units.push(AssetRenderUnit {
                    frame_position: position,
                    frame_id: frame.frame_id().into(),
                    layer_id: layer.layer_id.clone(),
                });
            }
        }
    }
    if units.is_empty() {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Asset bundle needs an available frame and visible layer",
        ));
    }
    let plan = AssetBundlePlan {
        schema_version: "asset-bundle-plan/2".into(),
        project: request.project.clone(),
        order_basis: document.order_basis,
        missing_frame_policy: request.missing_frames,
        frames,
        units,
    };
    control.checkpoint()?;
    Ok(plan)
}
pub struct RenderedAssetBundleFrame<L: AssetProjectLease, P: AssetInputAdmission> {
    project: CapturedAssetProject<L>,
    unit_index: usize,
    rendered: RenderedAssetFrame<P::Image, P::Mask, P::Review>,
}
impl<L: AssetProjectLease> CapturedAssetProject<L> {
    pub fn plan(&self) -> &AssetBundlePlan {
        &self.inner.plan
    }
    pub fn publication(&self) -> &AssetPublication {
        self.inner.lease.publication()
    }
    pub fn render_frame<P: AssetInputAdmission>(
        &self,
        index: usize,
        inputs: &P,
        raster: &dyn AssetRasterPort,
        control: &dyn MutationControl,
    ) -> Result<RenderedAssetBundleFrame<L, P>, ApiError> {
        control.checkpoint()?;
        let unit = self.inner.plan.units.get(index).ok_or_else(|| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Asset render unit index out of bounds",
            )
        })?;
        let request = self
            .publication()
            .document
            .frame_request(&unit.frame_id, &unit.layer_id)
            .map_err(|_| {
                ApiError::new(
                    ErrorCode::SourceExpired,
                    "Original asset frame/layer selection differs",
                )
            })?;
        let rendered =
            asset_generation::prepare(&request, inputs, control)?.render(raster, control)?;
        Ok(RenderedAssetBundleFrame {
            project: self.clone(),
            unit_index: index,
            rendered,
        })
    }
}
impl<L: AssetProjectLease, P: AssetInputAdmission> RenderedAssetBundleFrame<L, P> {
    pub fn retained_original_png_bytes(&self) -> u64 {
        self.rendered.retained_original_png_bytes()
    }
    pub fn plan(&self) -> &AssetBundlePlan {
        self.project.plan()
    }
    pub fn unit(&self) -> &AssetRenderUnit {
        &self.project.inner.plan.units[self.unit_index]
    }
    pub fn pixels(&self) -> &asset_generation::AssetRasterOutput {
        self.rendered.pixels()
    }
    pub fn request(&self) -> &AssetFrameRequest {
        self.rendered.request()
    }
    pub fn unreviewed_mask_warning(&self) -> bool {
        self.rendered.unreviewed_mask_warning()
    }
}
