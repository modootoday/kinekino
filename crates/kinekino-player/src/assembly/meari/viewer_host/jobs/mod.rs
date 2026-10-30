//! Route-owned JOB consumers; root execution worker and document remain parent-owned.
mod codec;
mod composition;
mod document;
mod handoff;
mod route;
pub(super) use route::JobsLedger;
pub(crate) mod consumer;
#[cfg(test)]
mod tests;
use crate::{
    application::{
        api::RequestContext,
        jobs::{JobPage, ListJobs},
        sdk_execution::ExecutionAttachment,
    },
    presentation::web_ui_adapter::studio::jobs::list::{JobsListProjection, ListStamp},
};
use consumer::{DecodedRead, ReadBinding, ReadConsumer};
use meari::{
    application::{
        host_callbacks::{CallbackBindings, PendingCallback},
        plugins::*,
    },
    domain::build::ContentHash,
};
use std::{num::NonZeroU64, sync::Arc};

pub(super) struct ListReadOwner {
    consumer: ReadConsumer,
    pub view: JobsListProjection,
    request: Option<ListStamp>,
    diagnostic: String,
}
impl ListReadOwner {
    pub(super) fn new(
        attachment: Arc<ExecutionAttachment>,
        scope: PluginScope,
        route: NonZeroU64,
        context: RequestContext,
    ) -> Result<Self, PluginError> {
        let consumer = ReadConsumer::new(
            scope,
            ReadBinding {
                manifest: codec::manifest(),
                command: codec::COMMAND,
                contract: ContentHash::of(b"studio-jobs-list-v1"),
                target: ListJobs {
                    cursor: None,
                    limit: None,
                },
                refresh: "onJobsListRpc",
                retry: "onJobsListRetained",
            },
            codec::adapter(attachment, context)?,
        )?;
        Ok(Self {
            consumer,
            view: JobsListProjection::new(route),
            request: None,
            diagnostic: String::new(),
        })
    }
    pub(super) fn bindings(&self) -> Result<CallbackBindings, PluginError> {
        self.consumer.bindings()
    }
    pub(super) fn bind_local(
        &mut self,
        name: &str,
        signature: meari::domain::host_callbacks::CallbackSignature,
    ) -> Result<meari::application::host_callbacks::LocalActionKey, PluginError> {
        self.consumer.bind_local(name, signature)
    }
    pub(super) fn local(
        &self,
        pending: &PendingCallback,
        scope: &PluginScope,
    ) -> Result<
        meari::application::host_callbacks::LocalActionInvocation,
        meari::application::host_callbacks::LocalDispatchError,
    > {
        self.consumer.local(pending, scope)
    }
    pub(super) fn poll(&mut self) -> Result<bool, PluginError> {
        let changed = self.consumer.poll();
        if let Err(error) = &changed {
            self.diagnostic = error.message.clone();
        }
        if !changed? {
            return Ok(false);
        }
        if self.consumer.blocked() {
            self.diagnostic = self
                .consumer
                .retained()
                .and_then(|accepted| accepted.owner_ack.as_ref().err())
                .or_else(|| self.consumer.first_fault())
                .map(|error| error.message.clone())
                .unwrap_or_default();
            return Ok(true);
        }
        let stamp = self.request.ok_or_else(|| {
            PluginError::new(PluginErrorCode::Stale, "Missing list display request")
        })?;
        let result = match self.consumer.decoded::<JobPage>()? {
            DecodedRead::Completed(page) => self
                .view
                .accept(stamp, &page)
                .map_err(|error| format!("{error:?}")),
            DecodedRead::Failed(message) => Err(message),
            DecodedRead::Cancelled => {
                Err("목록 조회를 취소했습니다. 작업 자체의 취소가 아닙니다.".into())
            }
        };
        if let Err(message) = result {
            self.view
                .reject(stamp, message)
                .map_err(|error| PluginError::new(PluginErrorCode::Stale, format!("{error:?}")))?;
        }
        self.request = None;
        self.diagnostic.clear();
        Ok(true)
    }
    pub(super) fn request_from_host(&mut self) -> Result<(), PluginError> {
        let stamp = self
            .view
            .begin()
            .map_err(|error| PluginError::new(PluginErrorCode::Busy, format!("{error:?}")))?;
        if let Err(error) = self.consumer.submit_from_host() {
            let _ = self.view.reject(stamp, error.message.clone());
            return Err(error);
        }
        self.request = Some(stamp);
        self.diagnostic.clear();
        Ok(())
    }
    pub(super) fn retry_retained(&mut self) -> Result<(), PluginError> {
        self.consumer.retry_retained()
    }
    pub(super) fn close(&mut self) -> Result<bool, PluginError> {
        self.view.begin_close();
        self.consumer.close()
    }
}
