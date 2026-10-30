//! Display facts from the existing review owner. No admission or mutation policy.
use super::{MutationReceipt, ReviewSession, ViewError, panel_model::PanelLabels};
use crate::{
    application::api::ApiError,
    domain::{
        api_values::DecimalU64,
        proposals::{DraftState, StoredProposal},
    },
};
use meari::domain::Value;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct FeedbackLabels {
    pub document_revision: String,
    pub output_revision: String,
    pub stored: String,
    pub accepted: String,
    pub dirty: String,
    pub pending: String,
    pub needs_repair: String,
    pub recheck: String,
    pub save_reply: String,
    pub accept_reply: String,
    pub sdk_error: String,
    pub local_error: String,
    pub separator: String,
    /// Stale, Busy, SaveRequired, NoDraft, InvalidSnapshot, Exhausted,
    /// RefreshRequired, InvalidSelection, ExistingDraft, in that order.
    pub view_errors: [String; 9],
}
impl FeedbackLabels {
    /// Explicit catalog selection by presentation, never inferred from other labels.
    pub fn korean() -> Self {
        Self {
            document_revision: "문서 revision".into(),
            output_revision: "출력 revision".into(),
            stored: "저장된 수정본".into(),
            accepted: "저장된 채택 결과".into(),
            dirty: "저장하지 않은 수정".into(),
            pending: "요청 처리 중".into(),
            needs_repair: "수정본 복구 필요".into(),
            recheck: "결과 재확인 필요".into(),
            save_reply: "저장 응답 확인".into(),
            accept_reply: "채택 응답 확인".into(),
            sdk_error: "SDK 오류".into(),
            local_error: "입력 오류".into(),
            separator: " · ".into(),
            view_errors: [
                "이전 화면의 입력입니다",
                "처리 중입니다",
                "수정본 저장이 필요합니다",
                "수정본이 없습니다",
                "결과를 확인할 수 없습니다",
                "입력 한도에 도달했습니다",
                "결과 재확인이 필요합니다",
                "선택을 확인하세요",
                "기존 수정본이 있습니다",
            ]
            .map(str::to_owned),
        }
    }
    pub fn english() -> Self {
        Self {
            document_revision: "Document revision".into(),
            output_revision: "Output revision".into(),
            stored: "Stored draft".into(),
            accepted: "Stored acceptance".into(),
            dirty: "Unsaved changes".into(),
            pending: "Request pending".into(),
            needs_repair: "Draft repair required".into(),
            recheck: "Result verification required".into(),
            save_reply: "Save response received".into(),
            accept_reply: "Accept response received".into(),
            sdk_error: "SDK error".into(),
            local_error: "Input error".into(),
            separator: " | ".into(),
            view_errors: [
                "Stale input",
                "Busy",
                "Save required",
                "No draft",
                "Invalid snapshot",
                "Capacity exhausted",
                "Refresh required",
                "Invalid selection",
                "Draft already exists",
            ]
            .map(str::to_owned),
        }
    }
    fn view_error(&self, error: ViewError) -> &str {
        &self.view_errors[match error {
            ViewError::Stale => 0,
            ViewError::Busy => 1,
            ViewError::SaveRequired => 2,
            ViewError::NoDraft => 3,
            ViewError::InvalidSnapshot => 4,
            ViewError::Exhausted => 5,
            ViewError::RefreshRequired => 6,
            ViewError::InvalidSelection => 7,
            ViewError::ExistingDraft => 8,
        }]
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmedKind {
    Save,
    Accept,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConfirmedResult {
    pub kind: ConfirmedKind,
    pub revision: DecimalU64,
}
pub struct ReviewFeedback<'a> {
    pub document_revision: DecimalU64,
    pub output_revision: Option<DecimalU64>,
    pub draft_state: DraftState,
    pub dirty: bool,
    pub pending: bool,
    pub confirmed: Option<ConfirmedResult>,
    pub sdk_error: Option<&'a ApiError>,
    pub local_error: Option<ViewError>,
}
pub struct RenderedFeedback {
    pub revision: String,
    pub result: String,
    pub errors: String,
}
impl<'a> ReviewFeedback<'a> {
    pub fn from_stored(stored: &StoredProposal) -> Self {
        Self {
            document_revision: stored.document.draft_revision,
            output_revision: stored
                .acceptance
                .as_ref()
                .map(|receipt| receipt.output_revision),
            draft_state: stored.document.state,
            dirty: false,
            pending: false,
            confirmed: None,
            sdk_error: None,
            local_error: None,
        }
    }
    pub fn from_review(review: &'a ReviewSession) -> Self {
        let mut facts = Self::from_stored(review.stored());
        facts.dirty = review.dirty();
        facts.pending = review.pending().is_some();
        facts.sdk_error = review.error();
        facts.confirmed = review.confirmed_mutation().map(|receipt| match receipt {
            MutationReceipt::Saved(document) => ConfirmedResult {
                kind: ConfirmedKind::Save,
                revision: document.draft_revision,
            },
            MutationReceipt::Accepted(receipt) => ConfirmedResult {
                kind: ConfirmedKind::Accept,
                revision: receipt.output_revision,
            },
        });
        facts
    }
    pub fn with_local_error(mut self, error: Option<ViewError>) -> Self {
        self.local_error = error;
        self
    }
    pub fn render(&self, labels: &PanelLabels) -> RenderedFeedback {
        let catalog = &labels.feedback;
        let document_label = if labels.revision.is_empty() {
            &catalog.document_revision
        } else {
            &labels.revision
        };
        let mut revision = format!("{} {}", document_label, self.document_revision.0);
        if let Some(output) = self.output_revision {
            revision.push_str(&format!(
                "{}{} {}",
                catalog.separator, catalog.output_revision, output.0
            ));
        }
        let mut parts = Vec::new();
        if !labels.result.is_empty() {
            parts.push(labels.result.clone());
        }
        if let Some(confirmed) = self.confirmed {
            let name = match confirmed.kind {
                ConfirmedKind::Save => &catalog.save_reply,
                ConfirmedKind::Accept => &catalog.accept_reply,
            };
            parts.push(format!("{name} {}", confirmed.revision.0));
            parts.push(catalog.recheck.clone());
        }
        // Errors remain independent facts even when a confirmed response needs inspection.
        let mut errors = Vec::new();
        if let Some(error) = self.sdk_error {
            errors.push(format!("{}: {}", catalog.sdk_error, error.message));
        }
        if let Some(error) = self.local_error {
            errors.push(format!(
                "{}: {}",
                catalog.local_error,
                catalog.view_error(error)
            ));
        }
        parts.extend(errors.iter().cloned());
        if self.confirmed.is_none() {
            if self.pending {
                parts.push(catalog.pending.clone());
            } else if self.dirty {
                parts.push(catalog.dirty.clone());
            } else if self.sdk_error.is_none() && self.local_error.is_none() {
                if self.output_revision.is_some() {
                    parts.push(catalog.accepted.clone());
                } else if self.draft_state == DraftState::NeedsRepair {
                    parts.push(catalog.needs_repair.clone());
                } else {
                    parts.push(catalog.stored.clone());
                }
            }
        }
        let result = parts.join(&catalog.separator);
        RenderedFeedback {
            revision,
            result,
            errors: errors.join(&catalog.separator),
        }
    }
    pub fn apply(&self, props: &mut BTreeMap<String, Value>, labels: &PanelLabels) {
        let rendered = self.render(labels);
        props.insert("revisionLabel".into(), Value::String(rendered.revision));
        props.insert("resultLabel".into(), Value::String(rendered.result));
        if props.contains_key("errorText") {
            props.insert("errorText".into(), Value::String(rendered.errors));
        }
    }
}
#[cfg(test)]
#[path = "review_feedback/tests.rs"]
mod tests;
