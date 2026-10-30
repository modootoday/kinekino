//! CLI-only transport shutdown ownership; dependency cleanup remains at each host call site.
mod carrier;
mod dependency;
mod driver;
pub(crate) use dependency::{
    DependencyIncomplete, DependencyObservation, dependency_deadline, failed_dependency,
    finish_dependency, observe_dependency, pending_jobs,
};
#[cfg(windows)]
pub(crate) use dependency::{
    pending_native, pending_playback, pending_provider, pending_workspace,
};
#[cfg(test)]
mod tests;
pub(crate) use carrier::LocalHostShutdownView;

#[cfg(test)]
pub(crate) use carrier::{ClosingHost, ReadyForDependencies};
#[cfg(test)]
pub(crate) use driver::{DriveOutcome, advance};
pub(crate) use driver::{
    HostRunOutcome, IoPhase, TransportOutcome, preserve_first_error, serve_transport,
};
