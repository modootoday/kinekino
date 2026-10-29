//! Concrete identity/prerequisite checks over independently configured release.
#[path = "windows_loaded_runtime/deployment.rs"]
mod deployment;
#[path = "windows_loaded_runtime/files.rs"]
mod files;
#[path = "windows_loaded_runtime/observation.rs"]
mod observation;
use super::LoadedRuntimeAdmission;
pub use deployment::{FixedNativeDeployment, ModulePhase, NativeModule, PinnedFile};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::{application::AnalysisControl, domain::VadProfile};
use std::{cell::RefCell, fs::File, rc::Rc, time::Instant};
fn check(control: &dyn AnalysisControl) -> Result<(), ApiError> {
    if control.checkpoint()? {
        return Err(ApiError::new(
            ErrorCode::Cancelled,
            "Original native admission cancelled",
        ));
    }
    if Instant::now() >= control.deadline() {
        return Err(ApiError::new(
            ErrorCode::Timeout,
            "Original native admission cutoff",
        ));
    }
    Ok(())
}
pub struct NativeAdmissionGuard {
    _modules: Vec<observation::ObservedModule>,
    _observed_files: Vec<File>,
    _observed_directories: Vec<File>,
    _files: Rc<files::GuardedDeployment>,
}
pub struct WindowsLoadedRuntimeAdmission<'a> {
    config: &'a FixedNativeDeployment,
    control: &'a dyn AnalysisControl,
    files: RefCell<Option<Rc<files::GuardedDeployment>>>,
}
impl<'a> WindowsLoadedRuntimeAdmission<'a> {
    pub fn new(
        config: &'a FixedNativeDeployment,
        control: &'a dyn AnalysisControl,
    ) -> Result<Self, ApiError> {
        config.validate()?;
        check(control)?;
        Ok(Self {
            config,
            control,
            files: RefCell::new(None),
        })
    }
    fn observe(
        &self,
        files: Rc<files::GuardedDeployment>,
        inference: bool,
    ) -> Result<NativeAdmissionGuard, ApiError> {
        use crate::staged_files::immutable_storage_paths;
        check(self.control)?;
        observation::check_windows(self.config.minimum_windows_build)?;
        let system = observation::system_root()?;
        let modules = observation::modules()?;
        let mut seen = vec![false; self.config.native_modules.len()];
        let mut main = false;
        let mut observed_files = Vec::new();
        let mut directories = Vec::new();
        for module in &modules {
            check(self.control)?;
            let parent = module.path.parent().ok_or_else(|| {
                ApiError::new(ErrorCode::SourceExpired, "Observed module parent absent")
            })?;
            directories.extend(immutable_storage_paths::root(parent)?.1);
            observed_files.push(immutable_storage_paths::file(&module.path)?);
            let actual = module
                .path
                .canonicalize()
                .map_err(|e| ApiError::new(ErrorCode::EngineFailed, e.to_string()))?;
            if module.is_main {
                if main || actual != files.worker.canonical {
                    return Err(ApiError::new(
                        ErrorCode::SourceExpired,
                        "Actual worker image differs from pinned deployment",
                    ));
                }
                main = true;
                continue;
            }
            let name = module
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| {
                    ApiError::new(ErrorCode::SourceExpired, "Native module filename invalid")
                })?;
            if let Some(index) = self.config.native_modules.iter().position(|m| {
                m.file
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|expected| name.eq_ignore_ascii_case(expected))
            }) {
                if seen[index] || actual != files.modules[index].canonical {
                    return Err(ApiError::new(
                        ErrorCode::SourceExpired,
                        "Actual native module path/identity differs",
                    ));
                }
                seen[index] = true;
            } else {
                if actual.parent() != Some(system.as_path())
                    || !self
                        .config
                        .system_module_names
                        .iter()
                        .any(|expected| name.eq_ignore_ascii_case(expected))
                {
                    return Err(ApiError::new(
                        ErrorCode::SourceExpired,
                        "Unconfigured loaded module observed",
                    ));
                }
            }
        }
        if !main
            || seen.iter().enumerate().any(|(index, seen)| {
                !*seen
                    && (inference
                        || self.config.native_modules[index].phase == ModulePhase::Startup)
            })
        {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Required original native module is not loaded",
            ));
        }
        check(self.control)?;
        Ok(NativeAdmissionGuard {
            _modules: modules,
            _observed_files: observed_files,
            _observed_directories: directories,
            _files: files,
        })
    }
    fn profile(&self, profile: &VadProfile) -> Result<(), ApiError> {
        if profile == &self.config.profile {
            Ok(())
        } else {
            Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "Native profile is not the independently configured release",
            ))
        }
    }
}
impl LoadedRuntimeAdmission for WindowsLoadedRuntimeAdmission<'_> {
    type PrerequisiteGuard = NativeAdmissionGuard;
    type ModuleGuard = NativeAdmissionGuard;
    fn before_session(&self, profile: &VadProfile) -> Result<NativeAdmissionGuard, ApiError> {
        self.profile(profile)?;
        let files = Rc::new(files::GuardedDeployment::open(self.config, self.control)?);
        let guard = self.observe(files.clone(), false)?;
        *self.files.borrow_mut() = Some(files);
        Ok(guard)
    }
    fn before_inference(&self, profile: &VadProfile) -> Result<NativeAdmissionGuard, ApiError> {
        self.profile(profile)?;
        let files = self.files.borrow().as_ref().cloned().ok_or_else(|| {
            ApiError::new(
                ErrorCode::RevisionConflict,
                "Original prerequisite guard has not been established",
            )
        })?;
        self.observe(files, true)
    }
}
