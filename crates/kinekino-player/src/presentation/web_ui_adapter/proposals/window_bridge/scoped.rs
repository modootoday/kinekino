//! Install an inspected scoped session on the original checked document turn.
use super::super::scoped_panel::ScopedCardPanel;
use super::*;

pub struct ScopedBridgeMount<L: ParentProposalLease> {
    pub handle: NativeDocumentHandle,
    pub turn: HostTurn,
    pub panel: ScopedCardPanel<L>,
    pub frame: Option<super::super::frame_composition::FrameProjection>,
    pub inventory: BTreeMap<BuildPath, String>,
    pub labels: PanelLabelProvider,
    pub close_policy: PanelClosePolicy,
    pub pins: IntentPins,
    pub commit: PanelCommitPort,
}

#[cfg(test)]
#[path = "scoped/task_tests.rs"]
mod task_tests;
#[cfg(test)]
#[path = "scoped/tests.rs"]
mod tests;

/// A rejected installation retains the original document and the complete scoped owner.
#[must_use]
pub struct ScopedBridgeMountFailure<L: ParentProposalLease> {
    pub error: PanelBridgeError,
    pub input: ScopedBridgeMount<L>,
    /// Retain the original explicit provider with a rejected mount for retry or cleanup.
    pub correction_labels: Option<CorrectionLabelProvider>,
}

impl<L: ParentProposalLease> ProposalWindowBridge<L> {
    pub fn document_handle(&self) -> NativeDocumentHandle {
        self.handle.clone()
    }
    pub fn scope_close_waiting(&self) -> bool {
        self.owners.session.close_waiting()
    }
    pub fn mount_scoped(
        input: ScopedBridgeMount<L>,
    ) -> Result<(Self, HostCommitReceipt), Box<ScopedBridgeMountFailure<L>>> {
        Self::mount_scoped_task(input, None)
    }

    /// Correction needs its explicit provider before any props commit or callback export.
    pub fn mount_scoped_task(
        mut input: ScopedBridgeMount<L>,
        correction_labels: Option<CorrectionLabelProvider>,
    ) -> Result<(Self, HostCommitReceipt), Box<ScopedBridgeMountFailure<L>>> {
        let attempt = (|| {
            if input.turn.remaining_callbacks != 0 {
                return Err(PanelBridgeError::Admission("Frame callback suffix remains"));
            }
            if input.turn.stamp.owner != input.panel.session.review().scope().view {
                return Err(PanelBridgeError::Admission("Card scope differs from frame"));
            }
            if input
                .panel
                .session
                .review()
                .stored()
                .document
                .generated
                .task
                == ProposalTask::Correction
                && correction_labels.is_none()
            {
                return Err(PanelBridgeError::Admission(
                    "Explicit correction labels are required",
                ));
            }
            if input
                .frame
                .as_ref()
                .is_some_and(|frame| frame.inspector_reference().is_some())
            {
                return Err(PanelBridgeError::Admission(
                    "Initial Card inspector must be closed",
                ));
            }
            let export = input
                .panel
                .session
                .export_callbacks()
                .map_err(PanelBridgeError::Plugin)?;
            let props = input.panel.initial_props().clone();
            let commit = &mut input.commit;
            let (_, receipt) = input
                .handle
                .try_reproject(&input.turn.stamp, |document| {
                    let props = match &input.frame {
                        Some(frame) => frame.root_props(
                            frame.compose_editor_slot(props, |key, reference, props| {
                                document.node_slot(key, reference, props)
                            })?,
                        ),
                        None => props,
                    };
                    commit(document, props, export)
                })
                .map_err(|error| match error {
                    HostUpdateError::Access(error) => PanelBridgeError::Access(error),
                    HostUpdateError::Apply(error) => PanelBridgeError::Commit(error),
                })?;
            Ok(receipt)
        })();
        let receipt = match attempt {
            Ok(receipt) => receipt,
            Err(error) => {
                return Err(Box::new(ScopedBridgeMountFailure {
                    error,
                    input,
                    correction_labels,
                }));
            }
        };
        let ScopedCardPanel {
            session,
            parent,
            metadata,
            ..
        } = input.panel;
        let owners = PanelBridgeOwners {
            frame: input.frame,
            session,
            parent,
            registry: metadata.registry,
            registry_pending: metadata.registry_pending,
            context: metadata.context,
            selection: metadata.selection,
            inventory: input.inventory,
        };
        let mut bridge = Self::from_owned_parts(
            input.handle,
            owners,
            input.labels,
            input.close_policy,
            metadata.budget,
            input.pins,
        )
        .with_commit_port(input.commit);
        if let Some(labels) = correction_labels {
            bridge = bridge.with_correction_labels(labels);
        }
        bridge.owners.session.projection_committed();
        bridge.observations.projection_commits = 1;
        Ok((bridge, receipt))
    }

    /// Navigation releases only this panel registration, never its parent SDK execution.
    pub fn unbind_scope(&mut self) -> Result<(), PluginError> {
        if !self.owned_session_closed {
            self.owners.session.close()?;
            self.owned_session_closed = true;
        }
        if !self.owned_parent_closed {
            self.owners.parent.unbind_for_retry()?;
            self.owned_parent_closed = true;
        }
        Ok(())
    }
}
