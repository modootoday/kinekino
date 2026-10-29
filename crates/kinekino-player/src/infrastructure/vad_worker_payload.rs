//! Trusted original admission → actual release bytes → guarded fixed sidecars.
use super::{
    vad_release_inputs::GuardedReleasedVadInputs,
    vad_worker_payload_files::{
        PinnedSidecar, SidecarRole, retain_original_request, write_sidecars,
    },
};
use kinekino_media_contracts::errors::ApiError;
use kinekino_vad::application::AnalysisControl;
use kinekino_vad::protocol::application::decode_worker_request;
use kinekino_vad_executor::{
    ArtifactPin,
    application::{PreparedVadAdmission, SnapshotLease, bind_static_worker},
    infrastructure::{StagedWorkerInput, WorkerPayloadPreparation},
};
use std::{path::Path, sync::Arc, time::Instant};
#[path = "vad_worker_payload/child_configuration.rs"]
mod child_configuration;
use kinekino_vad_executor::application::ParentSelectedRelease;

pub(super) struct ReleasedWorkerPayload {
    inputs: Arc<GuardedReleasedVadInputs>,
    admission: Arc<PreparedVadAdmission>,
    worker: ArtifactPin,
    selected: Option<Arc<ParentSelectedRelease>>,
}
impl ReleasedWorkerPayload {
    pub(super) fn new(
        inputs: Arc<GuardedReleasedVadInputs>,
        admission: Arc<PreparedVadAdmission>,
        worker: ArtifactPin,
    ) -> Result<Self, ApiError> {
        let profile = admission.descriptor().decoded().profile();
        bind_static_worker(profile, profile, &worker)?;
        Ok(Self {
            inputs,
            admission,
            worker,
            selected: None,
        })
    }
    pub(super) fn with_selected_release(
        mut self,
        selected: Arc<ParentSelectedRelease>,
    ) -> Result<Self, ApiError> {
        selected.check_before_spawn(
            self.admission.descriptor().decoded().profile(),
            self.admission.profile_manifest(),
            &self.worker,
        )?;
        self.selected = Some(selected);
        Ok(self)
    }
}
struct OriginalControl<'a>(&'a dyn kinekino_vad_executor::application::RunControl);
impl AnalysisControl for OriginalControl<'_> {
    fn deadline(&self) -> Instant {
        self.0.deadline()
    }
    fn cancellation_requested(&self) -> bool {
        self.0.cancellation_requested()
    }
}
struct PayloadLease {
    _admission: Arc<PreparedVadAdmission>,
    _sources: Vec<Arc<dyn SnapshotLease>>,
    _staged: Vec<Arc<dyn SnapshotLease>>,
}
impl SnapshotLease for PayloadLease {}
impl WorkerPayloadPreparation for ReleasedWorkerPayload {
    fn stage(
        &self,
        request: &[u8],
        owned_root: &Path,
        original: &dyn kinekino_vad_executor::application::RunControl,
    ) -> Result<StagedWorkerInput, ApiError> {
        let control = OriginalControl(original);
        let decoded = decode_worker_request(
            request,
            request.len() as u64,
            self.admission.descriptor().raw().as_bytes(),
        )?;
        let profile = self.admission.descriptor().decoded().profile();
        bind_static_worker(profile, decoded.expected().profile(), &self.worker)?;
        if let Some(selected) = &self.selected {
            selected.check_before_spawn(
                decoded.expected().profile(),
                self.admission.profile_manifest(),
                &self.worker,
            )?;
        }
        let request_lease =
            retain_original_request(owned_root, decoded.expected().request(), &control)?;
        let sources = self
            .inputs
            .worker_payload_sources(&self.admission, &control)?;
        let mut staged = write_sidecars(
            owned_root,
            vec![
                PinnedSidecar {
                    role: SidecarRole::Descriptor,
                    pin: self.admission.descriptor().decoded().raw().clone(),
                    bytes: self.admission.descriptor().raw().as_bytes().to_vec(),
                },
                PinnedSidecar {
                    role: SidecarRole::Manifest,
                    pin: sources.manifest.clone(),
                    bytes: sources.manifest_bytes,
                },
                PinnedSidecar {
                    role: SidecarRole::Model,
                    pin: profile.weights().clone(),
                    bytes: sources.model,
                },
                PinnedSidecar {
                    role: SidecarRole::License,
                    pin: profile.license_document().clone(),
                    bytes: sources.license,
                },
                PinnedSidecar {
                    role: SidecarRole::Wav,
                    pin: decoded.expected().input().wav().clone(),
                    bytes: sources.wav,
                },
            ],
            &control,
        )?;
        staged.push(request_lease);
        let configuration = match &self.selected {
            Some(selected) => {
                let (bytes, pin) = selected.encode_for_attempt(
                    decoded.expected().request(),
                    decoded.expected().profile(),
                    self.admission.profile_manifest(),
                    &self.worker,
                )?;
                let lease = child_configuration::stage(bytes, &pin, owned_root, &control)?;
                staged.push(lease);
                Some(pin)
            }
            None => None,
        };
        let staged = StagedWorkerInput::new(
            sources.manifest,
            Arc::new(PayloadLease {
                _admission: self.admission.clone(),
                _sources: sources.leases,
                _staged: staged,
            }),
        );
        match configuration {
            Some(pin) => staged.with_child_configuration(pin),
            None => Ok(staged),
        }
    }
}

#[cfg(test)]
#[path = "vad_worker_payload_tests.rs"]
mod tests;
