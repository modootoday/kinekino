use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_player::{
    application::{
        api::{Request, RequestContext},
        proposals::{AcceptProposal, EditProposal, InspectProposal},
    },
    domain::{api_values::DecimalU64, proposals::*},
};
use meari::application::plugins::*;
use serde::{Serialize, de::DeserializeOwned};
use std::{cell::RefCell, rc::Rc};

use super::ReviewHost;

/// Registry and registration are owned by the real document's composition root.
/// The same context slot must be supplied to the existing SDK adapter's ticket factory.
pub struct RegistryProposalHost {
    registry: Rc<RefCell<PluginRegistry>>,
    registration: RegistrationHandle,
    scope: PluginScope,
    context: Rc<RefCell<RequestContext>>,
    request_namespace: String,
}

impl RegistryProposalHost {
    pub(super) fn uses_registry(&self, registry: &Rc<RefCell<PluginRegistry>>) -> bool {
        Rc::ptr_eq(&self.registry, registry)
    }
    pub fn new(
        registry: Rc<RefCell<PluginRegistry>>,
        registration: RegistrationHandle,
        scope: PluginScope,
        context: Rc<RefCell<RequestContext>>,
        request_namespace: String,
    ) -> Result<Self, PluginError> {
        {
            let current = registry.try_borrow().map_err(|_| borrowed())?;
            if current.scope() != &scope {
                return Err(PluginError::new(
                    PluginErrorCode::Stale,
                    "Proposal document scope differs",
                ));
            }
            let manifest = current.manifest(&registration)?;
            for capability in ["proposals.edit", "proposals.accept", "proposals.inspect"] {
                if !manifest.capabilities.contains_key(capability) {
                    return Err(PluginError::new(
                        PluginErrorCode::MissingCapability,
                        "Proposal capability is not registered",
                    ));
                }
            }
        }
        Ok(Self {
            registry,
            registration,
            scope,
            context,
            request_namespace,
        })
    }

    fn invoke<I: Serialize, O: DeserializeOwned>(
        &mut self,
        command: &str,
        revision: Option<DecimalU64>,
        input: I,
    ) -> Result<O, ApiError> {
        let request = Request {
            command: command.into(),
            payload: serde_json::to_value(input).map_err(codec_error)?,
            context: RequestContext {
                request_id: self.request_namespace.clone(),
                expected_revision: revision.map(|value| value.0.to_string()),
                ..Default::default()
            },
        };
        request.validate()?;
        let schema = {
            let current = self
                .registry
                .try_borrow()
                .map_err(|_| transport_error(borrowed()))?;
            if current.scope() != &self.scope {
                return Err(transport_error(PluginError::new(
                    PluginErrorCode::Stale,
                    "Proposal document scope changed",
                )));
            }
            current
                .manifest(&self.registration)
                .map_err(transport_error)?
                .capabilities
                .get(command)
                .ok_or_else(|| {
                    transport_error(PluginError::new(
                        PluginErrorCode::MissingCapability,
                        "Proposal capability is not registered",
                    ))
                })?
                .request_schema
                .clone()
        };
        {
            let mut slot = self
                .context
                .try_borrow_mut()
                .map_err(|_| transport_error(borrowed()))?;
            *slot = request.context;
        }
        // The context borrow ends before the SDK adapter reads it. Callback dispatch must
        // also have returned before this relay acquires the document registry.
        let receipt = self
            .registry
            .try_borrow_mut()
            .map_err(|_| transport_error(borrowed()))?
            .dispatch(
                &self.registration,
                command,
                PluginPayload {
                    schema,
                    value: request.payload,
                },
            )
            .map_err(transport_error)?;
        match receipt.outcome {
            PluginDispatch::Immediate(Ok(reply)) => {
                serde_json::from_value(reply.value).map_err(codec_error)
            }
            PluginDispatch::Immediate(Err(error)) => {
                if let Some(details) = &error.details
                    && let Ok(canonical) = serde_json::from_value::<ApiError>(details.value.clone())
                {
                    return Err(canonical);
                }
                Err(transport_error(error))
            }
            PluginDispatch::Pending => Err(ApiError::new(
                ErrorCode::Internal,
                "Unexpected pending proposal response",
            )),
        }
    }
}

fn borrowed() -> PluginError {
    PluginError::new(
        PluginErrorCode::Adapter,
        "Proposal registry or request context is already borrowed",
    )
}
fn codec_error(error: serde_json::Error) -> ApiError {
    ApiError::new(ErrorCode::Internal, error.to_string())
}
fn transport_error(error: PluginError) -> ApiError {
    let mut result = ApiError::new(ErrorCode::Internal, error.message.clone());
    result.details = serde_json::json!({"plugin_error": error});
    result
}

impl ReviewHost for RegistryProposalHost {
    fn edit(
        &mut self,
        revision: DecimalU64,
        request: EditProposal,
    ) -> Result<ProposalDocument, ApiError> {
        self.invoke("proposals.edit", Some(revision), request)
    }
    fn accept(
        &mut self,
        revision: DecimalU64,
        request: AcceptProposal,
    ) -> Result<AcceptanceReceipt, ApiError> {
        self.invoke("proposals.accept", Some(revision), request)
    }
    fn inspect(&mut self, request: InspectProposal) -> Result<StoredProposal, ApiError> {
        self.invoke("proposals.inspect", None, request)
    }
}
