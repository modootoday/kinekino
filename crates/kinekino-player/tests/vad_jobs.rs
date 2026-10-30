#![cfg(windows)]
//! Compiles the exact concrete product adapters as siblings, just like the
//! existing Media product_guards target. This does not install the operation.
#[path = "support/cli_exe.rs"]
mod cli_exe;
mod application {
    pub use kinekino_player::application::audio_vad;
}
#[path = "vad_parent_worker/child_configuration.rs"]
mod child_configuration;
#[path = "vad_jobs/cli_cases.rs"]
mod cli_cases;
#[path = "support/local_host_cli.rs"]
pub mod cli_fixture;

#[test]
#[ignore = "Explicit actual CLI / named-pipe consumption gate; no external model worker"]
fn actual_cli_vad_failed_readmission_reopen_and_retry_preserve_original_source() {
    kinekino_platform::install();
    cli_cases::exercise();
}
#[path = "vad_jobs/admission_cases.rs"]
mod admission_cases;
#[path = "support/retaining_job_host.rs"]
mod host_closeout;
#[path = "../src/infrastructure/immutable_storage_paths.rs"]
mod immutable_storage_paths;
#[path = "../src/infrastructure/vad_fixed_inputs.rs"]
mod vad_fixed_inputs;
#[path = "../src/infrastructure/vad_job_asset.rs"]
mod vad_job_asset;
#[path = "../src/infrastructure/vad_job_attempts.rs"]
mod vad_job_attempts;
#[path = "../src/infrastructure/vad_job_executor.rs"]
mod vad_job_executor;
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

#[test]
fn concrete_guarded_adapter_satisfies_original_owned_host_send_sync_contract() {
    kinekino_platform::install();
    use kinekino_jobs::application::{
        JobExecutor,
        owned_attempt::{OwnedAttemptExecutor, OwnedOutputCandidate},
    };
    fn requires_original_host<T>()
    where
        T: JobExecutor
            + OwnedAttemptExecutor<Candidate = Box<dyn OwnedOutputCandidate>>
            + Send
            + Sync,
    {
    }
    requires_original_host::<vad_job_executor::VadJobExecutor>();
    let _trusted_constructor = vad_job_executor::VadJobExecutor::new;
    let _unadmitted_constructor = vad_job_attempts::assemble_attempt_executor;
    let _admitted_constructor = vad_job_attempts::assemble_admitted_attempt_executor;
}

#[test]
fn original_scope_mid_admission_cancel_fails_without_worker_or_cleanup_ack() {
    kinekino_platform::install();
    admission_cases::exercise(
        vad_release_fixture::release_file_fixture(),
        admission_cases::Failure::MidAdmissionCancel,
    );
}

#[test]
fn original_audio_changed_after_submit_is_rejected_on_attempt_readmission() {
    kinekino_platform::install();
    admission_cases::exercise(
        vad_release_fixture::release_file_fixture(),
        admission_cases::Failure::ChangedOriginalSource,
    );
}

#[test]
fn expired_original_scope_never_creates_attempt_resources_or_publishes() {
    kinekino_platform::install();
    admission_cases::exercise(
        vad_release_fixture::release_file_fixture(),
        admission_cases::Failure::OriginalDeadline,
    );
}
#[path = "support/os_policy_oracle.rs"]
mod os_policy_oracle;
mod infrastructure {
    pub(crate) use crate::os_policy_oracle as os_policy;
}
