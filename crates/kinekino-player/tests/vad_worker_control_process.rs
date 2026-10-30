#![cfg(windows)]
//! Explicit actual-child gate. No operation registration or model inference.
pub use kinekino_vad_executor::{application, domain};
#[path = "../../kinekino-vad-worker/tests/support/fixture_barrier.rs"]
pub mod fixture_barrier;
#[path = "../../kinekino-vad-worker/tests/support/fixture_data.rs"]
pub mod fixture_data;
#[path = "../../kinekino-vad-worker/tests/support/fixture_executable_path.rs"]
pub mod fixture_executable_path;
#[path = "../../kinekino-vad-worker/tests/support/fixture_payload.rs"]
pub mod fixture_payload;
#[path = "vad_worker_control_process/parent.rs"]
mod infrastructure;

macro_rules! actual {
    ($name:ident, $case:ident) => {
        #[test]
        #[ignore = "Central joint actual-child slot and fixed fixture binary/evidence pins required"]
        fn $name() { infrastructure::exercise(infrastructure::CaseKind::$case); }
    };
}
actual!(
    actual_native_transport_complete_control_and_cleanup,
    Complete
);
actual!(
    actual_original_parent_cancel_reaches_running_child,
    RunningCancel
);
actual!(
    actual_cancel_before_engine_keeps_original_child_owner,
    BeforeEngineCancel
);
actual!(
    actual_delayed_child_start_does_not_renew_operation_cutoff,
    DelayedStart
);
actual!(
    actual_running_child_expires_under_original_operation_cutoff,
    RunningExpiry
);
actual!(
    actual_signal_io_fault_is_not_encoded_as_cancellation,
    SignalFault
);
actual!(
    actual_control_pin_mismatch_rejects_before_engine,
    BindingMismatch
);
actual!(
    actual_fixture_flush_failure_retains_empty_stdout_and_family_receipt,
    FlushFailure
);
