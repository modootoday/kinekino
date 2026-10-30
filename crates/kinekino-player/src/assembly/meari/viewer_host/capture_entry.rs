//! Explicit stored capture entry; the backend guarded use case remains mutation authority.
use super::*;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        proposals::guarded::CaptureAnchor,
    },
    presentation::web_ui_adapter::{
        player_shell::{NavigationGuards, RouteCatalog, ShellState},
        proposals::{
            capture_mutation::CaptureMutationAdapter, card_selection_queue::CardSelectionRoute,
        },
    },
};
use meari::application::plugins::PluginScope;

pub(super) struct CapturedEntry {
    pub(super) route: CardSelectionRoute,
    pub(super) adapter: Option<CaptureMutationAdapter>,
    scope: PluginScope,
    attached: bool,
    error: Option<Arc<ApiError>>,
}
impl CapturedEntry {
    pub(super) fn matches_return_scope(&self, scope: &PluginScope) -> bool {
        self.attached
            && &self.scope == scope
            && self
                .route
                .target
                .stored_capture()
                .is_some_and(|(_, snapshot)| snapshot.proposal_origin.is_some())
    }
}
fn unavailable() -> ViewerHostError {
    ViewerHostError::Sdk(Arc::new(ApiError::new(
        ErrorCode::NeedsInput,
        "Explicit captured entry requires a stored new-origin selection; legacy remains read-only",
    )))
}
impl ViewerHost {
    /// A typed, explicit opt-in. The old captured route entry stays unsupported.
    /// No current inspection, state refresh, origin issuance or working-draft substitution.
    pub fn request_explicit_captured_edit(
        &mut self,
        state: &mut ShellState,
        catalog: &RouteCatalog,
        guards: NavigationGuards,
    ) -> Result<bool, ViewerHostError> {
        if self.shutdown_started {
            return Err(unavailable());
        }
        let mut cards = self
            .cards
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        if cards.capture_entry.is_some()
            || cards.capture.is_some()
            || cards.active.is_some()
            || cards.opening.is_some()
            || cards.inspection.is_some()
            || cards.viewing_return.is_some()
            || cards.mount_failure.is_some()
            || cards.rejected_bridge.is_some()
        {
            return Err(unavailable());
        }
        let queue = self.card_selection_queue().ok_or_else(unavailable)?;
        let mut queue = queue
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        let Some(target) = queue.front().cloned() else {
            return Ok(false);
        };
        let (stored, snapshot) = target.stored_capture().ok_or_else(unavailable)?;
        if snapshot.proposal_origin.is_none() {
            return Err(unavailable());
        }
        let anchor = CaptureAnchor {
            project_id: stored.project.project_id.clone(),
            project_revision: stored.project.revision,
            project_revision_sha256: stored.revision_sha256.clone(),
            proposal_id: snapshot.captured.document.proposal_id.clone(),
            capture_sha256: snapshot.capture_sha256.clone(),
        };
        let adapter = CaptureMutationAdapter::new(anchor, snapshot.captured.clone());
        let scope = queue.scope().clone();
        let route = queue
            .request_route(state, catalog, guards)
            .map_err(ViewerHostError::Navigation)?
            .ok_or_else(unavailable)?;
        cards.capture_entry = Some(CapturedEntry {
            route,
            adapter: Some(adapter),
            scope,
            attached: false,
            error: None,
        });
        Ok(true)
    }
    pub fn captured_entry_route(
        &self,
    ) -> Option<crate::presentation::web_ui_adapter::player_shell::RouteTicket> {
        self.cards
            .try_borrow()
            .ok()?
            .capture_entry
            .as_ref()
            .map(|entry| entry.route.ticket.clone())
    }
    pub fn captured_entry_error(&self) -> Option<Arc<ApiError>> {
        self.cards
            .try_borrow()
            .ok()?
            .capture_entry
            .as_ref()?
            .error
            .clone()
    }
    /// A retained entry owns this turn until the original read consumer actually releases.
    /// Does not mark a route/frame committed or bind mutation input callbacks.
    pub fn advance_explicit_captured_entry(&mut self) -> Result<bool, ViewerHostError> {
        let scope = {
            let cards = self
                .cards
                .try_borrow()
                .map_err(|_| ViewerHostError::UnsupportedCallback)?;
            let Some(entry) = &cards.capture_entry else {
                return Ok(false);
            };
            if entry.attached {
                return Ok(false);
            }
            entry.scope.clone()
        };
        let mut callbacks = self
            .callbacks
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?;
        callbacks
            .consume_reads()
            .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?;
        if let Some(delay) = callbacks.next_consumer_retry() {
            return Err(ViewerHostError::ReadConsumerRetry(delay));
        }
        if callbacks
            .prepare_route_release(&scope)
            .map_err(|error| ViewerHostError::Callback(Arc::new(error)))?
            .is_some()
        {
            return Ok(true);
        }
        drop(callbacks);
        let adapter = self
            .cards
            .try_borrow_mut()
            .map_err(|_| ViewerHostError::UnsupportedCallback)?
            .capture_entry
            .as_mut()
            .ok_or_else(unavailable)?
            .adapter
            .take()
            .ok_or_else(unavailable)?;
        match self.attach_capture_adapter(adapter) {
            Ok(()) => {
                self.cards
                    .try_borrow_mut()
                    .map_err(|_| ViewerHostError::UnsupportedCallback)?
                    .capture_entry
                    .as_mut()
                    .ok_or_else(unavailable)?
                    .attached = true;
                Ok(true)
            }
            Err(failure) => {
                let error = Arc::new(failure.error);
                let mut cards = self
                    .cards
                    .try_borrow_mut()
                    .map_err(|_| ViewerHostError::UnsupportedCallback)?;
                let entry = cards.capture_entry.as_mut().ok_or_else(unavailable)?;
                entry.adapter = Some(failure.adapter);
                entry.error = Some(error.clone());
                Err(ViewerHostError::Sdk(error))
            }
        }
    }
}
