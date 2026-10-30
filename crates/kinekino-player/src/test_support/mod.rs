//! Test-only fixture generation; never a product layer or an evidence verifier.
//! Shared by crate unit tests; integration-test fixtures remain separate.
pub(crate) mod historical_fixture;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub(crate) mod native_window_host;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub(crate) mod native_window_supervision;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub(crate) mod review_package;
