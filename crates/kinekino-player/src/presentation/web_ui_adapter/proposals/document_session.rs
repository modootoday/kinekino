#[path = "document_session/local_composition.rs"]
mod local_composition;
use meari::{
    application::{
        host_callbacks::{CallbackBindings, CallbackHandle, HostCallbacks},
        plugins::*,
    },
    domain::{
        build::{ContentHash, TargetKind},
        host_callbacks::{CallbackArguments, CallbackReference, CallbackSignature},
    },
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use super::callback_adapter::REVIEW_INTENT_PLUGIN_ID;
use super::document_controller::{DocumentController, InteractionController, InteractionError};
use super::pending_controller::PendingReviewController;
use super::{
    DrainReport, IntentPins, QueueBinding, QueueCallbackAdapter, RegistryProposalHost,
    ReviewController, ReviewIntents, ReviewSession, bind_review_callbacks,
};

/// Host-created node/field bindings and explicit resource budgets. No renderer authority in JSON.
pub struct ReviewBindings {
    pub queue: Rc<RefCell<ReviewIntents>>,
    pub bindings: BTreeMap<String, QueueBinding>,
    pub pins: IntentPins,
    pub targets: BTreeSet<TargetKind>,
}

struct MountedBindings {
    callbacks: HostCallbacks,
    registration: RegistrationHandle,
    queue: Rc<RefCell<ReviewIntents>>,
    closed: Arc<AtomicBool>,
    signatures: BTreeMap<String, CallbackSignature>,
    callbacks_revoked: bool,
    local_keys: BTreeMap<String, meari::application::host_callbacks::LocalActionKey>,
}
impl MountedBindings {
    fn revoke_callbacks(&mut self) -> Result<(), PluginError> {
        self.closed.store(true, Ordering::Release);
        if self.callbacks_revoked {
            return Ok(());
        }
        // The owner is invalidated before HostCallbacks' checked revision increment.
        self.callbacks_revoked = true;
        self.callbacks.invalidate()
    }
}
struct PreparedBindings {
    adapter: QueueCallbackAdapter,
    scope: PluginScope,
    queue: Rc<RefCell<ReviewIntents>>,
    closed: Arc<AtomicBool>,
    signatures: BTreeMap<String, CallbackSignature>,
    schema: ContentHash,
}

/// Owns review interaction lifetime; the parent document owns the registry and SDK registration.
pub struct ProposalDocumentSession {
    registry: Rc<RefCell<PluginRegistry>>,
    controller: DocumentController,
    mounted: Option<MountedBindings>,
    retired: Option<MountedBindings>,
    next_binding: u64,
    local_signatures: BTreeMap<String, CallbackSignature>,
}
/// Original failure plus partial owner, including any registration awaiting cleanup.
pub struct SessionConstructionFailure {
    pub error: PluginError,
    pub partial: Option<Box<ProposalDocumentSession>>,
}
impl SessionConstructionFailure {
    pub fn retry_cleanup(&mut self) -> Result<(), PluginError> {
        if let Some(partial) = &mut self.partial {
            partial.close()?;
        }
        self.partial = None;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ReprojectionReceipt {
    pub cleanup_pending: bool,
}
#[derive(Debug)]
pub enum ReprojectionError {
    Prepare(PluginError),
    Renderer(String),
}

fn borrowed() -> PluginError {
    PluginError::new(
        PluginErrorCode::Adapter,
        "Review document resource is borrowed; cleanup must be retried after dispatch",
    )
}
fn closed() -> PluginError {
    PluginError::new(
        PluginErrorCode::Stale,
        "Review document bindings are closed",
    )
}

impl ProposalDocumentSession {
    pub fn new(
        registry: Rc<RefCell<PluginRegistry>>,
        controller: ReviewController<RegistryProposalHost>,
        bindings: ReviewBindings,
    ) -> Result<Self, PluginError> {
        // Compatibility only: this cannot return a failed cleanup owner to the caller.
        // Product composition must use new_retaining and retain its failure until cleanup ends.
        Self::new_retaining(registry, controller, bindings).map_err(|failure| failure.error)
    }
    pub fn new_retaining(
        registry: Rc<RefCell<PluginRegistry>>,
        controller: ReviewController<RegistryProposalHost>,
        bindings: ReviewBindings,
    ) -> Result<Self, Box<SessionConstructionFailure>> {
        Self::new_composed_retaining(
            registry,
            DocumentController::Immediate(Box::new(controller)),
            bindings,
        )
    }
    pub fn new_pending_retaining(
        registry: Rc<RefCell<PluginRegistry>>,
        controller: PendingReviewController,
        bindings: ReviewBindings,
    ) -> Result<Self, Box<SessionConstructionFailure>> {
        Self::new_composed_retaining(
            registry,
            DocumentController::Pending(Box::new(controller)),
            bindings,
        )
    }
    fn new_composed_retaining(
        registry: Rc<RefCell<PluginRegistry>>,
        controller: DocumentController,
        bindings: ReviewBindings,
    ) -> Result<Self, Box<SessionConstructionFailure>> {
        Self::new_composed_retaining_with_local(registry, controller, bindings, BTreeMap::new())
    }
    fn new_composed_retaining_with_local(
        registry: Rc<RefCell<PluginRegistry>>,
        controller: DocumentController,
        bindings: ReviewBindings,
        mut local_signatures: BTreeMap<String, CallbackSignature>,
    ) -> Result<Self, Box<SessionConstructionFailure>> {
        for (name, _) in crate::presentation::web_ui_adapter::player_shell::window_caption::BINDINGS
        {
            local_signatures
                .entry(name.into())
                .or_insert(CallbackSignature::Action);
        }
        if !controller.uses_registry(&registry) {
            return Err(Box::new(SessionConstructionFailure {
                error: PluginError::new(
                    PluginErrorCode::Stale,
                    "Review host uses another document registry",
                ),
                partial: None,
            }));
        }
        let mut session = Self {
            registry,
            controller,
            mounted: None,
            retired: None,
            next_binding: 0,
            local_signatures,
        };
        if let Err(error) = session.mount(bindings) {
            return Err(Box::new(SessionConstructionFailure {
                error,
                partial: Some(Box::new(session)),
            }));
        }
        Ok(session)
    }
    pub fn review(&self) -> &ReviewSession {
        self.controller.session()
    }
    pub fn raw_review(&self) -> Option<&super::raw_review_state::RawReviewState> {
        self.controller.raw_review()
    }
    pub fn raw_recovery(&self) -> Option<&super::pending_controller::RawRecovery> {
        self.controller.raw_recovery()
    }
    pub fn raw_review_unprojected(&self) -> bool {
        self.controller.raw_review_unprojected()
    }
    pub fn raw_review_retained(&self) -> Option<&super::pending_controller::RawReviewRetained> {
        self.controller.raw_review_retained()
    }
    pub fn output_review(&self) -> Option<&super::output_review_state::OutputReviewState> {
        self.controller.output_review()
    }
    pub fn pump_after_dispatch(&mut self) -> Result<bool, InteractionError> {
        self.controller.poll()
    }
    pub fn completion_guard_blocked(&self) -> bool {
        self.controller.completion_guard_blocked()
    }
    pub fn close_waiting(&self) -> bool {
        self.controller.close_waiting()
    }
    pub fn projection_needed(&self) -> bool {
        self.controller.projection_needed()
    }
    pub fn operation_pending(&self) -> bool {
        self.controller.operation_pending()
    }
    pub fn projection_committed(&mut self) {
        self.controller.projection_committed();
    }

    fn mount(&mut self, bindings: ReviewBindings) -> Result<(), PluginError> {
        let prepared = self.prepare(bindings)?;
        self.install(prepared)
    }
    fn prepare(&mut self, bindings: ReviewBindings) -> Result<PreparedBindings, PluginError> {
        if self.retired.is_some() {
            return Err(borrowed());
        }
        if self
            .mounted
            .as_ref()
            .is_some_and(|mounted| Rc::ptr_eq(&mounted.queue, &bindings.queue))
        {
            return Err(PluginError::new(
                PluginErrorCode::InvalidPayload,
                "Reprojection requires a fresh review queue",
            ));
        }
        let scope = self.review().scope().view.clone();
        let close_signal = {
            let queue = bindings.queue.try_borrow().map_err(|_| borrowed())?;
            if !queue.belongs_to(self.review()) {
                return Err(PluginError::new(
                    PluginErrorCode::Stale,
                    "Review queue belongs to another session",
                ));
            }
            queue.close_signal()
        };
        if self.registry.try_borrow().map_err(|_| borrowed())?.scope() != &scope {
            return Err(PluginError::new(
                PluginErrorCode::Stale,
                "Review registry document differs",
            ));
        }
        let signatures: BTreeMap<_, _> = bindings
            .bindings
            .iter()
            .map(|(name, binding)| (name.clone(), binding.signature()))
            .collect();
        for (prop, signature) in &signatures {
            CallbackReference {
                binding: prop.clone(),
                signature: *signature,
            }
            .validate()
            .map_err(|message| PluginError::new(PluginErrorCode::InvalidPayload, message))?;
        }
        let schema = bindings.pins.request.clone();
        let epoch = self.next_binding;
        let next = epoch.checked_add(1).ok_or_else(|| {
            PluginError::new(
                PluginErrorCode::InvalidPayload,
                "Review binding identity exhausted",
            )
        })?;
        let adapter = QueueCallbackAdapter::with_plugin_id(
            scope.clone(),
            bindings.queue.clone(),
            bindings.bindings,
            bindings.pins,
            bindings.targets,
            format!("{REVIEW_INTENT_PLUGIN_ID}.{epoch}"),
        )?;
        self.next_binding = next;
        Ok(PreparedBindings {
            adapter,
            scope,
            queue: bindings.queue,
            closed: close_signal,
            signatures,
            schema,
        })
    }
    fn install(&mut self, prepared: PreparedBindings) -> Result<(), PluginError> {
        self.mounted = Some(self.stage(prepared)?);
        Ok(())
    }
    fn stage(&mut self, prepared: PreparedBindings) -> Result<MountedBindings, PluginError> {
        let registration = self
            .registry
            .try_borrow_mut()
            .map_err(|_| borrowed())?
            .register(
                Box::new(prepared.adapter),
                prepared.signatures.keys().cloned().collect(),
            )?;
        let mut mounted = MountedBindings {
            callbacks: HostCallbacks::new(prepared.scope.clone()),
            registration,
            queue: prepared.queue,
            closed: prepared.closed,
            signatures: prepared.signatures,
            callbacks_revoked: false,
            local_keys: BTreeMap::new(),
        };
        let bound = (|| {
            mounted.callbacks = bind_review_callbacks(
                prepared.scope,
                &mounted.registration,
                &mounted.signatures,
                &prepared.schema,
            )?;
            for (prop, signature) in &self.local_signatures {
                let key = mounted.callbacks.bind_local(prop.clone(), *signature)?;
                mounted.local_keys.insert(prop.clone(), key);
            }
            Ok::<(), PluginError>(())
        })();
        if let Err(error) = bound {
            self.retired = Some(mounted);
            let _ = self.retry_cleanup();
            return Err(error);
        }
        Ok(mounted)
    }
    /// The actual renderer must retain this revocable export for its mounted document.
    pub fn export_callbacks(&self) -> Result<CallbackBindings, PluginError> {
        self.mounted
            .as_ref()
            .ok_or_else(closed)?
            .callbacks
            .export_bindings()
    }
    pub fn references(&self) -> Result<BTreeMap<String, CallbackReference>, PluginError> {
        let mounted = self.mounted.as_ref().ok_or_else(closed)?;
        if mounted.closed.load(Ordering::Acquire) {
            return Err(closed());
        }
        Ok(mounted
            .signatures
            .iter()
            .chain(self.local_signatures.iter())
            .map(|(name, signature)| {
                (
                    name.clone(),
                    CallbackReference {
                        binding: name.clone(),
                        signature: *signature,
                    },
                )
            })
            .collect())
    }
    /// Port for an already guarded renderer event. Does not itself validate a real node/disabled state.
    pub fn dispatch_after_renderer_guard(
        &self,
        export: &CallbackBindings,
        reference: &CallbackReference,
        arguments: CallbackArguments,
    ) -> Result<DispatchReceipt, PluginError> {
        let handle = export.resolve(reference)?;
        self.dispatch_handle_after_renderer_guard(&handle, arguments)
    }
    /// Renderer-owned exports may resolve before handing their opaque handle to this port.
    pub fn dispatch_handle_after_renderer_guard(
        &self,
        handle: &CallbackHandle,
        arguments: CallbackArguments,
    ) -> Result<DispatchReceipt, PluginError> {
        let mounted = self.mounted.as_ref().ok_or_else(closed)?;
        let mut registry = self.registry.try_borrow_mut().map_err(|_| borrowed())?;
        mounted.callbacks.dispatch(handle, arguments, &mut registry)
    }
    /// Call only after callback dispatch releases the shared registry borrow.
    pub fn drain_after_dispatch(&mut self) -> Result<DrainReport, PluginError> {
        let mounted = self.mounted.as_ref().ok_or_else(closed)?;
        Ok(mounted
            .queue
            .try_borrow_mut()
            .map_err(|_| borrowed())?
            .drain_interactions(&mut self.controller))
    }
    /// Stage new authority, let the renderer commit, then retire old local bindings.
    pub fn reproject_with(
        &mut self,
        bindings: ReviewBindings,
        apply: impl FnOnce(CallbackBindings) -> Result<(), String>,
    ) -> Result<ReprojectionReceipt, ReprojectionError> {
        let prepared = self.prepare(bindings).map_err(ReprojectionError::Prepare)?;
        let staged = self.stage(prepared).map_err(ReprojectionError::Prepare)?;
        let export = match staged.callbacks.export_bindings() {
            Ok(export) => export,
            Err(error) => {
                self.retired = Some(staged);
                let _ = self.retry_cleanup();
                return Err(ReprojectionError::Prepare(error));
            }
        };
        if let Err(error) = apply(export) {
            self.retired = Some(staged);
            let _ = self.retry_cleanup();
            return Err(ReprojectionError::Renderer(error));
        }
        self.retired = self.mounted.replace(staged);
        let cleanup_pending = self.retry_cleanup().is_err();
        Ok(ReprojectionReceipt { cleanup_pending })
    }
    pub fn retry_cleanup(&mut self) -> Result<(), PluginError> {
        if let Some(retired) = &mut self.retired {
            retired.revoke_callbacks()?;
            self.registry
                .try_borrow_mut()
                .map_err(|_| borrowed())?
                .unregister(&retired.registration)?;
        }
        self.retired = None;
        Ok(())
    }
    pub fn close(&mut self) -> Result<(), PluginError> {
        // Revoke both before any fallible registry cleanup.
        let retired_error = self
            .retired
            .as_mut()
            .and_then(|retired| retired.revoke_callbacks().err());
        let mounted_error = self
            .mounted
            .as_mut()
            .and_then(|mounted| mounted.revoke_callbacks().err());
        if let Some(error) = retired_error.or(mounted_error) {
            return Err(error);
        }
        // Revoked callbacks do not discard the controller's actual pending ledger.
        // The parent lease alone owns SDK unregister/shutdown after quiescence.
        self.controller.quiesce_for_parent_close()?;
        if let Some(mounted) = &mut self.mounted {
            self.registry
                .try_borrow_mut()
                .map_err(|_| borrowed())?
                .unregister(&mounted.registration)?;
        }
        self.mounted = None;
        self.retry_cleanup()
    }
}
impl Drop for ProposalDocumentSession {
    fn drop(&mut self) {
        // Revoke exports and queued work even if a caller is still holding the registry borrow.
        // Explicit close reports cleanup failure; parent registry teardown releases residual adapters.
        let _ = self.close();
    }
}
