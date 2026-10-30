//! Capture-specific guards feed the existing retained Viewing return transaction.
use super::*;
use crate::presentation::web_ui_adapter::player_shell::NavigationGuards;

impl CapturedSurface {
    pub(in crate::assembly::meari::viewer_host) fn navigation_guards(&self) -> NavigationGuards {
        NavigationGuards {
            dirty: self.model.dirty(),
            pending: !self.inputs.queue.is_empty()
                || self.progress != Some(CaptureExecutionProgress::Idle),
            recovery_blocked: self.evidence_blocked
                || !self.acknowledged.is_empty()
                || self.rejected_intent.is_some(),
        }
    }
}
impl ViewerHost {
    pub(super) fn capture_return_after_input(
        &mut self,
        turn: &HostTurn,
    ) -> Result<Option<HostEffects>, ViewerHostError> {
        match self.request_viewing_return(turn) {
            Ok(()) => self.advance_viewing_return(turn),
            Err(error) => {
                {
                    let mut cards = self
                        .cards
                        .try_borrow_mut()
                        .map_err(|_| ViewerHostError::UnsupportedCallback)?;
                    let surface = cards
                        .capture_surface
                        .as_mut()
                        .ok_or(ViewerHostError::UnsupportedCallback)?;
                    surface.error.get_or_insert(error);
                    surface.model.message =
                        "시청으로 돌아가기 전에 수정 문구와 처리 결과를 확인하세요.".into();
                    surface.dirty = true;
                }
                // The request bit was consumed before retrying projection; no recursive intent.
                self.advance_capture_surface(turn)
            }
        }
    }
}

impl cards::CardRoute {
    pub(in crate::assembly::meari::viewer_host) fn capture_return_guards(
        &mut self,
    ) -> Result<NavigationGuards, ViewerHostError> {
        let capture = self
            .capture
            .as_mut()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        let progress = capture.poll_execution().map_err(ViewerHostError::Sdk)?;
        let surface = self
            .capture_surface
            .as_ref()
            .ok_or(ViewerHostError::UnsupportedCallback)?;
        let mut guards = surface.navigation_guards();
        guards.pending |= progress != CaptureExecutionProgress::Idle || capture.execution_closing();
        Ok(guards)
    }
}
