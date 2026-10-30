//! Shared controller construction for a new document and an existing scoped frame.
use super::super::{
    ProposalDocumentSession, RegistryProposalHost, ReviewBindings, ReviewController, ReviewSession,
    pending_controller::PendingReviewController,
    pending_host::{PendingHostInput, PendingRegistryProposalHost},
};
use super::FactoryCause;
use kinekino_player::application::api::RequestContext;
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::AsyncSdkPump;
use meari::application::plugins::*;
use std::{cell::RefCell, rc::Rc};

pub(crate) struct SessionAssemblyInput {
    pub registry: Rc<RefCell<PluginRegistry>>,
    pub registration: RegistrationHandle,
    pub scope: PluginScope,
    pub context: Rc<RefCell<RequestContext>>,
    pub namespace: String,
    pub limit: usize,
    pub review: ReviewSession,
    pub bindings: ReviewBindings,
    pub pump: Option<AsyncSdkPump>,
    pub local_signatures:
        std::collections::BTreeMap<String, meari::domain::host_callbacks::CallbackSignature>,
}

pub(crate) struct SessionAssemblyFailure {
    pub cause: FactoryCause,
    pub partial: Option<Box<ProposalDocumentSession>>,
}
impl From<FactoryCause> for SessionAssemblyFailure {
    fn from(cause: FactoryCause) -> Self {
        Self {
            cause,
            partial: None,
        }
    }
}

pub(crate) fn create_session(
    input: SessionAssemblyInput,
) -> Result<ProposalDocumentSession, SessionAssemblyFailure> {
    let created = if let Some(pump) = input.pump {
        let (output_reviews, raw_reviews) = {
            let current = input.registry.try_borrow().map_err(|_| {
                FactoryCause::Plugin(PluginError::new(
                    PluginErrorCode::Busy,
                    "Output review registration is borrowed",
                ))
            })?;
            let manifest = current
                .manifest(&input.registration)
                .map_err(FactoryCause::Plugin)?;
            let output =
                kinekino_player::assembly::meari::proposal_manifest::OUTPUT_REVIEW_CAPABILITIES
                    .iter()
                    .all(|name| manifest.capabilities.contains_key(*name));
            let raw = kinekino_player::assembly::meari::proposal_manifest::RAW_REVIEW_CAPABILITIES
                .iter()
                .all(|name| manifest.capabilities.contains_key(*name));
            (output, raw)
        };
        let host = PendingRegistryProposalHost::new(PendingHostInput {
            registry: input.registry.clone(),
            registration: input.registration,
            scope: input.scope,
            context: input.context,
            namespace: input.namespace,
            pump,
            limit: input.limit,
        })
        .map_err(FactoryCause::Plugin)?;
        let mut controller =
            PendingReviewController::new(host, input.review).map_err(FactoryCause::View)?;
        if output_reviews {
            controller
                .enable_output_reviews(Box::new(
                    kinekino_player::infrastructure::proposals::output_review_ids::next_decision_id,
                ))
                .map_err(|error| {
                    FactoryCause::Plugin(PluginError::new(
                        PluginErrorCode::Adapter,
                        format!("Output review activation failed: {error:?}"),
                    ))
                })?;
        }
        if raw_reviews {
            controller
                .enable_raw_reviews(Box::new(
                    kinekino_player::infrastructure::proposals::output_review_ids::next_decision_id,
                ))
                .map_err(|error| {
                    FactoryCause::Plugin(PluginError::new(
                        PluginErrorCode::Adapter,
                        format!("Raw review activation failed: {error:?}"),
                    ))
                })?;
        }
        ProposalDocumentSession::new_pending_retaining_with_local_callbacks(
            input.registry,
            controller,
            input.bindings,
            input.local_signatures,
        )
    } else {
        let host = RegistryProposalHost::new(
            input.registry.clone(),
            input.registration,
            input.scope,
            input.context,
            input.namespace,
        )
        .map_err(FactoryCause::Plugin)?;
        ProposalDocumentSession::new_retaining_with_local_callbacks(
            input.registry,
            ReviewController::new(host, input.review),
            input.bindings,
            input.local_signatures,
        )
    };
    created.map_err(|failure| SessionAssemblyFailure {
        cause: FactoryCause::SessionConstruction(failure.error),
        partial: failure.partial,
    })
}
