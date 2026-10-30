//! Card compatibility entry; shared scoped ownership lives in scoped_task_panel.
pub use super::scoped_task_panel::{
    ScopedPanel as ScopedCardPanel, ScopedPanelCause as ScopedCardCause,
    ScopedPanelFailure as ScopedCardFailure, ScopedPanelInput as ScopedCardInput,
};
use super::{
    bootstrap::{BootstrapError, BootstrapTarget},
    document_factory::ParentProposalLease,
    scoped_task_panel::bind_task_panel,
};
#[cfg(test)]
use super::{
    bootstrap::{BootstrapLabels, BootstrapSelection},
    panel_model::PanelBudget,
};
use kinekino_player::{
    application::api::{ApplicationClientHandle, RequestContext},
    domain::proposals::StoredProposal,
};
use meari::application::plugins::*;
#[cfg(test)]
use meari::domain::Value;
#[cfg(test)]
use std::collections::BTreeMap;
use std::{cell::RefCell, rc::Rc};

/// The original Card entry remains Card-only; inspection does not grant live authority.
pub fn bind_card_panel<L: ParentProposalLease>(
    input: ScopedCardInput,
    stored: StoredProposal,
    register_parent: impl FnOnce(
        ApplicationClientHandle,
        Rc<RefCell<PluginRegistry>>,
        &PluginScope,
        Rc<RefCell<RequestContext>>,
    ) -> Result<L, PluginError>,
) -> Result<ScopedCardPanel<L>, Box<ScopedCardFailure<L>>> {
    if !matches!(&input.selection.target, BootstrapTarget::Card { .. }) {
        return Err(Box::new(ScopedCardFailure {
            cause: ScopedCardCause::Selection(BootstrapError::TargetMismatch),
            parent: None,
            session: None,
            registry: input.registry,
        }));
    }
    bind_task_panel(input, stored, None, register_parent)
}

#[cfg(test)]
pub(super) mod tests;
