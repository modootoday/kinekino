use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::domain::{
    ArtifactPin, MAX_RUNTIME_NOTICE_FILES, VadProfile, validate_runtime_notice_sizes,
};
use std::path::PathBuf;
pub struct PinnedFile {
    pub path: PathBuf,
    pub pin: ArtifactPin,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ModulePhase {
    Startup,
    BeforeInference,
}
pub struct NativeModule {
    pub file: PinnedFile,
    pub phase: ModulePhase,
}
/// Rust product configuration only, not an admission grant or JSON DTO. The
/// caller must supply the same independently approved release/deployment.
pub struct FixedNativeDeployment {
    pub profile: VadProfile,
    pub manifest: PinnedFile,
    pub worker: PinnedFile,
    pub native_modules: Vec<NativeModule>,
    pub notices: Vec<PinnedFile>,
    pub system_module_names: Vec<String>,
    pub minimum_windows_build: u32,
}
impl FixedNativeDeployment {
    pub(super) fn validate(&self) -> Result<(), ApiError> {
        if self.worker.pin != *self.profile.runtime()
            || self.native_modules.is_empty()
            || self.native_modules.len() > 16
            || self.notices.is_empty()
            || self.notices.len() > MAX_RUNTIME_NOTICE_FILES
            || self.system_module_names.is_empty()
            || self.system_module_names.len() > 128
            || self.minimum_windows_build == 0
        {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Complete fixed native deployment and supported Windows baseline are required",
            ));
        }
        validate_runtime_notice_sizes(self.notices.iter().map(|n| n.pin.bytes())).map_err(
            |_| {
                ApiError::new(
                    ErrorCode::NeedsInput,
                    "Fixed native notice sizes exceed bounds",
                )
            },
        )?;
        let mut names = Vec::new();
        for path in std::iter::once(&self.worker.path)
            .chain(self.native_modules.iter().map(|m| &m.file.path))
        {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| {
                    ApiError::new(ErrorCode::InvalidArgument, "Native filename missing")
                })?
                .to_ascii_lowercase();
            if !path.is_absolute() || names.contains(&name) {
                return Err(ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Native deployment paths/names are ambiguous",
                ));
            }
            names.push(name);
        }
        for name in &self.system_module_names {
            let normalized = name.to_ascii_lowercase();
            if name.is_empty()
                || !name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
                || names.contains(&normalized)
            {
                return Err(ApiError::new(
                    ErrorCode::InvalidArgument,
                    "System module selection is ambiguous",
                ));
            }
            names.push(normalized);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinekino_media_contracts::api_values::DecimalU64;
    fn pin(id: &str) -> ArtifactPin {
        ArtifactPin::new(id.into(), "a".repeat(64), DecimalU64(16)).unwrap()
    }
    fn config() -> FixedNativeDeployment {
        let profile = VadProfile::cached_silero(pin("license"), pin("worker")).unwrap();
        FixedNativeDeployment {
            worker: PinnedFile {
                path: "C:/fixed/worker.exe".into(),
                pin: profile.runtime().clone(),
            },
            profile,
            manifest: PinnedFile {
                path: "C:/fixed/release.json".into(),
                pin: pin("manifest"),
            },
            native_modules: vec![NativeModule {
                file: PinnedFile {
                    path: "C:/fixed/DirectML.dll".into(),
                    pin: pin("directml"),
                },
                phase: ModulePhase::Startup,
            }],
            notices: vec![PinnedFile {
                path: "C:/fixed/native-notice.txt".into(),
                pin: pin("notice"),
            }],
            system_module_names: vec!["ntdll.dll".into()],
            minimum_windows_build: 1,
        }
    }
    #[test]
    fn missing_deployment_or_os_baseline_does_not_become_implicit_permission() {
        let mut c = config();
        c.minimum_windows_build = 0;
        assert_eq!(c.validate().unwrap_err().code, ErrorCode::NeedsInput);
        c.minimum_windows_build = 1;
        c.system_module_names.clear();
        assert_eq!(c.validate().unwrap_err().code, ErrorCode::NeedsInput);
    }
    #[test]
    fn system_name_cannot_bypass_an_explicit_pinned_native_module() {
        let mut c = config();
        c.system_module_names.push("directml.DLL".into());
        assert_eq!(c.validate().unwrap_err().code, ErrorCode::InvalidArgument);
    }
    fn with_notices(sizes: &[u64]) -> FixedNativeDeployment {
        let mut deployment = config();
        deployment.notices = sizes
            .iter()
            .enumerate()
            .map(|(i, size)| PinnedFile {
                path: format!("C:/fixed/native-notice-{i}.txt").into(),
                pin: ArtifactPin::new(format!("notice-{i}"), "f".repeat(64), DecimalU64(*size))
                    .unwrap(),
            })
            .collect();
        deployment
    }
    #[test]
    fn actual_native_configuration_uses_separate_file_and_total_notice_bounds() {
        use kinekino_vad::domain::{MAX_RUNTIME_NOTICE_BYTES, MAX_RUNTIME_NOTICES_TOTAL_BYTES};
        let half = MAX_RUNTIME_NOTICES_TOTAL_BYTES / 2;
        for sizes in [
            vec![338_088, 1073],
            vec![MAX_RUNTIME_NOTICE_BYTES],
            vec![half, half],
            vec![65_536; 16],
        ] {
            with_notices(&sizes).validate().unwrap();
        }
        for sizes in [
            vec![],
            vec![MAX_RUNTIME_NOTICE_BYTES + 1],
            vec![half, half + 1],
            vec![1; 17],
        ] {
            assert_eq!(
                with_notices(&sizes).validate().err().unwrap().code,
                ErrorCode::NeedsInput
            );
        }
    }
}
