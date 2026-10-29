use kinekino_media_contracts::errors::ApiError;
use kinekino_vad::domain::VadProfile;

/// Trusted native composition, never implemented by request JSON. Concrete
/// Windows module/path/identity/SHA verification is still unimplemented.
/// Staged dependency bytes alone cannot satisfy either phase.
pub trait LoadedRuntimeAdmission {
    type PrerequisiteGuard;
    type ModuleGuard;
    /// Check approved deployment/prerequisites before the first ORT API call.
    fn before_session(&self, profile: &VadProfile) -> Result<Self::PrerequisiteGuard, ApiError>;
    /// Verify actual loader-opened modules against the approved payload before
    /// any Session::run. Retain original module/file identity evidence.
    fn before_inference(&self, profile: &VadProfile) -> Result<Self::ModuleGuard, ApiError>;
}
