//! Bind an explicitly inspected live item to the original frame; no worker or authority is created.
use super::{
    IntentPins, ProposalDocumentSession, ReviewSession,
    bootstrap::{
        BootstrapError, BootstrapLabels, BootstrapSelection, BootstrapTarget, project_initial,
    },
    correction_panel::{CorrectionLabels, CorrectionSelection, prepare_correction},
    document_factory::{
        FactoryCause, ParentProposalLease,
        session_assembly::{SessionAssemblyInput, create_session},
    },
    panel_model::{PanelBudget, PanelPreparationError, PanelSelection, prepare_panel},
};
use kinekino_player::{
    application::api::{ApplicationClientHandle, RequestContext},
    domain::proposals::StoredProposal,
};
use meari::{application::plugins::*, domain::Value};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

pub struct ScopedPanelInput {
    pub client: ApplicationClientHandle,
    pub context: RequestContext,
    /// Must come from the original live document, never from a source callback payload.
    pub scope: PluginScope,
    pub registry: Rc<RefCell<PluginRegistry>>,
    pub selection: BootstrapSelection,
    pub labels: BootstrapLabels,
    pub budget: PanelBudget,
    pub pins: IntentPins,
    pub namespace: String,
    pub pending_limit: usize,
    /// Trusted frame declarations; local targets never grant SDK capabilities.
    pub local_signatures: BTreeMap<String, meari::domain::host_callbacks::CallbackSignature>,
}

#[derive(Debug)]
pub enum ScopedPanelCause {
    Selection(BootstrapError),
    Panel(PanelPreparationError),
    Session(super::ViewError),
    Plugin(PluginError),
    Assembly(FactoryCause),
    InvalidConfiguration,
    MissingCorrectionLabels,
    UnsupportedRepair,
}

#[must_use = "Retain the registration and session until explicit quiescent unbind succeeds"]
pub struct ScopedPanel<L: ParentProposalLease> {
    pub session: ProposalDocumentSession,
    pub parent: L,
    initial_props: BTreeMap<String, Value>,
    pub(super) metadata: ScopedPanelMetadata,
}
pub(super) struct ScopedPanelMetadata {
    pub registry: Rc<RefCell<PluginRegistry>>,
    pub context: Rc<RefCell<RequestContext>>,
    pub registry_pending: usize,
    pub budget: PanelBudget,
    pub selection: super::document_factory::ExplicitPanelSelection,
}
impl<L: ParentProposalLease> ScopedPanel<L> {
    pub fn initial_props(&self) -> &BTreeMap<String, Value> {
        &self.initial_props
    }
    /// Scope-only unbind; the caller still owns the original document, endpoint and SDK worker.
    pub fn unbind(&mut self) -> Result<(), PluginError> {
        self.session.close()?;
        self.parent.unbind_for_retry()
    }
}

#[must_use = "A failed bind may retain the original registration and partial session"]
pub struct ScopedPanelFailure<L: ParentProposalLease> {
    pub cause: ScopedPanelCause,
    pub parent: Option<L>,
    pub session: Option<Box<ProposalDocumentSession>>,
    pub registry: Rc<RefCell<PluginRegistry>>,
}
impl<L: ParentProposalLease> ScopedPanelFailure<L> {
    pub fn retry_unbind(&mut self) -> Result<(), PluginError> {
        if let Some(session) = &mut self.session {
            session.close()?;
        }
        if let Some(parent) = &mut self.parent {
            parent.unbind_for_retry()?;
        }
        Ok(())
    }
}

/// The host has validated an explicit live entry and consumed/ACKed its original inspect result.
/// Matching captured identity is not authority, and ACK is not proposal acceptance.
/// This prepares callbacks/props only. The caller must install them with the original checked
/// document reproject/slot API and retain this panel if that commit fails.
pub fn bind_task_panel<L: ParentProposalLease>(
    input: ScopedPanelInput,
    stored: StoredProposal,
    correction_labels: Option<CorrectionLabels>,
    register_parent: impl FnOnce(
        ApplicationClientHandle,
        Rc<RefCell<PluginRegistry>>,
        &PluginScope,
        Rc<RefCell<RequestContext>>,
    ) -> Result<L, PluginError>,
) -> Result<ScopedPanel<L>, Box<ScopedPanelFailure<L>>> {
    let registry = input.registry.clone();
    let fail = |cause| {
        Box::new(ScopedPanelFailure {
            cause,
            parent: None,
            session: None,
            registry: registry.clone(),
        })
    };
    if input.namespace.is_empty() || input.pending_limit == 0 || input.scope.source_generation == 0
    {
        return Err(fail(ScopedPanelCause::InvalidConfiguration));
    }
    {
        let current = registry.try_borrow().map_err(|_| {
            fail(ScopedPanelCause::Plugin(PluginError::new(
                PluginErrorCode::Busy,
                "Proposal frame registry is borrowed",
            )))
        })?;
        if current.scope() != &input.scope {
            return Err(fail(ScopedPanelCause::Plugin(PluginError::new(
                PluginErrorCode::Stale,
                "Proposal frame registry scope differs",
            ))));
        }
    }
    let manual_repair = matches!(
        &input.selection.target,
        BootstrapTarget::ManualRepair { .. }
    );
    let (index, correction) = match &input.selection.target {
        BootstrapTarget::ManualRepair { .. } => (0, false),
        BootstrapTarget::Card { index } | BootstrapTarget::Highlight { index } => (*index, false),
        BootstrapTarget::CueCorrection { index, .. }
        | BootstrapTarget::DocumentCorrection { index, .. } => (*index, true),
        BootstrapTarget::DocumentRepair { .. } => {
            return Err(fail(ScopedPanelCause::UnsupportedRepair));
        }
    };
    if correction && correction_labels.is_none() {
        return Err(fail(ScopedPanelCause::MissingCorrectionLabels));
    }
    let labels = input.labels.panel.clone();
    project_initial(&stored, &input.selection, input.labels)
        .map_err(|error| fail(ScopedPanelCause::Selection(error)))?;
    let review = ReviewSession::new(input.scope.clone(), stored)
        .map_err(|error| fail(ScopedPanelCause::Session(error)))?;
    let metadata_budget = PanelBudget {
        entries: input.budget.entries,
        text_bytes: input.budget.text_bytes,
    };
    let selection = super::document_factory::ExplicitPanelSelection {
        proposal: kinekino_player::application::proposals::InspectProposal {
            proposal_id: input.selection.proposal_id,
        },
        source_id: input.selection.source_id,
        source_revision: input.selection.source_revision,
        source_sha256: input.selection.source_sha256,
        item_index: (!manual_repair).then_some(index),
        mode: if manual_repair {
            super::document_factory::PanelSelectionMode::ManualRepair
        } else {
            super::document_factory::PanelSelectionMode::Item
        },
    };
    let mut prepared = if manual_repair {
        super::repair_panel::prepare_repair_panel(
            &review,
            review.scope(),
            labels,
            input.budget,
            input.pins,
        )
        .map_err(|error| fail(ScopedPanelCause::Panel(error)))?
    } else if correction {
        let selected = CorrectionSelection::at(&review, review.scope(), index)
            .map_err(|error| fail(ScopedPanelCause::Session(error)))?;
        prepare_correction(
            &review,
            review.scope(),
            &selected,
            correction_labels.expect("checked explicit correction labels"),
            input.budget,
            input.pins,
        )
        .map_err(|error| fail(ScopedPanelCause::Panel(error)))?
    } else {
        let selected = PanelSelection::at(&review, review.scope(), index)
            .map_err(|error| fail(ScopedPanelCause::Session(error)))?;
        prepare_panel(
            &review,
            review.scope(),
            &selected,
            labels,
            input.budget,
            input.pins,
        )
        .map_err(|error| fail(ScopedPanelCause::Panel(error)))?
    };
    let context = Rc::new(RefCell::new(input.context));
    let parent = register_parent(
        input.client,
        registry.clone(),
        &input.scope,
        context.clone(),
    )
    .map_err(|error| fail(ScopedPanelCause::Plugin(error)))?;
    let registration = match parent.registration() {
        Ok(handle) => handle.clone(),
        Err(error) => {
            return Err(Box::new(ScopedPanelFailure {
                cause: ScopedPanelCause::Plugin(error),
                parent: Some(parent),
                session: None,
                registry,
            }));
        }
    };
    let raw_reviews = (|| -> Result<bool, PluginError> {
        let current = registry.try_borrow().map_err(|_| {
            PluginError::new(
                PluginErrorCode::Busy,
                "Original raw review manifest borrowed",
            )
        })?;
        let manifest = current.manifest(&registration)?;
        Ok(
            kinekino_player::assembly::meari::proposal_manifest::RAW_REVIEW_CAPABILITIES
                .iter()
                .all(|name| manifest.capabilities.contains_key(*name)),
        )
    })();
    let raw_reviews = match raw_reviews {
        Ok(enabled) => enabled,
        Err(error) => {
            return Err(Box::new(ScopedPanelFailure {
                cause: ScopedPanelCause::Plugin(error),
                parent: Some(parent),
                session: None,
                registry,
            }));
        }
    };
    if raw_reviews {
        let node = (|| -> Result<_, ScopedPanelCause> {
            prepared
                .bindings
                .queue
                .try_borrow_mut()
                .map_err(|_| {
                    ScopedPanelCause::Plugin(PluginError::new(
                        PluginErrorCode::Busy,
                        "Original raw review intent queue borrowed",
                    ))
                })?
                .bind_node()
                .map_err(|error| ScopedPanelCause::Panel(PanelPreparationError::Queue(error)))
        })();
        let node = match node {
            Ok(node) => node,
            Err(cause) => {
                return Err(Box::new(ScopedPanelFailure {
                    cause,
                    parent: Some(parent),
                    session: None,
                    registry,
                }));
            }
        };
        super::raw_review_panel::bind_callbacks(&node, &mut prepared.bindings.bindings);
    }
    match create_session(SessionAssemblyInput {
        registry: registry.clone(),
        registration,
        scope: input.scope,
        context: context.clone(),
        namespace: input.namespace,
        limit: input.pending_limit,
        review,
        bindings: prepared.bindings,
        pump: parent.async_pump(),
        local_signatures: input.local_signatures,
    }) {
        Ok(session) => {
            let mut initial_props = prepared.props;
            if raw_reviews {
                super::raw_review_panel::project_component_props(
                    &mut initial_props,
                    session.raw_review(),
                    session.operation_pending(),
                    session.raw_review_retained(),
                    session.raw_recovery(),
                );
            }
            super::output_review_panel::project_component_props(
                &mut initial_props,
                session.output_review(),
                session.operation_pending(),
            );
            Ok(ScopedPanel {
                session,
                parent,
                initial_props,
                metadata: ScopedPanelMetadata {
                    registry,
                    context,
                    registry_pending: input.pending_limit,
                    budget: metadata_budget,
                    selection,
                },
            })
        }
        Err(failure) => Err(Box::new(ScopedPanelFailure {
            cause: ScopedPanelCause::Assembly(failure.cause),
            parent: Some(parent),
            session: failure.partial,
            registry,
        })),
    }
}

#[cfg(test)]
pub(super) mod tests;
