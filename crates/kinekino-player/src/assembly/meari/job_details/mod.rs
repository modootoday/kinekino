//! Thin owned jobs.get view over the shared original single-query lifecycle.
mod codec;
pub mod document;
#[cfg(test)]
use crate::application::api::ApiError;
use crate::{
    application::{
        api::RequestContext,
        jobs::{GetJob, JobRecord},
        sdk_execution::ExecutionAttachment,
    },
    assembly::meari::viewer_host::jobs::consumer::{DecodedRead, ReadBinding, ReadConsumer},
    presentation::web_ui_adapter::studio::job_details::JobDetailsProjection,
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::TerminalAcceptance;
use meari::{
    application::{
        host_callbacks::{CallbackBindings, PendingCallback},
        plugins::*,
    },
    domain::build::ContentHash,
};
use std::sync::Arc;

fn error(code: PluginErrorCode, message: &str) -> PluginError {
    PluginError::new(code, message)
}

#[must_use = "Retain the same consumer until close returns true; parent owns the SDK worker"]
pub struct JobDetailsOwner {
    consumer: ReadConsumer,
    invalidated: bool,
    pub view: JobDetailsProjection,
    pub dirty: bool,
}
impl JobDetailsOwner {
    pub(in crate::assembly::meari) fn new(
        attachment: Arc<ExecutionAttachment>,
        scope: PluginScope,
        target: GetJob,
        context: RequestContext,
    ) -> Result<Self, PluginError> {
        Self::with_bindings(
            attachment,
            scope,
            target,
            context,
            ("onRefreshJob", "onRetryJobResult"),
        )
    }
    pub(in crate::assembly::meari) fn new_embedded(
        attachment: Arc<ExecutionAttachment>,
        scope: PluginScope,
        target: GetJob,
        context: RequestContext,
    ) -> Result<Self, PluginError> {
        Self::with_bindings(
            attachment,
            scope,
            target,
            context,
            ("onJobsGetRpc", "onJobsGetRetained"),
        )
    }
    fn with_bindings(
        attachment: Arc<ExecutionAttachment>,
        scope: PluginScope,
        target: GetJob,
        context: RequestContext,
        names: (&'static str, &'static str),
    ) -> Result<Self, PluginError> {
        let view = JobDetailsProjection::new(target.job_id.clone());
        let consumer = ReadConsumer::new(
            scope,
            ReadBinding {
                manifest: codec::manifest(),
                command: codec::COMMAND,
                contract: ContentHash::of(b"job-details-panel-v1"),
                target,
                refresh: names.0,
                retry: names.1,
            },
            codec::adapter(attachment, context)?,
        )?;
        Ok(Self {
            consumer,
            invalidated: false,
            view,
            dirty: true,
        })
    }
    pub fn bindings(&self) -> Result<CallbackBindings, PluginError> {
        self.consumer.bindings()
    }
    pub(in crate::assembly::meari) fn bind_local(
        &mut self,
        name: &str,
        signature: meari::domain::host_callbacks::CallbackSignature,
    ) -> Result<meari::application::host_callbacks::LocalActionKey, PluginError> {
        self.consumer.bind_local(name, signature)
    }
    pub(in crate::assembly::meari) fn local(
        &self,
        pending: &PendingCallback,
        scope: &PluginScope,
    ) -> Result<
        meari::application::host_callbacks::LocalActionInvocation,
        meari::application::host_callbacks::LocalDispatchError,
    > {
        self.consumer.local(pending, scope)
    }
    pub fn retained(&self) -> Option<&TerminalAcceptance> {
        self.consumer.retained()
    }
    pub fn pending(&self) -> Option<&PluginTicket> {
        self.consumer.pending()
    }
    pub fn first_fault(&self) -> Option<&PluginError> {
        self.consumer.first_fault()
    }
    pub(in crate::assembly::meari) fn blocked(&self) -> bool {
        self.consumer.blocked()
    }
    pub fn dispatch(
        &mut self,
        event: &PendingCallback,
        scope: &PluginScope,
    ) -> Result<(), PluginError> {
        if self.view.closing {
            return Err(error(
                PluginErrorCode::Closing,
                "Job detail consumer is closing",
            ));
        }
        if self.consumer.dispatch(event, scope)? {
            self.requested();
        }
        Ok(())
    }
    fn requested(&mut self) {
        self.view.loading = true;
        self.view.status = "작업 정보를 조회하고 있습니다.".into();
        self.view.diagnostic.clear();
        self.view.retry_result = false;
        self.dirty = true;
    }
    pub(in crate::assembly::meari) fn request_from_host(&mut self) -> Result<(), PluginError> {
        if self.view.closing {
            return Err(error(
                PluginErrorCode::Closing,
                "Job detail consumer is closing",
            ));
        }
        self.consumer.submit_from_host()?;
        self.requested();
        Ok(())
    }
    pub fn retry_retained(&mut self) -> Result<(), PluginError> {
        self.consumer.retry_retained()
    }
    pub fn poll(&mut self) -> Result<bool, PluginError> {
        match self.consumer.poll() {
            Ok(false) => return Ok(false),
            Err(failure) => {
                self.view.loading = false;
                self.view.retry_result = true;
                self.view.status = "조회 결과 확인을 마치지 못했습니다.".into();
                self.view.diagnostic = failure.message.clone();
                self.dirty = true;
                return Err(failure);
            }
            Ok(true) => {}
        }
        self.view.loading = false;
        self.view.retry_result = self.consumer.blocked();
        let decode = match self.consumer.decoded::<JobRecord>()? {
            DecodedRead::Completed(record) => self.view.received(record),
            DecodedRead::Failed(message) => Err(message),
            DecodedRead::Cancelled => {
                self.view.status =
                    "작업 정보 조회를 취소했습니다. 작업 자체의 취소가 아닙니다.".into();
                Ok(())
            }
        };
        if let Err(message) = decode {
            self.view.status = "작업 정보를 확인하지 못했습니다.".into();
            self.view.diagnostic = message;
        }
        if let Some(accepted) = self.consumer.retained()
            && let Err(failure) = &accepted.owner_ack
        {
            self.view.status = "조회 결과 확인을 마치지 못했습니다.".into();
            self.view.diagnostic = failure.message.clone();
        }
        self.dirty = true;
        Ok(true)
    }
    pub fn close(&mut self) -> Result<bool, PluginError> {
        self.view.closing = true;
        self.dirty = true;
        let result = self.consumer.close();
        self.invalidated = self.consumer.invalidated();
        result
    }
}

#[cfg(test)]
mod tests;
