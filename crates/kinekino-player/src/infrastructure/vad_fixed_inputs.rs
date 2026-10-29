//! Guarded fixed offline inputs, not a release registry or a downloader.
//! Profile input is explicitly an existing VadDescriptorV1 snapshot file; it is
//! never presented as an accepted general ModelManifest/native license closure.
pub(super) use super::vad_job_asset::FixedVadFile;
use super::vad_job_asset::GuardedAsset;
use super::vad_job_files::VadJobFile;
#[cfg(test)]
use kinekino_jobs::application::SourceBlobStore;
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::application::AnalysisControl;
#[cfg(test)]
use kinekino_vad_executor::application::{
    PrepareVadRequest, PreparedVadAdmission, VadAdmissionPorts, prepare_vad_admission,
};
use kinekino_vad_executor::{
    ArtifactPin, MAX_AUDIO_BYTES, MAX_DESCRIPTOR_BYTES, VadInput,
    application::{
        AudioArtifactLookup, AudioSnapshot, DestinationLookup, DestinationSnapshot, ProfileLookup,
        ProfileSnapshot, SnapshotLease,
    },
    decode_descriptor,
    domain::{ExpectedDigest, OpaqueReference},
    infrastructure::JsonOutputPublication,
};
use std::{path::PathBuf, sync::Arc};

pub(super) struct FixedVadInputs {
    pub profile_descriptor: FixedVadFile,
    pub audio_id: OpaqueReference,
    pub original_audio: FixedVadFile,
    pub wav: FixedVadFile,
    pub pcm: FixedVadFile,
    pub samples: DecimalU64,
    pub destination_id: OpaqueReference,
    pub destination: PathBuf,
}
struct CombinedSnapshotLease {
    _leases: Vec<Arc<dyn SnapshotLease>>,
}
impl SnapshotLease for CombinedSnapshotLease {}
struct DestinationLease {
    _path: Arc<VadJobFile>,
}
impl SnapshotLease for DestinationLease {}
pub(super) struct GuardedFixedVadInputs {
    profile: GuardedAsset,
    audio_id: OpaqueReference,
    original_audio: GuardedAsset,
    wav: GuardedAsset,
    pcm: GuardedAsset,
    input: VadInput,
    destination_id: OpaqueReference,
    destination: Arc<VadJobFile>,
}
impl GuardedFixedVadInputs {
    pub(super) fn has_input_ids(
        &self,
        audio: &OpaqueReference,
        destination: &OpaqueReference,
    ) -> bool {
        audio == &self.audio_id && destination == &self.destination_id
    }
    pub(super) fn check_audio_binding(
        &self,
        id: &OpaqueReference,
        expected: &ExpectedDigest,
    ) -> Result<(), ApiError> {
        if id != &self.audio_id {
            return Err(ApiError::new(
                ErrorCode::NotFound,
                "Fixed audio identity is unavailable",
            ));
        }
        if self.original_audio.pin().sha256() != expected.as_str() {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original audio source digest differs",
            ));
        }
        Ok(())
    }
    pub(super) fn new(files: FixedVadInputs) -> Result<Self, ApiError> {
        let input = VadInput::new(
            files.wav.pin.clone(),
            files.pcm.pin.clone(),
            files.samples,
            DecimalU64(0),
            files.samples,
        )?;
        Ok(Self {
            profile: GuardedAsset::new(files.profile_descriptor, MAX_DESCRIPTOR_BYTES)?,
            audio_id: files.audio_id,
            original_audio: GuardedAsset::new(files.original_audio, MAX_AUDIO_BYTES)?,
            wav: GuardedAsset::new(files.wav, MAX_AUDIO_BYTES)?,
            pcm: GuardedAsset::new(files.pcm, MAX_AUDIO_BYTES)?,
            input,
            destination_id: files.destination_id,
            destination: Arc::new(VadJobFile::from_application_path(&files.destination)?),
        })
    }
}
impl GuardedFixedVadInputs {
    pub(super) fn profile_snapshot(
        &self,
        id: &OpaqueReference,
        expected: &ExpectedDigest,
        control: Option<&dyn AnalysisControl>,
    ) -> Result<ProfileSnapshot, ApiError> {
        if self.profile.pin().sha256() != expected.as_str() {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Fixed profile snapshot digest differs",
            ));
        };
        let (bytes, lease) = self.profile.snapshot_with_control(control)?;
        let decoded = decode_descriptor(bytes.as_slice(), self.profile.pin())?;
        if decoded.profile().id() != id.as_str() {
            return Err(ApiError::new(
                ErrorCode::NotFound,
                "Fixed profile identity is unavailable",
            ));
        };
        Ok(ProfileSnapshot {
            profile: decoded.profile().clone(),
            manifest: self.profile.pin().clone(),
            lease,
        })
    }
}
impl GuardedFixedVadInputs {
    pub(super) fn worker_wav_snapshot(
        &self,
        expected: &ArtifactPin,
        control: &dyn AnalysisControl,
    ) -> Result<(Vec<u8>, Arc<dyn SnapshotLease>), ApiError> {
        if self.wav.pin() != expected {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Worker WAV differs from admitted input",
            ));
        }
        self.wav.snapshot_with_control(Some(control))
    }
}
impl GuardedFixedVadInputs {
    pub(super) fn audio_snapshot(
        &self,
        id: &OpaqueReference,
        expected: &ExpectedDigest,
        control: Option<&dyn AnalysisControl>,
    ) -> Result<AudioSnapshot, ApiError> {
        self.check_audio_binding(id, expected)?;
        let (_, source) = self.original_audio.snapshot_with_control(control)?;
        let (_, wav) = self.wav.snapshot_with_control(control)?;
        let (_, pcm) = self.pcm.snapshot_with_control(control)?;
        Ok(AudioSnapshot {
            id: self.audio_id.clone(),
            source: self.original_audio.pin().clone(),
            wav: self.input.wav().clone(),
            pcm: self.input.pcm().clone(),
            samples: DecimalU64(self.input.samples()),
            lease: Arc::new(CombinedSnapshotLease {
                _leases: vec![source, wav, pcm],
            }),
        })
    }
}
impl ProfileLookup for GuardedFixedVadInputs {
    fn lookup(
        &self,
        id: &OpaqueReference,
        expected: &ExpectedDigest,
    ) -> Result<ProfileSnapshot, ApiError> {
        self.profile_snapshot(id, expected, None)
    }
}
impl AudioArtifactLookup for GuardedFixedVadInputs {
    fn lookup(
        &self,
        id: &OpaqueReference,
        expected: &ExpectedDigest,
    ) -> Result<AudioSnapshot, ApiError> {
        self.audio_snapshot(id, expected, None)
    }
}
impl DestinationLookup for GuardedFixedVadInputs {
    fn lookup(&self, id: &OpaqueReference) -> Result<DestinationSnapshot, ApiError> {
        if id != &self.destination_id {
            return Err(ApiError::new(
                ErrorCode::NotFound,
                "Fixed destination identity is unavailable",
            ));
        };
        Ok(DestinationSnapshot {
            id: self.destination_id.clone(),
            publication: Arc::new(JsonOutputPublication::new(self.destination.clone())),
            lease: Arc::new(DestinationLease {
                _path: self.destination.clone(),
            }),
        })
    }
}
#[cfg(test)]
pub(super) fn prepare_fixed_vad(
    request: &PrepareVadRequest,
    inputs: &GuardedFixedVadInputs,
    admission: &dyn kinekino_vad_executor::ProfileAdmission,
    blobs: &dyn SourceBlobStore,
) -> Result<PreparedVadAdmission, ApiError> {
    prepare_vad_admission(
        request,
        VadAdmissionPorts {
            profiles: inputs,
            audio: inputs,
            destinations: inputs,
            admission,
            blobs,
        },
    )
}

#[cfg(test)]
pub(super) struct VadReleaseTestFixture {
    pub inputs: Arc<super::vad_release_inputs::GuardedReleasedVadInputs>,
    pub root: PathBuf,
    pub blobs: Arc<kinekino_jobs::infrastructure::JobFiles>,
    pub request: PrepareVadRequest,
    pub admission: Arc<dyn kinekino_vad::application::ProfileAdmission + Send + Sync>,
    pub execution: super::vad_job_attempts::VadAttemptConfiguration,
    pub public_profile_id: &'static str,
    pub public_audio_id: &'static str,
    pub public_destination_id: &'static str,
}
/// Reuses the exact authored synthetic files/admission; no new SDK fixture policy.
#[cfg(test)]
pub(super) fn release_file_fixture() -> VadReleaseTestFixture {
    tests::release_file_fixture()
}

#[cfg(test)]
#[path = "vad_fixed_inputs/tests"]
mod tests {
    mod release_profile;
    use super::super::{
        immutable_storage_paths as paths,
        vad_job_attempts::{VadAttemptConfiguration, assemble_admitted_attempt_executor},
    };
    use super::*;
    use kinekino_jobs::{
        domain::PreparedJobInput,
        infrastructure::{JobFiles, RootedJobPaths},
    };
    use kinekino_vad_executor::{
        ArtifactPin, ExpectedWorkerContext, ProfileAdmission, VadProfile,
        application::persist_vad_descriptor, decode_worker_outcome,
    };
    use sha2::{Digest, Sha256};
    use std::{
        fs::{self, File},
        path::Path,
        sync::atomic::{AtomicUsize, Ordering},
        time::{Duration, Instant},
    };
    struct Paths;
    impl RootedJobPaths for Paths {
        fn root(&self, p: &Path) -> Result<(PathBuf, Vec<File>), ApiError> {
            paths::root(p)
        }
        fn directory(&self, p: &Path) -> Result<File, ApiError> {
            paths::directory(p)
        }
        fn file(&self, p: &Path) -> Result<File, ApiError> {
            paths::file(p)
        }
        fn lock(&self, p: &Path) -> Result<File, ApiError> {
            paths::lock(p)
        }
    }
    fn pin(id: &str, bytes: &[u8]) -> ArtifactPin {
        ArtifactPin::new(
            id.into(),
            format!("{:x}", Sha256::digest(bytes)),
            DecimalU64(bytes.len() as u64),
        )
        .unwrap()
    }
    fn write(root: &Path, name: &str, id: &str, bytes: &[u8]) -> FixedVadFile {
        let path = root.join(name);
        fs::write(&path, bytes).unwrap();
        FixedVadFile {
            path,
            pin: pin(id, bytes),
        }
    }
    fn wav(pcm: &[u8]) -> Vec<u8> {
        let n = pcm.len() as u32;
        let mut b = Vec::new();
        b.extend_from_slice(b"RIFF");
        b.extend_from_slice(&(36 + n).to_le_bytes());
        b.extend_from_slice(b"WAVEfmt ");
        b.extend_from_slice(&16u32.to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&16000u32.to_le_bytes());
        b.extend_from_slice(&32000u32.to_le_bytes());
        b.extend_from_slice(&2u16.to_le_bytes());
        b.extend_from_slice(&16u16.to_le_bytes());
        b.extend_from_slice(b"data");
        b.extend_from_slice(&n.to_le_bytes());
        b.extend_from_slice(pcm);
        b
    }
    struct Fixture {
        inputs: GuardedFixedVadInputs,
        root: PathBuf,
        manifest: ArtifactPin,
        source: ArtifactPin,
        blobs: Arc<JobFiles>,
    }
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let root = std::env::temp_dir().join(format!(
                "vad-fixed-inputs-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            fs::create_dir(&root).unwrap();
            let jobs = root.join("jobs");
            fs::create_dir(&jobs).unwrap();
            let blobs = Arc::new(JobFiles::open(&jobs, Arc::new(Paths)).unwrap());
            let source = write(&root, "source.bin", "source", b"original-audio-fixture");
            let pcm = write(&root, "audio.pcm", "pcm", &[0; 1026]);
            let wav = write(&root, "audio.wav", "wav", &wav(&[0; 1026]));
            let input = VadInput::new(
                wav.pin.clone(),
                pcm.pin.clone(),
                DecimalU64(513),
                DecimalU64(0),
                DecimalU64(513),
            )
            .unwrap();
            let profile = VadProfile::new(
                "fixed-profile".into(),
                "onnx-community/silero-vad".into(),
                "1".repeat(40),
                pin("model", b"synthetic"),
                pin("license", b"synthetic-license"),
                pin("runtime", b"synthetic-runtime"),
            )
            .unwrap();
            let saved = persist_vad_descriptor(&profile, &input, blobs.as_ref()).unwrap();
            let manifest = write(
                &root,
                "profile.snapshot.json",
                "profile-snapshot",
                saved.raw().as_bytes(),
            );
            let manifest_pin = manifest.pin.clone();
            let source_pin = source.pin.clone();
            let inputs = GuardedFixedVadInputs::new(FixedVadInputs {
                profile_descriptor: manifest,
                audio_id: OpaqueReference::new("audio".into()).unwrap(),
                original_audio: source,
                wav,
                pcm,
                samples: DecimalU64(513),
                destination_id: OpaqueReference::new("destination".into()).unwrap(),
                destination: root.join("result.json"),
            })
            .unwrap();
            Self {
                inputs,
                root,
                manifest: manifest_pin,
                source: source_pin,
                blobs,
            }
        }
        fn request(&self) -> PrepareVadRequest {
            PrepareVadRequest {
                profile: OpaqueReference::new("fixed-profile".into()).unwrap(),
                profile_manifest: ExpectedDigest::new(self.manifest.sha256().into()).unwrap(),
                audio: OpaqueReference::new("audio".into()).unwrap(),
                audio_source: ExpectedDigest::new(self.source.sha256().into()).unwrap(),
                start: DecimalU64(0),
                end: DecimalU64(513),
                destination: OpaqueReference::new("destination".into()).unwrap(),
            }
        }
        fn remove(self) {
            let root = self.root.clone();
            drop(self);
            fs::remove_dir_all(root).unwrap();
        }
    }
    struct ExactProfile(VadProfile);
    pub(super) fn release_file_fixture() -> VadReleaseTestFixture {
        let (inputs, root, blobs, request, profile) = release_profile::released(false);
        let evidence = root.join("disabled-worker-evidence");
        fs::create_dir(&evidence).unwrap();
        // This file deliberately does not exist: an unexpected run must fail
        // actual artifact open before any external process can be launched.
        let execution = VadAttemptConfiguration {
            worker: root.join("worker-disabled.fixture"),
            worker_pin: pin("disabled-worker", b"disabled"),
            evidence_root: evidence,
            cleanup_reserve: Duration::from_millis(1),
            environment: Default::default(),
        };
        VadReleaseTestFixture {
            inputs: Arc::new(inputs),
            root,
            blobs,
            request,
            admission: Arc::new(ExactProfile(profile)),
            execution,
            public_profile_id: release_profile::PUBLIC_PROFILE,
            public_audio_id: release_profile::PUBLIC_AUDIO,
            public_destination_id: release_profile::PUBLIC_DESTINATION,
        }
    }
    impl ProfileAdmission for ExactProfile {
        fn authorize(&self, p: &VadProfile) -> Result<(), ApiError> {
            if p == &self.0 {
                Ok(())
            } else {
                Err(ApiError::new(
                    ErrorCode::PermissionDenied,
                    "Synthetic profile mismatch",
                ))
            }
        }
    }
    #[test]
    fn released_public_ids_and_legacy_assembly_retain_original_evidence_lease() {
        let fixture = release_file_fixture();
        assert_eq!(
            fixture
                .inputs
                .resolve_profile(fixture.public_profile_id, &fixture.request.profile_manifest)
                .unwrap(),
            fixture.request.profile
        );
        assert_eq!(
            fixture
                .inputs
                .resolve_audio(fixture.public_audio_id, &fixture.request.audio_source)
                .unwrap(),
            fixture.request.audio
        );
        assert_eq!(
            fixture
                .inputs
                .resolve_destination(fixture.public_destination_id)
                .unwrap(),
            fixture.request.destination
        );
        let root = fixture.root.clone();
        let evidence = fixture.execution.evidence_root.clone();
        assert!(!fixture.execution.worker.exists());
        let executor =
            super::super::vad_job_attempts::assemble_attempt_executor(fixture.execution).unwrap();
        assert!(
            super::super::vad_job_files::VadJobEvidenceRoot::from_application_root(&evidence)
                .is_err()
        );
        drop(fixture.inputs);
        drop(fixture.blobs);
        drop(fixture.admission);
        assert!(
            super::super::vad_job_files::VadJobEvidenceRoot::from_application_root(&evidence)
                .is_err()
        );
        drop(executor);
        let reopened =
            super::super::vad_job_files::VadJobEvidenceRoot::from_application_root(&evidence)
                .unwrap();
        drop(reopened);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn actual_profile_file_is_pinned_and_changed_bytes_are_rejected() {
        let fixture = Fixture::new();
        let request = fixture.request();
        let snapshot =
            ProfileLookup::lookup(&fixture.inputs, &request.profile, &request.profile_manifest)
                .unwrap();
        crate::infrastructure::os_policy::guarded_mutation(fs::write(
            fixture.root.join("profile.snapshot.json"),
            b"changed",
        ));
        drop(snapshot);
        fs::write(fixture.root.join("profile.snapshot.json"), b"changed").unwrap();
        assert!(
            ProfileLookup::lookup(&fixture.inputs, &request.profile, &request.profile_manifest)
                .is_err()
        );
        fixture.remove();
    }
    #[test]
    fn actual_audio_snapshot_keeps_original_file_guards_after_producer_release() {
        let fixture = Fixture::new();
        let request = fixture.request();
        let snapshot =
            AudioArtifactLookup::lookup(&fixture.inputs, &request.audio, &request.audio_source)
                .unwrap();
        let Fixture {
            inputs,
            root,
            blobs,
            ..
        } = fixture;
        drop(inputs);
        let applied = crate::infrastructure::os_policy::guarded_mutation(fs::remove_file(
            root.join("audio.pcm"),
        ));
        drop(snapshot);
        if !applied {
            fs::remove_file(root.join("audio.pcm")).unwrap();
        }
        drop(blobs);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn actual_pcm_digest_mismatch_is_not_replaced_by_declared_metadata() {
        let fixture = Fixture::new();
        let request = fixture.request();
        fs::write(fixture.root.join("audio.pcm"), b"changed").unwrap();
        assert!(
            AudioArtifactLookup::lookup(&fixture.inputs, &request.audio, &request.audio_source)
                .is_err()
        );
        fixture.remove();
    }
    #[test]
    fn concrete_producers_use_existing_job_files_reread_and_bind_session_resources() {
        let fixture = Fixture::new();
        let request = fixture.request();
        let profile =
            ProfileLookup::lookup(&fixture.inputs, &request.profile, &request.profile_manifest)
                .unwrap()
                .profile;
        let admission = Arc::new(
            prepare_fixed_vad(
                &request,
                &fixture.inputs,
                &ExactProfile(profile),
                fixture.blobs.as_ref(),
            )
            .unwrap(),
        );
        assert_eq!(
            fixture
                .blobs
                .read_utf8(admission.descriptor().source())
                .unwrap(),
            admission.descriptor().raw()
        );
        assert_ne!(
            admission.audio_source().sha256(),
            admission.descriptor().decoded().input().wav().sha256()
        );
        let evidence = fixture.root.join("evidence");
        fs::create_dir(&evidence).unwrap();
        let worker = write(&fixture.root, "worker.bin", "worker", b"not executable");
        let input = PreparedJobInput {
            operation: "audio.vad.candidate".into(),
            parameters: serde_json::json!({"synthetic_only":true}),
            source: admission.descriptor().source().clone(),
            execution_timeout_ms: DecimalU64(1000),
        };
        let executor = assemble_admitted_attempt_executor(
            VadAttemptConfiguration {
                worker: worker.path,
                worker_pin: worker.pin,
                evidence_root: evidence,
                cleanup_reserve: Duration::from_secs(1),
                environment: Default::default(),
            },
            &input,
            admission.clone(),
        )
        .unwrap();
        drop(admission);
        let applied = crate::infrastructure::os_policy::guarded_mutation(fs::remove_file(
            fixture.root.join("audio.wav"),
        ));
        drop(executor);
        if !applied {
            fs::remove_file(fixture.root.join("audio.wav")).unwrap();
        }
        fixture.remove();
    }
    #[test]
    fn fixed_destination_publishes_verifies_actual_bytes_and_refuses_overwrite() {
        let fixture = Fixture::new();
        let request = fixture.request();
        let raw = fs::read(fixture.root.join("profile.snapshot.json")).unwrap();
        let descriptor = decode_descriptor(raw.as_slice(), &fixture.manifest).unwrap();
        let context = ExpectedWorkerContext::new(pin("request", b"request"), descriptor);
        let bytes=serde_json::to_vec(&serde_json::json!({"schema_version":"1","request_sha256":context.request().sha256(),"descriptor_sha256":context.descriptor().raw().sha256(),"context":serde_json::from_slice::<serde_json::Value>(&raw).unwrap(),"kind":"complete","processed_samples":"513","scores":[0.1,0.9],"failure":null,"actual_runtime":null})).unwrap();
        let wire = decode_worker_outcome(bytes.as_slice(), bytes.len() as u64, &context).unwrap();
        let destination = DestinationLookup::lookup(&fixture.inputs, &request.destination).unwrap();
        let end = Instant::now() + Duration::from_secs(5);
        destination.publication.publish(&wire, end).unwrap();
        let output = destination.publication.verify(&wire, end).unwrap();
        assert_eq!(
            output.sha256,
            format!("{:x}", Sha256::digest(fs::read(&output.path).unwrap()))
        );
        assert!(destination.publication.publish(&wire, end).is_err());
        drop(destination);
        fixture.remove();
    }
}
