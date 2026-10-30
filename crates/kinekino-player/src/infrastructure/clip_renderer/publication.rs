use super::*;
mod owned;
use crate::{
    application::{clip_render::SavedClipProjectPin, jobs::ExecutionControl},
    domain::{
        api_values::DecimalU64,
        identifiers::{valid_id, valid_sha256},
    },
    infrastructure::{
        content_digest, immutable_storage_paths as paths, native_immutable_publication,
    },
};
pub(crate) use owned::ClipOwnedOutputCandidate;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, Serialize)]
pub struct PublishedClipArtifact {
    pub artifact_id: String,
    pub manifest_sha256: String,
    pub manifest_bytes: DecimalU64,
    pub review_artifact_only: bool,
    pub quality_approved: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    name: String,
    sha256: String,
    bytes: DecimalU64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: String,
    artifact_id: String,
    project: SavedClipProjectPin,
    document_sha256: String,
    source_sha256: String,
    output_sha256: String,
    files: Vec<Entry>,
    execution: Option<Value>,
    review_artifact_only: bool,
    quality_approved: bool,
}
pub struct RetainedClipArtifact {
    receipt: PublishedClipArtifact,
    render: Value,
    verification: Value,
    execution: Option<Value>,
    _guards: Vec<File>,
}
impl RetainedClipArtifact {
    pub fn receipt(&self) -> &PublishedClipArtifact {
        &self.receipt
    }
    pub fn render_facts(&self) -> &Value {
        &self.render
    }
    pub fn verification(&self) -> &Value {
        &self.verification
    }
    pub fn execution(&self) -> Option<&Value> {
        self.execution.as_ref()
    }
}
pub struct ClipArtifactStore {
    configured_root: PathBuf,
    root: PathBuf,
    _guards: Vec<File>,
}
/// Owns original staged files/root/writer guards; cannot be rebuilt from a manifest.
struct PreparedClipPublication {
    directory: PathBuf,
    data: Vec<u8>,
    expected: String,
    attempted: bool,
    _writer: File,
    _directory: File,
    _guards: Vec<File>,
    _root_guards: Vec<File>,
}
impl PreparedClipPublication {
    // Only the existing publish path invokes this after its original reservation.
    fn commit(&mut self) -> Result<(), ApiError> {
        if self.attempted {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original clip manifest commit was already attempted",
            ));
        }
        self.attempted = true;
        native_immutable_publication::publish(&self.directory, "manifest.json", &self.data)
    }
}
fn error(message: &str) -> ApiError {
    ApiError::new(ErrorCode::NeedsInput, message)
}
fn bytes(file: &mut File, maximum: u64) -> Result<Vec<u8>, ApiError> {
    file.seek(SeekFrom::Start(0))
        .map_err(kinekino_core::file_publication::io_error)?;
    let mut data = Vec::new();
    Read::by_ref(file)
        .take(maximum + 1)
        .read_to_end(&mut data)
        .map_err(kinekino_core::file_publication::io_error)?;
    if data.len() as u64 > maximum {
        return Err(error("Clip artifact record exceeds its bound"));
    }
    Ok(data)
}
fn hash(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}
struct Check<'a>(&'a dyn ExecutionControl);
impl MutationControl for Check<'_> {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if self.0.cancellation_requested() {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Clip publication cancelled before commit",
            ));
        }
        if Instant::now() >= self.0.deadline() {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Clip publication deadline elapsed",
            ));
        }
        Ok(())
    }
}
struct PostCommit<'a>(&'a dyn ExecutionControl);
impl MutationControl for PostCommit<'_> {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if Instant::now() >= self.0.deadline() {
            return Err(ApiError::new(
                ErrorCode::Timeout,
                "Clip output is committed; post-publication verification deadline elapsed",
            ));
        }
        Ok(())
    }
}
impl ClipArtifactStore {
    pub fn open(root: &Path) -> Result<Self, ApiError> {
        let configured_root = root.to_path_buf();
        let (root, guards) = paths::root(root)?;
        Ok(Self {
            configured_root,
            root,
            _guards: guards,
        })
    }
    pub fn publish(
        &self,
        id: &str,
        verified: &VerifiedClipCandidate<'_>,
        control: &dyn ExecutionControl,
    ) -> Result<RetainedClipArtifact, ApiError> {
        let check = Check(control);
        let mut staged = self.stage(id, verified, control.execution_identity(), &check)?;
        check.checkpoint()?;
        control.reserve_publication()?;
        staged.commit()?;
        let expected = staged.expected.clone();
        let result = (|| {
            control.begin_verification()?;
            self.inspect(id, &expected, &PostCommit(control))
        })();
        result.map_err(|mut error| {
            error.details["publication_committed"] = json!(true);
            error.details["artifact_id"] = json!(id);
            error.details["manifest_sha256"] = json!(expected);
            error
        })
    }
    fn stage(
        &self,
        id: &str,
        verified: &VerifiedClipCandidate<'_>,
        identity: Option<&crate::application::jobs::JobAttemptIdentityV1>,
        control: &dyn MutationControl,
    ) -> Result<PreparedClipPublication, ApiError> {
        if !valid_id(id) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid clip artifact identity",
            ));
        }
        control.checkpoint()?;
        let (_, root_guards) = paths::root(&self.configured_root)?;
        let _writer = paths::lock(&self.root.join("clip-output-writer.lock"))?;
        let directory = self.root.join(format!("clip-{id}"));
        fs::create_dir(&directory).map_err(kinekino_core::file_publication::io_error)?;
        let _directory = paths::directory(&directory)?;
        let candidate = verified.candidate();
        let facts = candidate.facts();
        let render = serde_json::to_vec(facts).map_err(|_| error("Invalid clip render record"))?;
        let verification = serde_json::to_vec(verified.verification())
            .map_err(|_| error("Invalid output verification record"))?;
        if render.len() > 1_048_576 || verification.len() > 1_048_576 {
            return Err(error("Clip records exceed publication bounds"));
        }
        let mut files = Vec::new();
        let mut guards = Vec::new();
        for (name, data) in [
            ("clip.mp4", candidate.bytes()),
            ("render.json", render.as_slice()),
            ("verification.json", verification.as_slice()),
        ] {
            control.checkpoint()?;
            native_immutable_publication::publish(&directory, name, data)?;
            let mut file = paths::file(&directory.join(name))?;
            let observed =
                content_digest::sha256_with_checkpoint(&mut file, data.len() as u64, || {
                    control.checkpoint()
                })
                .map_err(|e| e.into_api_error())?;
            let expected = hash(data);
            if observed != (data.len() as u64, expected.clone()) {
                return Err(error("Published clip file differs from verified candidate"));
            }
            files.push(Entry {
                name: name.into(),
                sha256: expected,
                bytes: DecimalU64(data.len() as u64),
            });
            guards.push(file);
        }
        let execution=identity.map(|identity|json!({"job_id":identity.job_id().as_str(),"attempt":DecimalU64(identity.attempt()),"input_sha256":identity.input_sha256(),"source_blob_sha256":identity.source_blob_sha256(),"submission_request_sha256":identity.submission_request_sha256()}));
        let manifest = Manifest {
            schema_version: "clip-artifact/1".into(),
            artifact_id: id.into(),
            project: facts.project.clone(),
            document_sha256: facts.plan.document_sha256.clone(),
            source_sha256: facts.plan.source_sha256.clone(),
            output_sha256: facts.output_sha256.clone(),
            files,
            execution,
            review_artifact_only: true,
            quality_approved: false,
        };
        let data = serde_json::to_vec(&manifest).map_err(|_| error("Invalid clip manifest"))?;
        let expected = hash(&data);

        Ok(PreparedClipPublication {
            directory,
            data,
            expected,
            attempted: false,
            _writer,
            _directory,
            _guards: guards,
            _root_guards: root_guards,
        })
    }
    pub fn inspect(
        &self,
        id: &str,
        expected_manifest_sha256: &str,
        control: &dyn MutationControl,
    ) -> Result<RetainedClipArtifact, ApiError> {
        if !valid_id(id) || !valid_sha256(expected_manifest_sha256) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Pinned clip artifact identity is required",
            ));
        }
        control.checkpoint()?;
        let (root, mut guards) = paths::root(&self.configured_root)?;
        let directory = root.join(format!("clip-{id}"));
        guards.push(paths::directory(&directory)?);
        let mut file = paths::file(&directory.join("manifest.json"))?;
        let data = bytes(&mut file, 1_048_576)?;
        if hash(&data) != expected_manifest_sha256 {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Clip manifest differs from its pinned identity",
            ));
        }
        let manifest: Manifest =
            serde_json::from_slice(&data).map_err(|_| error("Invalid pinned clip manifest"))?;
        guards.push(file);
        if manifest.schema_version != "clip-artifact/1"
            || manifest.artifact_id != id
            || !manifest.review_artifact_only
            || manifest.quality_approved
            || manifest.files.len() != 3
            || !valid_id(&manifest.project.document_id)
            || manifest.project.publication_revision.0 == 0
            || !valid_sha256(&manifest.project.publication_sha256)
            || !valid_sha256(&manifest.document_sha256)
            || !valid_sha256(&manifest.source_sha256)
            || !valid_sha256(&manifest.output_sha256)
        {
            return Err(error("Pinned clip manifest fields are invalid"));
        }
        let mut render = None;
        let mut verification = None;
        for (entry, name) in
            manifest
                .files
                .iter()
                .zip(["clip.mp4", "render.json", "verification.json"])
        {
            if entry.name != name
                || !valid_sha256(&entry.sha256)
                || entry.bytes.0 == 0
                || entry.bytes.0
                    > if name == "clip.mp4" {
                        128 * 1024 * 1024
                    } else {
                        1_048_576
                    }
            {
                return Err(error("Invalid clip artifact file entry"));
            }
            control.checkpoint()?;
            let mut held = paths::file(&directory.join(name))?;
            let observed = content_digest::sha256_with_checkpoint(&mut held, entry.bytes.0, || {
                control.checkpoint()
            })
            .map_err(|e| e.into_api_error())?;
            if observed != (entry.bytes.0, entry.sha256.clone()) {
                return Err(ApiError::new(
                    ErrorCode::SourceExpired,
                    "Clip artifact file differs from its manifest",
                ));
            }
            if name == "clip.mp4" {
                if entry.sha256 != manifest.output_sha256 {
                    return Err(error("Clip output hash binding differs"));
                }
            } else {
                let value: Value = serde_json::from_slice(&bytes(&mut held, 1_048_576)?)
                    .map_err(|_| error("Invalid clip artifact JSON"))?;
                if name == "render.json" {
                    render = Some(value)
                } else {
                    verification = Some(value)
                }
            }
            guards.push(held);
        }
        let render = render.ok_or_else(|| error("Missing clip render record"))?;
        let verification = verification.ok_or_else(|| error("Missing clip verification record"))?;
        if render["output_sha256"] != manifest.output_sha256
            || render["plan"]["document_sha256"] != manifest.document_sha256
            || render["plan"]["source_sha256"] != manifest.source_sha256
            || render["project"]
                != serde_json::to_value(&manifest.project)
                    .map_err(|_| error("Invalid project pin"))?
            || verification["output_sha256"] != manifest.output_sha256
            || verification["schema_version"] != "clip-output-verification/1"
            || verification["decode_integrity_verified"] != true
            || verification["quality_approved"] != false
            || render["quality_approved"] != false
        {
            return Err(error(
                "Clip artifact lineage or verification binding differs",
            ));
        }
        control.checkpoint()?;
        Ok(RetainedClipArtifact {
            receipt: PublishedClipArtifact {
                artifact_id: id.into(),
                manifest_sha256: expected_manifest_sha256.into(),
                manifest_bytes: DecimalU64(data.len() as u64),
                review_artifact_only: true,
                quality_approved: false,
            },
            render,
            verification,
            execution: manifest.execution,
            _guards: guards,
        })
    }
}
