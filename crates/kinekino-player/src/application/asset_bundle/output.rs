//! Output bytes and manifest retain original project and source leases until publication.
use super::*;
use sha2::{Digest, Sha256};
pub const MAX_ASSET_BUNDLE_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_ASSET_RETAINED_BYTES: u64 = 512 * 1024 * 1024;
pub struct AssetEncodedBuffers {
    pub rgba_png: Vec<u8>,
    pub mask_png: Vec<u8>,
}
/// Trusted host codec verifies actual PNG round trips; caller JSON supplies no codec.
pub trait AssetLayerEncoder {
    fn encode(
        &self,
        pixels: &asset_generation::AssetRasterOutput,
        control: &dyn MutationControl,
    ) -> Result<AssetEncodedBuffers, ApiError>;
}
pub struct EncodedAssetBundleFrame<L: AssetProjectLease, P: AssetInputAdmission> {
    rendered: RenderedAssetBundleFrame<L, P>,
    encoded: AssetEncodedBuffers,
}
impl<L: AssetProjectLease, P: AssetInputAdmission> RenderedAssetBundleFrame<L, P> {
    pub fn encode(
        self,
        encoder: &dyn AssetLayerEncoder,
        control: &dyn MutationControl,
    ) -> Result<EncodedAssetBundleFrame<L, P>, ApiError> {
        control.checkpoint()?;
        let encoded = encoder.encode(self.pixels(), control)?;
        for bytes in [&encoded.rgba_png, &encoded.mask_png] {
            if bytes.is_empty()
                || bytes.len() as u64 > crate::domain::frame_capture::MAX_FRAME_PNG_BYTES
            {
                return Err(error(
                    ErrorCode::ResourceExhausted,
                    "Asset encoded PNG size limit exceeded",
                ));
            }
        }
        control.checkpoint()?;
        Ok(EncodedAssetBundleFrame {
            rendered: self,
            encoded,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetBundleFile {
    pub name: String,
    pub sha256: String,
    pub bytes: DecimalU64,
    pub media_type: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetPngConversion {
    None,
    UnassociateSrgb8WithQuantization,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetLayerOutput {
    pub unit: AssetRenderUnit,
    pub request: AssetFrameRequest,
    pub geometry: AssetGeometry,
    pub raw_alpha_mode: AlphaMode,
    pub png_alpha_mode: AlphaMode,
    pub png_conversion: AssetPngConversion,
    pub color: AssetColor,
    pub unreviewed_mask_warning: bool,
    pub rgba8: AssetBundleFile,
    pub rgba_png: AssetBundleFile,
    pub mask_png: AssetBundleFile,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetBundleManifest {
    pub schema_version: String,
    pub plan: AssetBundlePlan,
    pub project_file: AssetBundleFile,
    pub layers: Vec<AssetLayerOutput>,
    pub state: String,
    pub quality_approved: bool,
    pub colorimetry_verified: bool,
}
pub struct AssetBundleOutputBuilder<L: AssetProjectLease, P: AssetInputAdmission> {
    project: CapturedAssetProject<L>,
    frames: Vec<EncodedAssetBundleFrame<L, P>>,
    retained_bytes: u64,
    output_bytes: u64,
}
pub struct PreparedAssetBundleOutput<L: AssetProjectLease, P: AssetInputAdmission> {
    frames: Vec<EncodedAssetBundleFrame<L, P>>,
    manifest: AssetBundleManifest,
    project_json: Vec<u8>,
    manifest_json: Vec<u8>,
    manifest_file: AssetBundleFile,
}
pub struct AssetBundleFileRef<'a> {
    pub descriptor: &'a AssetBundleFile,
    pub contents: &'a [u8],
}
fn error(code: ErrorCode, message: &str) -> ApiError {
    ApiError::new(code, message)
}
fn descriptor(name: String, bytes: &[u8], media_type: &str) -> AssetBundleFile {
    AssetBundleFile {
        name,
        sha256: format!("{:x}", Sha256::digest(bytes)),
        bytes: DecimalU64(bytes.len() as u64),
        media_type: media_type.into(),
    }
}
fn add(total: u64, bytes: u64, limit: u64) -> Result<u64, ApiError> {
    total
        .checked_add(bytes)
        .filter(|sum| *sum <= limit)
        .ok_or_else(|| {
            error(
                ErrorCode::ResourceExhausted,
                "Asset bundle output/retained memory budget exceeded",
            )
        })
}
impl<L: AssetProjectLease, P: AssetInputAdmission> AssetBundleOutputBuilder<L, P> {
    pub fn new(project: &CapturedAssetProject<L>) -> Self {
        Self {
            project: project.clone(),
            frames: Vec::new(),
            retained_bytes: 0,
            output_bytes: 0,
        }
    }
    pub fn push(
        &mut self,
        frame: EncodedAssetBundleFrame<L, P>,
        control: &dyn MutationControl,
    ) -> Result<(), ApiError> {
        control.checkpoint()?;
        if !std::sync::Arc::ptr_eq(&self.project.inner, &frame.rendered.project.inner)
            || frame.rendered.unit_index != self.frames.len()
        {
            return Err(error(
                ErrorCode::SourceExpired,
                "Asset output must retain the original project and exact planned unit order",
            ));
        }
        let pixels = frame.rendered.pixels();
        let request = frame.rendered.request();
        let encoded_bytes =
            frame.encoded.rgba_png.len() as u64 + frame.encoded.mask_png.len() as u64;
        let output_bytes = add(
            self.output_bytes,
            encoded_bytes + pixels.rgba.len() as u64,
            MAX_ASSET_BUNDLE_BYTES,
        )?;
        let retained = frame.rendered.retained_original_png_bytes()
            + u64::from(request.image.width) * u64::from(request.image.height) * 5
            + pixels.rgba.len() as u64
            + pixels.mask.len() as u64
            + encoded_bytes;
        let retained_bytes = add(self.retained_bytes, retained, MAX_ASSET_RETAINED_BYTES)?;
        self.output_bytes = output_bytes;
        self.retained_bytes = retained_bytes;
        self.frames.push(frame);
        Ok(())
    }
    pub fn finish(
        self,
        control: &dyn MutationControl,
    ) -> Result<PreparedAssetBundleOutput<L, P>, ApiError> {
        control.checkpoint()?;
        if self.frames.len() != self.project.plan().units.len() {
            return Err(error(
                ErrorCode::NeedsInput,
                "Asset bundle has missing rendered units",
            ));
        }
        let project_json = serde_json::to_vec(self.project.publication())
            .map_err(|_| error(ErrorCode::Internal, "Asset publication encoding failed"))?;
        let project_file = descriptor("project.json".into(), &project_json, "application/json");
        if project_file.sha256 != self.project.plan().project.publication_sha256 {
            return Err(error(
                ErrorCode::SourceExpired,
                "Output project bytes differ from original publication pin",
            ));
        }
        let mut layers = Vec::new();
        for (index, frame) in self.frames.iter().enumerate() {
            control.checkpoint()?;
            let stem = format!(
                "frame-{:06}-unit-{index:06}",
                frame.rendered.unit().frame_position.0
            );
            let pixels = frame.rendered.pixels();
            layers.push(AssetLayerOutput {
                unit: frame.rendered.unit().clone(),
                request: frame.rendered.request().clone(),
                geometry: pixels.geometry.clone(),
                raw_alpha_mode: pixels.alpha_mode,
                png_alpha_mode: AlphaMode::Straight,
                png_conversion: match pixels.alpha_mode {
                    AlphaMode::Straight => AssetPngConversion::None,
                    AlphaMode::Premultiplied => {
                        AssetPngConversion::UnassociateSrgb8WithQuantization
                    }
                },
                color: pixels.color,
                unreviewed_mask_warning: frame.rendered.unreviewed_mask_warning(),
                rgba8: descriptor(
                    format!("{stem}.rgba8"),
                    &pixels.rgba,
                    "application/octet-stream",
                ),
                rgba_png: descriptor(format!("{stem}.png"), &frame.encoded.rgba_png, "image/png"),
                mask_png: descriptor(
                    format!("{stem}.mask.png"),
                    &frame.encoded.mask_png,
                    "image/png",
                ),
            });
        }
        let manifest = AssetBundleManifest {
            schema_version: "asset-bundle/1".into(),
            plan: self.project.plan().clone(),
            project_file,
            layers,
            state: "review_artifact".into(),
            quality_approved: false,
            colorimetry_verified: false,
        };
        let manifest_json = serde_json::to_vec(&manifest)
            .map_err(|_| error(ErrorCode::Internal, "Asset manifest encoding failed"))?;
        add(
            add(
                self.output_bytes,
                project_json.len() as u64,
                MAX_ASSET_BUNDLE_BYTES,
            )?,
            manifest_json.len() as u64,
            MAX_ASSET_BUNDLE_BYTES,
        )?;
        let manifest_file = descriptor("manifest.json".into(), &manifest_json, "application/json");
        control.checkpoint()?;
        Ok(PreparedAssetBundleOutput {
            frames: self.frames,
            manifest,
            project_json,
            manifest_json,
            manifest_file,
        })
    }
}
impl<L: AssetProjectLease, P: AssetInputAdmission> PreparedAssetBundleOutput<L, P> {
    pub fn manifest(&self) -> &AssetBundleManifest {
        &self.manifest
    }
    pub fn manifest_file(&self) -> &AssetBundleFile {
        &self.manifest_file
    }
    pub fn files(&self) -> Vec<AssetBundleFileRef<'_>> {
        let mut files = vec![AssetBundleFileRef {
            descriptor: &self.manifest.project_file,
            contents: &self.project_json,
        }];
        for (frame, record) in self.frames.iter().zip(&self.manifest.layers) {
            files.push(AssetBundleFileRef {
                descriptor: &record.rgba8,
                contents: &frame.rendered.pixels().rgba,
            });
            files.push(AssetBundleFileRef {
                descriptor: &record.rgba_png,
                contents: &frame.encoded.rgba_png,
            });
            files.push(AssetBundleFileRef {
                descriptor: &record.mask_png,
                contents: &frame.encoded.mask_png,
            });
        }
        files.push(AssetBundleFileRef {
            descriptor: &self.manifest_file,
            contents: &self.manifest_json,
        });
        files
    }
}
