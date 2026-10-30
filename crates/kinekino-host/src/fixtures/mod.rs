//! Child-process fixtures and workers that used to be bins of domain crates. Their bins now live
//! in `kinekino-app` (thin entries that install the OS policy first, ADR p1-port-inversion).
#[cfg(feature = "worker-ipc")]
pub mod media_source_worker;
#[cfg(feature = "test-support")]
pub mod owned_process_test_child;
#[cfg(feature = "source-adapter-test-support")]
pub mod source_sdk_fixture_worker;
