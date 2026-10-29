use super::super::super::vad_release_inputs::{
    FixedVadInputMapping, FixedVadRelease, GuardedReleasedVadInputs,
};
use super::*;
use kinekino_vad_executor::application::prepare_vad_admission;
use serde_json::json;
struct OriginalControl {
    checks: AtomicUsize,
    cancel_at: usize,
    end: Instant,
}
impl AnalysisControl for OriginalControl {
    fn cancellation_requested(&self) -> bool {
        self.checks.fetch_add(1, Ordering::SeqCst) + 1 >= self.cancel_at
    }
    fn deadline(&self) -> Instant {
        self.end
    }
}
fn control(cancel_at: usize, end: Instant) -> OriginalControl {
    OriginalControl {
        checks: AtomicUsize::new(0),
        cancel_at,
        end,
    }
}

pub(super) const PUBLIC_PROFILE: &str = "public.silero.v1";
pub(super) const PUBLIC_AUDIO: &str = "public.audio.v1";
pub(super) const PUBLIC_DESTINATION: &str = "public.destination.v1";

pub(super) fn released(
    changed_profile: bool,
) -> (
    GuardedReleasedVadInputs,
    PathBuf,
    Arc<JobFiles>,
    PrepareVadRequest,
    VadProfile,
) {
    let Fixture {
        inputs,
        root,
        manifest,
        source,
        blobs,
    } = Fixture::new();
    let snapshot = ProfileLookup::lookup(
        &inputs,
        &OpaqueReference::new("fixed-profile".into()).unwrap(),
        &ExpectedDigest::new(manifest.sha256().into()).unwrap(),
    )
    .unwrap();
    let profile = snapshot.profile.clone();
    drop(snapshot);
    let artifacts = vec![
        write(&root, "model.bin", "model", b"synthetic"),
        write(&root, "runtime.bin", "runtime", b"synthetic-runtime"),
        write(&root, "model-license.txt", "license", b"synthetic-license"),
        write(
            &root,
            "runtime-notice.txt",
            "native-notice",
            b"synthetic-runtime-notice",
        ),
    ];
    let p = |pin: &ArtifactPin| json!({"id":pin.id(),"sha256":pin.sha256(),"bytes":pin.bytes().to_string()});
    let wire = json!({"schema_version":"vad.release/1","task":"audio.vad","profile_id":profile.id(),
        "repo_id":profile.repo(),"immutable_revision":if changed_profile {"2".repeat(40)} else {profile.revision().into()},
        "format":"onnx","quantization":profile.precision(),"license_id":profile.license_id(),
        "files":{"weights":p(profile.weights()),"runtime":p(profile.runtime()),"license_document":p(profile.license_document()),
            "runtime_dependencies":[],"runtime_notices":[p(&artifacts[3].pin)]},
        "runtime":{"minimum_version":"1.31.0","provider":profile.provider(),"intra_threads":2,"inter_threads":1},
        "provenance":{"source_model":"declared","source_revision":"1".repeat(40),"conversion_recipe":"synthetic","conversion_version":"1"},
        "evaluation_id":null,"measured_memory_bytes":null});
    let release = write(
        &root,
        "release.json",
        "release-manifest",
        &serde_json::to_vec(&wire).unwrap(),
    );
    let release_hash = release.pin.sha256().to_owned();
    let internal = OpaqueReference::new("fixed-profile".into()).unwrap();
    let adapter = GuardedReleasedVadInputs::new(
        FixedVadRelease {
            public_profile_id: PUBLIC_PROFILE.into(),
            internal_profile: internal.clone(),
            private_snapshot: manifest,
            manifest: release,
            artifacts,
        },
        Arc::new(inputs),
        FixedVadInputMapping {
            public_audio_id: PUBLIC_AUDIO.into(),
            internal_audio: OpaqueReference::new("audio".into()).unwrap(),
            public_destination_id: PUBLIC_DESTINATION.into(),
            internal_destination: OpaqueReference::new("destination".into()).unwrap(),
        },
    )
    .unwrap();
    let profile_manifest = ExpectedDigest::new(release_hash).unwrap();
    let audio_source = ExpectedDigest::new(source.sha256().into()).unwrap();
    let request = PrepareVadRequest {
        profile: adapter
            .resolve_profile("public.silero.v1", &profile_manifest)
            .unwrap(),
        profile_manifest,
        audio: adapter
            .resolve_audio("public.audio.v1", &audio_source)
            .unwrap(),
        audio_source,
        start: DecimalU64(0),
        end: DecimalU64(513),
        destination: adapter
            .resolve_destination("public.destination.v1")
            .unwrap(),
    };
    (adapter, root, blobs, request, profile)
}

#[test]
fn real_release_mapping_admits_existing_store_and_retains_original_assets() {
    let (adapter, root, blobs, request, profile) = released(false);
    assert!(
        adapter
            .resolve_audio("audio", &request.audio_source)
            .is_err()
    );
    assert!(
        adapter
            .resolve_audio(
                "public.audio.v1",
                &ExpectedDigest::new("0".repeat(64)).unwrap()
            )
            .is_err()
    );
    assert!(adapter.resolve_destination("destination").is_err());
    assert_eq!(
        adapter
            .resolve_profile("public.silero.v1", &request.profile_manifest)
            .unwrap(),
        request.profile
    );
    assert!(
        adapter
            .resolve_profile("fixed-profile", &request.profile_manifest)
            .is_err()
    );
    let private = pin(
        "snapshot",
        &fs::read(root.join("profile.snapshot.json")).unwrap(),
    );
    assert_ne!(private.sha256(), request.profile_manifest.as_str());
    assert!(
        adapter
            .resolve_profile(
                "public.silero.v1",
                &ExpectedDigest::new(private.sha256().into()).unwrap()
            )
            .is_err()
    );
    let original = control(usize::MAX, Instant::now() + Duration::from_secs(5));
    let admission = {
        let controlled = adapter.controlled(&original);
        prepare_vad_admission(
            &request,
            VadAdmissionPorts {
                profiles: &controlled,
                audio: &controlled,
                destinations: &controlled,
                admission: &ExactProfile(profile),
                blobs: blobs.as_ref(),
            },
        )
        .unwrap()
    };
    assert_eq!(
        admission.profile_manifest().sha256(),
        request.profile_manifest.as_str()
    );
    assert_eq!(
        blobs.read_utf8(admission.descriptor().source()).unwrap(),
        admission.descriptor().raw()
    );
    drop(adapter);
    let model =
        crate::infrastructure::os_policy::guarded_mutation(fs::remove_file(root.join("model.bin")));
    let release = crate::infrastructure::os_policy::guarded_mutation(fs::remove_file(
        root.join("release.json"),
    ));
    drop(admission);
    if !model {
        fs::remove_file(root.join("model.bin")).unwrap();
    }
    if !release {
        fs::remove_file(root.join("release.json")).unwrap();
    }
    drop(blobs);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn actual_release_asset_corruption_cannot_use_declared_pin() {
    let (adapter, root, blobs, request, _) = released(false);
    fs::write(root.join("model.bin"), b"corrupted").unwrap();
    assert_eq!(
        ProfileLookup::lookup(&adapter, &request.profile, &request.profile_manifest)
            .err()
            .unwrap()
            .code,
        ErrorCode::SourceExpired
    );
    drop(adapter);
    drop(blobs);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn valid_release_with_a_different_profile_cannot_authorize_private_snapshot() {
    let (adapter, root, blobs, request, _) = released(true);
    assert_eq!(
        ProfileLookup::lookup(&adapter, &request.profile, &request.profile_manifest)
            .err()
            .unwrap()
            .code,
        ErrorCode::SourceExpired
    );
    drop(adapter);
    drop(blobs);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn original_cancel_precedes_release_read_and_does_not_mint_cleanup_ack() {
    let (adapter, root, blobs, request, _) = released(false);
    fs::remove_file(root.join("release.json")).unwrap();
    let original = control(1, Instant::now() + Duration::from_secs(5));
    let error = ProfileLookup::lookup(
        &adapter.controlled(&original),
        &request.profile,
        &request.profile_manifest,
    )
    .err()
    .unwrap();
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert_eq!(original.checks.load(Ordering::SeqCst), 1);
    drop(adapter);
    drop(blobs);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn original_expired_cutoff_is_not_renewed_before_opening_release() {
    let (adapter, root, blobs, request, _) = released(false);
    fs::remove_file(root.join("release.json")).unwrap();
    let original = control(usize::MAX, Instant::now() - Duration::from_secs(1));
    let error = ProfileLookup::lookup(
        &adapter.controlled(&original),
        &request.profile,
        &request.profile_manifest,
    )
    .err()
    .unwrap();
    assert_eq!(error.code, ErrorCode::Timeout);
    drop(adapter);
    drop(blobs);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn cancellation_between_original_reads_releases_failed_preparation_guards() {
    let (adapter, root, blobs, request, _) = released(false);
    let original = control(3, Instant::now() + Duration::from_secs(5));
    let error = ProfileLookup::lookup(
        &adapter.controlled(&original),
        &request.profile,
        &request.profile_manifest,
    )
    .err()
    .unwrap();
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert_eq!(original.checks.load(Ordering::SeqCst), 3);
    fs::write(
        root.join("release.json"),
        b"failed preparation has no escaped file guard",
    )
    .unwrap();
    drop(adapter);
    drop(blobs);
    fs::remove_dir_all(root).unwrap();
}
