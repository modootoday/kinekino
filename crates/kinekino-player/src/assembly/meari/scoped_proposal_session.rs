//! Scoped proposal registration borrows the original SDK execution without owning shutdown.
use super::async_proposals::AsyncProposalRegistration;
use crate::{
    application::sdk_execution::ExecutionHandle,
    domain::proposals::StoredProposal,
    presentation::web_ui_adapter::proposals::{
        correction_panel::CorrectionLabels,
        document_factory::ParentProposalLease,
        scoped_task_panel::{ScopedPanel, ScopedPanelFailure, ScopedPanelInput, bind_task_panel},
    },
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::{AsyncCodecBudget, AsyncSdkPump};
use meari::application::plugins::{PluginError, PluginRegistry, PluginScope, RegistrationHandle};
use std::{cell::RefCell, rc::Rc};

pub struct ScopedProposalLease(AsyncProposalRegistration);
impl ScopedProposalLease {
    pub(super) fn register(
        registry: Rc<RefCell<PluginRegistry>>,
        scope: &PluginScope,
        context: Rc<RefCell<crate::application::api::RequestContext>>,
        execution: ExecutionHandle,
        budget: AsyncCodecBudget,
    ) -> Result<Self, PluginError> {
        AsyncProposalRegistration::register_output_reviews(
            registry, scope, context, execution, budget,
        )
        .map(Self)
    }
}
impl ParentProposalLease for ScopedProposalLease {
    fn registration(&self) -> Result<&RegistrationHandle, PluginError> {
        self.0.handle()
    }
    fn close(&mut self) -> Result<(), PluginError> {
        self.0.unbind()
    }
    fn async_pump(&self) -> Option<AsyncSdkPump> {
        Some(self.0.pump().clone())
    }
}

/// Caller has accepted the original live inspect terminal and released the prior attachment.
/// Captured rows/current-inspect identity alone are never authority to call this entry.
pub fn bind_scoped_task(
    input: ScopedPanelInput,
    stored: StoredProposal,
    correction_labels: Option<CorrectionLabels>,
    execution: ExecutionHandle,
    budget: AsyncCodecBudget,
) -> Result<ScopedPanel<ScopedProposalLease>, Box<ScopedPanelFailure<ScopedProposalLease>>> {
    let raw_reviews = matches!(
        &input.selection.target,
        crate::presentation::web_ui_adapter::proposals::bootstrap::BootstrapTarget::ManualRepair { .. }
            | crate::presentation::web_ui_adapter::proposals::bootstrap::BootstrapTarget::Card { .. }
            | crate::presentation::web_ui_adapter::proposals::bootstrap::BootstrapTarget::Highlight { .. }
            | crate::presentation::web_ui_adapter::proposals::bootstrap::BootstrapTarget::CueCorrection { .. }
            | crate::presentation::web_ui_adapter::proposals::bootstrap::BootstrapTarget::DocumentCorrection { .. }
    );
    bind_task_panel(
        input,
        stored,
        correction_labels,
        move |_, registry, scope, context| {
            if raw_reviews {
                AsyncProposalRegistration::register_raw_reviews(
                    registry, scope, context, execution, budget,
                )
                .map(ScopedProposalLease)
            } else {
                ScopedProposalLease::register(registry, scope, context, execution, budget)
            }
        },
    )
}
