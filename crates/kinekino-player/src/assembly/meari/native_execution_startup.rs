//! One same-loop startup and SDK owner; no proposal policy or UI controller lives here.
use crate::{
    application::{
        api::{ApiError, ApplicationClientHandle},
        sdk_execution::{CompletionNotifier, ExecutionBudget, ExecutionOwner},
    },
    infrastructure::meari::completion_notifier::NativeStartupCompletionNotifier,
};
use meari_native::infrastructure::{
    native_host::HostScheduleLimits,
    window::{NativeLoopOwner, NativeStartup, NativeStartupError},
};
use std::{fmt, sync::Arc};

#[must_use = "Move this same owner into initial UI preparation and run the retained loop/startup"]
pub struct PreparedNativeExecution {
    loop_owner: NativeLoopOwner,
    startup: NativeStartup,
    execution: ExecutionOwner,
}
impl PreparedNativeExecution {
    pub fn into_parts(self) -> (NativeLoopOwner, NativeStartup, ExecutionOwner) {
        (self.loop_owner, self.startup, self.execution)
    }
}
#[derive(Debug)]
pub enum NativeExecutionStartError {
    Native(NativeStartupError),
    Sdk(ApiError),
}
impl fmt::Display for NativeExecutionStartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native(error) => write!(f, "{error}"),
            Self::Sdk(error) => write!(f, "SDK execution startup: {}", error.message),
        }
    }
}
impl std::error::Error for NativeExecutionStartError {}

#[must_use = "Retain startup authority on failure; Drop is not a cleanup receipt"]
pub struct NativeExecutionFailure {
    pub cause: NativeExecutionStartError,
    pub loop_owner: Option<NativeLoopOwner>,
    pub startup: Option<NativeStartup>,
    pub client: ApplicationClientHandle,
    notifier: Option<Arc<dyn CompletionNotifier>>,
}
impl fmt::Debug for NativeExecutionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProposalExecutionFailure")
            .field("cause", &self.cause)
            .field("loop_retained", &self.loop_owner.is_some())
            .field("startup_retained", &self.startup.is_some())
            .finish_non_exhaustive()
    }
}
impl NativeExecutionFailure {
    /// Only a failed SDK start can reuse the already issued startup. No new loop is created.
    pub fn retry_execution(
        mut self: Box<Self>,
        execution_budget: ExecutionBudget,
    ) -> Result<PreparedNativeExecution, Box<Self>> {
        if !matches!(&self.cause, NativeExecutionStartError::Sdk(_))
            || self.loop_owner.is_none()
            || self.startup.is_none()
            || self.notifier.is_none()
        {
            return Err(self);
        }
        let loop_owner = self.loop_owner.take().expect("checked retained loop");
        let startup = self.startup.take().expect("checked retained startup");
        let notifier = self.notifier.take().expect("checked retained notifier");
        start_execution(loop_owner, startup, self.client, execution_budget, notifier)
    }
}

pub fn prepare_native_execution(
    client: ApplicationClientHandle,
    execution_budget: ExecutionBudget,
    schedule_limits: HostScheduleLimits,
) -> Result<PreparedNativeExecution, Box<NativeExecutionFailure>> {
    prepare_native_execution_with_notifier_wrapper(
        client,
        execution_budget,
        schedule_limits,
        |notifier| notifier,
    )
}
/// Trusted assembly seam, applied once before worker creation; never a late notification target setter.
/// A wrapper must retain/delegate to the supplied Native notifier to preserve actual wake routing.
pub fn prepare_native_execution_with_notifier_wrapper(
    client: ApplicationClientHandle,
    execution_budget: ExecutionBudget,
    schedule_limits: HostScheduleLimits,
    wrap: impl FnOnce(Arc<dyn CompletionNotifier>) -> Arc<dyn CompletionNotifier>,
) -> Result<PreparedNativeExecution, Box<NativeExecutionFailure>> {
    let mut loop_owner = NativeLoopOwner::new(schedule_limits).map_err(|error| {
        Box::new(NativeExecutionFailure {
            cause: NativeExecutionStartError::Native(error),
            loop_owner: None,
            startup: None,
            client: client.clone(),
            notifier: None,
        })
    })?;
    let startup = match loop_owner.startup() {
        Ok(startup) => startup,
        Err(error) => {
            return Err(Box::new(NativeExecutionFailure {
                cause: NativeExecutionStartError::Native(error),
                loop_owner: Some(loop_owner),
                startup: None,
                client,
                notifier: None,
            }));
        }
    };
    let notifier = Arc::new(NativeStartupCompletionNotifier::new(startup.wake_handle()));
    let notifier = wrap(notifier);
    start_execution(loop_owner, startup, client, execution_budget, notifier)
}
fn start_execution(
    loop_owner: NativeLoopOwner,
    startup: NativeStartup,
    client: ApplicationClientHandle,
    execution_budget: ExecutionBudget,
    notifier: Arc<dyn CompletionNotifier>,
) -> Result<PreparedNativeExecution, Box<NativeExecutionFailure>> {
    match ExecutionOwner::start(client.clone(), execution_budget, notifier.clone()) {
        Ok(execution) => Ok(PreparedNativeExecution {
            loop_owner,
            startup,
            execution,
        }),
        Err(error) => Err(Box::new(NativeExecutionFailure {
            cause: NativeExecutionStartError::Sdk(error),
            loop_owner: Some(loop_owner),
            startup: Some(startup),
            client,
            notifier: Some(notifier),
        })),
    }
}

#[cfg(test)]
mod tests;
