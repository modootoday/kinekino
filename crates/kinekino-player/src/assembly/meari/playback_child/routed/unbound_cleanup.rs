//! Existing ParentCleanupTask on the original dispatcher; no route or new deadline is minted.
#[cfg(test)]
mod tests;
use super::errors::RoutedChildError;
use crate::{
    application::api::{ApiError, ErrorCode},
    infrastructure::mpv::{
        NativeEpisodeEngine, ParentCleanupObservation, ParentCleanupStatus, ParentCleanupTask,
        ParentEngineRouter,
    },
};
use meari_native::application::window_lifecycle::{CleanupProgress, CompletionSignal};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

pub(super) struct UnboundCleanup {
    task: ParentCleanupTask,
    wakes: Arc<Mutex<VecDeque<CompletionSignal>>>,
    dispatch_error: Option<RoutedChildError>,
}
fn internal() -> RoutedChildError {
    RoutedChildError::Api(Arc::new(ApiError::new(
        ErrorCode::Internal,
        "Original partial cleanup notification ownership failed",
    )))
}
impl UnboundCleanup {
    pub(super) fn new(
        router: &ParentEngineRouter,
        engine: &Arc<NativeEpisodeEngine>,
    ) -> Result<Self, RoutedChildError> {
        let wakes: Arc<Mutex<VecDeque<CompletionSignal>>> = Arc::new(Mutex::new(VecDeque::new()));
        let pending = wakes.clone();
        let wake = Arc::new(move || {
            let signal = pending
                .lock()
                .map_err(|_| "Partial cleanup wake queue poisoned".to_string())?
                .pop_front()
                .ok_or_else(|| "Partial cleanup has no original admitted signal".to_string())?;
            signal.notify();
            Ok(())
        });
        let task = router
            .prepare_failed_engine_cleanup(engine, wake)
            .map_err(|error| RoutedChildError::Api(Arc::new(error)))?;
        Ok(Self {
            task,
            wakes,
            dispatch_error: None,
        })
    }
    pub(super) fn begin(
        &mut self,
        signal: CompletionSignal,
    ) -> Result<CleanupProgress, RoutedChildError> {
        self.wakes.lock().map_err(|_| internal())?.push_back(signal);
        if let Err(error) = self.task.start_attempt() {
            self.wakes.lock().map_err(|_| internal())?.pop_back();
            let error = RoutedChildError::Api(Arc::new(error));
            self.dispatch_error = Some(error.clone());
            return Err(error);
        }
        self.dispatch_error = None;
        self.observe()
    }
    pub(super) fn observe(&mut self) -> Result<CleanupProgress, RoutedChildError> {
        if let Some(error) = &self.dispatch_error {
            return Err(error.clone());
        }
        match self.task.observe() {
            ParentCleanupObservation::Pending => Ok(CleanupProgress::Pending),
            ParentCleanupObservation::Finished(Ok(ParentCleanupStatus::Complete(_))) => {
                Ok(CleanupProgress::Complete)
            }
            ParentCleanupObservation::Finished(Ok(ParentCleanupStatus::Incomplete(receipt))) => {
                Err(RoutedChildError::Cleanup(
                    super::super::cleanup::PlaybackCleanupError::Incomplete(Arc::new(
                        receipt.clone(),
                    )),
                ))
            }
            ParentCleanupObservation::Finished(Err(error)) => Err(RoutedChildError::Api(
                super::errors::retain_api_error(error),
            )),
            ParentCleanupObservation::NotStarted
            | ParentCleanupObservation::Finished(Ok(ParentCleanupStatus::Pending)) => {
                Err(RoutedChildError::NotReady)
            }
        }
    }
}
