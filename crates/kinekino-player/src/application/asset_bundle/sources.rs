//! Exact original PNG export owns original project/image/mask leases; no review authority.
mod validation;
use super::*;
use asset_generation::{AssetImageLease, AssetMaskLease};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetSourceBinding {
    pub frame_id: String,
    pub image: AssetImagePin,
    pub mask: AssetMaskPin,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetSourceManifest {
    pub schema_version: String,
    pub project: SavedAssetProjectPin,
    pub project_file: AssetBundleFile,
    pub bindings: Vec<AssetSourceBinding>,
    pub originals: Vec<AssetBundleFile>,
    pub missing_frames: Vec<String>,
    pub quality_approved: bool,
    pub review_admitted: bool,
}
enum Original<I, M> {
    Image(I),
    Mask(M),
}
impl<I: AssetImageLease, M: AssetMaskLease> Original<I, M> {
    fn bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Image(i) => i.original_png(),
            Self::Mask(m) => m.original_png(),
        }
    }
}
pub struct PreparedAssetSources<L: AssetProjectLease, P: AssetInputAdmission> {
    _project: std::sync::Arc<L>,
    originals: Vec<Original<P::Image, P::Mask>>,
    file_sources: Vec<usize>,
    manifest: AssetSourceManifest,
    project_bytes: Vec<u8>,
    manifest_bytes: Vec<u8>,
    manifest_file: AssetBundleFile,
}
fn descriptor(name: String, bytes: &[u8], media: &str) -> AssetBundleFile {
    AssetBundleFile {
        name,
        sha256: format!("{:x}", Sha256::digest(bytes)),
        bytes: DecimalU64(bytes.len() as u64),
        media_type: media.into(),
    }
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::SourceExpired,
        "Original asset source PNG bytes/pin differ",
    )
}
fn add(total: &mut u64, bytes: u64) -> Result<(), ApiError> {
    *total = total
        .checked_add(bytes)
        .filter(|n| *n <= MAX_ASSET_RETAINED_BYTES)
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::ResourceExhausted,
                "Asset original source archive budget exceeded",
            )
        })?;
    Ok(())
}
impl<L: AssetProjectLease> CapturedAssetProject<L> {
    pub fn capture_sources<P: AssetInputAdmission>(
        &self,
        inputs: &P,
        control: &dyn MutationControl,
    ) -> Result<PreparedAssetSources<L, P>, ApiError> {
        CapturedAssetSourceProject {
            lease: self.inner.lease.clone(),
            pin: self.plan().project.clone(),
        }
        .capture_sources(inputs, control)
    }
}
/// Storage-only original lease; cannot be passed to a renderer/output builder.
pub struct CapturedAssetSourceProject<L: AssetProjectLease> {
    lease: std::sync::Arc<L>,
    pin: SavedAssetProjectPin,
}
pub fn capture_source_project<P: AssetProjectAdmission>(
    ports: &P,
    pin: &SavedAssetProjectPin,
    missing: MissingAssetFramePolicy,
    control: &dyn MutationControl,
) -> Result<CapturedAssetSourceProject<P::Project>, ApiError> {
    pin.validate()?;
    control.checkpoint()?;
    let lease = ports.open_project(pin, control)?;
    let publication = lease.publication();
    if publication.schema_version != "asset-project-publication/1"
        || publication.document.document_id != pin.document_id
        || publication.publication_revision != pin.publication_revision
        || lease.publication_sha256() != pin.publication_sha256
        || crate::application::asset_projects::publication_sha256(publication)?
            != pin.publication_sha256
    {
        return Err(invalid());
    }
    publication.document.validate().map_err(|_| invalid())?;
    if missing == MissingAssetFramePolicy::Reject
        && publication
            .document
            .frames
            .iter()
            .any(|frame| matches!(frame, AssetFrameEntry::Missing { .. }))
    {
        return Err(ApiError::new(
            ErrorCode::NeedsInput,
            "Source archive has missing frames; explicit gap preservation required",
        ));
    }
    control.checkpoint()?;
    Ok(CapturedAssetSourceProject {
        lease: std::sync::Arc::new(lease),
        pin: pin.clone(),
    })
}
impl<L: AssetProjectLease> CapturedAssetSourceProject<L> {
    fn publication(&self) -> &AssetPublication {
        self.lease.publication()
    }
    pub fn capture_sources<P: AssetInputAdmission>(
        &self,
        inputs: &P,
        control: &dyn MutationControl,
    ) -> Result<PreparedAssetSources<L, P>, ApiError> {
        let project_bytes = serde_json::to_vec(self.publication()).map_err(|_| invalid())?;
        let project_file = descriptor("project.json".into(), &project_bytes, "application/json");
        if project_file.sha256 != self.pin.publication_sha256 {
            return Err(invalid());
        }
        let mut originals = Vec::new();
        let mut file_sources = Vec::new();
        let mut admitted = std::collections::BTreeSet::new();
        let mut files = Vec::new();
        let mut seen = BTreeMap::new();
        let mut bindings = Vec::new();
        let mut missing = Vec::new();
        let mut retained = project_bytes.len() as u64;
        for frame in &self.publication().document.frames {
            control.checkpoint()?;
            let AssetFrameEntry::Available { frame_id, input } = frame else {
                missing.push(frame.frame_id().to_owned());
                continue;
            };
            for mask in [false, true] {
                let pin = if mask {
                    &input.mask.blob
                } else {
                    &input.image.blob
                };
                let identity = if mask {
                    serde_json::to_vec(&input.mask)
                } else {
                    serde_json::to_vec(&input.image)
                }
                .map_err(|_| invalid())?;
                if !admitted.insert((mask, identity)) {
                    continue;
                }
                let original = if mask {
                    let lease = inputs.open_mask(&input.mask, control)?;
                    if lease.pin() != &input.mask {
                        return Err(invalid());
                    }
                    add(&mut retained, lease.coverage().len() as u64)?;
                    Original::Mask(lease)
                } else {
                    let lease = inputs.open_image(&input.image, control)?;
                    if lease.pin() != &input.image {
                        return Err(invalid());
                    }
                    add(&mut retained, lease.rgba().len() as u64)?;
                    Original::Image(lease)
                };
                let bytes = original.bytes().ok_or_else(|| {
                    ApiError::new(
                        ErrorCode::Unsupported,
                        "Source lease does not expose exact original PNG bytes",
                    )
                })?;
                if bytes.is_empty()
                    || bytes.len() as u64 > crate::domain::frame_capture::MAX_FRAME_PNG_BYTES
                {
                    return Err(invalid());
                }
                let file = descriptor(format!("source-{}.png", pin.sha256), bytes, "image/png");
                if file.sha256 != pin.sha256 {
                    return Err(invalid());
                }
                add(&mut retained, file.bytes.0)?;
                if !seen.contains_key(&pin.sha256) {
                    seen.insert(pin.sha256.clone(), files.len());
                    files.push(file);
                    file_sources.push(originals.len());
                }
                originals.push(original);
            }
            bindings.push(AssetSourceBinding {
                frame_id: frame_id.clone(),
                image: input.image.clone(),
                mask: input.mask.clone(),
            });
        }
        let manifest = AssetSourceManifest {
            schema_version: "asset-sources/1".into(),
            project: self.pin.clone(),
            project_file,
            bindings,
            originals: files,
            missing_frames: missing,
            quality_approved: false,
            review_admitted: false,
        };
        let manifest_bytes = serde_json::to_vec(&manifest).map_err(|_| invalid())?;
        manifest.validate_project(&project_bytes, manifest_bytes.len() as u64, control)?;
        add(&mut retained, manifest_bytes.len() as u64)?;
        let manifest_file = descriptor("sources.json".into(), &manifest_bytes, "application/json");
        control.checkpoint()?;
        Ok(PreparedAssetSources {
            _project: self.lease.clone(),
            originals,
            file_sources,
            manifest,
            project_bytes,
            manifest_bytes,
            manifest_file,
        })
    }
}
impl<L: AssetProjectLease, P: AssetInputAdmission> PreparedAssetSources<L, P> {
    pub fn manifest(&self) -> &AssetSourceManifest {
        &self.manifest
    }
    pub fn files(&self) -> Vec<AssetBundleFileRef<'_>> {
        let mut files = vec![AssetBundleFileRef {
            descriptor: &self.manifest.project_file,
            contents: &self.project_bytes,
        }];
        for (index, file) in self.file_sources.iter().zip(&self.manifest.originals) {
            let source = &self.originals[*index];
            files.push(AssetBundleFileRef {
                descriptor: file,
                contents: source
                    .bytes()
                    .expect("original bytes retained by immutable lease"),
            });
        }
        files.push(AssetBundleFileRef {
            descriptor: &self.manifest_file,
            contents: &self.manifest_bytes,
        });
        files
    }
}
