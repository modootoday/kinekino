use crate::entry::WorkerLaunch;
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::domain::ArtifactPin;
use kinekino_vad_executor::application::MAX_CHILD_CONFIGURATION_BYTES;
use std::ffi::OsString;
pub struct ConfiguredWorkerLaunch {
    pub worker: WorkerLaunch,
    pub configuration: ArtifactPin,
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid parent-selected release launch",
    )
}
impl ConfiguredWorkerLaunch {
    pub fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Self, ApiError> {
        let mut args: Vec<_> = arguments.into_iter().take(23).collect();
        if args.len() != 22
            || args[16] != "--parent-release-id"
            || args[18] != "--parent-release-sha256"
            || args[20] != "--parent-release-bytes"
        {
            return Err(invalid());
        }
        let bytes = args[21].to_str().ok_or_else(invalid)?;
        if bytes.is_empty()
            || !bytes.bytes().all(|b| b.is_ascii_digit())
            || (bytes.len() > 1 && bytes.starts_with('0'))
        {
            return Err(invalid());
        }
        let configuration = ArtifactPin::new(
            args[17].to_str().ok_or_else(invalid)?.into(),
            args[19].to_str().ok_or_else(invalid)?.into(),
            DecimalU64(bytes.parse().map_err(|_| invalid())?),
        )?;
        if configuration.id() != "parent-release-configuration"
            || configuration.bytes() > MAX_CHILD_CONFIGURATION_BYTES
        {
            return Err(invalid());
        }
        args.truncate(16);
        Ok(Self {
            worker: WorkerLaunch::parse(args)?,
            configuration,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn arguments() -> Vec<OsString> {
        let fixed = |name: &str| {
            std::env::temp_dir()
                .join("fixed")
                .join(name)
                .display()
                .to_string()
        };
        [
            "--vad-request-v1",
            &fixed("stdin.raw"),
            "--cancel-file",
            &fixed("cancel.signal"),
            "--release-manifest-id",
            "manifest",
            "--release-manifest-sha256",
            &"a".repeat(64),
            "--release-manifest-bytes",
            "16",
            "--worker-control-v1",
            &fixed("worker-control.v1.json"),
            "--worker-control-sha256",
            &"b".repeat(64),
            "--worker-control-bytes",
            "16",
            "--parent-release-id",
            "parent-release-configuration",
            "--parent-release-sha256",
            &"c".repeat(64),
            "--parent-release-bytes",
            "16",
        ]
        .into_iter()
        .map(OsString::from)
        .collect()
    }
    #[test]
    fn configured_suffix_preserves_original_sixteen_argument_contract() {
        let args = arguments();
        let original = WorkerLaunch::parse(args[..16].to_vec()).unwrap();
        let configured = ConfiguredWorkerLaunch::parse(args).unwrap();
        assert_eq!(configured.worker.request, original.request);
        assert_eq!(configured.worker.manifest, original.manifest);
        assert_eq!(configured.worker.control_pin, original.control_pin);
        assert_eq!(
            configured.configuration.id(),
            "parent-release-configuration"
        );
    }
    #[test]
    fn missing_extra_wrong_identity_or_noncanonical_length_is_rejected() {
        let args = arguments();
        assert!(ConfiguredWorkerLaunch::parse(args[..16].to_vec()).is_err());
        let mut extra = args.clone();
        extra.push("extra".into());
        assert!(ConfiguredWorkerLaunch::parse(extra).is_err());
        let mut identity = args.clone();
        identity[17] = "other-configuration".into();
        assert!(ConfiguredWorkerLaunch::parse(identity).is_err());
        let mut length = args;
        length[21] = "016".into();
        assert!(ConfiguredWorkerLaunch::parse(length).is_err());
    }
}
