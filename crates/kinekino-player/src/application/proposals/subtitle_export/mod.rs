//! Export one explicitly selected immutable accepted correction without mutating its store.
pub(crate) mod schema;
use super::{InspectProposal, ProposalStore};
use crate::{
    application::{
        api::{ApiError, ErrorCode, Request},
        subtitles::storage::{MAX_SUBTITLE_BYTES, SubtitleStorage},
    },
    domain::{
        api_values::DecimalU64,
        proposals::{
            accepted_subtitle_projection::{self, ProjectionError},
            output_review::OutputReviewTarget,
        },
        subtitles::{
            Cue, CueDocument, Loss, SubtitleError, SubtitleFormat,
            range::{self, SubtitleRange},
            render,
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const COMMAND: &str = "proposals.export_subtitles";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportAcceptedSubtitles {
    pub target: OutputReviewTarget,
    pub output_path: String,
    pub output_format: SubtitleFormat,
    #[serde(default)]
    pub allow_loss: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportedAcceptedSubtitles {
    pub target: OutputReviewTarget,
    pub output_path: String,
    pub output_format: SubtitleFormat,
    pub output_sha256: String,
    pub output_bytes: DecimalU64,
    pub cue_count: usize,
    pub losses: Vec<Loss>,
    pub encoding: String,
    pub overwrite: bool,
}

pub(crate) fn execute_request(
    request: &Request,
    store: &dyn ProposalStore,
    storage: &dyn SubtitleStorage,
) -> Result<Value, ApiError> {
    request.validate()?;
    if request.command != COMMAND
        || request.context.expected_revision.is_some()
        || request.context.idempotency_key.is_some()
        || request.context.timeout_ms.is_some()
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Accepted export uses the explicit immutable target, not mutation, retry or timeout context",
        ));
    }
    let input = serde_json::from_value(request.payload.clone()).map_err(|_| ApiError::new(ErrorCode::InvalidArgument, "Expected an accepted output target, output path, SRT/VTT format and loss acknowledgement"))?;
    serde_json::to_value(execute(store, storage, input)?).map_err(|_| {
        ApiError::new(
            ErrorCode::Internal,
            "Cannot encode accepted subtitle export result",
        )
    })
}

pub fn execute(
    store: &dyn ProposalStore,
    storage: &dyn SubtitleStorage,
    input: ExportAcceptedSubtitles,
) -> Result<ExportedAcceptedSubtitles, ApiError> {
    if !store.available() || !storage.available() {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Accepted subtitle export requires proposal and subtitle storage adapters",
        ));
    }
    input
        .target
        .validate_stored()
        .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
    if input.output_path.trim().is_empty() {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Explicit nonempty output path required",
        ));
    }
    // Existing inspect includes historical evidence verification; no writer/CAS or Accept is added.
    let state = super::inspect(
        store,
        InspectProposal {
            proposal_id: input.target.proposal_id.clone(),
        },
    )?;
    state
        .validate()
        .map_err(|message| ApiError::new(ErrorCode::Internal, message))?;
    input
        .target
        .validate_against(&state)
        .map_err(|message| ApiError::new(ErrorCode::RevisionConflict, message))?;
    let receipt = state
        .acceptance
        .as_ref()
        .ok_or_else(|| ApiError::new(ErrorCode::NeedsInput, "Explicit accepted output required"))?;
    let projection =
        accepted_subtitle_projection::project(&state, receipt).map_err(projection_error)?;
    let text_bytes = projection
        .cues()
        .iter()
        .try_fold(0usize, |size, cue| size.checked_add(cue.text().len()))
        .ok_or_else(too_large)?;
    if text_bytes > MAX_SUBTITLE_BYTES {
        return Err(too_large());
    }
    let document = CueDocument {
        cues: projection
            .cues()
            .iter()
            .map(|cue| Cue {
                in_us: cue.in_us(),
                out_us: cue.out_us(),
                text: cue.text().to_owned(),
            })
            .collect(),
        losses: Vec::new(),
    };
    // Whole-domain selection preserves each original us value/order while reusing precision checks.
    let document = range::select(
        &document,
        SubtitleRange {
            in_us: DecimalU64(0),
            out_us: DecimalU64(u64::MAX),
        },
    )
    .map_err(subtitle_error)?;
    if !document.losses.is_empty() && !input.allow_loss {
        let mut error = ApiError::new(
            ErrorCode::NeedsInput,
            "Review subtitle export losses and explicitly acknowledge allow_loss before publication",
        );
        error.details = json!({"losses":document.losses});
        error.remediation = "Resubmit the same immutable target with allow_loss=true only after reviewing these losses".into();
        return Err(error);
    }
    let output = render::render(&document, input.output_format);
    if output.len() > MAX_SUBTITLE_BYTES {
        return Err(too_large());
    }
    let output_sha256 = format!("{:x}", Sha256::digest(output.as_bytes()));
    let result = ExportedAcceptedSubtitles {
        target: input.target,
        output_path: input.output_path,
        output_format: input.output_format,
        output_sha256,
        output_bytes: DecimalU64(output.len() as u64),
        cue_count: document.cues.len(),
        losses: document.losses,
        encoding: "utf-8".into(),
        overwrite: false,
    };
    storage.publish_new(&result.output_path, &output)?;
    Ok(result)
}

fn too_large() -> ApiError {
    ApiError::new(
        ErrorCode::ResourceExhausted,
        "Rendered subtitle output exceeds 8 MiB",
    )
}
fn subtitle_error(error: SubtitleError) -> ApiError {
    match error {
        SubtitleError::Invalid(message) => ApiError::new(ErrorCode::InvalidArgument, message),
        SubtitleError::Unsupported(message) => ApiError::new(ErrorCode::Unsupported, message),
    }
}
fn projection_error(error: ProjectionError) -> ApiError {
    match error {
        ProjectionError::InvalidState(message) => ApiError::new(ErrorCode::Internal, message),
        ProjectionError::AcceptanceMismatch => ApiError::new(
            ErrorCode::RevisionConflict,
            "Accepted output identity changed",
        ),
        ProjectionError::NotAccepted => ApiError::new(
            ErrorCode::NeedsInput,
            "Explicit accepted correction required",
        ),
        ProjectionError::UnsupportedTask | ProjectionError::UnsupportedSource => ApiError::new(
            ErrorCode::Unsupported,
            "Only accepted timed-cue corrections can be exported as subtitles",
        ),
        ProjectionError::MissingCueTiming { cue_index } => {
            let mut error = ApiError::new(
                ErrorCode::NeedsInput,
                "Accepted cue timing remains unknown; export cannot infer timing",
            );
            error.details = json!({"cue_index":cue_index});
            error
        }
    }
}

#[cfg(test)]
mod tests;
