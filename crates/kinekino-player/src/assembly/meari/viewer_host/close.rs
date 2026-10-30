use crate::assembly::meari::viewer_callbacks::ViewerCallbacks;
use crate::{
    application::{
        api::ApiError,
        sdk_execution::{ExecutionCloseProgress, ExecutionCloseReceipt, ExecutionOwner},
    },
    infrastructure::local_host::{
        LocalHost, LocalHostShutdownStatus, PendingLocalHost, WorkerJoin,
    },
};
use meari::application::plugins::{PluginError, PluginTicket};
use std::sync::Arc;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub enum ViewerCloseProgress {
    CallbackPending {
        tickets: Vec<PluginTicket>,
    },
    EndpointPending(LocalHostShutdownStatus),
    ExecutionPending(ExecutionCloseProgress),
    CardScopePending,
    JobsScopePending,
    Complete {
        endpoint: LocalHostShutdownStatus,
        execution: ExecutionCloseReceipt,
    },
}
#[derive(Clone, Debug)]
pub enum ViewerCloseError {
    Callbacks(Arc<PluginError>),
    CardScope(Arc<PluginError>),
    JobsScope(Arc<PluginError>),
    EndpointJoin(LocalHostShutdownStatus),
    Sdk(Arc<ApiError>),
    InvalidExecutionReceipt(ExecutionCloseReceipt),
}
pub(super) trait EndpointPort {
    fn poll_shutdown(&mut self) -> LocalHostShutdownStatus;
}
pub(super) trait ExecutionPort {
    fn prepare_shutdown(&mut self) -> Result<Option<Vec<PluginTicket>>, ViewerCloseError> {
        Ok(None)
    }
    fn poll_shutdown(&mut self) -> Result<ExecutionCloseProgress, ApiError>;
    fn prepare_card_shutdown(&mut self) -> Result<bool, ViewerCloseError> {
        Ok(true)
    }
    fn prepare_jobs_shutdown(&mut self) -> Result<bool, ViewerCloseError> {
        Ok(true)
    }
}
pub(super) struct ViewerExecution {
    pub(super) owner: ExecutionOwner,
    pub(super) callbacks: Rc<RefCell<ViewerCallbacks>>,
    pub(super) cards: Rc<RefCell<super::cards::CardRoute>>,
    pub(super) jobs: Rc<RefCell<super::jobs::JobsLedger>>,
}
impl ExecutionPort for ViewerExecution {
    fn prepare_jobs_shutdown(&mut self) -> Result<bool, ViewerCloseError> {
        self.jobs
            .try_borrow_mut()
            .map_err(|_| {
                ViewerCloseError::JobsScope(Arc::new(PluginError::new(
                    meari::application::plugins::PluginErrorCode::Busy,
                    "Original JOB owner borrowed",
                )))
            })?
            .poll_close()
            .map_err(|error| ViewerCloseError::JobsScope(Arc::new(error)))
    }
    fn prepare_card_shutdown(&mut self) -> Result<bool, ViewerCloseError> {
        let mut cards = self.cards.try_borrow_mut().map_err(|_| {
            ViewerCloseError::CardScope(Arc::new(PluginError::new(
                meari::application::plugins::PluginErrorCode::Busy,
                "Active Card owner borrowed",
            )))
        })?;
        cards.prepare_live_close();
        cards
            .poll_release()
            .map_err(|error| ViewerCloseError::CardScope(Arc::new(error)))
    }
    fn prepare_shutdown(&mut self) -> Result<Option<Vec<PluginTicket>>, ViewerCloseError> {
        self.callbacks
            .try_borrow_mut()
            .map_err(|_| {
                ViewerCloseError::Callbacks(Arc::new(PluginError::new(
                    meari::application::plugins::PluginErrorCode::Busy,
                    "Viewer callback owner borrowed",
                )))
            })?
            .prepare_release()
            .map_err(|error| ViewerCloseError::Callbacks(Arc::new(error)))
    }
    fn poll_shutdown(&mut self) -> Result<ExecutionCloseProgress, ApiError> {
        self.owner.poll_close()
    }
}
pub(super) struct LocalEndpoint {
    host: Option<LocalHost>,
    pending: Option<PendingLocalHost>,
}
impl LocalEndpoint {
    pub(super) fn new(host: LocalHost) -> Self {
        Self {
            host: Some(host),
            pending: None,
        }
    }
}
impl EndpointPort for LocalEndpoint {
    fn poll_shutdown(&mut self) -> LocalHostShutdownStatus {
        if let Some(host) = self.host.take() {
            self.pending = Some(host.begin_shutdown());
        }
        self.pending
            .as_mut()
            .expect("original shutdown owner retained")
            .poll()
    }
}
impl ExecutionPort for ExecutionOwner {
    fn poll_shutdown(&mut self) -> Result<ExecutionCloseProgress, ApiError> {
        self.poll_close()
    }
}
pub(super) struct OrderedClose<E, X> {
    endpoint: E,
    execution: X,
    endpoint_receipt: Option<LocalHostShutdownStatus>,
    pub(super) last: Option<ViewerCloseProgress>,
}
impl<E> OrderedClose<E, ViewerExecution> {
    pub(super) fn original_capture_execution(
        &self,
    ) -> crate::application::sdk_execution::ExecutionHandle {
        self.execution.owner.handle()
    }
}
impl<E: EndpointPort, X: ExecutionPort> OrderedClose<E, X> {
    pub(super) fn new(endpoint: E, execution: X) -> Self {
        Self {
            endpoint,
            execution,
            endpoint_receipt: None,
            last: None,
        }
    }
    pub(super) fn poll(&mut self) -> Result<ViewerCloseProgress, ViewerCloseError> {
        if let Some(done @ ViewerCloseProgress::Complete { .. }) = &self.last {
            return Ok(done.clone());
        }
        if self.endpoint_receipt.is_none() {
            let status = self.endpoint.poll_shutdown();
            if status.pipe == WorkerJoin::Panicked || status.sdk == WorkerJoin::Panicked {
                return Err(ViewerCloseError::EndpointJoin(status));
            }
            if !status.is_quiescent() {
                let progress = ViewerCloseProgress::EndpointPending(status);
                self.last = Some(progress.clone());
                return Ok(progress);
            }
            self.endpoint_receipt = Some(status);
        }
        // Endpoint joins precede explicit callback terminal drain/release, then owner close.
        if let Some(tickets) = self.execution.prepare_shutdown()? {
            let progress = ViewerCloseProgress::CallbackPending { tickets };
            self.last = Some(progress.clone());
            return Ok(progress);
        }
        if !self.execution.prepare_jobs_shutdown()? {
            let progress = ViewerCloseProgress::JobsScopePending;
            self.last = Some(progress.clone());
            return Ok(progress);
        }
        if !self.execution.prepare_card_shutdown()? {
            let progress = ViewerCloseProgress::CardScopePending;
            self.last = Some(progress.clone());
            return Ok(progress);
        }
        let progress = self
            .execution
            .poll_shutdown()
            .map_err(|error| ViewerCloseError::Sdk(Arc::new(error)))?;
        let next = match progress {
            ExecutionCloseProgress::Closed(receipt) => {
                if !receipt.worker_stopped || receipt.unacknowledged != 0 {
                    return Err(ViewerCloseError::InvalidExecutionReceipt(receipt));
                }
                ViewerCloseProgress::Complete {
                    endpoint: self.endpoint_receipt.expect("joined endpoint"),
                    execution: receipt,
                }
            }
            ExecutionCloseProgress::Faulted { ref error, .. } => {
                return Err(ViewerCloseError::Sdk(error.clone()));
            }
            pending => ViewerCloseProgress::ExecutionPending(pending),
        };
        self.last = Some(next.clone());
        Ok(next)
    }
}
