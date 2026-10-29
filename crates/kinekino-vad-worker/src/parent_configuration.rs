//! Configuration delegated by the existing trusted host, not arbitrary-parent
//! authentication and not a caller JSON admission API.
mod launch;
use crate::{
    control::changed,
    production::ReleaseInputs,
    staged_files::{WorkerRequestFiles, immutable_storage_paths},
};
use kinekino_media_contracts::errors::ApiError;
use kinekino_vad::protocol::application::ValidatedWorkerRequest;
use kinekino_vad::{
    application::{
        AnalysisControl, ArtifactReader, ProfileAdmission, VadEngine, read_verified_artifact,
    },
    domain::{ArtifactPin, VadProfile},
};
use kinekino_vad_executor::application::{
    CHILD_CONFIGURATION_FILE, DelegatedReleaseConfiguration, MAX_CHILD_CONFIGURATION_BYTES,
    decode_parent_configuration,
};
pub use launch::ConfiguredWorkerLaunch;
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

struct Reader {
    path: PathBuf,
    pin: ArtifactPin,
    _file: File,
}
impl ArtifactReader for Reader {
    fn open(&self, pin: &ArtifactPin) -> Result<Box<dyn Read + '_>, ApiError> {
        if pin != &self.pin {
            return Err(changed("Parent configuration pin differs"));
        }
        Ok(Box::new(
            File::open(&self.path).map_err(crate::control::io)?,
        ))
    }
}
struct GuardedParentConfiguration {
    configuration: DelegatedReleaseConfiguration,
    _reader: Reader,
    _directories: Vec<File>,
}
impl GuardedParentConfiguration {
    /// Only invoked inside serve_controlled_request's original verified-parent
    /// callback. No public constructor from decoded configuration grants access.
    fn open(
        launch: &ConfiguredWorkerLaunch,
        request: &ValidatedWorkerRequest,
        control: &dyn AnalysisControl,
    ) -> Result<Self, ApiError> {
        let root = launch
            .worker
            .request
            .parent()
            .ok_or_else(|| changed("Original attempt root missing"))?;
        let (_, directories) = immutable_storage_paths::root(root)?;
        let path = root.join(CHILD_CONFIGURATION_FILE);
        let reader = Reader {
            _file: immutable_storage_paths::file(&path)?,
            path,
            pin: launch.configuration.clone(),
        };
        let bytes = read_verified_artifact(
            &reader,
            &launch.configuration,
            MAX_CHILD_CONFIGURATION_BYTES,
            control,
        )?;
        let configuration = decode_parent_configuration(
            &bytes,
            &launch.configuration,
            request.expected().request(),
        )?;
        configuration.check_request(request.expected().profile(), &launch.worker.manifest)?;
        Ok(Self {
            configuration,
            _reader: reader,
            _directories: directories,
        })
    }
}
impl ProfileAdmission for GuardedParentConfiguration {
    fn authorize(&self, requested: &VadProfile) -> Result<(), ApiError> {
        self.configuration
            .check_request(requested, self.configuration.manifest())
    }
}
/// Caller supplies the independently configured native engine port; this function
/// neither invents a native deployment nor enables production main/registration.
pub fn serve_parent_configured_request(
    reader: impl Read,
    files: &WorkerRequestFiles,
    launch: &ConfiguredWorkerLaunch,
    actual_executable: &Path,
    writer: impl Write,
    engine: &dyn VadEngine,
) -> Result<(), ApiError> {
    crate::control::serve_controlled_request(
        reader,
        files,
        &launch.worker,
        writer,
        |request, control| {
            let guarded = GuardedParentConfiguration::open(launch, request, control)?;
            let release = ReleaseInputs {
                profile: guarded.configuration.profile(),
                manifest: guarded.configuration.manifest(),
                admission: &guarded,
            };
            let root = launch
                .worker
                .request
                .parent()
                .ok_or_else(|| changed("Original attempt root missing"))?;
            crate::production::run_released_request(
                request,
                root,
                actual_executable,
                &release,
                engine,
                control,
            )
        },
    )
}

/// Actual configured main dispatch. Native engine composition happens only after
/// original parent/control and guarded configuration checks in the same callback.
pub fn serve_parent_configured_native_request(
    reader: impl Read,
    files: &WorkerRequestFiles,
    launch: &ConfiguredWorkerLaunch,
    writer: impl Write,
) -> Result<(), ApiError> {
    crate::control::serve_controlled_request(
        reader,
        files,
        &launch.worker,
        writer,
        |request, control| {
            let guarded = GuardedParentConfiguration::open(launch, request, control)?;
            #[cfg(feature = "native-ort")]
            {
                let root = launch
                    .worker
                    .request
                    .parent()
                    .ok_or_else(|| changed("Original attempt root missing"))?;
                let config = guarded.configuration.native_deployment().ok_or_else(|| kinekino_media_contracts::errors::ApiError::new(kinekino_media_contracts::errors::ErrorCode::NeedsInput, "Parent-owned native deployment, notice pins and Windows baseline are missing"))?;
                let deployment = native_deployment(&guarded.configuration, config, root);
                let runtime = crate::infrastructure::WindowsLoadedRuntimeAdmission::new(
                    &deployment,
                    control,
                )?;
                let engine = crate::infrastructure::OrtVadEngine::new(runtime);
                let release = ReleaseInputs {
                    profile: guarded.configuration.profile(),
                    manifest: guarded.configuration.manifest(),
                    admission: &guarded,
                };
                crate::production::run_released_request(
                    request,
                    root,
                    &deployment.worker.path,
                    &release,
                    &engine,
                    control,
                )
            }
            #[cfg(not(feature = "native-ort"))]
            {
                let _ = guarded;
                Err(ApiError::new(
                    kinekino_media_contracts::errors::ErrorCode::Unsupported,
                    "Parent configuration checked; native-ort adapter is not enabled",
                ))
            }
        },
    )
}

#[cfg(feature = "native-ort")]
fn native_deployment(
    release: &DelegatedReleaseConfiguration,
    config: &kinekino_vad_executor::application::ChildNativeDeployment,
    root: &Path,
) -> crate::infrastructure::FixedNativeDeployment {
    use crate::infrastructure::{FixedNativeDeployment, ModulePhase, NativeModule, PinnedFile};
    FixedNativeDeployment {
        profile: release.profile().clone(),
        manifest: PinnedFile {
            path: root.join("release.manifest.json"),
            pin: release.manifest().clone(),
        },
        worker: PinnedFile {
            path: config.worker_path.clone().into(),
            pin: release.profile().runtime().clone(),
        },
        native_modules: config
            .modules
            .iter()
            .map(|m| NativeModule {
                file: PinnedFile {
                    path: m.file.path.clone().into(),
                    pin: m.file.pin.clone(),
                },
                phase: if m.required_at_startup {
                    ModulePhase::Startup
                } else {
                    ModulePhase::BeforeInference
                },
            })
            .collect(),
        notices: config
            .notices
            .iter()
            .map(|f| PinnedFile {
                path: f.path.clone().into(),
                pin: f.pin.clone(),
            })
            .collect(),
        system_module_names: config.system_module_names.clone(),
        minimum_windows_build: config.minimum_windows_build,
    }
}
