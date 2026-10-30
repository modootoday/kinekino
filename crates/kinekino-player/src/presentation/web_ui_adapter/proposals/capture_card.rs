//! Local Card text and display projection only; guarded SDK mutation stays in the adapter.
use super::{
    capture_mutation::{CaptureExecutionProgress, CaptureMutationAdapter},
    project,
};
use crate::domain::proposals::{Card, ProposalPayload};
use meari::{domain::Value, runtime::IntoValue};
use std::collections::BTreeMap;

pub struct CaptureCardModel {
    labels: super::capture_labels::CaptureLabels,
    draft: ProposalPayload,
    index: usize,
    baseline: (String, String),
    pub retry_accept: bool,
    pub message: String,
}
impl CaptureCardModel {
    pub fn new(adapter: &CaptureMutationAdapter, index: u32) -> Result<Self, String> {
        Self::with_labels(
            adapter,
            index,
            super::capture_labels::CaptureLabels::korean(),
        )
    }
    pub fn with_labels(
        adapter: &CaptureMutationAdapter,
        index: u32,
        labels: super::capture_labels::CaptureLabels,
    ) -> Result<Self, String> {
        let draft = adapter
            .display()
            .state
            .document
            .draft
            .clone()
            .ok_or("Captured Card has no draft")?;
        let card = Self::selected(&draft, index as usize)?;
        let baseline = (card.front.clone(), card.back.clone());
        Ok(Self {
            labels,
            draft,
            index: index as usize,
            baseline,
            retry_accept: false,
            message: String::new(),
        })
    }
    fn selected(draft: &ProposalPayload, index: usize) -> Result<&Card, String> {
        match draft {
            ProposalPayload::Cards { cards } => cards
                .get(index)
                .ok_or_else(|| "Selected Card is absent".into()),
            _ => Err("Capture is not a Card proposal".into()),
        }
    }
    pub fn dirty(&self) -> bool {
        Self::selected(&self.draft, self.index)
            .is_ok_and(|card| card.front != self.baseline.0 || card.back != self.baseline.1)
    }
    pub fn text(&mut self, front: bool, text: String) -> Result<(), String> {
        if text.len() > 8192 {
            return Err("Card text exceeds the presentation input budget".into());
        }
        let ProposalPayload::Cards { cards } = &mut self.draft else {
            return Err("Capture is not a Card proposal".into());
        };
        let card = cards.get_mut(self.index).ok_or("Selected Card is absent")?;
        if front {
            card.front = text;
        } else {
            card.back = text;
        }
        Ok(())
    }
    pub fn payload(&self) -> ProposalPayload {
        self.draft.clone()
    }
    pub fn revert_local(&mut self, adapter: &CaptureMutationAdapter) -> Result<(), String> {
        *self = Self::with_labels(adapter, self.index as u32, self.labels.clone())?;
        Ok(())
    }
    /// Only call after an explicit successful consume; failed SDK results retain local text.
    pub fn adopted(&mut self, adapter: &CaptureMutationAdapter) -> Result<(), String> {
        if adapter.display().sdk_error.is_some() {
            return Ok(());
        }
        let retry_accept = self.retry_accept;
        *self = Self::with_labels(adapter, self.index as u32, self.labels.clone())?;
        self.retry_accept = retry_accept;
        self.message = "처리 결과를 확인했습니다.".into();
        Ok(())
    }
    pub fn props(
        &self,
        adapter: &CaptureMutationAdapter,
        progress: CaptureExecutionProgress,
    ) -> Result<BTreeMap<String, Value>, String> {
        let display = adapter.display();
        let view = project(display.state).map_err(str::to_owned)?;
        let card = Self::selected(&self.draft, self.index)?;
        let idle = progress == CaptureExecutionProgress::Idle && !adapter.execution_closing();
        let ready = matches!(progress, CaptureExecutionProgress::TerminalReady { .. });
        let decided = matches!(
            progress,
            CaptureExecutionProgress::ConsumedAckPending
                | CaptureExecutionProgress::DiscardedAckPending
        );
        let source_text = view
            .original
            .iter()
            .filter(|original| match &original.reference {
                super::SourceReference::Cue { cue_id } => card.source_refs.contains(cue_id),
                _ => false,
            })
            .map(|original| original.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let result = if adapter.execution_fault().is_some() {
            "처리 결과를 확인하지 못했습니다. 결과를 확인하거나 버리세요.".into()
        } else if let Some(error) = display.sdk_error {
            match error.code {
                crate::application::api::ErrorCode::RevisionConflict => {
                    "저장된 제안이 변경되어 요청을 적용할 수 없습니다.".into()
                }
                crate::application::api::ErrorCode::NeedsInput => {
                    "이 저장본의 편집 조건을 확인하세요.".into()
                }
                _ => "요청을 완료하지 못했습니다. 입력과 처리 상태를 확인하세요.".into(),
            }
        } else if ready {
            "처리 결과가 도착했습니다. 확인하거나 버리세요.".into()
        } else if decided {
            "처리 결과 확인을 마치지 못했습니다. 다시 시도하세요.".into()
        } else if matches!(
            progress,
            CaptureExecutionProgress::Queued | CaptureExecutionProgress::Running { .. }
        ) {
            "요청을 처리하고 있습니다.".into()
        } else {
            self.message.clone()
        };
        let mut props = BTreeMap::from([
            (
                "returnViewingLabel".into(),
                Value::String(self.labels.return_viewing.clone()),
            ),
            (
                "revertLabel".into(),
                Value::String(self.labels.revert.clone()),
            ),
            ("captureCanRevert".into(), Value::Bool(idle && self.dirty())),
            (
                "captureTitle".into(),
                Value::String(self.labels.title.clone()),
            ),
            ("saveLabel".into(), Value::String(self.labels.save.clone())),
            (
                "acceptLabel".into(),
                Value::String(self.labels.accept.clone()),
            ),
            (
                "consumeLabel".into(),
                Value::String(self.labels.consume.clone()),
            ),
            (
                "discardLabel".into(),
                Value::String(self.labels.discard.clone()),
            ),
            (
                "retryLabel".into(),
                Value::String(self.labels.retry.clone()),
            ),
            ("proposalId".into(), Value::String(view.proposal_id)),
            (
                "revisionLabel".into(),
                Value::String(format!("저장된 제안 문서 {}", view.document_revision.0)),
            ),
            (
                "sourceIdentityLabel".into(),
                Value::String("저장된 capture 원본".into()),
            ),
            ("sourceRefs".into(), card.source_refs.clone().into_value()),
            ("sourceText".into(), Value::String(source_text)),
            ("generatedRaw".into(), Value::String(view.generated_raw)),
            ("qualityLabel".into(), Value::String("품질 미판정".into())),
            (
                "selectedCardLabel".into(),
                Value::String(format!("명시 선택 카드 {}", self.index + 1)),
            ),
            ("front".into(), Value::String(card.front.clone())),
            ("back".into(), Value::String(card.back.clone())),
            ("canEdit".into(), Value::Bool(idle)),
            ("canSave".into(), Value::Bool(idle && self.dirty())),
            ("canAccept".into(), Value::Bool(idle && !self.dirty())),
            ("busy".into(), Value::Bool(!idle)),
            ("refreshRequired".into(), Value::Bool(false)),
            ("resultLabel".into(), Value::String(result)),
            ("captureCanConsume".into(), Value::Bool(ready)),
            (
                "captureCanDiscard".into(),
                Value::Bool(ready || progress == CaptureExecutionProgress::Unsubmitted),
            ),
            (
                "captureCanRetry".into(),
                Value::Bool(
                    decided
                        || progress == CaptureExecutionProgress::Unsubmitted
                        || (idle && self.retry_accept && !self.dirty()),
                ),
            ),
        ]);
        props.extend(super::historical_source_props(
            view.historical_reference.as_ref(),
        ));
        Ok(props)
    }
}
