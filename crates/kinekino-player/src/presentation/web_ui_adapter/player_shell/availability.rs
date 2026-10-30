use super::routes::RouteDescriptor;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SourceObservation {
    #[default]
    Unknown,
    NotIntegrated,
    Integrated,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CapabilityObservation {
    #[default]
    Unknown,
    Unsupported,
    Supported,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BackendObservation {
    #[default]
    Unknown,
    Unavailable,
    Ready,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MountObservation {
    #[default]
    Unknown,
    Unavailable,
    ReportedByHost,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct AvailabilityObservation {
    pub source: SourceObservation,
    pub capability: CapabilityObservation,
    pub backend: BackendObservation,
    pub mount: MountObservation,
    pub recovery_blocked: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Availability {
    Unknown,
    SourceMissing,
    SourceNotIntegrated,
    CapabilityUnsupported,
    BackendUnavailable,
    HostUnavailable,
    RecoveryBlocked,
    Available,
}
/// Display projection of trusted typed observations, not playback readiness or OS proof.
pub fn availability(route: &RouteDescriptor, observation: AvailabilityObservation) -> Availability {
    if observation.recovery_blocked {
        return Availability::RecoveryBlocked;
    }
    if route.source().is_none() {
        return Availability::SourceMissing;
    }
    match observation.source {
        SourceObservation::Unknown => return Availability::Unknown,
        SourceObservation::NotIntegrated => return Availability::SourceNotIntegrated,
        SourceObservation::Integrated => {}
    }
    match observation.capability {
        CapabilityObservation::Unknown => return Availability::Unknown,
        CapabilityObservation::Unsupported => return Availability::CapabilityUnsupported,
        CapabilityObservation::Supported => {}
    }
    match observation.backend {
        BackendObservation::Unknown => return Availability::Unknown,
        BackendObservation::Unavailable => return Availability::BackendUnavailable,
        BackendObservation::Ready => {}
    }
    match observation.mount {
        MountObservation::Unknown => Availability::Unknown,
        MountObservation::Unavailable => Availability::HostUnavailable,
        MountObservation::ReportedByHost => Availability::Available,
    }
}
