//! Card composition borrows the original execution handle; it never owns its shutdown.
pub use super::scoped_proposal_session::ScopedProposalLease;
use crate::{
    application::sdk_execution::ExecutionHandle,
    domain::proposals::StoredProposal,
    presentation::web_ui_adapter::proposals::scoped_panel::{
        ScopedCardFailure, ScopedCardInput, ScopedCardPanel, bind_card_panel,
    },
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::AsyncCodecBudget;

/// Viewing attachment must already be released after its original terminals and ACKs.
/// Failure keeps the existing bind_card_panel retention/retry contract unchanged.
pub fn bind_scoped_card(
    input: ScopedCardInput,
    stored: StoredProposal,
    execution: ExecutionHandle,
    budget: AsyncCodecBudget,
) -> Result<ScopedCardPanel<ScopedProposalLease>, Box<ScopedCardFailure<ScopedProposalLease>>> {
    bind_card_panel(input, stored, move |_, registry, scope, context| {
        ScopedProposalLease::register(registry, scope, context, execution, budget)
    })
}
