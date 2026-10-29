mod admission;
mod child_configuration;
pub use child_configuration::{
    CHILD_CONFIGURATION_FILE, ChildNativeDeployment, ChildNativeModule, ChildPinnedFile,
    DelegatedReleaseConfiguration, MAX_CHILD_CONFIGURATION_BYTES, ParentSelectedRelease,
    decode_parent_configuration,
};
mod descriptor;
mod executor;
mod ports;
mod release_manifest;
#[cfg(test)]
pub(crate) mod source_tests;
mod worker_binding;
mod worker_control;
pub use admission::{
    AudioArtifactLookup, AudioSnapshot, DestinationLookup, DestinationSnapshot, PrepareVadRequest,
    PreparedVadAdmission, ProfileLookup, ProfileSnapshot, SnapshotLease, VadAdmissionPorts,
    prepare_vad_admission,
};
pub(crate) use descriptor::snapshot as descriptor_snapshot;
pub use descriptor::{SavedVadDescriptor, persist_vad_descriptor};
pub use executor::VadExecutor;
pub use ports::{
    AttemptResourcesFactory, OutputPublication, PreparedResources, RequestEncoder, RunControl,
    WorkerRead, WorkerTransport, WorkerTransportFactory,
};
pub use release_manifest::{
    MAX_RELEASE_MANIFEST_BYTES, VadReleaseManifest, decode_release_manifest,
};
pub use worker_binding::bind_static_worker;
pub use worker_control::{
    CounterClock, MAX_WORKER_CONTROL_BYTES, WorkerCancellation, WorkerControl, WorkerControlView,
    decode_worker_control, prepare_worker_cutoff, project_worker_deadline,
};
