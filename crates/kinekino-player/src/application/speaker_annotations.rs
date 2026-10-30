//! Shared manual annotation usecases; inferred speaker identity is never asserted.
use crate::{
    application::{
        api::{ApiError, ErrorCode, Request},
        review_source::MutationControl,
    },
    domain::{api_values::DecimalU64, speaker_annotations::*},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};
pub trait SpeakerInput {
    fn bytes(&self) -> &[u8];
    fn check_current(&self) -> Result<(), ApiError>;
}
pub trait SpeakerDocumentFiles: Send + Sync {
    fn retain(
        &self,
        path: &str,
        control: &dyn MutationControl,
    ) -> Result<Box<dyn SpeakerInput>, ApiError>;
    fn publish_new(
        &self,
        path: &str,
        bytes: &[u8],
        input: &dyn SpeakerInput,
        control: &dyn MutationControl,
    ) -> Result<(), ApiError>;
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadSpeakers {
    pub input_path: String,
    pub input_sha256: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditSpeakers {
    pub input_path: String,
    pub input_sha256: String,
    pub output_path: String,
    pub edits: Vec<SpeakerEdit>,
}
pub fn is_command(s: &str) -> bool {
    matches!(s, "subtitles.speakers" | "subtitles.edit_speakers")
}
fn invalid(s: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, s)
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn load(
    input: &dyn SpeakerInput,
    pin: Option<&str>,
) -> Result<(SpeakerDocument, String), ApiError> {
    if input.bytes().len() > MAX_DOCUMENT_BYTES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Speaker document exceeds 512KiB",
        ));
    }
    let sha = hash(input.bytes());
    if let Some(pin) = pin {
        if !crate::domain::identifiers::valid_sha256(pin) {
            return Err(invalid("Exact input SHA256 required"));
        }
        if pin != sha {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Speaker input changed",
            ));
        }
    }
    let document: SpeakerDocument = serde_json::from_slice(input.bytes())
        .map_err(|_| invalid("Expected strict speaker-annotations/1 JSON"))?;
    document.current().map_err(invalid)?;
    input.check_current()?;
    Ok((document, sha))
}
fn view(document: SpeakerDocument, sha256: String) -> Result<Value, ApiError> {
    let current = document.current().map_err(invalid)?;
    Ok(
        json!({"document":document,"current":current,"sha256":sha256,"media_binding_verified":false,"parent_chain_verified":false,"inference_performed":false,"quality_approved":false}),
    )
}
pub fn read(
    files: &dyn SpeakerDocumentFiles,
    input: ReadSpeakers,
    control: &dyn MutationControl,
) -> Result<Value, ApiError> {
    control.checkpoint()?;
    let retained = files.retain(&input.input_path, control)?;
    let (document, sha) = load(retained.as_ref(), input.input_sha256.as_deref())?;
    let result = view(document, sha)?;
    control.checkpoint()?;
    Ok(result)
}
pub fn edit(
    files: &dyn SpeakerDocumentFiles,
    expected: u64,
    input: EditSpeakers,
    control: &dyn MutationControl,
) -> Result<Value, ApiError> {
    control.checkpoint()?;
    if input.input_path == input.output_path || expected == 0 {
        return Err(invalid("New output and exact positive revision required"));
    }
    let retained = files.retain(&input.input_path, control)?;
    let (document, sha) = load(retained.as_ref(), Some(&input.input_sha256))?;
    if document.revision != DecimalU64(expected) {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Speaker document revision changed",
        ));
    }
    let next = document.edit(sha, input.edits).map_err(invalid)?;
    let bytes =
        serde_json::to_vec(&next).map_err(|_| invalid("Speaker document serialization failed"))?;
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(ApiError::new(
            ErrorCode::ResourceExhausted,
            "Speaker history exceeds document byte capacity",
        ));
    }
    control.checkpoint()?;
    retained.check_current()?;
    let mut result = view(next, hash(&bytes))?;
    result["output_path"] = json!(input.output_path);
    // Assemble the complete response before the single create-only publication.
    files.publish_new(&input.output_path, &bytes, retained.as_ref(), control)?;
    Ok(result)
}
struct Deadline<'a> {
    at: Instant,
    external: Option<&'a dyn MutationControl>,
}
impl MutationControl for Deadline<'_> {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if let Some(external) = self.external {
            external.checkpoint()?;
        }
        if Instant::now() >= self.at {
            Err(ApiError::new(
                ErrorCode::Timeout,
                "Speaker annotation deadline expired",
            ))
        } else {
            Ok(())
        }
    }
}
pub fn execute(
    files: Option<&dyn SpeakerDocumentFiles>,
    request: &Request,
) -> Result<Value, ApiError> {
    execute_controlled(files, request, None)
}
pub fn execute_controlled(
    files: Option<&dyn SpeakerDocumentFiles>,
    request: &Request,
    external: Option<&dyn MutationControl>,
) -> Result<Value, ApiError> {
    let files = files.ok_or_else(|| {
        ApiError::new(
            ErrorCode::Unsupported,
            "Speaker annotation file adapter unavailable",
        )
    })?;
    if request.context.idempotency_key.is_some()
        || request.command == "subtitles.speakers" && request.context.expected_revision.is_some()
    {
        return Err(invalid("Unsupported speaker read/retry context"));
    }
    let ms = request.context.timeout_ms.unwrap_or(30000);
    if !(1..=120000).contains(&ms) {
        return Err(invalid("Speaker timeout must be1..120000ms"));
    }
    let control = Deadline {
        at: Instant::now() + Duration::from_millis(ms),
        external,
    };
    match request.command.as_str() {
        "subtitles.speakers" => read(
            files,
            serde_json::from_value(request.payload.clone())
                .map_err(|_| invalid("Expected speaker input path and optional hash"))?,
            &control,
        ),
        "subtitles.edit_speakers" => edit(
            files,
            request
                .context
                .expected_revision
                .as_deref()
                .and_then(|s| s.parse().ok())
                .filter(|n| *n > 0)
                .ok_or_else(|| invalid("Speaker edit requires expected_revision"))?,
            serde_json::from_value(request.payload.clone())
                .map_err(|_| invalid("Expected pinned speaker input, new output and edits"))?,
            &control,
        ),
        _ => Err(invalid("Unknown speaker annotation command")),
    }
}
pub fn capabilities(value: &mut Value, available: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if command["command"].as_str().is_some_and(is_command) {
                command["status"] = json!(if available {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["speaker_annotations"] = json!({"available":available,"format":"speaker-annotations/1","manual_only":true,"inference":false,"document_max_bytes":MAX_DOCUMENT_BYTES,"history_max_edits":MAX_HISTORY,"editing_max_speakers":MAX_SPEAKERS,"editing_max_spans":MAX_SPANS,"editing_max_cues":MAX_CUES,"limits_are_model_capabilities":false,"overwrite":false,"global_latest_cas":false,"in_process_sdk_execution_cooperative_cancel":available,"wire_cancel_ack":false,"cancel_after_publication":"actual_success_retained"});
}
fn edit_schema() -> Value {
    fn object(op: &str, mut fields: Value, required: &[&str]) -> Value {
        fields["op"] = json!({"const":op});
        let mut names = vec!["op"];
        names.extend_from_slice(required);
        json!({"type":"object","additionalProperties":false,"required":names,"properties":fields})
    }
    let id = json!({"type":"string","minLength":1,"maxLength":80,"pattern":"^[A-Za-z0-9_.-]+$"});
    let label = json!({"type":"string","minLength":1,"maxLength":1024,"description":"Nonempty user label, at most1024 UTF8 bytes; no identity inference"});
    let speaker = json!({"type":"object","additionalProperties":false,"required":["id","label"],"properties":{"id":id,"label":label}});
    let ids = json!({"type":"array","maxItems":MAX_SPEAKERS,"uniqueItems":true,"items":id});
    let spans = json!({"type":"array","maxItems":MAX_SPANS,"uniqueItems":true,"items":id});
    let cues = json!({"type":"array","maxItems":MAX_CUES,"uniqueItems":true,"items":id});
    json!({"oneOf":[
        object("add",json!({"speaker":speaker}),&["speaker"]),
        object("rename",json!({"speaker_id":id,"label":label}),&["speaker_id","label"]),
        object("merge",json!({"speaker_ids":ids,"target":id}),&["speaker_ids","target"]),
        object("split",json!({"speaker_id":id,"new_speaker":speaker,"span_ids":spans,"cue_ids":cues}),&["speaker_id","new_speaker","span_ids","cue_ids"]),
        object("assign_cues",json!({"cue_ids":cues,"speaker_ids":ids}),&["cue_ids","speaker_ids"]),
        object("delete",json!({"speaker_id":id,"reassign":{"anyOf":[ids,{"type":"null"}]}}),&["speaker_id"]),
        object("undo",json!({}),&[]),object("redo",json!({}),&[])
    ]})
}
pub fn schema(payload: &Value, available: bool) -> Result<Value, ApiError> {
    let command = payload["command"]
        .as_str()
        .filter(|s| is_command(s))
        .ok_or_else(|| invalid("Unknown speaker schema"))?;
    if payload.as_object().is_none_or(|p| p.len() != 1) {
        return Err(invalid("Unexpected schema fields"));
    }
    let path = json!({"type":"string","minLength":1,"maxLength":32768});
    let sha = json!({"type":"string","pattern":"^[a-f0-9]{64}$"});
    let mut input = if command == "subtitles.speakers" {
        json!({"type":"object","additionalProperties":false,"required":["input_path"],"properties":{"input_path":path,"input_sha256":{"anyOf":[sha,{"type":"null"}]}}})
    } else {
        json!({"type":"object","additionalProperties":false,"required":["input_path","input_sha256","output_path","edits"],"properties":{"input_path":path,"input_sha256":sha,"output_path":path,"edits":{"type":"array","minItems":1,"maxItems":32,"items":{"type":"object","required":["op"],"properties":{"op":{"enum":["add","rename","merge","split","assign_cues","delete","undo","redo"]}},"description":"Strict tagged SpeakerEdit DTO; unknown fields rejected by domain deserializer"}}}})
    };
    if command == "subtitles.edit_speakers" {
        input["properties"]["edits"]["items"] = edit_schema();
    }
    let mut context = crate::application::api::schema::context_schema();
    context["properties"]["idempotency_key"] = json!({"type":"null"});
    context["properties"]["timeout_ms"] =
        json!({"type":["integer","null"],"minimum":1,"maximum":120000});
    if command == "subtitles.speakers" {
        context["properties"]["expected_revision"] = json!({"type":"null"});
    } else {
        context["required"] = json!(["request_id", "api_version", "expected_revision"]);
    }
    Ok(
        json!({"schema_version":"1","command":command,"status":if available{"supported"}else{"unsupported"},"request":{"type":"object","additionalProperties":false,"required":["context","command","payload"],"properties":{"context":context,"command":{"const":command},"payload":input}},"response":{"type":"object","required":["api_version","request_id","ok","data","error"],"properties":{"data":{"type":["object","null"]}}},"error_codes":ErrorCode::ALL,"scope":"Imported manual annotation JSON only; no diarization, model/media binding, SRT voice export, global latest CAS or quality approval"}),
    )
}
