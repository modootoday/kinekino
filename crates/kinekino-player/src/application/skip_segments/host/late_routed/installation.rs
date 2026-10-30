use super::{LateRoutedPort, State, internal};
use crate::application::{
    api::{ApiError, ErrorCode},
    skip_segments::host::{PreparedRoutedSkipOwner, SkipHostOwner, SkipHostRegistry},
};
use std::sync::Arc;

/// Crate-only, non-Clone/non-Serde, consumed by one successful original host installation.
pub(crate) struct LateRoutedInstaller {
    pub(super) port: Arc<LateRoutedPort>,
}
pub(crate) struct InstalledRoutedHost {
    pub(crate) registry: Arc<SkipHostRegistry>,
    pub(crate) owner: SkipHostOwner,
    pub(crate) rearm: LateRoutedRearm,
}
/// Original registry identity only. Native assembly may resume after its exact child joins.
pub(crate) struct LateRoutedRearm {
    port: Arc<LateRoutedPort>,
    registry: Arc<SkipHostRegistry>,
}
impl LateRoutedRearm {
    pub(crate) fn suspend(&self) -> Result<(), ApiError> {
        let mut state = self.port.state.lock().map_err(|_| internal())?;
        match &*state {
            State::Live(registry) if Arc::ptr_eq(registry, &self.registry) => {
                self.registry.begin_close();
                *state = State::Retired(self.registry.clone());
                Ok(())
            }
            State::Retired(registry) if Arc::ptr_eq(registry, &self.registry) => Ok(()),
            _ => Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original native registry rearm differs",
            )),
        }
    }
    pub(crate) fn resume(&self) -> Result<LateRoutedInstaller, ApiError> {
        let mut state = self.port.state.lock().map_err(|_| internal())?;
        match &*state {
            State::Retired(registry) if Arc::ptr_eq(registry, &self.registry) => {
                *state = State::Pending;
                Ok(LateRoutedInstaller {
                    port: self.port.clone(),
                })
            }
            _ => Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original native registry has not retired or rearm was consumed",
            )),
        }
    }
}
#[must_use = "Retain the installer and original prepared owner on setup failure"]
pub(crate) struct LateInstallFailure {
    pub(crate) error: ApiError,
    pub(crate) installer: LateRoutedInstaller,
    pub(crate) prepared: PreparedRoutedSkipOwner,
}
struct FailureParts {
    error: ApiError,
    prepared: PreparedRoutedSkipOwner,
}
impl LateRoutedInstaller {
    pub(crate) fn install(
        self,
        generation: String,
        prepared: PreparedRoutedSkipOwner,
    ) -> Result<InstalledRoutedHost, Box<LateInstallFailure>> {
        match self.install_original(generation, prepared) {
            Ok(installed) => Ok(installed),
            Err(failure) => Err(Box::new(LateInstallFailure {
                error: failure.error,
                installer: self,
                prepared: failure.prepared,
            })),
        }
    }
    fn install_original(
        &self,
        generation: String,
        prepared: PreparedRoutedSkipOwner,
    ) -> Result<InstalledRoutedHost, Box<FailureParts>> {
        if !Arc::ptr_eq(&self.port.sessions, &prepared.sessions()) {
            return Err(Box::new(FailureParts {
                error: ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Native skip activation belongs to different original sessions",
                ),
                prepared,
            }));
        }
        let mut state = match self.port.state.lock() {
            Ok(state) => state,
            Err(_) => {
                return Err(Box::new(FailureParts {
                    error: internal(),
                    prepared,
                }));
            }
        };
        match &*state {
            State::Live(_) => {
                return Err(Box::new(FailureParts {
                    error: ApiError::new(
                        ErrorCode::RevisionConflict,
                        "Original native skip host is already installed",
                    ),
                    prepared,
                }));
            }
            State::Closed | State::Retired(_) => {
                return Err(Box::new(FailureParts {
                    error: ApiError::new(
                        ErrorCode::Cancelled,
                        "Original native skip activation is closed",
                    ),
                    prepared,
                }));
            }
            State::Pending => {}
        }
        let registry = match SkipHostRegistry::new_routed_export_host(generation, prepared, 1) {
            Ok(registry) => Arc::new(registry),
            Err(failure) => {
                return Err(Box::new(FailureParts {
                    error: failure.error,
                    prepared: failure.prepared,
                }));
            }
        };
        let owner = registry.owner();
        *state = State::Live(registry.clone());
        let rearm = LateRoutedRearm {
            port: self.port.clone(),
            registry: registry.clone(),
        };
        Ok(InstalledRoutedHost {
            registry,
            owner,
            rearm,
        })
    }
    /// This closes only pending installation; native parent cleanup remains with its original owner.
    pub(crate) fn close_pending(&self) -> Result<(), ApiError> {
        let mut state = self.port.state.lock().map_err(|_| internal())?;
        match &*state {
            State::Pending => {
                *state = State::Closed;
                Ok(())
            }
            State::Closed | State::Retired(_) => Ok(()),
            State::Live(_) => Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Installed native host must end through its original owner",
            )),
        }
    }
}
