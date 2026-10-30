//! Compatibility names for the shared Native execution startup; no second implementation.
pub use super::native_execution_startup::{
    NativeExecutionFailure as ProposalExecutionFailure,
    NativeExecutionStartError as ProposalExecutionStartError,
    PreparedNativeExecution as PreparedProposalExecution,
    prepare_native_execution as prepare_proposal_execution,
    prepare_native_execution_with_notifier_wrapper as prepare_proposal_execution_with_notifier_wrapper,
};
