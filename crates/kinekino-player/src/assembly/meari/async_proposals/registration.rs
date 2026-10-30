use super::*;
use crate::assembly::meari::request_context::scoped_context;
use std::cell::RefCell;
pub struct AsyncProposalRegistration {
    registry: Rc<RefCell<PluginRegistry>>,
    handle: Option<RegistrationHandle>,
    pump: AsyncSdkPump,
}
fn busy() -> PluginError {
    PluginError::new(PluginErrorCode::Busy, "Document registry is borrowed")
}
impl AsyncProposalRegistration {
    pub fn register(
        registry: Rc<RefCell<PluginRegistry>>,
        scope: &PluginScope,
        context: Rc<RefCell<RequestContext>>,
        execution: ExecutionHandle,
        budget: AsyncCodecBudget,
    ) -> Result<Self, PluginError> {
        Self::register_profile(registry, scope, context, execution, budget, false, false)
    }
    /// Explicit product opt-in; the original register() retains the Native5 profile.
    pub fn register_output_reviews(
        registry: Rc<RefCell<PluginRegistry>>,
        scope: &PluginScope,
        context: Rc<RefCell<RequestContext>>,
        execution: ExecutionHandle,
        budget: AsyncCodecBudget,
    ) -> Result<Self, PluginError> {
        Self::register_profile(registry, scope, context, execution, budget, true, false)
    }
    pub fn register_raw_reviews(
        registry: Rc<RefCell<PluginRegistry>>,
        scope: &PluginScope,
        context: Rc<RefCell<RequestContext>>,
        execution: ExecutionHandle,
        budget: AsyncCodecBudget,
    ) -> Result<Self, PluginError> {
        Self::register_profile(registry, scope, context, execution, budget, true, true)
    }
    fn register_profile(
        registry: Rc<RefCell<PluginRegistry>>,
        scope: &PluginScope,
        context: Rc<RefCell<RequestContext>>,
        execution: ExecutionHandle,
        budget: AsyncCodecBudget,
        output_reviews: bool,
        raw_reviews: bool,
    ) -> Result<Self, PluginError> {
        if registry.try_borrow().map_err(|_| busy())?.scope() != scope {
            return Err(PluginError::new(
                PluginErrorCode::Stale,
                "Async SDK scope differs",
            ));
        }
        let request_context = move |ticket: &PluginTicket| {
            let current = context.try_borrow().map_err(|_| {
                ApiError::new(
                    crate::application::api::ErrorCode::NeedsInput,
                    "Document context is borrowed",
                )
            })?;
            scoped_context(&current, ticket)
        };
        let (plugin, pump) = if raw_reviews {
            raw_review_adapter(execution, budget, request_context)
        } else if output_reviews {
            output_review_adapter(execution, budget, request_context)
        } else {
            adapter(execution, budget, request_context)
        }?;
        let grants = plugin.manifest().capabilities.keys().cloned().collect();
        let handle = registry
            .try_borrow_mut()
            .map_err(|_| busy())?
            .register(Box::new(plugin), grants)?;
        Ok(Self {
            registry,
            handle: Some(handle),
            pump,
        })
    }
    pub fn handle(&self) -> Result<&RegistrationHandle, PluginError> {
        self.handle.as_ref().ok_or_else(|| {
            PluginError::new(PluginErrorCode::Stale, "Async SDK registration closed")
        })
    }
    pub fn pump(&self) -> &AsyncSdkPump {
        &self.pump
    }
    /// Document unbinding only; composition separately closes and joins ExecutionOwner.
    pub fn unbind(&mut self) -> Result<(), PluginError> {
        self.close()
    }
    pub fn close(&mut self) -> Result<(), PluginError> {
        if let Some(handle) = self.handle.as_ref() {
            self.pump.begin_close()?;
            let tickets = self.pump.tickets()?;
            let mut registry = self.registry.try_borrow_mut().map_err(|_| busy())?;
            for ticket in tickets {
                registry.cancel(&ticket)?;
            }
            registry.unregister(handle)?;
            self.handle = None;
        }
        Ok(())
    }
}
impl Drop for AsyncProposalRegistration {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
