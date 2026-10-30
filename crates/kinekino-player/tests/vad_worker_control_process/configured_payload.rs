//! Test-only typed selection and immutable sidecar, retaining the original lease.
use super::*;
use crate::fixture_data::{FixtureAdmission, pin};
use kinekino_media_contracts::api_values::DecimalU64;
use kinekino_vad_executor::application::{
    CHILD_CONFIGURATION_FILE, ParentSelectedRelease, SnapshotLease,
};

#[derive(Clone, Copy)]
pub(super) enum Fault {
    None,
    Request,
    Configuration,
    Manifest,
    Worker,
}
pub(super) struct ConfiguredPayload {
    data: Arc<FixtureData>,
    staged_data: Arc<FixtureData>,
    bytes: Vec<u8>,
    pin: kinekino_vad::domain::ArtifactPin,
}
struct Lease {
    _base: StagedWorkerInput,
    _configuration: File,
    _directories: Vec<File>,
}
impl SnapshotLease for Lease {}
impl ConfiguredPayload {
    pub(super) fn new(data: Arc<FixtureData>, fault: Fault) -> Result<Self, ApiError> {
        let selected = ParentSelectedRelease::authorize(
            data.profile.clone(),
            data.manifest_pin.clone(),
            &FixtureAdmission(&data),
        )?;
        let (mut bytes, mut configuration_pin) = if matches!(fault, Fault::Request) {
            let original = data.expected.request();
            let mut other_sha = original.sha256().to_owned();
            other_sha.replace_range(
                ..1,
                if original.sha256().starts_with('0') {
                    "1"
                } else {
                    "0"
                },
            );
            let other = kinekino_vad::domain::ArtifactPin::new(
                original.id().into(),
                other_sha,
                DecimalU64(original.bytes()),
            )?;
            selected.encode_for_request(&other)?
        } else {
            selected.encode_for_attempt(
                data.expected.request(),
                &data.profile,
                &data.manifest_pin,
                data.profile.runtime(),
            )?
        };
        if matches!(fault, Fault::Worker) {
            // Deliberate invalid wire fixture, never another authorized profile.
            // Same digest/length, different worker artifact ID tests full binding.
            let mut wire: serde_json::Value = serde_json::from_slice(&bytes)
                .map_err(|e| ApiError::new(ErrorCode::Internal, e.to_string()))?;
            wire["worker"]["id"] = "other-worker-identity".into();
            bytes = serde_json::to_vec(&wire)
                .map_err(|e| ApiError::new(ErrorCode::Internal, e.to_string()))?;
            configuration_pin = pin("parent-release-configuration", &bytes)?;
        }
        if matches!(fault, Fault::Configuration) {
            bytes[0] = b'['; // Keep original declared pin; child must reject SHA.
        }
        let staged_data = if matches!(fault, Fault::Manifest) {
            // Independent fixed expectations remain unchanged. Only the actual
            // staged manifest bytes are faulty, before their read guards exist.
            let mut bad = FixtureData::load(&data.executable)?;
            let manifest = bad
                .staged
                .iter_mut()
                .find(|(name, _)| *name == "release.manifest.json")
                .ok_or_else(|| ApiError::new(ErrorCode::Internal, "Fixture manifest absent"))?;
            manifest.1[0] = b'[';
            Arc::new(bad)
        } else {
            data.clone()
        };
        Ok(Self {
            data,
            staged_data,
            bytes,
            pin: configuration_pin,
        })
    }
    pub(super) fn arguments(&self) -> Vec<OsString> {
        vec![
            "--parent-release-id".into(),
            self.pin.id().into(),
            "--parent-release-sha256".into(),
            self.pin.sha256().into(),
            "--parent-release-bytes".into(),
            self.pin.bytes().to_string().into(),
        ]
    }
}
fn check(control: &dyn RunControl) -> Result<(), ApiError> {
    if control.cancellation_requested() {
        return Err(ApiError::new(
            ErrorCode::Cancelled,
            "Original configured staging cancelled",
        ));
    }
    if Instant::now() >= control.deadline() {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Original configured staging cutoff",
        ));
    }
    Ok(())
}
impl WorkerPayloadPreparation for ConfiguredPayload {
    fn stage(
        &self,
        request: &[u8],
        root: &Path,
        control: &dyn RunControl,
    ) -> Result<StagedWorkerInput, ApiError> {
        check(control)?;
        let base = FixturePayload::new(self.staged_data.clone()).stage(request, root, control)?;
        let (_, directories) = crate::fixture_data::ordinary_paths::root(root)?;
        let path = root.join(CHILD_CONFIGURATION_FILE);
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(crate::fixture_data::io)?;
        output
            .write_all(&self.bytes)
            .and_then(|()| output.sync_all())
            .map_err(crate::fixture_data::io)?;
        drop(output);
        let mut guard = crate::fixture_data::ordinary_paths::file(&path)?;
        let mut actual = Vec::new();
        Read::by_ref(&mut guard)
            .take(kinekino_vad_executor::application::MAX_CHILD_CONFIGURATION_BYTES + 1)
            .read_to_end(&mut actual)
            .map_err(crate::fixture_data::io)?;
        if actual != self.bytes {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Staged configuration changed",
            ));
        }
        check(control)?;
        StagedWorkerInput::new(
            self.data.manifest_pin.clone(),
            Arc::new(Lease {
                _base: base,
                _configuration: guard,
                _directories: directories,
            }),
        )
        .with_child_configuration(self.pin.clone())
    }
}
