//! Explicit asynchronous-operation host over the existing authenticated pipe.
use super::{LocalHost, LocalHostShutdownStatus, PendingLocalHost, TransportOptions, security};
use crate::application::{
    api::{ApiError, ApplicationClient, ApplicationClientHandle, Request, Response},
    sdk_execution::{
        CompletionNotifier, ExecutionBudget, ExecutionCloseProgress, host::HostExecution,
    },
};
use std::sync::Arc;
struct PollNotifier;
impl CompletionNotifier for PollNotifier {
    fn notify(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
struct RpcClient(Arc<HostExecution>);
impl ApplicationClient for RpcClient {
    fn execute(&self, request: Request) -> Response {
        self.0.execute_rpc(request)
    }
}
#[must_use = "Keep the original host through explicit shutdown and both joins"]
pub struct ExecutionLocalHost {
    execution: Arc<HostExecution>,
    pipe: LocalHost,
}
impl ExecutionLocalHost {
    pub fn start(
        client: ApplicationClientHandle,
        budget: ExecutionBudget,
        options: TransportOptions,
    ) -> Result<Self, ApiError> {
        // Reuse OS CSPRNG and process-identity validation. This generated identity
        // is only a scope; LocalHost independently owns its real pipe endpoint.
        let identity = security::endpoint()?;
        let scope = identity
            .as_str()
            .rsplit('.')
            .next()
            .expect("validated endpoint nonce")
            .to_owned();
        let execution = Arc::new(HostExecution::start(
            scope,
            client,
            budget,
            Arc::new(PollNotifier),
        )?);
        let pipe = LocalHost::start(Arc::new(RpcClient(execution.clone())), options)?;
        Ok(Self { execution, pipe })
    }
    pub fn endpoint(&self) -> &str {
        self.pipe.endpoint()
    }
    pub fn host_scope(&self) -> &str {
        self.execution.host_scope()
    }
    pub fn begin_shutdown(self) -> PendingExecutionLocalHost {
        PendingExecutionLocalHost {
            execution: self.execution,
            pipe: self.pipe.begin_shutdown(),
        }
    }
}
#[must_use = "Pending operations and unacknowledged results are not cleanup success"]
pub struct PendingExecutionLocalHost {
    execution: Arc<HostExecution>,
    pipe: PendingLocalHost,
}
#[derive(Debug)]
pub struct ExecutionLocalHostShutdown {
    pub pipe: LocalHostShutdownStatus,
    pub execution: ExecutionCloseProgress,
}
impl ExecutionLocalHostShutdown {
    pub fn is_complete(&self) -> bool {
        self.pipe.is_quiescent() && matches!(self.execution, ExecutionCloseProgress::Closed(_))
    }
}
impl PendingExecutionLocalHost {
    pub fn execution(&self) -> &HostExecution {
        &self.execution
    }
    pub fn poll(&mut self) -> Result<ExecutionLocalHostShutdown, ApiError> {
        let execution = self.execution.poll_close()?;
        Ok(ExecutionLocalHostShutdown {
            execution,
            pipe: self.pipe.poll(),
        })
    }
}
#[cfg(test)]
mod tests;
