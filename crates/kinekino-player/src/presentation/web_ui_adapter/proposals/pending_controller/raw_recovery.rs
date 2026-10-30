//! Read-only recovery stays on the original host; it never replays the retained mutation.
use super::*;
use kinekino_player::application::model_review::{
    InspectReviewPackage, ListReviewDecisions, ReviewPackageView, StoredDecision,
};

#[derive(Clone, Copy)]
pub(super) enum RecoveryRead {
    Inspect,
    History,
}

#[derive(Default)]
pub struct RawRecovery {
    generation: u64,
    pub identity: String,
    package: String,
    item: Option<String>,
    package_read_only: bool,
    pub inspected: Option<Box<ReviewPackageView>>,
    pub history: Option<Vec<StoredDecision>>,
    pub error: Option<ApiError>,
    pub unmatched: Option<Result<PendingOutput, ApiError>>,
    pub released: bool,
    pub transport: Option<(PluginTicket, bool)>,
}
impl RawRecovery {
    pub fn can_read_history(&self) -> bool {
        self.inspected.is_some() && self.item.is_some() && self.unmatched.is_none()
    }
    pub fn can_release(&self) -> bool {
        !self.released
            && self.inspected.is_some()
            && ((self.item.is_some() && self.history.is_some())
                || (self.item.is_none() && self.package_read_only))
            && self.error.is_none()
            && self.unmatched.is_none()
    }
    pub fn summary(&self) -> String {
        let state = if self.released && self.package_read_only {
            "항목 선택 전 원패키지 조회 결과를 해제했어요. 항목 이력은 조회하지 않았고 출판 여부도 판단하지 않았어요. 자료와 후보를 다시 선택해 주세요."
        } else if self.released {
            "보관 결과를 정리했어요. 현재 목록에서 후보를 다시 선택해 주세요."
        } else if self.unmatched.is_some() {
            "조회 응답의 범위가 달라 원 타입으로 보관했어요."
        } else if self.error.is_some() {
            "원자료 조회에 실패했어요. 보관 결과는 유지했어요."
        } else if self.can_release() {
            "원자료 조회를 마쳤어요. 빈 이력도 과거 기록이 없었다는 뜻은 아니에요."
        } else {
            "보관한 응답의 원자료와 이력을 차례로 확인해 주세요."
        };
        format!("{}\n{}", self.identity, state)
    }
    pub(super) fn complete(
        &mut self,
        generation: u64,
        phase: RecoveryRead,
        outcome: Result<PendingOutput, ApiError>,
    ) {
        if generation != self.generation {
            // Admission is blocked with an unmatched slot, so a second terminal cannot arrive.
            self.unmatched = Some(outcome);
            return;
        }
        match (phase, outcome) {
            (_, Err(error)) => self.error = Some(error),
            (RecoveryRead::Inspect, Ok(PendingOutput::RawReviewInspected(view)))
                if view.package_sha256 == self.package
                    && self.item.as_ref().is_none_or(|id| {
                        view.package.items.iter().any(|item| &item.item_id == id)
                    }) =>
            {
                self.inspected = Some(view);
                self.error = None;
            }
            (RecoveryRead::History, Ok(PendingOutput::RawReviewHistory(history)))
                if self.inspected.is_some()
                    && self.item.as_ref().is_some_and(|id| {
                        history.iter().all(|entry| {
                            entry.decision.package_sha256 == self.package
                                && &entry.decision.item_id == id
                        })
                    }) =>
            {
                self.history = Some(history);
                self.error = None;
            }
            (_, outcome) => self.unmatched = Some(outcome),
        }
    }
}

impl PendingReviewController {
    pub(super) fn recovery_intent(
        &mut self,
        scope: &ReviewScope,
        action: super::super::raw_review_state::RawReviewAction,
    ) -> Result<(), PendingControllerError> {
        use super::super::raw_review_state::RawReviewAction;
        if self.closing
            || self.active.is_some()
            || !self.host.pending_tickets().is_empty()
            || self.host.completion_guard_blocked()
            || self.output_review_scope_mismatch.is_some()
        {
            return Err(PendingControllerError::View(ViewError::Busy));
        }
        if scope != self.session.scope() {
            return Err(PendingControllerError::View(ViewError::Stale));
        }
        let carrier = self
            .unprojected_raw_terminal
            .as_ref()
            .ok_or(PendingControllerError::View(ViewError::InvalidSelection))?;
        if matches!(action, RawReviewAction::ReleaseRetained) {
            if !self
                .raw_recovery
                .as_ref()
                .is_some_and(RawRecovery::can_release)
            {
                return Err(PendingControllerError::View(ViewError::Busy));
            }
            let raw = self
                .raw_review
                .as_mut()
                .ok_or(PendingControllerError::View(ViewError::InvalidSelection))?;
            // Read-only recovery is never promoted into an editable selection.
            raw.clear_after_recovery();
            self.raw_recovery
                .as_mut()
                .expect("validated recovery")
                .released = true;
            drop(self.unprojected_raw_terminal.take());
            self.projection_needed = true;
            return Ok(());
        }
        if self.raw_recovery.is_none() {
            let (package, item) = carrier.rejected.token.original_selection();
            self.raw_recovery = Some(RawRecovery {
                identity: carrier.rejected.token.identity_label(),
                package: package.into(),
                item: item.map(str::to_owned),
                package_read_only: carrier.rejected.token.package_only_read(),
                ..RawRecovery::default()
            });
        }
        let recovery = self.raw_recovery.as_mut().expect("initialized recovery");
        if recovery.unmatched.is_some() || recovery.released {
            return Err(PendingControllerError::View(ViewError::Busy));
        }
        let (phase, request) = match action {
            RawReviewAction::InspectRetained => {
                recovery.inspected = None;
                recovery.history = None;
                (
                    RecoveryRead::Inspect,
                    PendingRequest::InspectRawReview(InspectReviewPackage {
                        package_sha256: recovery.package.clone(),
                    }),
                )
            }
            RawReviewAction::HistoryRetained if recovery.can_read_history() => (
                RecoveryRead::History,
                PendingRequest::RawReviewDecisions(ListReviewDecisions {
                    package_sha256: recovery.package.clone(),
                    item_id: recovery.item.clone().expect("validated item"),
                }),
            ),
            _ => return Err(PendingControllerError::View(ViewError::InvalidSelection)),
        };
        recovery.generation = recovery
            .generation
            .checked_add(1)
            .ok_or(PendingControllerError::View(ViewError::InvalidSelection))?;
        let generation = recovery.generation;
        recovery.error = None;
        self.projection_needed = true;
        match self.host.submit(request) {
            Ok(ProposalAdmission::Pending(ticket)) => {
                self.active = Some(Active {
                    ticket,
                    kind: ActiveKind::RawRecovery {
                        generation,
                        phase,
                        scope: scope.clone(),
                    },
                });
            }
            Ok(ProposalAdmission::Immediate(outcome)) => {
                recovery.complete(generation, phase, *outcome)
            }
            Err(error) => recovery.complete(generation, phase, Err(error)),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_recovery_generation_preserves_original_typed_error_without_readiness() {
        let mut recovery = RawRecovery {
            generation: 2,
            ..RawRecovery::default()
        };
        let error = ApiError::new(ErrorCode::EngineFailed, "original recovery read error");
        let allocation = error.message.as_ptr();
        recovery.complete(1, RecoveryRead::History, Err(error));
        assert!(!recovery.can_release());
        assert_eq!(
            recovery
                .unmatched
                .as_ref()
                .unwrap()
                .as_ref()
                .err()
                .unwrap()
                .message
                .as_ptr(),
            allocation
        );
        assert!(recovery.history.is_none());
    }
}
