use super::ProposalTask;
use serde::Deserialize;
pub const MAX_DOCUMENT_BYTES: usize = 65_536;
const MARKER: &str = "/no_think\n";
const ASR_INSTRUCTION: &str = r#"Edit this ASR draft. Correct punctuation only. Do not guess unclear words. JSON only: {"proposed":string,"uncertain_words":[string]}. Under100words. "#;
const CARDS_INSTRUCTION: &str = r#"Using only these Korean subtitle cues, write a short Korean headline and one-sentence summary for a community card. Do not invent visual events. JSON only: {"headline":string,"summary":string,"cue_ids":[string]}. "#;
const HIGHLIGHTS_INSTRUCTION: &str = r#"Choose at most two interesting dialogue ranges from these actual cues. Do not invent timestamps. Return JSON only {"ranges":[{"cue_ids":[string],"reason":string}]}. Reasons under15words. "#;
#[derive(Deserialize)]
struct RequestEnvelope {
    messages: Vec<SourceMessage>,
}
#[derive(Deserialize)]
struct SourceMessage {
    role: String,
    content: String,
}
/// Exact saved-request profile. Returns the whole suffix; never searches for a bracket or repairs escapes.
pub fn request_source_payload(asset: &str, task: ProposalTask) -> Result<String, &'static str> {
    if asset.len() > MAX_DOCUMENT_BYTES {
        return Err("Document source exceeds 64 KiB");
    }
    let request: RequestEnvelope = serde_json::from_str(asset)
        .map_err(|_| "Invalid source JSON or duplicate message fields")?;
    if request.messages.len() != 1 || request.messages[0].role != "user" {
        return Err("Source profile requires exactly one user message");
    }
    let content = &request.messages[0].content;
    if content.matches(MARKER).count() != 1 {
        return Err("Source requires exactly one real-linefeed no_think boundary");
    }
    let instruction = match task {
        ProposalTask::Correction => ASR_INSTRUCTION,
        ProposalTask::Cards => CARDS_INSTRUCTION,
        ProposalTask::Highlights => HIGHLIGHTS_INSTRUCTION,
    };
    content
        .strip_prefix(instruction)
        .and_then(|s| s.strip_prefix(MARKER))
        .map(str::to_owned)
        .ok_or("Source instruction differs from the pinned extraction profile")
}
