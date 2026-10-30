#![cfg(windows)]
//! Same product sources and common crates; no process or operation registration.
#[path = "vad_parent_worker/child_configuration.rs"]
mod child_configuration;
#[path = "../src/infrastructure/immutable_storage_paths.rs"]
mod immutable_storage_paths;
#[path = "support/os_policy_oracle.rs"]
mod os_policy_oracle;
#[path = "vad_parent_worker/roundtrip.rs"]
mod roundtrip;
#[path = "../src/infrastructure/vad_fixed_inputs.rs"]
mod vad_fixed_inputs;
#[path = "../src/infrastructure/vad_job_asset.rs"]
mod vad_job_asset;
#[path = "../src/infrastructure/vad_job_attempts.rs"]
mod vad_job_attempts;
#[path = "../src/infrastructure/vad_job_files.rs"]
mod vad_job_files;
#[path = "support/vad_release_fixture.rs"]
mod vad_release_fixture;
#[path = "../src/infrastructure/vad_release_inputs.rs"]
mod vad_release_inputs;
#[path = "../src/infrastructure/vad_worker_payload.rs"]
mod vad_worker_payload;
#[path = "../src/infrastructure/vad_worker_payload_files.rs"]
mod vad_worker_payload_files;
mod infrastructure {
    pub(crate) use crate::os_policy_oracle as os_policy;
}
