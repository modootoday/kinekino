//! Product per-attempt resource composition candidate; registry remains disabled.
use super::vad_job_files::{VadJobEvidenceRoot, VadJobFile};
use super::{
    vad_release_inputs::GuardedReleasedVadInputs, vad_worker_payload::ReleasedWorkerPayload,
};
use kinekino_jobs::{application::JobAttemptIdentityV1, domain::JobId};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad_executor::{
    ArtifactPin, Deadline,
    application::{
        AttemptResourcesFactory, PreparedResources, VadExecutor, WorkerTransportFactory,
    },
    infrastructure::{
        EvidenceRootGuard, GuardedOutputFile, JsonOutputPublication, NativeTransportFactory,
        PinnedRequestEncoder, WorkerPayloadPreparation,
    },
};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{self, File},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

pub(super) struct VadAttemptConfiguration {
    pub worker: PathBuf,
    pub worker_pin: ArtifactPin,
    pub evidence_root: PathBuf,
    pub cleanup_reserve: Duration,
    pub environment: BTreeMap<String, OsString>,
}
#[cfg(test)]
pub(super) fn assemble_attempt_executor(
    configuration: VadAttemptConfiguration,
) -> Result<VadExecutor, ApiError> {
    Ok(VadExecutor::for_attempts(Arc::new(
        VadAttemptResources::new(configuration)?,
    )))
}
/// One admitted original prepared input, retaining its concrete snapshots across
/// all of its attempts. Public operation/schema and registration stay disabled.
#[cfg(test)]
pub(super) fn assemble_admitted_attempt_executor(
    configuration: VadAttemptConfiguration,
    input: &kinekino_jobs::domain::PreparedJobInput,
    admission: Arc<kinekino_vad_executor::application::PreparedVadAdmission>,
) -> Result<VadExecutor, ApiError> {
    let resources = kinekino_vad_executor::infrastructure::admitted_attempt_resources(
        Arc::new(VadAttemptResources::new(configuration)?),
        input,
        admission,
    )?;
    Ok(VadExecutor::for_attempts(resources))
}
/// Explicit static-worker composition. Original admission/source identity is
/// checked by the existing resources wrapper; registration stays disabled.
pub(super) fn assemble_static_worker_attempt_executor(
    configuration: VadAttemptConfiguration,
    input: &kinekino_jobs::domain::PreparedJobInput,
    admission: Arc<kinekino_vad_executor::application::PreparedVadAdmission>,
    inputs: Arc<GuardedReleasedVadInputs>,
) -> Result<VadExecutor, ApiError> {
    let payload = Arc::new(ReleasedWorkerPayload::new(
        inputs,
        admission.clone(),
        configuration.worker_pin.clone(),
    )?);
    let manifest = admission.profile_manifest().clone();
    let mut inner = VadAttemptResources::new(configuration)?;
    inner.payload = Some((manifest, payload));
    let resources = kinekino_vad_executor::infrastructure::admitted_attempt_resources(
        Arc::new(inner),
        input,
        admission,
    )?;
    Ok(VadExecutor::for_attempts(resources))
}
/// Concrete opt-in host assembly. `selected` comes from existing Rust issuer,
/// independently of `input`/request. The original admitted leases remain owned
/// by the same resources wrapper and NativeTransport through terminal cleanup.
pub(super) fn assemble_parent_configured_attempt_executor(
    configuration: VadAttemptConfiguration,
    input: &kinekino_jobs::domain::PreparedJobInput,
    admission: Arc<kinekino_vad_executor::application::PreparedVadAdmission>,
    inputs: Arc<GuardedReleasedVadInputs>,
    selected: Arc<kinekino_vad_executor::application::ParentSelectedRelease>,
) -> Result<VadExecutor, ApiError> {
    if let Some(deployment) = selected.native_deployment()
        && std::path::Path::new(&deployment.worker_path) != configuration.worker.as_path()
    {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Host worker path differs from selected native deployment",
        ));
    }
    selected.check_before_spawn(
        admission.descriptor().decoded().profile(),
        admission.profile_manifest(),
        &configuration.worker_pin,
    )?;
    let payload =
        ReleasedWorkerPayload::new(inputs, admission.clone(), configuration.worker_pin.clone())?
            .with_selected_release(selected)?;
    let mut inner = VadAttemptResources::new(configuration)?;
    inner.payload = Some((admission.profile_manifest().clone(), Arc::new(payload)));
    let resources = kinekino_vad_executor::infrastructure::admitted_attempt_resources(
        Arc::new(inner),
        input,
        admission,
    )?;
    Ok(VadExecutor::for_attempts(resources))
}
struct VadAttemptResources {
    worker: Arc<VadJobFile>,
    pin: ArtifactPin,
    parent: Arc<VadJobEvidenceRoot>,
    cleanup: Duration,
    environment: BTreeMap<String, OsString>,
    payload: Option<(ArtifactPin, Arc<dyn WorkerPayloadPreparation>)>,
}
struct AttemptDirectory {
    child: Arc<VadJobEvidenceRoot>,
    _parent: Arc<VadJobEvidenceRoot>,
}
impl EvidenceRootGuard for AttemptDirectory {
    fn path(&self) -> &Path {
        EvidenceRootGuard::path(self.child.as_ref())
    }
}
struct AttemptOutput {
    file: VadJobFile,
    _directory: Arc<AttemptDirectory>,
}
impl GuardedOutputFile for AttemptOutput {
    fn path(&self) -> &str {
        GuardedOutputFile::path(&self.file)
    }
    fn open_actual(&self) -> Result<File, ApiError> {
        self.file.open_actual()
    }
}
fn original_deadline(end: Instant, cleanup: Duration) -> Result<Deadline, ApiError> {
    let start = Instant::now();
    let remaining = end.saturating_duration_since(start);
    if cleanup.is_zero() {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Explicit VAD cleanup reserve required",
        ));
    };
    if cleanup >= remaining {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Original VAD cutoff has no remaining operation budget",
        ));
    };
    // End stays exactly the original scope end. Reserve is inside it, not added.
    Deadline::new(start, remaining, cleanup)
        .map_err(|e| ApiError::new(ErrorCode::InvalidArgument, e.to_string()))
}
impl VadAttemptResources {
    fn new(c: VadAttemptConfiguration) -> Result<Self, ApiError> {
        if c.cleanup_reserve.is_zero() {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Explicit VAD cleanup reserve required",
            ));
        };
        Ok(Self {
            worker: Arc::new(VadJobFile::from_application_path(&c.worker)?),
            pin: c.worker_pin,
            parent: Arc::new(VadJobEvidenceRoot::from_application_root(&c.evidence_root)?),
            cleanup: c.cleanup_reserve,
            environment: c.environment,
            payload: None,
        })
    }
    fn build(
        &self,
        job: &JobId,
        attempt: u64,
        end: Instant,
    ) -> Result<PreparedResources, ApiError> {
        let original = original_deadline(end, self.cleanup)?;
        // JobId is an existing checked domain value. No user path/worker JSON
        // creates a directory or replaces a prior attempt's evidence/output.
        let path = EvidenceRootGuard::path(self.parent.as_ref())
            .join(format!("{}-attempt-{attempt}", job.as_str()));
        fs::create_dir(&path).map_err(|e| ApiError::new(ErrorCode::EngineFailed, e.to_string()))?;
        let directory = Arc::new(AttemptDirectory {
            child: Arc::new(VadJobEvidenceRoot::from_application_root(&path)?),
            _parent: self.parent.clone(),
        });
        let output = Arc::new(AttemptOutput {
            file: VadJobFile::from_application_path(&path.join("observations.v1.json"))?,
            _directory: directory.clone(),
        });
        let factory = NativeTransportFactory::new(
            self.worker.clone(),
            self.pin.clone(),
            path,
            directory,
            original,
            self.environment.clone(),
        );
        let factory = match &self.payload {
            Some((manifest, payload)) => factory.with_payload(manifest.clone(), payload.clone())?,
            None => factory,
        };
        let transport = factory.prepare()?;
        Ok(PreparedResources {
            encoder: Arc::new(PinnedRequestEncoder),
            transport,
            publication: Arc::new(JsonOutputPublication::new(output)),
        })
    }
}
impl AttemptResourcesFactory for VadAttemptResources {
    fn prepare(
        &self,
        identity: &JobAttemptIdentityV1,
        original_end: Instant,
    ) -> Result<PreparedResources, ApiError> {
        self.build(identity.job_id(), identity.attempt(), original_end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    fn fixture() -> (VadAttemptResources, PathBuf) {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "vad-attempt-resources-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&root).unwrap();
        let worker = root.join("worker.bin");
        fs::write(&worker, b"fake").unwrap();
        let c = VadAttemptConfiguration {
            worker,
            worker_pin: ArtifactPin::new(
                "worker".into(),
                "a".repeat(64),
                kinekino_media_contracts::api_values::DecimalU64(4),
            )
            .unwrap(),
            evidence_root: root.clone(),
            cleanup_reserve: Duration::from_secs(1),
            environment: BTreeMap::new(),
        };
        (VadAttemptResources::new(c).unwrap(), root)
    }
    #[test]
    fn resources_separate_attempt_paths_and_keep_parent_lease_after_factory_drop() {
        let (factory, root) = fixture();
        let job = JobId::try_from(format!("job-{}", "a".repeat(32))).unwrap();
        let end = Instant::now() + Duration::from_secs(10);
        let one = factory.build(&job, 1, end).unwrap();
        let two = factory.build(&job, 2, end).unwrap();
        assert!(root.join(format!("{}-attempt-1", job.as_str())).is_dir());
        assert!(root.join(format!("{}-attempt-2", job.as_str())).is_dir());
        drop(factory);
        assert!(VadJobEvidenceRoot::from_application_root(&root).is_err());
        drop(one);
        assert!(VadJobEvidenceRoot::from_application_root(&root).is_err());
        drop(two);
        let guard = VadJobEvidenceRoot::from_application_root(&root).unwrap();
        drop(guard);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn repeated_same_attempt_never_overwrites_original_directory() {
        let (factory, root) = fixture();
        let job = JobId::try_from(format!("job-{}", "b".repeat(32))).unwrap();
        let one = factory
            .build(&job, 1, Instant::now() + Duration::from_secs(10))
            .unwrap();
        assert!(
            factory
                .build(&job, 1, Instant::now() + Duration::from_secs(10))
                .is_err()
        );
        drop(one);
        drop(factory);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn cutoff_is_not_renewed_and_cleanup_cannot_consume_all_operation_budget() {
        assert!(original_deadline(Instant::now(), Duration::from_secs(1)).is_err());
        let now = Instant::now();
        let end = now + Duration::from_secs(5);
        let original = original_deadline(end, Duration::from_secs(1)).unwrap();
        assert!(original.remaining(end).is_zero());
        assert!(original.operation_remaining(now) <= Duration::from_secs(4));
        assert!(original_deadline(end, Duration::ZERO).is_err());
    }
}
