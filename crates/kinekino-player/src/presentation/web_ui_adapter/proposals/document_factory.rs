//! Unregistered product composition using existing Native document APIs only.
use crate::application::api::ProposalClient;
#[path = "document_factory/session_assembly.rs"]
pub(super) mod session_assembly;
use super::correction_panel::{
    CorrectionBudget, CorrectionLabels, CorrectionPreparationError, CorrectionSelection,
    prepare_correction,
};
use super::frame_composition::{FrameComposition, FrameProjection};
use super::{
    ProposalDocumentSession, ReviewSession, ViewError,
    panel_model::{PanelBudget, PanelLabels, PanelPreparationError, PanelSelection, prepare_panel},
};
use kinekino_media_contracts::errors::ApiError;
use kinekino_player::domain::proposals::{ProposalTask, StoredProposal};
use kinekino_player::{
    application::{
        api::{ApplicationClientHandle, RequestContext},
        proposals::InspectProposal,
    },
    domain::api_values::DecimalU64,
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::AsyncSdkPump;
use meari::{
    application::{
        assets::VerifiedAssets,
        plugins::*,
        source_modules::{ModuleDiagnostic, ModuleProgram},
    },
    domain::{
        Value,
        build::{BuildPath, ContentHash, TargetKind},
    },
};
use meari_native::{domain::Viewport, infrastructure::document::InteractiveDocument};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

/// Implemented by the SDK composition adapter around its parent registration lease.
/// No SDK/client/store is created by this presentation port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParentCloseProgress {
    Complete,
    TerminalAckPending,
    WorkerExitPending,
}

pub trait ParentProposalLease {
    fn registration(&self) -> Result<&RegistrationHandle, PluginError>;
    fn close(&mut self) -> Result<(), PluginError>;
    fn poll_close(&mut self) -> Result<ParentCloseProgress, PluginError> {
        self.close()?;
        Ok(ParentCloseProgress::Complete)
    }
    /// Explicit async assembly opts in; legacy Immediate leases keep their existing contract.
    fn async_pump(&self) -> Option<AsyncSdkPump> {
        None
    }
    fn unbind_for_retry(&mut self) -> Result<(), PluginError> {
        self.close()
    }
    fn close_error(&self) -> Option<&ApiError> {
        None
    }
    fn close_progress(
        &self,
    ) -> Option<&kinekino_player::application::sdk_execution::ExecutionCloseProgress> {
        None
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelSelectionMode {
    Item,
    DocumentRepair,
    ManualRepair,
}
pub struct ExplicitPanelSelection {
    pub proposal: InspectProposal,
    pub source_id: String,
    pub source_revision: DecimalU64,
    pub source_sha256: String,
    pub item_index: Option<usize>,
    pub mode: PanelSelectionMode,
}
pub struct PanelModuleInput {
    pub entry: BuildPath,
    pub inventory: BTreeMap<BuildPath, String>,
    /// Must equal the actual session-derived projection; not an unchecked renderer fallback.
    pub initial_props: BTreeMap<String, Value>,
    pub instance_namespace: String,
    pub epoch: u64,
    pub source_generation: u64,
    pub css: String,
    pub viewport: Viewport,
    pub assets: Option<VerifiedAssets>,
}
pub struct PanelFactoryInput {
    pub client: ApplicationClientHandle,
    pub context: RequestContext,
    pub selection: ExplicitPanelSelection,
    pub modules: PanelModuleInput,
    pub labels: PanelLabels,
    pub budget: PanelBudget,
    pub pins: super::IntentPins,
    pub registry_pending: usize,
    pub request_namespace: String,
}
#[derive(Debug)]
pub enum FactoryCause {
    Sdk(ApiError),
    View(ViewError),
    Panel(PanelPreparationError),
    Module(ModuleDiagnostic),
    Plugin(PluginError),
    SessionConstruction(PluginError),
    Renderer(String),
    Input(&'static str),
}
#[derive(Default, Debug, PartialEq, Eq)]
pub enum CallbackCleanupEvidence {
    #[default]
    NotCreated,
    Unconfirmed,
    Active,
    NativeCloseCalled,
    SessionCloseConfirmed,
    NativeAndSessionClosed,
}
#[derive(Default, Debug)]
pub struct LocalCleanupProgress {
    pub callbacks: CallbackCleanupEvidence,
    pub session_closed: bool,
    pub parent_closed: bool,
}
/// Retains partial owners on error. This is not a successful Native CloseReceipt.
pub struct PanelFactoryFailure<L: ParentProposalLease> {
    pub cause: FactoryCause,
    pub cleanup_error: Option<PluginError>,
    pub cleanup: LocalCleanupProgress,
    pub registry: Option<Rc<RefCell<PluginRegistry>>>,
    session: Option<ProposalDocumentSession>,
    parent: Option<L>,
    parent_unbound: bool,
}
impl<L: ParentProposalLease> PanelFactoryFailure<L> {
    pub fn parent_close_progress(
        &self,
    ) -> Option<&kinekino_player::application::sdk_execution::ExecutionCloseProgress> {
        self.parent
            .as_ref()
            .and_then(ParentProposalLease::close_progress)
    }
    pub fn parent_close_error(&self) -> Option<&ApiError> {
        self.parent
            .as_ref()
            .and_then(ParentProposalLease::close_error)
    }
    pub(super) fn before_mount(cause: FactoryCause) -> Box<Self> {
        Box::new(Self {
            cause,
            cleanup_error: None,
            cleanup: LocalCleanupProgress::default(),
            registry: None,
            session: None,
            parent: None,
            parent_unbound: false,
        })
    }
    fn retry_local_cleanup(&mut self) -> Result<(), PluginError> {
        self.cleanup_error = None;
        if !self.cleanup.session_closed {
            if let Some(session) = &mut self.session {
                if let Err(error) = session.close() {
                    self.cleanup_error = Some(error.clone());
                    return Err(error);
                }
                // This certifies session revocation, not an unobserved Native close call.
                self.cleanup.callbacks =
                    if self.cleanup.callbacks == CallbackCleanupEvidence::NativeCloseCalled {
                        CallbackCleanupEvidence::NativeAndSessionClosed
                    } else {
                        CallbackCleanupEvidence::SessionCloseConfirmed
                    };
            }
            self.cleanup.session_closed = true;
        }
        Ok(())
    }
    pub fn retry_cleanup(&mut self) -> Result<(), PluginError> {
        self.retry_local_cleanup()?;
        if !self.cleanup.parent_closed {
            if let Some(parent) = &mut self.parent
                && let Err(error) = parent.close()
            {
                self.cleanup_error = Some(error.clone());
                return Err(error);
            }
            self.cleanup.parent_closed = true;
        }
        Ok(())
    }
    pub fn poll_cleanup(&mut self) -> Result<ParentCloseProgress, PluginError> {
        self.retry_local_cleanup()?;
        if !self.cleanup.parent_closed {
            if let Some(parent) = &mut self.parent {
                let progress = parent.poll_close()?;
                if progress != ParentCloseProgress::Complete {
                    return Ok(progress);
                }
            }
            self.cleanup.parent_closed = true;
        }
        Ok(ParentCloseProgress::Complete)
    }
    pub fn retry_unbind(&mut self) -> Result<(), PluginError> {
        if self.cleanup.parent_closed {
            return Err(PluginError::new(
                PluginErrorCode::Stale,
                "Final parent close cannot be recovered as a live startup",
            ));
        }
        self.retry_local_cleanup()?;
        if !self.parent_unbound {
            if let Some(parent) = &mut self.parent {
                parent.unbind_for_retry()?;
            }
            self.parent_unbound = true;
        }
        Ok(())
    }
    pub fn take_unbound_parent(&mut self) -> Option<L> {
        if self.parent_unbound && self.cleanup.session_closed {
            self.parent.take()
        } else {
            None
        }
    }
}
pub struct MountedProposalPanel<L: ParentProposalLease> {
    pub(super) frame: Option<FrameProjection>,
    pub document: InteractiveDocument,
    pub session: ProposalDocumentSession,
    pub registry: Rc<RefCell<PluginRegistry>>,
    registry_pending: usize,
    budget: PanelBudget,
    pub context: Rc<RefCell<RequestContext>>,
    pub selection: ExplicitPanelSelection,
    pub inventory: BTreeMap<BuildPath, String>,
    pub cleanup: LocalCleanupProgress,
    parent: L,
}
/// Ownership transfer to the public Native host adapter; no second document or registry.
pub struct PanelHostParts<L: ParentProposalLease> {
    pub frame: Option<FrameProjection>,
    pub document: InteractiveDocument,
    pub session: ProposalDocumentSession,
    pub parent: L,
    pub registry: Rc<RefCell<PluginRegistry>>,
    pub(super) registry_pending: usize,
    pub(super) budget: PanelBudget,
    pub context: Rc<RefCell<RequestContext>>,
    pub selection: ExplicitPanelSelection,
    pub inventory: BTreeMap<BuildPath, String>,
    pub cleanup: LocalCleanupProgress,
}
impl<L: ParentProposalLease> MountedProposalPanel<L> {
    pub fn parent_close_error(&self) -> Option<&ApiError> {
        self.parent.close_error()
    }
    pub fn into_host_parts(self) -> PanelHostParts<L> {
        PanelHostParts {
            frame: self.frame,
            document: self.document,
            session: self.session,
            parent: self.parent,
            registry: self.registry,
            registry_pending: self.registry_pending,
            budget: self.budget,
            context: self.context,
            selection: self.selection,
            inventory: self.inventory,
            cleanup: self.cleanup,
        }
    }
    /// Explicit local teardown only. Driver removal/forced destruction needs the future host port.
    pub fn close(&mut self) -> Result<(), PluginError> {
        if self.cleanup.callbacks == CallbackCleanupEvidence::Active {
            self.document.close_callbacks();
            self.cleanup.callbacks = CallbackCleanupEvidence::NativeCloseCalled;
        }
        if !self.cleanup.session_closed {
            self.session.close()?;
            self.cleanup.session_closed = true;
            self.cleanup.callbacks = CallbackCleanupEvidence::NativeAndSessionClosed;
        }
        if !self.cleanup.parent_closed {
            self.parent.close()?;
            self.cleanup.parent_closed = true;
        }
        Ok(())
    }
}

/// The caller registers its SDK-owned parent lease in the supplied actual registry/scope/context.
pub fn mount_proposal_panel<L: ParentProposalLease>(
    input: PanelFactoryInput,
    register_parent: impl FnOnce(
        ApplicationClientHandle,
        Rc<RefCell<PluginRegistry>>,
        &PluginScope,
        Rc<RefCell<RequestContext>>,
    ) -> Result<L, PluginError>,
) -> Result<MountedProposalPanel<L>, Box<PanelFactoryFailure<L>>> {
    let fail = PanelFactoryFailure::before_mount;
    let stored = input
        .client
        .inspect_proposal(input.context.clone(), input.selection.proposal.clone())
        .map_err(|error| fail(FactoryCause::Sdk(error)))?;
    let revision = stored.document.draft_revision;
    mount_inspected_panel(input, stored, revision, None, None, None, register_parent)
}

/// SDK-IO-free assembly candidate. Initial execution/ACK handoff remains a separate port.
pub(super) fn mount_inspected_panel<L: ParentProposalLease>(
    mut input: PanelFactoryInput,
    stored: StoredProposal,
    expected_revision: DecimalU64,
    correction_labels: Option<CorrectionLabels>,
    initial_repair: Option<&kinekino_player::domain::proposals::DocumentTextRef>,
    frame: Option<FrameComposition>,
    register_parent: impl FnOnce(
        ApplicationClientHandle,
        Rc<RefCell<PluginRegistry>>,
        &PluginScope,
        Rc<RefCell<RequestContext>>,
    ) -> Result<L, PluginError>,
) -> Result<MountedProposalPanel<L>, Box<PanelFactoryFailure<L>>> {
    let fail = PanelFactoryFailure::before_mount;
    stored
        .validate()
        .map_err(|_| fail(FactoryCause::View(ViewError::InvalidSnapshot)))?;
    if stored.document.proposal_id != input.selection.proposal.proposal_id
        || stored.document.draft_revision != expected_revision
    {
        return Err(fail(FactoryCause::View(ViewError::Stale)));
    }
    match input.selection.mode {
        PanelSelectionMode::ManualRepair
            if input.selection.item_index.is_none()
                && stored.document.draft.is_none()
                && stored.document.state
                    == kinekino_player::domain::proposals::DraftState::NeedsRepair => {}
        PanelSelectionMode::Item
            if input.selection.item_index.is_some() && stored.document.draft.is_some() => {}
        PanelSelectionMode::DocumentRepair
            if input.selection.item_index.is_none()
                && stored.document.draft.is_none()
                && stored.document.generated.task == ProposalTask::Correction
                && stored.document.source.as_document().is_some() => {}
        _ => return Err(fail(FactoryCause::View(ViewError::InvalidSelection))),
    }
    if initial_repair.is_some() && input.selection.mode != PanelSelectionMode::DocumentRepair {
        return Err(fail(FactoryCause::View(ViewError::InvalidSelection)));
    }
    if stored.document.source.source_id() != input.selection.source_id
        || stored.document.source.source_revision() != input.selection.source_revision
        || stored.document.source.source_sha256() != input.selection.source_sha256
    {
        return Err(fail(FactoryCause::View(ViewError::Stale)));
    }
    if input.modules.source_generation == 0
        || input.modules.instance_namespace.is_empty()
        || input.request_namespace.is_empty()
    {
        return Err(fail(FactoryCause::Input(
            "Explicit positive generation and namespaces are required",
        )));
    }
    let graph = meari::parse_module_graph(&input.modules.entry, &input.modules.inventory)
        .map_err(|error| fail(FactoryCause::Module(error)))?;
    let (graph, frame) = match frame {
        Some(frame) => {
            let (graph, projection) = frame
                .prepare(graph, &mut input.modules.inventory)
                .map_err(|error| fail(FactoryCause::Module(error)))?;
            (graph, Some(projection))
        }
        None => (graph, None),
    };
    let provider = ContentHash::of(
        &serde_json::to_vec(&input.modules.inventory)
            .map_err(|_| fail(FactoryCause::Input("Module inventory encoding failed")))?,
    );
    let program = if let Some(frame) = &frame {
        frame.mount(
            graph,
            input.modules.initial_props.clone(),
            input.modules.instance_namespace,
            input.modules.epoch,
        )
    } else {
        ModuleProgram::mount(
            graph,
            input.modules.initial_props.clone(),
            input.modules.instance_namespace,
            input.modules.epoch,
        )
    }
    .map_err(|error| fail(FactoryCause::Module(error)))?;
    // This is the actual mounted root identity, exactly the one Native passes to its factory.
    let scope = PluginScope {
        instance: program
            .project()
            .map_err(|error| fail(FactoryCause::Module(error)))?
            .instances
            .get("root")
            .ok_or_else(|| fail(FactoryCause::Input("Mounted root is missing")))?
            .instance
            .clone(),
        source_generation: input.modules.source_generation,
    };
    let mut review = ReviewSession::new(scope.clone(), stored)
        .map_err(|error| fail(FactoryCause::View(error)))?;
    if let Some(reference) = initial_repair {
        CorrectionSelection::repair(&review, review.scope())
            .map_err(|error| fail(FactoryCause::View(error)))?;
        let source = review
            .stored()
            .document
            .source
            .as_document()
            .ok_or_else(|| fail(FactoryCause::View(ViewError::InvalidSelection)))?;
        let position = source
            .texts
            .iter()
            .position(|text| source.text_reference(&text.inner_pointer).as_ref() == Some(reference))
            .ok_or_else(|| fail(FactoryCause::View(ViewError::InvalidSelection)))?;
        let review_scope = review.scope().clone();
        review
            .choose_document_text(&review_scope, (position + 1).to_string())
            .map_err(|error| fail(FactoryCause::View(error)))?;
    }
    let budget = PanelBudget {
        entries: input.budget.entries,
        text_bytes: input.budget.text_bytes,
    };
    let mut prepared = if input.selection.mode == PanelSelectionMode::ManualRepair {
        super::repair_panel::prepare_repair_panel(
            &review,
            review.scope(),
            input.labels,
            PanelBudget {
                entries: budget.entries,
                text_bytes: budget.text_bytes,
            },
            input.pins,
        )
        .map_err(|error| fail(FactoryCause::Panel(error)))?
    } else if review.stored().document.generated.task == ProposalTask::Correction {
        let labels = correction_labels
            .ok_or_else(|| fail(FactoryCause::Input("Explicit correction labels required")))?;
        let selected = if review.text_fields().is_empty() {
            CorrectionSelection::repair(&review, review.scope())
        } else {
            CorrectionSelection::at(
                &review,
                review.scope(),
                input
                    .selection
                    .item_index
                    .ok_or_else(|| fail(FactoryCause::View(ViewError::InvalidSelection)))?,
            )
        }
        .map_err(|error| fail(FactoryCause::View(error)))?;
        let prepared = prepare_correction(
            &review,
            review.scope(),
            &selected,
            labels,
            CorrectionBudget {
                entries: input.budget.entries,
                text_bytes: input.budget.text_bytes,
            },
            input.pins,
        )
        .map_err(|error| {
            fail(match error {
                CorrectionPreparationError::View(error) => FactoryCause::View(error),
                CorrectionPreparationError::Queue(error) => {
                    FactoryCause::Panel(PanelPreparationError::Queue(error))
                }
            })
        })?;
        super::panel_model::PreparedPanel {
            props: prepared.props,
            bindings: prepared.bindings,
        }
    } else {
        let selected = PanelSelection::at(
            &review,
            review.scope(),
            input
                .selection
                .item_index
                .ok_or_else(|| fail(FactoryCause::View(ViewError::InvalidSelection)))?,
        )
        .map_err(|error| fail(FactoryCause::View(error)))?;
        prepare_panel(
            &review,
            review.scope(),
            &selected,
            input.labels,
            input.budget,
            input.pins,
        )
        .map_err(|error| fail(FactoryCause::Panel(error)))?
    };
    if prepared.props != input.modules.initial_props {
        return Err(fail(FactoryCause::Input(
            "Initial props differ from the explicitly selected canonical panel",
        )));
    }
    if let Some(frame) = &frame {
        frame
            .bind_inspector(&mut prepared.bindings)
            .map_err(|error| fail(FactoryCause::Panel(PanelPreparationError::Queue(error))))?;
    }
    let registry = Rc::new(RefCell::new(
        PluginRegistry::new(
            provider,
            TargetKind::Native,
            scope.clone(),
            input.registry_pending,
        )
        .map_err(|error| fail(FactoryCause::Plugin(error)))?,
    ));
    if frame
        .as_ref()
        .is_some_and(|frame| frame.raw_review_enabled())
    {
        let node = prepared
            .bindings
            .queue
            .try_borrow_mut()
            .map_err(|_| {
                fail(FactoryCause::Plugin(PluginError::new(
                    PluginErrorCode::Busy,
                    "Original raw review intent queue borrowed",
                )))
            })?
            .bind_node()
            .map_err(|error| fail(FactoryCause::Panel(PanelPreparationError::Queue(error))))?;
        super::raw_review_panel::bind_callbacks(&node, &mut prepared.bindings.bindings);
    }
    let context = Rc::new(RefCell::new(input.context));
    let mut parent = None;
    let mut session = None;
    let mut callback_cause = None;
    let document = InteractiveDocument::from_module_program_with_callbacks(
        program,
        &input.modules.css,
        input.modules.viewport,
        input.modules.assets,
        input.modules.source_generation,
        |actual_scope| {
            let result = (|| -> Result<_, FactoryCause> {
                if actual_scope != scope {
                    return Err(FactoryCause::View(ViewError::Stale));
                }
                parent = Some(
                    register_parent(
                        input.client,
                        registry.clone(),
                        &actual_scope,
                        context.clone(),
                    )
                    .map_err(FactoryCause::Plugin)?,
                );
                let registration = parent
                    .as_ref()
                    .expect("parent assigned")
                    .registration()
                    .map_err(FactoryCause::Plugin)?
                    .clone();
                let created =
                    session_assembly::create_session(session_assembly::SessionAssemblyInput {
                        registry: registry.clone(),
                        registration,
                        scope: actual_scope,
                        context: context.clone(),
                        namespace: input.request_namespace,
                        limit: input.registry_pending,
                        review,
                        bindings: prepared.bindings,
                        pump: parent.as_ref().expect("parent assigned").async_pump(),
                        local_signatures: BTreeMap::new(),
                    });
                match created {
                    Ok(created) => session = Some(created),
                    Err(mut failure) => {
                        session = failure.partial.take().map(|partial| *partial);
                        return Err(failure.cause);
                    }
                }
                session
                    .as_ref()
                    .expect("session assigned")
                    .export_callbacks()
                    .map_err(FactoryCause::Plugin)
            })();
            match result {
                Ok(export) => Ok(export),
                Err(error) => {
                    callback_cause = Some(error);
                    Err("Product document factory failed; typed cause retained by parent".into())
                }
            }
        },
    );
    match document {
        Ok(document) => Ok(MountedProposalPanel {
            frame,
            document,
            session: session.expect("successful export owns a session"),
            parent: parent.expect("successful export owns a parent lease"),
            registry,
            registry_pending: input.registry_pending,
            budget,
            context,
            selection: input.selection,
            inventory: input.modules.inventory,
            cleanup: LocalCleanupProgress {
                callbacks: CallbackCleanupEvidence::Active,
                ..Default::default()
            },
        }),
        Err(error) => {
            let cause = callback_cause.unwrap_or(FactoryCause::Renderer(error));
            let callbacks = if session.is_some() {
                CallbackCleanupEvidence::Unconfirmed
            } else {
                CallbackCleanupEvidence::NotCreated
            };
            let mut failure = PanelFactoryFailure {
                cause,
                cleanup_error: None,
                cleanup: LocalCleanupProgress {
                    callbacks,
                    ..Default::default()
                },
                registry: Some(registry),
                session,
                parent,
                parent_unbound: false,
            };
            // Preserve failure and partially cleaned owners; caller can retry after registry borrows end.
            // Async parent failure must retain its exclusive worker/registration until
            // the explicit quiescent-unbind recovery contract is available.
            if !failure
                .parent
                .as_ref()
                .is_some_and(|parent| parent.async_pump().is_some())
            {
                let _ = failure.retry_cleanup();
            }
            Err(Box::new(failure))
        }
    }
}
