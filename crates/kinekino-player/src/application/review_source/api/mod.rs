//! Transport-neutral command binding. Domain policies remain in the shared usecases.
#[path = "ports.rs"]
mod ports;
use super::timeline_evidence::TimelineEvidenceResolver;
use super::*;
use crate::{
    application::api::Request,
    domain::{api_values::DecimalU64, review_source::*},
};
pub use ports::ReviewSourceApi;
use serde::Deserialize;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

pub const COMMANDS: &[&str] = &[
    "reviews.association.inspect",
    "reviews.association.create",
    "reviews.association.revise",
    "reviews.range_draft.read",
    "reviews.range_draft.save",
];
pub fn is_command(command: &str) -> bool {
    COMMANDS.contains(&command)
}
pub fn is_read(command: &str) -> bool {
    matches!(
        command,
        "reviews.association.inspect" | "reviews.range_draft.read"
    )
}
pub struct ReviewSourceService {
    cases: Box<dyn CaseResolver>,
    media: Box<dyn MediaResolver>,
    store: Box<dyn UserReviewStore>,
    writable: bool,
    evidence: Option<Box<dyn TimelineEvidenceResolver>>,
    media_preparation: Option<Box<dyn super::media_api::MediaPreparationPort>>,
}
impl ReviewSourceService {
    /// Trusted assembly supplies adapter availability; readonly mode never calls mutation.
    pub fn new(
        cases: impl CaseResolver + 'static,
        media: impl MediaResolver + 'static,
        store: impl UserReviewStore + 'static,
        writable: bool,
    ) -> Self {
        Self {
            cases: Box::new(cases),
            media: Box::new(media),
            store: Box::new(store),
            writable,
            evidence: None,
            media_preparation: None,
        }
    }
    pub fn with_timeline_evidence(
        mut self,
        resolver: impl TimelineEvidenceResolver + 'static,
    ) -> Self {
        self.evidence = Some(Box::new(resolver));
        self
    }
    pub fn with_media_preparation(
        mut self,
        port: impl super::media_api::MediaPreparationPort + 'static,
    ) -> Self {
        self.media_preparation = Some(Box::new(port));
        self
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectAssociation {
    pub case: ReviewCase,
    pub association_id: String,
    pub revision: Option<DecimalU64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadDraft {
    pub case: ReviewCase,
    pub draft_id: String,
    pub revision: Option<DecimalU64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WriteAssociation {
    pub case: ReviewCase,
    pub association_id: String,
    pub media: MediaReference,
    pub note: String,
    pub timeline_evidence: Option<EvidenceReference>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveDraft {
    pub case: ReviewCase,
    pub draft: RangeDraft,
    pub create_association: Option<AssociationRevision>,
}
fn decode<T: serde::de::DeserializeOwned>(value: &Value) -> Result<T, ApiError> {
    serde_json::from_value(value.clone())
        .map_err(|_| invalid("Invalid review source payload or unknown field"))
}
fn encode(value: impl serde::Serialize) -> Result<Value, ApiError> {
    serde_json::to_value(value).map_err(|_| {
        ApiError::new(
            ErrorCode::Internal,
            "Review source response encoding failed",
        )
    })
}
struct Deadline(Instant);
impl MutationControl for Deadline {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if Instant::now() >= self.0 {
            Err(ApiError::new(
                ErrorCode::Timeout,
                "Review source operation deadline elapsed before publication",
            ))
        } else {
            Ok(())
        }
    }
}
fn mutation(request: &Request) -> Result<MutationRequest, ApiError> {
    let expected = request
        .context
        .expected_revision
        .as_deref()
        .map(|value| {
            value
                .parse::<u64>()
                .map(DecimalU64)
                .map_err(|_| invalid("Invalid expected revision"))
        })
        .transpose()?;
    let key = request
        .context
        .idempotency_key
        .clone()
        .ok_or_else(|| invalid("Review mutation requires idempotency_key"))?;
    let (case, mutation) = match request.command.as_str() {
        "reviews.association.create" | "reviews.association.revise" => {
            let create = request.command == "reviews.association.create";
            if create != expected.is_none() {
                return Err(invalid(
                    "Create requires no revision; revise requires expected_revision",
                ));
            }
            let input: WriteAssociation = decode(&request.payload)?;
            if let Some(evidence) = input.timeline_evidence {
                if create {
                    return Err(invalid(
                        "Timeline verification requires an existing association",
                    ));
                }
                let mutation = Mutation::VerifyAssociation {
                    association_id: input.association_id,
                    media: input.media,
                    evidence,
                    note: input.note,
                };
                let result = MutationRequest {
                    case: input.case,
                    idempotency_key: key,
                    expected_revision: expected,
                    mutation,
                };
                result.validate()?;
                return Ok(result);
            }
            let association = Box::new(AssociationRevision {
                association_id: input.association_id,
                case: input.case.clone(),
                media: input.media,
                revision: next_revision(expected).map_err(validate)?,
                timeline_status: TimelineStatus::Unverified,
                evidence_refs: vec![],
                note: input.note,
            });
            (
                input.case,
                if create {
                    Mutation::CreateAssociation(association)
                } else {
                    Mutation::ReviseAssociation(association)
                },
            )
        }
        "reviews.range_draft.save" => {
            let input: SaveDraft = decode(&request.payload)?;
            (
                input.case,
                Mutation::SaveDraft {
                    draft: Box::new(input.draft),
                    create_association: input.create_association.map(Box::new),
                },
            )
        }
        _ => return Err(invalid("Unknown review source mutation")),
    };
    let request = MutationRequest {
        case,
        idempotency_key: key,
        expected_revision: expected,
        mutation,
    };
    request.validate()?;
    Ok(request)
}
impl ReviewSourceApi for ReviewSourceService {
    fn media_preparation(&self) -> Option<&dyn super::media_api::MediaPreparationPort> {
        self.media_preparation.as_deref()
    }
    fn timeline_verification_available(&self) -> bool {
        self.writable && self.evidence.is_some()
    }
    fn storage_limits(&self) -> Option<StoreLimits> {
        self.store.limits()
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn execute(&self, request: &Request) -> Result<Value, ApiError> {
        request.validate()?;
        if !is_command(&request.command) {
            return Err(invalid("Unknown review source command"));
        }
        if is_read(&request.command) {
            if request.context.expected_revision.is_some()
                || request.context.idempotency_key.is_some()
                || request.context.timeout_ms.is_some()
            {
                return Err(invalid(
                    "Review reads reject mutation revision, key and timeout context",
                ));
            }
            let data = if request.command == "reviews.association.inspect" {
                let input: InspectAssociation = decode(&request.payload)?;
                json!({"association":inspect_association(self.cases.as_ref(), self.store.as_ref(), &input.case, &input.association_id, input.revision)?})
            } else {
                let input: ReadDraft = decode(&request.payload)?;
                encode(read_draft(
                    self.cases.as_ref(),
                    self.store.as_ref(),
                    &input.case,
                    &input.draft_id,
                    input.revision,
                )?)?
            };
            return Ok(
                json!({"stored":data,"media_check":"not_checked","playback_ready":false,"automatic_application_enabled":false}),
            );
        }
        let mutation = mutation(request)?;
        let timeout = request.context.timeout_ms.unwrap_or(30_000);
        if timeout > 120_000 {
            return Err(invalid("Review mutation timeout exceeds 120000ms"));
        }
        if !self.writable {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Review source writes unavailable",
            ));
        }
        let deadline = Instant::now()
            .checked_add(Duration::from_millis(timeout))
            .ok_or_else(|| invalid("Invalid operation timeout"))?;
        let receipt = mutate_with_evidence(
            self.cases.as_ref(),
            self.media.as_ref(),
            self.store.as_ref(),
            &Deadline(deadline),
            mutation,
            self.evidence.as_deref(),
        )?;
        Ok(json!({"receipt":receipt,"playback_ready":false,"automatic_application_enabled":false}))
    }
}
