//! Final publication marker commits only when Core invokes this original candidate.
use super::*;
use crate::infrastructure::os_policy;
use crate::infrastructure::{
    content_digest, immutable_storage_paths as paths, native_immutable_publication,
    storage_directory_identity::StorageDirectoryIdentity,
};
use kinekino_jobs::application::{
    JobAttemptIdentityV1,
    owned_attempt::{AttemptScope, OwnedOutputCandidate},
};
use serde_json::json;
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{Seek, SeekFrom},
    path::Path,
    time::Instant,
};
pub(super) struct AssetArtifacts {
    root: PathBuf,
    guards: Vec<File>,
}
impl AssetArtifacts {
    pub fn open(root: &Path) -> Result<Self, ApiError> {
        let (root, guards) = paths::root(root)?;
        Ok(Self { root, guards })
    }
    fn begin(&self, id: &str) -> Result<Stage, ApiError> {
        let writer = paths::lock(&self.root.join("asset-output-writer.lock"))?;
        let directory = self.root.join(format!("asset-{id}"));
        fs::create_dir(&directory).map_err(kinekino_core::file_publication::io_error)?;
        let guard = paths::directory(&directory)?;
        let identity = StorageDirectoryIdentity::observe(&guard)?;
        let roots = self
            .guards
            .iter()
            .map(File::try_clone)
            .collect::<Result<Vec<_>, _>>()
            .map_err(kinekino_core::file_publication::io_error)?;
        Ok(Stage {
            directory,
            guard,
            identity,
            _roots: roots,
            _writer: writer,
            records: Vec::new(),
            publication: None,
        })
    }
}
struct Record {
    descriptor: AssetBundleFile,
    identity: StorageDirectoryIdentity,
    file: File,
}
struct Stage {
    directory: PathBuf,
    guard: File,
    identity: StorageDirectoryIdentity,
    _roots: Vec<File>,
    _writer: File,
    records: Vec<Record>,
    publication: Option<Record>,
}
impl Stage {
    fn names(&self, published: bool) -> Result<(), ApiError> {
        if StorageDirectoryIdentity::observe(&self.guard)? != self.identity {
            return Err(error(
                ErrorCode::SourceExpired,
                "Original asset output directory changed",
            ));
        }
        let expected: BTreeSet<_> = self
            .records
            .iter()
            .map(|r| r.descriptor.name.clone())
            .chain(published.then_some("publication.json".into()))
            .collect();
        let actual = fs::read_dir(&self.directory)
            .map_err(kinekino_core::file_publication::io_error)?
            .map(|entry| {
                entry
                    .map_err(kinekino_core::file_publication::io_error)?
                    .file_name()
                    .into_string()
                    .map_err(|_| error(ErrorCode::SourceExpired, "Non-UTF8 asset artifact entry"))
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        if actual != expected {
            return Err(error(
                ErrorCode::SourceExpired,
                "Asset artifact entries differ from original output",
            ));
        }
        Ok(())
    }
    fn record(
        &mut self,
        descriptor: &AssetBundleFile,
        bytes: &[u8],
        control: &dyn MutationControl,
    ) -> Result<(), ApiError> {
        control.checkpoint()?;
        let file = native_immutable_publication::publish_retained(
            &self.directory,
            &descriptor.name,
            bytes,
        )?;
        let identity = StorageDirectoryIdentity::observe(&file)?;
        self.records.push(Record {
            descriptor: descriptor.clone(),
            identity,
            file,
        });
        Self::verify_record(
            self.records.last_mut().expect("Original asset writer"),
            control,
        )
    }
    fn verify_record(record: &mut Record, control: &dyn MutationControl) -> Result<(), ApiError> {
        control.checkpoint()?;
        let metadata = record
            .file
            .metadata()
            .map_err(kinekino_core::file_publication::io_error)?;
        if !metadata.is_file()
            || os_policy::is_reparse_point(&metadata)
            || metadata.len() != record.descriptor.bytes.0
            || StorageDirectoryIdentity::observe(&record.file)? != record.identity
        {
            return Err(error(
                ErrorCode::SourceExpired,
                "Original asset output identity/size changed",
            ));
        }
        record
            .file
            .seek(SeekFrom::Start(0))
            .map_err(kinekino_core::file_publication::io_error)?;
        let observed = content_digest::sha256_with_checkpoint(
            &mut record.file,
            record.descriptor.bytes.0,
            || control.checkpoint(),
        )
        .map_err(|error| error.into_api_error())?;
        if observed != (record.descriptor.bytes.0, record.descriptor.sha256.clone()) {
            return Err(error(
                ErrorCode::SourceExpired,
                "Original asset output bytes changed",
            ));
        }
        Ok(())
    }
    fn verify(&mut self, published: bool, control: &dyn MutationControl) -> Result<(), ApiError> {
        self.names(published)?;
        for record in &mut self.records {
            Self::verify_record(record, control)?;
        }
        if published {
            Self::verify_record(
                self.publication.as_mut().ok_or_else(|| {
                    error(ErrorCode::NeedsInput, "Original publication writer absent")
                })?,
                control,
            )?;
        }
        self.names(published)?;
        control.checkpoint()
    }
}
struct Deadline(Instant);
impl MutationControl for Deadline {
    fn checkpoint(&self) -> Result<(), ApiError> {
        owned::deadline(self.0)
    }
}
pub(super) struct AssetCandidate {
    store: Arc<AssetArtifacts>,
    output: Option<NativeOutput>,
    stage: Option<Stage>,
    identity: JobAttemptIdentityV1,
    deadline: Instant,
    id: String,
    data: Vec<u8>,
    descriptor: AssetBundleFile,
    stage_attempted: bool,
    ready: bool,
    commit_attempted: bool,
    published: bool,
    verification_attempted: bool,
    released: bool,
}
impl AssetCandidate {
    pub fn new(
        store: Arc<AssetArtifacts>,
        output: NativeOutput,
        scope: &AttemptScope<'_>,
    ) -> Result<Self, ApiError> {
        Self::new_bound(store, output, scope.identity().clone(), scope.deadline())
    }
    fn new_bound(
        store: Arc<AssetArtifacts>,
        output: NativeOutput,
        identity: JobAttemptIdentityV1,
        deadline: Instant,
    ) -> Result<Self, ApiError> {
        let id = format!(
            "{}-attempt-{}",
            identity.job_id().as_str(),
            identity.attempt()
        );
        if !crate::domain::identifiers::valid_id(&id) {
            return Err(error(
                ErrorCode::InvalidArgument,
                "Invalid asset attempt artifact identity",
            ));
        }
        let files = output
            .files()
            .iter()
            .map(|file| file.descriptor.clone())
            .collect::<Vec<_>>();
        let data=serde_json::to_vec(&json!({"schema_version":"asset-artifact/1","artifact_id":id,"execution":{"job_id":identity.job_id().as_str(),"attempt":DecimalU64(identity.attempt()),"input_sha256":identity.input_sha256(),"source_blob_sha256":identity.source_blob_sha256(),"submission_request_sha256":identity.submission_request_sha256()},"bundle_manifest":output.manifest_file(),"files":files,"review_artifact_only":true,"quality_approved":false})).map_err(|_|error(ErrorCode::Internal,"Asset publication encoding failed"))?;
        let total = files
            .iter()
            .try_fold(data.len() as u64, |sum, file| sum.checked_add(file.bytes.0))
            .filter(|total| *total <= MAX_ASSET_BUNDLE_BYTES);
        if data.len() > 32 * 1024 * 1024 || total.is_none() {
            return Err(error(
                ErrorCode::ResourceExhausted,
                "Asset publication byte budget exceeded",
            ));
        }
        let descriptor = AssetBundleFile {
            name: "publication.json".into(),
            sha256: hash(&data),
            bytes: DecimalU64(data.len() as u64),
            media_type: "application/json".into(),
        };
        Ok(Self {
            store,
            output: Some(output),
            stage: None,
            identity,
            deadline,
            id,
            data,
            descriptor,
            stage_attempted: false,
            ready: false,
            commit_attempted: false,
            published: false,
            verification_attempted: false,
            released: false,
        })
    }
    fn recovery(&self, mut failure: ApiError) -> ApiError {
        if !failure.details.is_object() {
            failure.details = json!({"original_details":failure.details});
        }
        failure.details["artifact_id"] = json!(self.id);
        failure.details["publication_sha256"] = json!(self.descriptor.sha256);
        failure.details["publication_committed"] = json!(self.published);
        failure.details["publication_may_exist"] = json!(self.commit_attempted);
        failure.details["partial_cleanup_supported"] = json!(false);
        failure
    }
    pub fn stage(&mut self, scope: &AttemptScope<'_>) -> Result<(), ApiError> {
        self.stage_bound(
            scope.identity(),
            scope.deadline(),
            &owned::ScopeControl(scope),
        )
    }
    fn stage_bound(
        &mut self,
        identity: &JobAttemptIdentityV1,
        deadline: Instant,
        control: &dyn MutationControl,
    ) -> Result<(), ApiError> {
        if self.stage_attempted
            || self.released
            || self.identity != *identity
            || self.deadline != deadline
        {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Asset staging consumed or rebound",
            ));
        }
        self.stage_attempted = true;
        let result = (|| {
            control.checkpoint()?;
            self.stage = Some(self.store.begin(&self.id)?);
            let stage = self.stage.as_mut().expect("Original asset stage");
            for file in self.output.as_ref().expect("Original asset output").files() {
                stage.record(file.descriptor, file.contents, control)?;
            }
            stage.verify(false, control)?;
            self.ready = true;
            Ok(())
        })();
        result.map_err(|failure| self.recovery(failure))
    }
}
pub(super) struct CpuControl<'a>(pub &'a dyn ExecutionControl);
impl MutationControl for CpuControl<'_> {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if self.0.cancellation_requested() {
            return Err(error(
                ErrorCode::Cancelled,
                "Original CPU asset attempt cancelled before CAS",
            ));
        }
        owned::deadline(self.0.deadline())
    }
}
impl AssetCandidate {
    pub fn execute_cpu(
        store: Arc<AssetArtifacts>,
        output: NativeOutput,
        identity: JobAttemptIdentityV1,
        control: &dyn ExecutionControl,
    ) -> Result<VerifiedJobOutput, ApiError> {
        if control.execution_identity() != Some(&identity) {
            return Err(error(
                ErrorCode::SourceExpired,
                "Original CPU attempt identity changed",
            ));
        }
        let mut candidate = Self::new_bound(store, output, identity.clone(), control.deadline())?;
        let result = (|| {
            let check = CpuControl(control);
            candidate.stage_bound(&identity, control.deadline(), &check)?;
            check.checkpoint()?;
            // Existing CPU Jobs route, exactly one original cancellation/publication CAS.
            control.reserve_publication()?;
            candidate.publish()?;
            control
                .begin_verification()
                .map_err(|failure| candidate.recovery(failure))?;
            candidate.verify()
        })();
        let release = candidate.poll_release(control.deadline());
        match (result, release) {
            (Ok(output), Ok(true)) => Ok(output),
            (Err(mut primary), Err(secondary)) => {
                if !primary.details.is_object() {
                    primary.details = json!({"original_details":primary.details});
                }
                primary.details["release_error"] = json!({"code":secondary.code,"message":secondary.message,"details":secondary.details});
                Err(candidate.recovery(primary))
            }
            (Err(primary), _) => Err(candidate.recovery(primary)),
            (Ok(_), Err(failure)) => Err(candidate.recovery(failure)),
            (Ok(_), Ok(false)) => Err(candidate.recovery(error(
                ErrorCode::NeedsInput,
                "CPU asset release remains incomplete",
            ))),
        }
    }
}

impl OwnedOutputCandidate for AssetCandidate {
    fn poll_release(&mut self, _deadline: Instant) -> Result<bool, ApiError> {
        self.output.take();
        self.stage.take();
        self.released = true;
        Ok(true)
    }
    fn publish(&mut self) -> Result<(), ApiError> {
        if !self.ready || self.commit_attempted || self.released {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Original staged asset candidate required for one commit",
            ));
        }
        self.commit_attempted = true;
        let result = (|| {
            let control = Deadline(self.deadline);
            control.checkpoint()?;
            let stage = self.stage.as_mut().expect("Ready original asset stage");
            stage.verify(false, &control)?;
            let file = native_immutable_publication::publish_retained(
                &stage.directory,
                "publication.json",
                &self.data,
            )?;
            self.published = true;
            let identity = StorageDirectoryIdentity::observe(&file)?;
            stage.publication = Some(Record {
                descriptor: self.descriptor.clone(),
                identity,
                file,
            });
            Ok(())
        })();
        result.map_err(|failure| self.recovery(failure))
    }
    fn verify(&mut self) -> Result<VerifiedJobOutput, ApiError> {
        if !self.published || self.verification_attempted || self.released {
            return Err(error(
                ErrorCode::RevisionConflict,
                "Original committed asset required for one verification",
            ));
        }
        self.verification_attempted = true;
        let result = (|| {
            let stage = self.stage.as_mut().expect("Original committed asset stage");
            stage.verify(true, &Deadline(self.deadline))?;
            let output = VerifiedJobOutput {
                path: stage
                    .directory
                    .join("publication.json")
                    .to_str()
                    .ok_or_else(|| {
                        error(
                            ErrorCode::InvalidArgument,
                            "Non-UTF8 asset publication path",
                        )
                    })?
                    .into(),
                sha256: self.descriptor.sha256.clone(),
                bytes: self.descriptor.bytes,
                receipt: json!({"schema_version":"asset-job-receipt/1","artifact_id":self.id,"bundle_manifest_sha256":self.output.as_ref().expect("Original output").manifest_file().sha256,"files":stage.records.len(),"directory_identity":stage.identity,"quality_approved":false,"colorimetry_verified":false,"partial_cleanup_supported":false}),
            };
            output
                .validate()
                .map_err(|message| error(ErrorCode::InvalidArgument, message))?;
            Ok(output)
        })();
        result.map_err(|failure| self.recovery(failure))
    }
}
