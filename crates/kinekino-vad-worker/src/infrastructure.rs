#[cfg(feature = "native-ort")]
#[path = "infrastructure/ort_engine.rs"]
mod ort_engine;
#[path = "infrastructure/os_policy.rs"]
pub(crate) mod os_policy;
#[path = "infrastructure/runtime_admission.rs"]
mod runtime_admission;
#[cfg(feature = "native-ort")]
pub use ort_engine::OrtVadEngine;
pub use runtime_admission::LoadedRuntimeAdmission;
#[cfg(windows)]
#[path = "infrastructure/windows_loaded_runtime.rs"]
mod windows_loaded_runtime;
#[cfg(windows)]
pub use windows_loaded_runtime::{
    FixedNativeDeployment, ModulePhase, NativeModule, PinnedFile, WindowsLoadedRuntimeAdmission,
};
