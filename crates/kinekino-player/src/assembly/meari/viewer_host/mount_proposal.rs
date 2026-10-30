//! One live-task composition over the original document, SDK execution and scoped lease.
use super::*;
use crate::{
    application::{
        api::RequestContext,
        launch::native_proposal::{NativeProposalSelection, NativeProposalTarget},
        sdk_execution::ExecutionHandle,
    },
    assembly::meari::scoped_proposal_session::{ScopedProposalLease, bind_scoped_task},
    domain::proposals::StoredProposal,
    presentation::web_ui_adapter::{
        player_shell::document::product_frame::{ProductFrameBlueprint, StudioTask},
        proposals::{
            bootstrap::{BootstrapPanel, BootstrapSelection},
            card_selection_queue::{CARD_SELECTION_BINDING, CardSelectionQueue},
            frame_composition::FrameComposition,
            product_window::{
                ProductWindowLabels, product_close_policy, product_codec_budget,
                product_intent_pins, product_modules_for_panel, product_panel_budget,
            },
            scoped_task_panel::{ScopedPanelFailure, ScopedPanelInput},
            window_bridge::{ProposalWindowBridge, ScopedBridgeMount, ScopedBridgeMountFailure},
        },
    },
};
use meari::{
    application::plugins::*,
    domain::{Value, build::TargetKind, host_callbacks::CallbackSignature},
};
use std::{cell::Cell, collections::BTreeMap, num::NonZeroU64};

pub(super) struct ProductProposalMountInput {
    pub document: NativeDocumentHandle,
    pub turn: HostTurn,
    pub frame: ProductFrameBlueprint,
    pub selected: NativeProposalSelection,
    pub stored: StoredProposal,
    pub client: ApplicationClientHandle,
    pub execution: ExecutionHandle,
    pub context: RequestContext,
    pub namespace: String,
    pub generation: NonZeroU64,
    pub labels: ProductWindowLabels,
    pub queue: Option<Rc<RefCell<CardSelectionQueue>>>,
    pub navigation: Option<Rc<RefCell<super::source_navigation::NavigationIntents>>>,
}
pub(super) enum ProductProposalMountPartial {
    Session(Box<ScopedPanelFailure<ScopedProposalLease>>),
    Mount(Box<ScopedBridgeMountFailure<ScopedProposalLease>>),
}
#[must_use]
pub(super) struct ProductProposalMountFailure {
    pub input: ProductProposalMountInput,
    pub diagnostic: String,
    pub partial: Option<ProductProposalMountPartial>,
}
impl ProductProposalMountFailure {
    pub fn poll_release(&mut self) -> Result<bool, PluginError> {
        match &mut self.partial {
            Some(ProductProposalMountPartial::Session(failure)) => match failure.retry_unbind() {
                Ok(()) => Ok(true),
                Err(_)
                    if failure
                        .session
                        .as_ref()
                        .is_some_and(|session| session.close_waiting()) =>
                {
                    Ok(false)
                }
                Err(error) => Err(error),
            },
            Some(ProductProposalMountPartial::Mount(failure)) => match failure.input.panel.unbind()
            {
                Ok(()) => Ok(true),
                Err(_) if failure.input.panel.session.close_waiting() => Ok(false),
                Err(error) => Err(error),
            },
            None => Ok(true),
        }
    }
}
#[must_use = "Commit success must return its exact synchronous receipt even if proof issuance fails"]
pub(super) struct MountedProposal {
    pub bridge: ActiveCardBridge,
    pub receipt: HostCommitReceipt,
    pub proof: Option<RetainedCommitProof>,
    pub proof_error: Option<Arc<HostAccessError>>,
}
pub(super) fn task(selected: &NativeProposalSelection) -> Result<StudioTask, String> {
    match &selected.target {
        NativeProposalTarget::ManualRepair { task } => Ok(match task {
            crate::domain::proposals::ProposalTask::Cards => StudioTask::Card,
            crate::domain::proposals::ProposalTask::Highlights => StudioTask::Highlight,
            crate::domain::proposals::ProposalTask::Correction => StudioTask::Correction,
        }),
        NativeProposalTarget::Card { .. } => Ok(StudioTask::Card),
        NativeProposalTarget::Highlight { .. } => Ok(StudioTask::Highlight),
        NativeProposalTarget::CueCorrection { .. }
        | NativeProposalTarget::DocumentCorrection { .. } => Ok(StudioTask::Correction),
        NativeProposalTarget::DocumentRepair { .. } => {
            Err("Live repair entry is not connected".into())
        }
    }
}
/// Presentation-only gating; the original SDK remains authoritative for every mutation.
pub(super) fn gate_props(props: &mut BTreeMap<String, Value>, active: bool) {
    if active {
        return;
    }
    for key in [
        "canEdit",
        "canSave",
        "canAccept",
        "outputReviewCanRecord",
        "outputReviewCanRetry",
    ] {
        if let Some(Value::Bool(value)) = props.get_mut(key) {
            *value = false;
        }
    }
    for key in [
        "inputDisabled",
        "saveDisabled",
        "acceptDisabled",
        "refreshDisabled",
        "selectionDisabled",
        "startDisabled",
        "outputReviewFieldsDisabled",
        "outputReviewBusy",
    ] {
        if let Some(Value::Bool(value)) = props.get_mut(key) {
            *value = true;
        }
    }
}

/// None preserves the old Card mount contract. Some gates a new explicit live entry until
/// its retained frame is observed; the closure owns its original closed StudioFrame.
pub(super) fn mount_current_proposal(
    input: ProductProposalMountInput,
    entry_gate: Option<Rc<Cell<bool>>>,
) -> Result<MountedProposal, Box<ProductProposalMountFailure>> {
    let prepare = || -> Result<_, String> {
        if input.navigation.is_some() && entry_gate.is_none() {
            return Err(
                "Source navigation requires the original live entry observation gate".into(),
            );
        }
        if input.turn.remaining_callbacks != 0
            || input.document.stamp().map_err(|error| error.to_string())? != input.turn.stamp
        {
            return Err("Original proposal mount turn differs".into());
        }
        let task = task(&input.selected)?;
        if task != StudioTask::Card && input.queue.is_some() {
            return Err("Captured Card queue cannot select a live task".into());
        }
        let selected = BootstrapSelection::try_from(input.selected.clone())
            .map_err(|error| format!("{error:?}"))?;
        let panel = if matches!(
            &input.selected.target,
            NativeProposalTarget::ManualRepair { .. }
        ) {
            BootstrapPanel::ManualRepair
        } else {
            match task {
                StudioTask::Card => BootstrapPanel::Card,
                StudioTask::Highlight => BootstrapPanel::Highlight,
                StudioTask::Correction => BootstrapPanel::Correction,
            }
        };
        let (entry, mut inventory) = product_modules_for_panel(panel);
        let provider =
            meari::parse_module_graph(&entry, &inventory).map_err(|error| format!("{error:?}"))?;
        let (graph, projection) = FrameComposition::new(
            crate::domain::product::PRODUCT.display_name.into(),
            String::new(),
        )
        .prepare(provider, &mut inventory)
        .map_err(|error| format!("{error:?}"))?;
        if inventory
            .iter()
            .any(|(path, source)| input.frame.inventory.get(path) != Some(source))
            || graph
                .modules
                .iter()
                .any(|(path, module)| input.frame.graph.modules.get(path) != Some(module))
        {
            return Err("Proposal provider differs from original closed product graph".into());
        }
        let registry = Rc::new(RefCell::new(
            PluginRegistry::new(
                input.frame.graph.fingerprint()?,
                TargetKind::Native,
                input.turn.stamp.owner.clone(),
                product_codec_budget().max_pending,
            )
            .map_err(|error| error.message)?,
        ));
        Ok((task, selected, projection, registry))
    };
    let (task, selected, projection, registry) = match prepare() {
        Ok(prepared) => prepared,
        Err(diagnostic) => {
            return Err(Box::new(ProductProposalMountFailure {
                input,
                diagnostic,
                partial: None,
            }));
        }
    };
    let correction = (task == StudioTask::Correction).then(|| input.labels.correction());
    let mut local_signatures = if input.queue.is_some() {
        BTreeMap::from([(CARD_SELECTION_BINDING.into(), CallbackSignature::Text)])
    } else {
        BTreeMap::new()
    };
    if input.navigation.is_some() {
        local_signatures.insert(
            super::source_navigation::NAVIGATION_BINDING.into(),
            CallbackSignature::Text,
        );
    }
    let panel = match bind_scoped_task(
        ScopedPanelInput {
            client: input.client.clone(),
            context: input.context.clone(),
            scope: input.turn.stamp.owner.clone(),
            registry,
            selection: selected,
            labels: input.labels.bootstrap(),
            budget: product_panel_budget(),
            pins: product_intent_pins(),
            namespace: input.namespace.clone(),
            pending_limit: product_codec_budget().max_pending,
            local_signatures,
        },
        input.stored.clone(),
        correction,
        input.execution.clone(),
        product_codec_budget(),
    ) {
        Ok(panel) => panel,
        Err(failure) => {
            return Err(Box::new(ProductProposalMountFailure {
                diagnostic: format!("{:?}", failure.cause),
                input,
                partial: Some(ProductProposalMountPartial::Session(failure)),
            }));
        }
    };
    let blueprint = input.frame.clone();
    // Declarative reference only; the original session owns its bound opaque key/export.
    let navigation_reference =
        input
            .navigation
            .is_some()
            .then(|| meari::domain::host_callbacks::CallbackReference {
                binding: super::source_navigation::NAVIGATION_BINDING.into(),
                signature: CallbackSignature::Text,
            });
    let generation = input.generation;
    let labels = input.labels.panel.clone();
    let correction_labels = input.labels.clone();
    let retain = entry_gate.is_some();
    let navigation_gate = entry_gate.clone();
    let original_frame = if retain {
        None
    } else {
        Some(projection.clone())
    };
    let correction_provider: Option<crate::presentation::web_ui_adapter::proposals::window_bridge::CorrectionLabelProvider> = (task == StudioTask::Correction).then(|| {
        let provider: crate::presentation::web_ui_adapter::proposals::window_bridge::CorrectionLabelProvider = Box::new(move |_, _, _| correction_labels.correction());
        provider
    });
    let mounted = ProposalWindowBridge::mount_scoped_task(
        ScopedBridgeMount {
            handle: input.document.clone(),
            turn: input.turn.clone(),
            panel,
            frame: original_frame,
            inventory: input.frame.inventory.clone(),
            labels: Box::new(move |_, _, _| labels.clone()),
            close_policy: product_close_policy(),
            pins: product_intent_pins(),
            commit: Box::new(move |document, mut props, export| {
                if let Some(gate) = &entry_gate {
                    gate_props(&mut props, gate.get());
                    let workspace = projection
                        .compose_editor_slot(props, |key, reference, props| {
                            document.node_slot(key, reference, props)
                        })?;
                    props = projection.root_props(workspace);
                    if let Some(reference) = &navigation_reference {
                        let selected_id = match task {
                            StudioTask::Card => "CARD",
                            StudioTask::Highlight => "CLIP",
                            StudioTask::Correction => "SUB",
                        };
                        let navigation = document.node_slot(
                            "proposal-navigation",
                            projection.navigation_reference().clone(),
                            projection.navigation_props(selected_id, reference.clone()),
                        )?;
                        props.insert("navigation".into(), navigation);
                    }
                }
                let props =
                    blueprint.workspace_props(document, task.workspace(), generation, props)?;
                document.update_module_props_with_callbacks(props, export)
            }),
        },
        correction_provider,
    );
    match mounted {
        Ok((bridge, receipt)) => {
            // No fallible binding/observer access or next turn precedes this issuance.
            let (proof, proof_error) = if retain {
                let retained = {
                    #[cfg(test)]
                    {
                        super::super::viewer_native_probe::live_retained::retain(
                            &input.document,
                            &receipt,
                            super::super::viewer_native_probe::live_retained::ProofStage::Entry,
                        )
                    }
                    #[cfg(not(test))]
                    {
                        input.document.retain_commit(&receipt)
                    }
                };
                match retained {
                    Ok(proof) => (Some(proof), None),
                    Err(error) => (None, Some(Arc::new(error))),
                }
            } else {
                (None, None)
            };
            let selection = input
                .queue
                .as_ref()
                .map(|queue| CardSelectionQueue::bridge_port(queue.clone()));
            let bridge = match &input.navigation {
                Some(navigation) => {
                    bridge.with_local_intents(super::source_navigation::bridge_port(
                        navigation.clone(),
                        input.document.clone(),
                        task,
                        navigation_gate,
                        selection,
                    ))
                }
                None => match selection {
                    Some(port) => bridge.with_local_intents(port),
                    None => bridge,
                },
            };
            Ok(MountedProposal {
                bridge,
                receipt,
                proof,
                proof_error,
            })
        }
        Err(failure) => Err(Box::new(ProductProposalMountFailure {
            diagnostic: failure.error.to_string(),
            input,
            partial: Some(ProductProposalMountPartial::Mount(failure)),
        })),
    }
}

#[cfg(test)]
mod tests;
