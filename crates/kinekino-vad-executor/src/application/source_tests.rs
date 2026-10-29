use super::*;
use crate::domain::{ExpectedDigest, OpaqueReference};
use kinekino_jobs::{
    application::SourceBlobStore,
    domain::{SourceBlobRef, VerifiedJobOutput},
};
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::{
    application::ProfileAdmission,
    domain::{ArtifactPin, VadInput, VadProfile},
};
use sha2::{Digest, Sha256};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};

fn pin(id: &str, bytes: &[u8]) -> ArtifactPin {
    ArtifactPin::new(
        id.into(),
        format!("{:x}", Sha256::digest(bytes)),
        DecimalU64(bytes.len() as u64),
    )
    .unwrap()
}
fn profile() -> VadProfile {
    VadProfile::new(
        "fixture-profile".into(),
        "onnx-community/silero-vad".into(),
        "1".repeat(40),
        pin("model", b"synthetic-model"),
        pin("license", b"synthetic-license"),
        pin("runtime", b"synthetic-runtime"),
    )
    .unwrap()
}
fn input() -> VadInput {
    VadInput::new(
        pin("wav", b"derived-synthetic-wav"),
        pin("pcm", &[0; 1026]),
        DecimalU64(513),
        DecimalU64(0),
        DecimalU64(513),
    )
    .unwrap()
}
#[derive(Default)]
struct Blobs {
    raw: Mutex<String>,
    writes: AtomicUsize,
    reads: AtomicUsize,
    bad_metadata: bool,
    bad_read: bool,
}
impl SourceBlobStore for Blobs {
    fn put_utf8(&self, source: &str) -> Result<SourceBlobRef, ApiError> {
        self.writes.fetch_add(1, Ordering::SeqCst);
        *self.raw.lock().unwrap() = source.to_owned();
        Ok(SourceBlobRef {
            sha256: if self.bad_metadata {
                "a".repeat(64)
            } else {
                format!("{:x}", Sha256::digest(source.as_bytes()))
            },
            bytes: DecimalU64(source.len() as u64),
        })
    }
    fn read_utf8(&self, _source: &SourceBlobRef) -> Result<String, ApiError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        let mut raw = self.raw.lock().unwrap().clone();
        if self.bad_read {
            raw.push(' ')
        };
        Ok(raw)
    }
}
struct Lease;
impl SnapshotLease for Lease {}
struct NoPublication;
impl OutputPublication for NoPublication {
    fn publish(
        &self,
        _: &kinekino_vad::protocol::domain::WorkerOutcomeCandidate,
        _: Instant,
    ) -> Result<(), ApiError> {
        panic!("Admission must not publish")
    }
    fn verify(
        &self,
        _: &kinekino_vad::protocol::domain::WorkerOutcomeCandidate,
        _: Instant,
    ) -> Result<VerifiedJobOutput, ApiError> {
        panic!("Admission must not verify output")
    }
}
struct Lookups {
    lease: Arc<dyn SnapshotLease>,
    deny: bool,
    bad_profile: bool,
    bad_audio: bool,
    bad_destination: bool,
    audio_calls: AtomicUsize,
    destination_calls: AtomicUsize,
}
impl Lookups {
    fn new(lease: Arc<dyn SnapshotLease>) -> Self {
        Self {
            lease,
            deny: false,
            bad_profile: false,
            bad_audio: false,
            bad_destination: false,
            audio_calls: AtomicUsize::new(0),
            destination_calls: AtomicUsize::new(0),
        }
    }
    fn ports<'a>(&'a self, blobs: &'a Blobs) -> VadAdmissionPorts<'a> {
        VadAdmissionPorts {
            profiles: self,
            audio: self,
            destinations: self,
            admission: self,
            blobs,
        }
    }
}
impl ProfileLookup for Lookups {
    fn lookup(&self, _: &OpaqueReference, _: &ExpectedDigest) -> Result<ProfileSnapshot, ApiError> {
        Ok(ProfileSnapshot {
            profile: profile(),
            manifest: pin(
                "manifest",
                if self.bad_profile {
                    b"changed"
                } else {
                    b"manifest"
                },
            ),
            lease: self.lease.clone(),
        })
    }
}
impl ProfileAdmission for Lookups {
    fn authorize(&self, _: &VadProfile) -> Result<(), ApiError> {
        if self.deny {
            Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "Profile not admitted",
            ))
        } else {
            Ok(())
        }
    }
}
impl AudioArtifactLookup for Lookups {
    fn lookup(&self, id: &OpaqueReference, _: &ExpectedDigest) -> Result<AudioSnapshot, ApiError> {
        self.audio_calls.fetch_add(1, Ordering::SeqCst);
        let i = input();
        Ok(AudioSnapshot {
            id: id.clone(),
            source: pin(
                "original-audio",
                if self.bad_audio {
                    b"changed"
                } else {
                    b"original-audio"
                },
            ),
            wav: i.wav().clone(),
            pcm: i.pcm().clone(),
            samples: DecimalU64(i.samples()),
            lease: self.lease.clone(),
        })
    }
}
impl DestinationLookup for Lookups {
    fn lookup(&self, id: &OpaqueReference) -> Result<DestinationSnapshot, ApiError> {
        self.destination_calls.fetch_add(1, Ordering::SeqCst);
        Ok(DestinationSnapshot {
            id: if self.bad_destination {
                OpaqueReference::new("changed".into()).unwrap()
            } else {
                id.clone()
            },
            publication: Arc::new(NoPublication),
            lease: self.lease.clone(),
        })
    }
}
fn request() -> PrepareVadRequest {
    PrepareVadRequest {
        profile: OpaqueReference::new("fixture-profile".into()).unwrap(),
        profile_manifest: ExpectedDigest::new(pin("manifest", b"manifest").sha256().into())
            .unwrap(),
        audio: OpaqueReference::new("audio".into()).unwrap(),
        audio_source: ExpectedDigest::new(pin("original-audio", b"original-audio").sha256().into())
            .unwrap(),
        start: DecimalU64(0),
        end: DecimalU64(513),
        destination: OpaqueReference::new("destination".into()).unwrap(),
    }
}
pub(crate) fn admitted_fixture() -> (PreparedVadAdmission, Arc<dyn SnapshotLease>) {
    let lease: Arc<dyn SnapshotLease> = Arc::new(Lease);
    let lookups = Lookups::new(lease.clone());
    let blobs = Blobs::default();
    let admitted = prepare_vad_admission(&request(), lookups.ports(&blobs)).unwrap();
    (admitted, lease)
}

#[test]
fn descriptor_stores_exact_original_bytes_and_reads_them_back() {
    let blobs = Blobs::default();
    let saved = persist_vad_descriptor(&profile(), &input(), &blobs).unwrap();
    assert_eq!(
        saved.source().sha256,
        format!("{:x}", Sha256::digest(saved.raw().as_bytes()))
    );
    assert_eq!(saved.source().bytes.0, saved.raw().len() as u64);
    assert_eq!(saved.decoded().profile(), &profile());
    assert_eq!(blobs.writes.load(Ordering::SeqCst), 1);
    assert_eq!(blobs.reads.load(Ordering::SeqCst), 1);
}
#[test]
fn wrong_stored_metadata_is_rejected_before_reread() {
    let blobs = Blobs {
        bad_metadata: true,
        ..Default::default()
    };
    assert!(persist_vad_descriptor(&profile(), &input(), &blobs).is_err());
    assert_eq!(blobs.reads.load(Ordering::SeqCst), 0);
}
#[test]
fn changed_reread_is_not_accepted_even_when_json_meaning_is_equal() {
    let blobs = Blobs {
        bad_read: true,
        ..Default::default()
    };
    assert!(persist_vad_descriptor(&profile(), &input(), &blobs).is_err());
    assert_eq!(blobs.reads.load(Ordering::SeqCst), 1);
}
#[test]
fn denied_profile_prevents_audio_lookup_and_source_write() {
    let lease = Arc::new(Lease);
    let mut lookups = Lookups::new(lease);
    lookups.deny = true;
    let blobs = Blobs::default();
    assert!(prepare_vad_admission(&request(), lookups.ports(&blobs)).is_err());
    assert_eq!(lookups.audio_calls.load(Ordering::SeqCst), 0);
    assert_eq!(blobs.writes.load(Ordering::SeqCst), 0);
}
#[test]
fn wrong_profile_manifest_or_audio_source_never_creates_descriptor() {
    for profile_failure in [true, false] {
        let mut lookups = Lookups::new(Arc::new(Lease));
        lookups.bad_profile = profile_failure;
        lookups.bad_audio = !profile_failure;
        let blobs = Blobs::default();
        assert!(prepare_vad_admission(&request(), lookups.ports(&blobs)).is_err());
        assert_eq!(blobs.writes.load(Ordering::SeqCst), 0);
    }
}
#[test]
fn original_audio_wav_and_descriptor_digests_stay_distinct_and_guards_survive_lookup_drop() {
    let lease = Arc::new(Lease);
    let lookups = Lookups::new(lease.clone());
    let blobs = Blobs::default();
    let admitted = prepare_vad_admission(&request(), lookups.ports(&blobs)).unwrap();
    assert_ne!(
        admitted.audio_source().sha256(),
        admitted.descriptor().decoded().input().wav().sha256()
    );
    assert_ne!(
        admitted.audio_source().sha256(),
        admitted.descriptor().source().sha256
    );
    drop(lookups);
    assert_eq!(Arc::strong_count(&lease), 4);
    drop(admitted);
    assert_eq!(Arc::strong_count(&lease), 1);
}
#[test]
fn invalid_range_uses_p0_rejection_before_destination_or_source_write() {
    let lookups = Lookups::new(Arc::new(Lease));
    let blobs = Blobs::default();
    let mut request = request();
    request.end = DecimalU64(514);
    assert!(prepare_vad_admission(&request, lookups.ports(&blobs)).is_err());
    assert_eq!(lookups.destination_calls.load(Ordering::SeqCst), 0);
    assert_eq!(blobs.writes.load(Ordering::SeqCst), 0);
}
#[test]
fn changed_destination_cannot_create_descriptor() {
    let mut lookups = Lookups::new(Arc::new(Lease));
    lookups.bad_destination = true;
    let blobs = Blobs::default();
    assert!(prepare_vad_admission(&request(), lookups.ports(&blobs)).is_err());
    assert_eq!(blobs.writes.load(Ordering::SeqCst), 0);
}
#[test]
fn opaque_references_never_accept_paths_and_digests_use_existing_shared_validation() {
    for value in ["", "../x", "C:\\worker.exe", "https://x", "x/y"] {
        assert!(OpaqueReference::new(value.into()).is_err())
    }
    assert!(ExpectedDigest::new("A".repeat(64)).is_err());
    assert!(ExpectedDigest::new("a".repeat(64)).is_ok());
}
