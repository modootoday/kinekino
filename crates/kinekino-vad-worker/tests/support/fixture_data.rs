//! Shared fixed synthetic data; only explicit test targets consume this module.
#[path = "../../../kinekino-player/src/infrastructure/immutable_storage_paths.rs"]
#[allow(dead_code)] // Exact shared ordinary-path module; its lock helper is unused.
pub(crate) mod ordinary_paths;
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::protocol::{application::decode_worker_request, domain::ExpectedWorkerContext};
use kinekino_vad::{
    application::ProfileAdmission,
    domain::{ArtifactPin, MAX_RUNTIME_BYTES, VadProfile},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub const MODEL: &[u8] = b"control-process-fixture-model";
pub const LICENSE: &[u8] = b"control-process-fixture-license-not-a-native-license-grant";
pub const NOTICE: &[u8] = b"synthetic test artifact; no ORT/model execution";
pub fn io(error: std::io::Error) -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, format!("Fixture IO: {error}"))
}
pub fn pin(id: &str, bytes: &[u8]) -> Result<ArtifactPin, ApiError> {
    ArtifactPin::new(
        id.into(),
        format!("{:x}", Sha256::digest(bytes)),
        DecimalU64(bytes.len() as u64),
    )
}
fn wire_pin(pin: &ArtifactPin) -> Value {
    json!({"id":pin.id(),"sha256":pin.sha256(),"bytes":pin.bytes().to_string()})
}
fn wav() -> Vec<u8> {
    let mut out = b"RIFF".to_vec();
    out.extend_from_slice(&(36_u32 + 1026).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt \x10\0\0\0\x01\0\x01\0\x80\x3e\0\0\0\x7d\0\0\x02\0\x10\0data");
    out.extend_from_slice(&1026_u32.to_le_bytes());
    out.resize(44 + 1026, 0);
    out
}
pub struct FixtureData {
    pub executable: PathBuf,
    pub profile: VadProfile,
    pub descriptor: Vec<u8>,
    pub request: Vec<u8>,
    pub expected: ExpectedWorkerContext,
    pub manifest_pin: ArtifactPin,
    pub staged: Vec<(&'static str, Vec<u8>)>,
    _directories: Vec<File>,
    _runtime: File,
}
impl FixtureData {
    /// Parent uses its fixed fixture BIN; child independently measures current_exe.
    /// Only synthetic test data is admitted, never request-copied profile fields.
    pub fn load(executable: &Path) -> Result<Self, ApiError> {
        let parent = executable.parent().ok_or_else(|| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Fixture executable parent absent",
            )
        })?;
        let (_, directories) = ordinary_paths::root(parent)?;
        let mut runtime = ordinary_paths::file(executable)?;
        let bytes = runtime.metadata().map_err(io)?.len();
        if bytes == 0 || bytes > MAX_RUNTIME_BYTES {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Fixture executable exceeds runtime bound",
            ));
        }
        let mut hash = Sha256::new();
        let mut actual = 0u64;
        let mut chunk = [0; 8192];
        loop {
            let count = runtime.read(&mut chunk).map_err(io)?;
            if count == 0 {
                break;
            }
            actual += count as u64;
            if actual > bytes {
                return Err(ApiError::new(
                    ErrorCode::SourceExpired,
                    "Fixture executable grew",
                ));
            }
            hash.update(&chunk[..count]);
        }
        if actual != bytes {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Fixture executable changed",
            ));
        }
        let runtime_pin = ArtifactPin::new(
            "fixture-executable".into(),
            format!("{:x}", hash.finalize()),
            DecimalU64(bytes),
        )?;
        let profile = VadProfile::new(
            "control-process-fixture".into(),
            "onnx-community/silero-vad".into(),
            "1".repeat(40),
            pin("fixture-model", MODEL)?,
            pin("fixture-license", LICENSE)?,
            runtime_pin,
        )?;
        let wav = wav();
        let pcm = vec![0; 1026];
        let context = json!({"schema_version":"1","profile":{"id":profile.id(),"repo":profile.repo(),"revision":profile.revision(),
            "precision":profile.precision(),"provider":profile.provider(),"intra_threads":2,"inter_threads":1,"license_id":profile.license_id(),
            "weights":wire_pin(profile.weights()),"license_document":wire_pin(profile.license_document()),"runtime":wire_pin(profile.runtime())},
            "input":{"wav":wire_pin(&pin("fixture-wav",&wav)?),"pcm":wire_pin(&pin("fixture-pcm",&pcm)?),"sample_rate":16000,"channels":1,"bits_per_sample":16,"samples":"513","start_sample":"0","end_sample":"513"}});
        let descriptor = serde_json::to_vec(&context)
            .map_err(|e| ApiError::new(ErrorCode::Internal, e.to_string()))?;
        let request = serde_json::to_vec(&json!({"schema_version":"1","job_id":"control-fixture-only","attempt":"1","input_sha256":"a".repeat(64),"source_blob_sha256":"b".repeat(64),"submission_request_sha256":null,
            "descriptor_sha256":pin("descriptor",&descriptor)?.sha256(),"context":context})).map_err(|e|ApiError::new(ErrorCode::Internal,e.to_string()))?;
        let expected =
            decode_worker_request(request.as_slice(), request.len() as u64, &descriptor)?
                .expected()
                .clone();
        let manifest = serde_json::to_vec(&json!({"schema_version":"vad.release/1","task":"audio.vad","profile_id":profile.id(),"repo_id":profile.repo(),"immutable_revision":profile.revision(),"format":"onnx","quantization":profile.precision(),"license_id":profile.license_id(),
            "files":{"weights":wire_pin(profile.weights()),"runtime":wire_pin(profile.runtime()),"license_document":wire_pin(profile.license_document()),"runtime_dependencies":[],"runtime_notices":[wire_pin(&pin("fixture-notice",NOTICE)?)]},
            "runtime":{"minimum_version":"fixture-only","provider":profile.provider(),"intra_threads":2,"inter_threads":1},"provenance":{"source_model":"synthetic-fixture","source_revision":"1".repeat(40),"conversion_recipe":"no-model","conversion_version":"1"},"evaluation_id":null,"measured_memory_bytes":null})).map_err(|e|ApiError::new(ErrorCode::Internal,e.to_string()))?;
        let manifest_pin = pin("fixture-release", &manifest)?;
        let declared =
            kinekino_vad_executor::application::decode_release_manifest(&manifest, &manifest_pin)?;
        if declared.profile() != &profile {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Fixture declaration differs",
            ));
        }
        let staged = vec![
            ("descriptor.json", descriptor.clone()),
            ("release.manifest.json", manifest),
            ("model.onnx", MODEL.to_vec()),
            ("model-license.txt", LICENSE.to_vec()),
            ("input.wav", wav),
            ("runtime-notice.txt", NOTICE.to_vec()),
        ];
        Ok(Self {
            executable: executable.to_owned(),
            profile,
            descriptor,
            request,
            expected,
            manifest_pin,
            staged,
            _directories: directories,
            _runtime: runtime,
        })
    }
    pub fn open_executable(&self) -> Result<File, ApiError> {
        ordinary_paths::file(&self.executable)
    }
}
pub struct FixtureAdmission<'a>(pub &'a FixtureData);
impl ProfileAdmission for FixtureAdmission<'_> {
    fn authorize(&self, profile: &VadProfile) -> Result<(), ApiError> {
        if profile == &self.0.profile {
            Ok(())
        } else {
            Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "Profile is not the fixed synthetic fixture",
            ))
        }
    }
}
