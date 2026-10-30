//! Existing review-package namespace; a committed custody handle is not a Verified grant.
mod objects;
mod package;
pub mod preparation;
mod reader;
use crate::infrastructure::timeline_evidence::live_contract::ResourceBudget;
use crate::{
    application::review_playback::ReviewInputIdentity,
    domain::review_source::timeline_evidence::TimelineObservations,
    infrastructure::timeline_evidence::live_contract::{ProducerPin, RoleContract, ToolPin},
};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        review_source::MutationControl,
    },
    domain::{
        api_values::DecimalU64,
        model_review::EvidenceAsset,
        review_source::{EvidenceReference, ReviewCase},
    },
    infrastructure::{
        content_digest, immutable_storage_paths as rooted,
        model_review::ReviewFiles,
        timeline_evidence::production::{ProducedArtifact, ProducedTimelineBundle},
    },
};
pub use reader::PublishedTimelineResolver;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};
pub struct TimelinePackageDestination {
    pub root: PathBuf,
    pub max_total_bytes: u64,
    pub excluded_roots: Vec<PathBuf>,
}
/// Constructor-private, Rust-only record of an actual package commit. Not a resolver lease.
pub struct PublishedTimelinePackage {
    root: PathBuf,
    case: ReviewCase,
    base_case: ReviewCase,
    reference: EvidenceReference,
    bundle_sha256: String,
    replayed: bool,
    snapshot: TimelineObservations,
    source_identity: ReviewInputIdentity,
    source_path: String,
    contracts: Vec<RoleContract>,
    tools: Vec<ToolPin>,
    producer: ProducerPin,
    budget: ResourceBudget,
    _guards: Vec<File>,
}
impl PublishedTimelinePackage {
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn case(&self) -> &ReviewCase {
        &self.case
    }
    pub fn base_case(&self) -> &ReviewCase {
        &self.base_case
    }
    pub fn reference(&self) -> &EvidenceReference {
        &self.reference
    }
    pub fn bundle_sha256(&self) -> &str {
        &self.bundle_sha256
    }
    pub fn replayed(&self) -> bool {
        self.replayed
    }
    pub fn check_current(&self) -> Result<(), ApiError> {
        let mut file = rooted::file(
            &self
                .root
                .join("packages")
                .join(format!("{}.json", self.case.package_sha256)),
        )?;
        if content_digest::sha256(&mut file, 524288)?.1 != self.case.package_sha256 {
            return Err(conflict("Committed package digest changed"));
        }
        Ok(())
    }
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ArtifactContent {
    Empty { bytes: u64, sha256: String },
    Asset { asset: EvidenceAsset },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    role: String,
    name: String,
    content: ArtifactContent,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: String,
    base_case: ReviewCase,
    media: crate::domain::review_source::MediaReference,
    base_package: EvidenceAsset,
    bundle_sha256: String,
    artifacts: Vec<Artifact>,
    mapping_revision: DecimalU64,
    contracts: Vec<RoleContract>,
    tools: Vec<ToolPin>,
    producer: ProducerPin,
    budget: ResourceBudget,
    video_stream_index: u32,
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
fn conflict(message: &str) -> ApiError {
    ApiError::new(ErrorCode::RevisionConflict, message)
}
fn io_error(error: std::io::Error) -> ApiError {
    crate::infrastructure::new_file::io_error(error)
}
fn blob(id: &str, body: &[u8], media_type: &str) -> EvidenceAsset {
    EvidenceAsset {
        asset_id: id.into(),
        sha256: format!("{:x}", Sha256::digest(body)),
        bytes: DecimalU64(body.len() as u64),
        media_type: media_type.into(),
    }
}
fn artifact_id(prefix: &str, role: &str, name: &str) -> Result<String, ApiError> {
    let label = format!("{prefix}-{role}-{name}");
    if !label
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    {
        return Err(invalid("Invalid internal artifact label"));
    }
    let id = label.replace('.', "-");
    if !crate::domain::identifiers::valid_id(&id)
        || !crate::domain::review_source::valid_review_object_id(&id)
    {
        return Err(invalid("Artifact ID exceeds shared package/case contract"));
    }
    Ok(id)
}
fn overlaps(a: &Path, b: &Path) -> bool {
    let a = PathBuf::from(a.to_string_lossy().to_lowercase());
    let b = PathBuf::from(b.to_string_lossy().to_lowercase());
    a.starts_with(&b) || b.starts_with(&a)
}
fn reserve_bytes(
    seen: &mut BTreeSet<String>,
    total: &mut u64,
    asset: &EvidenceAsset,
    limit: u64,
) -> Result<(), ApiError> {
    if seen.insert(asset.sha256.clone()) {
        *total = total
            .checked_add(asset.bytes.0)
            .ok_or_else(|| invalid("Package byte budget overflow"))?;
    }
    if *total > limit {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Timeline package byte budget exceeded",
        ));
    }
    Ok(())
}
pub fn publish(
    bundle: &ProducedTimelineBundle<'_, '_>,
    source: &ReviewFiles,
    destination: TimelinePackageDestination,
    control: &dyn MutationControl,
) -> Result<PublishedTimelinePackage, ApiError> {
    if destination.max_total_bytes == 0 || destination.max_total_bytes > 4_294_967_296 {
        return Err(invalid("Explicit bounded package byte budget required"));
    }
    let (canonical, mut guards) = rooted::root(&destination.root)?;
    if overlaps(&canonical, source.canonical_export_root()) {
        return Err(invalid(
            "Publication and original review roots must be disjoint",
        ));
    }
    for root in bundle.protected_roots() {
        if overlaps(&canonical, &root.canonicalize().map_err(io_error)?) {
            return Err(invalid(
                "Publication overlaps producer runtime or tool roots",
            ));
        }
    }
    for excluded in &destination.excluded_roots {
        let (root, held) = rooted::root(excluded)?;
        guards.extend(held);
        if overlaps(&canonical, &root) {
            return Err(invalid("Publication overlaps a protected host root"));
        }
    }
    bundle.within_publication(control, |artifacts: Vec<ProducedArtifact>| {
        let _writer = rooted::lock(&canonical.join(".timeline-package-publish.lock"))?;
        for name in ["objects", "packages", "decisions"] {
            let path = canonical.join(name);
            match std::fs::create_dir(&path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(io_error(e)),
            }
            guards.push(rooted::directory(&path)?);
        }
        let identity = bundle.source_identity();
        let mut exported = source.export_case(&identity.case)?;
        let prefix = format!("live-{}", &bundle.receipt_sha256()[..12]);
        exported
            .package_file
            .seek(SeekFrom::Start(0))
            .map_err(io_error)?;
        let mut base_bytes = Vec::new();
        Read::by_ref(&mut exported.package_file)
            .take(524289)
            .read_to_end(&mut base_bytes)
            .map_err(io_error)?;
        let base_asset = blob(
            &format!("{prefix}-base-package"),
            &base_bytes,
            "application/json",
        );
        if base_asset.sha256 != identity.case.package_sha256 {
            return Err(conflict("Retained base package bytes differ"));
        }
        let mut seen = BTreeSet::new();
        let mut total = 0u64;
        let checkpoint = || bundle.publication_checkpoint(control);
        for (asset, file) in exported.objects {
            reserve_bytes(&mut seen, &mut total, &asset, destination.max_total_bytes)?;
            guards.push(objects::retain_or_copy(
                &canonical.join("objects"),
                &asset,
                file,
                &checkpoint,
            )?);
            bundle.check_publication(control)?;
        }
        let mut added = vec![base_asset.clone()];
        reserve_bytes(
            &mut seen,
            &mut total,
            &base_asset,
            destination.max_total_bytes,
        )?;
        guards.push(objects::retain_or_publish(
            &canonical.join("objects"),
            &base_asset,
            &base_bytes,
            &checkpoint,
        )?);
        let mut declared = Vec::new();
        for artifact in artifacts {
            let content = if artifact.bytes == 0 {
                ArtifactContent::Empty {
                    bytes: 0,
                    sha256: artifact.sha256,
                }
            } else {
                let asset = EvidenceAsset {
                    asset_id: artifact_id(&prefix, &artifact.role, &artifact.name)?,
                    sha256: artifact.sha256,
                    bytes: DecimalU64(artifact.bytes),
                    media_type: artifact.media_type,
                };
                reserve_bytes(&mut seen, &mut total, &asset, destination.max_total_bytes)?;
                guards.push(objects::retain_or_copy(
                    &canonical.join("objects"),
                    &asset,
                    artifact.file,
                    &checkpoint,
                )?);
                added.push(asset.clone());
                ArtifactContent::Asset { asset }
            };
            declared.push(Artifact {
                role: artifact.role,
                name: artifact.name,
                content,
            });
            bundle.check_publication(control)?;
        }
        let manifest = Manifest {
            schema_version: "live_2_1".into(),
            base_case: identity.case.clone(),
            media: identity.media.clone(),
            base_package: base_asset,
            bundle_sha256: bundle.receipt_sha256().to_owned(),
            artifacts: declared,
            mapping_revision: DecimalU64(1),
            contracts: bundle.publication_contracts(),
            tools: bundle.publication_tools(),
            producer: bundle.publication_producer(),
            budget: bundle.publication_budget(),
            video_stream_index: bundle.publication_video_index(),
        };
        let bytes = serde_json::to_vec(&manifest)
            .map_err(|e| ApiError::new(ErrorCode::EngineFailed, e.to_string()))?;
        if bytes.len() > 65536 {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Live manifest exceeds existing 64KiB bound",
            ));
        }
        let manifest_asset = blob(&format!("{prefix}-manifest"), &bytes, "application/json");
        reserve_bytes(
            &mut seen,
            &mut total,
            &manifest_asset,
            destination.max_total_bytes,
        )?;
        guards.push(objects::retain_or_publish(
            &canonical.join("objects"),
            &manifest_asset,
            &bytes,
            &checkpoint,
        )?);
        added.push(manifest_asset.clone());
        let body = package::derive(&exported.package, &identity.case, &prefix, added)?;
        let sha = format!("{:x}", Sha256::digest(&body));
        let path = canonical.join("packages").join(format!("{sha}.json"));
        let case = ReviewCase {
            package_sha256: sha.clone(),
            item_id: identity.case.item_id.clone(),
            source_request_sha256: identity.case.source_request_sha256.clone(),
        };
        let reference = EvidenceReference {
            asset_id: manifest_asset.asset_id.clone(),
            sha256: manifest_asset.sha256.clone(),
        };
        let snapshot = bundle.publication_snapshot(
            case.clone(),
            reference.clone(),
            manifest.mapping_revision,
        )?;
        // Final create-only link is the commit. Never turn a later ACK/cancel into rollback.
        let replayed = if path.try_exists().map_err(io_error)? {
            let mut file = rooted::file(&path)?;
            if content_digest::sha256(&mut file, 524288)? != (body.len() as u64, sha.clone()) {
                return Err(conflict("Existing package differs from exact publication"));
            }
            guards.push(file);
            true
        } else {
            crate::infrastructure::new_file::publish_guarded(
                path.to_str()
                    .ok_or_else(|| invalid("Unicode package path required"))?,
                &body,
                || bundle.check_publication(control),
            )?;
            false
        };
        Ok(PublishedTimelinePackage {
            root: destination.root,
            case,
            base_case: identity.case.clone(),
            reference,
            bundle_sha256: bundle.receipt_sha256().to_owned(),
            replayed,
            snapshot,
            source_identity: identity.clone(),
            source_path: bundle.source_path(),
            contracts: manifest.contracts,
            tools: manifest.tools,
            producer: manifest.producer,
            budget: manifest.budget,
            _guards: guards,
        })
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn artifact_labels_obey_both_package_and_case_reference_contracts() {
        for name in ["INPUT.json", "stdout.raw", "clip.s16le", "engine.bin"] {
            let id = artifact_id("live-123456789abc", "replay-pcm", name).unwrap();
            assert!(crate::domain::identifiers::valid_id(&id));
            assert!(crate::domain::review_source::valid_review_object_id(&id));
        }
        assert!(artifact_id("live", "video", "../outside").is_err());
        assert!(artifact_id(&"a".repeat(80), "video", "input").is_err());
    }
    #[test]
    fn live_manifest_requires_revision_contracts_producer_and_budget_without_defaults() {
        let sha = "a".repeat(64);
        let value = serde_json::json!({"schema_version":"live_2_1","base_case":{"package_sha256":sha,"item_id":"item","source_request_sha256":sha},"media":{"asset_id":"media","sha256":sha},
            "base_package":{"asset_id":"base","sha256":sha,"bytes":"1","media_type":"application/json"},"bundle_sha256":sha,"artifacts":[],"mapping_revision":"1","contracts":[],"tools":[],
            "producer":{"path":"C:/producer.exe","sha256":sha,"bytes":1,"file_identity":"test-id","pid":1},"budget":{"memory_committed_bytes":1,"affinity_mask":1,"max_output_bytes":1,"cpu_threads":1},"video_stream_index":0});
        assert!(serde_json::from_value::<Manifest>(value.clone()).is_ok());
        for key in [
            "mapping_revision",
            "contracts",
            "tools",
            "producer",
            "budget",
            "video_stream_index",
        ] {
            let mut missing = value.clone();
            missing.as_object_mut().unwrap().remove(key);
            assert!(
                serde_json::from_value::<Manifest>(missing).is_err(),
                "missing {key} accepted"
            );
        }
        assert!(
            serde_json::from_value::<
                crate::infrastructure::timeline_evidence::live_contract::LiveExecutionContract,
            >(serde_json::json!({"kind":"video_first_stream_window"}))
            .is_err()
        );
    }
    #[test]
    fn byte_budget_deduplicates_content_and_rejects_new_content_overflow() {
        let a = blob("a", b"1234", "application/octet-stream");
        let b = blob("b", b"5678", "application/octet-stream");
        let mut seen = BTreeSet::new();
        let mut total = 0;
        reserve_bytes(&mut seen, &mut total, &a, 4).unwrap();
        reserve_bytes(&mut seen, &mut total, &a, 4).unwrap();
        assert_eq!(total, 4);
        assert_eq!(
            reserve_bytes(&mut seen, &mut total, &b, 4)
                .unwrap_err()
                .code,
            ErrorCode::ResourceExhausted
        );
        let mut seen = BTreeSet::new();
        let mut total = u64::MAX;
        assert!(reserve_bytes(&mut seen, &mut total, &a, u64::MAX).is_err());
    }
}
