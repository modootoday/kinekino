//! Recoverable product SDK registration owned by the document parent.
use super::{proposal_manifest::proposal_manifest, proposals, request_context::scoped_context};
use crate::application::api::{ApiError, ApplicationClientHandle, ErrorCode, RequestContext};
use meari::application::plugins::{
    PluginError, PluginErrorCode, PluginRegistry, PluginScope, RegistrationHandle,
};
use std::{cell::RefCell, rc::Rc};

#[must_use = "Keep the SDK registration in the document parent and explicitly close it after UI cleanup"]
pub struct ProposalSdkRegistration {
    registry: Rc<RefCell<PluginRegistry>>,
    handle: Option<RegistrationHandle>,
}
impl ProposalSdkRegistration {
    pub fn register(
        client: ApplicationClientHandle,
        registry: Rc<RefCell<PluginRegistry>>,
        scope: &PluginScope,
        context: Rc<RefCell<RequestContext>>,
    ) -> Result<Self, PluginError> {
        {
            let current = registry.try_borrow().map_err(|_| borrowed())?;
            if current.scope() != scope {
                return Err(PluginError::new(
                    PluginErrorCode::Stale,
                    "Proposal SDK scope differs from its document registry",
                ));
            }
        }
        let manifest = proposal_manifest();
        let grants = manifest.capabilities.keys().cloned().collect();
        let adapter = proposals::adapter(client, manifest, move |ticket| {
            let current = context.try_borrow().map_err(|_| {
                ApiError::new(
                    ErrorCode::NeedsInput,
                    "Document request context is currently in use",
                )
            })?;
            scoped_context(&current, ticket)
        })?;
        let handle = registry
            .try_borrow_mut()
            .map_err(|_| borrowed())?
            .register(Box::new(adapter), grants)?;
        Ok(Self {
            registry,
            handle: Some(handle),
        })
    }
    pub fn handle(&self) -> Result<&RegistrationHandle, PluginError> {
        self.handle.as_ref().ok_or_else(|| {
            PluginError::new(
                PluginErrorCode::Stale,
                "Proposal SDK registration is closed",
            )
        })
    }
    /// Invoke after document callbacks/session have closed and all registry borrows returned.
    /// On failure retain the handle and registry so the parent can report/retry cleanup.
    pub fn close(&mut self) -> Result<(), PluginError> {
        if let Some(handle) = self.handle.as_ref() {
            self.registry
                .try_borrow_mut()
                .map_err(|_| borrowed())?
                .unregister(handle)?;
            self.handle = None;
        }
        Ok(())
    }
}
impl Drop for ProposalSdkRegistration {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
fn borrowed() -> PluginError {
    PluginError::new(
        PluginErrorCode::Busy,
        "Document registry is currently in use",
    )
}

#[cfg(test)]
mod tests;
