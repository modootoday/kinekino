//! Owned by intent_queue via an explicit path module. No second queue, SDK host or poller.
use super::super::{
    output_review_state::{
        DecisionFields, OutputReviewInputError, OutputReviewState, OutputReviewToken,
    },
    pending_host::PendingRequest,
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_player::domain::{api_values::DecimalU64, proposals::output_review::OutputVerdict};
use meari::{
    application::plugins::{PluginError, PluginErrorCode, PluginPayload},
    domain::build::ContentHash,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputReviewAction {
    Inspect,
    Select,
    History,
    Verdict(OutputVerdict),
    Record,
    Retry,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputReviewField {
    Note,
    ReviewerLabel,
    ReviewerId,
    ClaimedTime,
}
impl OutputReviewAction {
    pub fn binding(self) -> &'static str {
        match self {
            Self::Inspect => "onInspect",
            Self::Select => "onSelect",
            Self::History => "onHistory",
            Self::Verdict(OutputVerdict::Usable) => "onUsable",
            Self::Verdict(OutputVerdict::NeedsChanges) => "onNeedsChanges",
            Self::Verdict(OutputVerdict::Deferred) => "onDeferred",
            Self::Record => "onRecord",
            Self::Retry => "onRetry",
        }
    }
}
impl OutputReviewField {
    pub fn binding(self) -> &'static str {
        match self {
            Self::Note => "onNoteChange",
            Self::ReviewerLabel => "onReviewerLabelChange",
            Self::ReviewerId => "onReviewerIdChange",
            Self::ClaimedTime => "onClaimedTimeChange",
        }
    }
}
pub enum OutputReviewIntent {
    Action(OutputReviewAction),
    Text {
        field: OutputReviewField,
        text: String,
    },
}
#[derive(Debug)]
pub enum OutputReviewIntentError {
    Input(OutputReviewInputError),
    Metadata(ApiError),
}
impl From<OutputReviewInputError> for OutputReviewIntentError {
    fn from(error: OutputReviewInputError) -> Self {
        Self::Input(error)
    }
}
/// Composition supplies a fresh decision ID only for an explicit new-record action.
pub type DecisionIdSource = Box<dyn FnMut() -> Result<String, ApiError>>;
pub struct OutputReviewInteraction {
    state: OutputReviewState,
    next_id: DecisionIdSource,
}
impl OutputReviewInteraction {
    pub fn new(proposal_id: String, next_id: DecisionIdSource) -> Self {
        Self {
            state: OutputReviewState::new(proposal_id),
            next_id,
        }
    }
    pub fn state(&self) -> &OutputReviewState {
        &self.state
    }
    pub fn state_mut(&mut self) -> &mut OutputReviewState {
        &mut self.state
    }
    fn fields(&self) -> DecisionFields {
        let fields = self.state.fields();
        DecisionFields {
            verdict: fields.verdict,
            note: fields.note.clone(),
            reviewer_label: fields.reviewer_label.clone(),
            reviewer_id: fields.reviewer_id.clone(),
            claimed_review_unix_ms: fields.claimed_review_unix_ms,
        }
    }
    pub fn prepare(
        &mut self,
        intent: OutputReviewIntent,
    ) -> Result<Option<(OutputReviewToken, PendingRequest)>, OutputReviewIntentError> {
        let request = match intent {
            OutputReviewIntent::Action(action) => match action {
                OutputReviewAction::Inspect => Some(self.state.inspect()?),
                OutputReviewAction::History => Some(self.state.load_history()?),
                OutputReviewAction::Select => {
                    self.state.select()?;
                    None
                }
                OutputReviewAction::Verdict(verdict) => {
                    let mut fields = self.fields();
                    fields.verdict = Some(verdict);
                    self.state.replace_fields(fields)?;
                    None
                }
                OutputReviewAction::Record => {
                    let id = (self.next_id)().map_err(OutputReviewIntentError::Metadata)?;
                    Some(self.state.record(id)?)
                }
                OutputReviewAction::Retry => Some(self.state.retry_record()?),
            },
            OutputReviewIntent::Text { field, text } => {
                let mut fields = self.fields();
                match field {
                    OutputReviewField::Note => fields.note = text,
                    OutputReviewField::ReviewerLabel => fields.reviewer_label = text,
                    OutputReviewField::ReviewerId => {
                        fields.reviewer_id = if text.is_empty() { None } else { Some(text) }
                    }
                    OutputReviewField::ClaimedTime => {
                        fields.claimed_review_unix_ms = if text.is_empty() {
                            None
                        } else {
                            Some(
                                serde_json::from_value::<DecimalU64>(serde_json::Value::String(
                                    text,
                                ))
                                .map_err(|error| {
                                    OutputReviewIntentError::Metadata(ApiError::new(
                                        ErrorCode::InvalidArgument,
                                        error.to_string(),
                                    ))
                                })?,
                            )
                        }
                    }
                }
                self.state.replace_fields(fields)?;
                None
            }
        };
        Ok(request)
    }
}
pub fn api_error(error: &ApiError) -> PluginError {
    let mut result = PluginError::new(PluginErrorCode::Adapter, error.message.clone());
    result.details = PluginPayload::encode(
        ContentHash::of(include_bytes!(
            "../../../../../kinekino-media-contracts/src/errors.rs"
        )),
        error,
    )
    .ok();
    result
}
pub fn unsupported() -> PluginError {
    api_error(&ApiError::new(
        ErrorCode::Unsupported,
        "Output quality review requires the explicit owned Native8 profile",
    ))
}
