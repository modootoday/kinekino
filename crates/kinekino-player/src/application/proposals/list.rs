//! Read witnesses only. Listing never authorizes inspection bypass or mutation.
use super::{ProposalStore, StoredProposal};
use crate::{
    application::api::{ApiError, ErrorCode, Response},
    domain::{
        api_values::DecimalU64,
        proposals::{DraftState, ProposalPayload, ProposalTask},
    },
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::io::{self, Write};

pub const MAX_CURSOR_BYTES: usize = 2048;
pub const MAX_ROW_BYTES: usize = 1024;
pub const MAX_RESPONSE_BYTES: usize = 256 * 1024;
pub const MAX_METADATA_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListProposals {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<ProposalTask>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}
impl ListProposals {
    pub fn decode(value: Value) -> Result<Self, ApiError> {
        if value
            .as_object()
            .is_none_or(|v| v.values().any(Value::is_null))
        {
            return Err(invalid("List fields must be omitted rather than null"));
        }
        let input: Self =
            serde_json::from_value(value).map_err(|_| invalid("Invalid proposal list request"))?;
        input.validate()?;
        Ok(input)
    }
    pub fn validate(&self) -> Result<(), ApiError> {
        if !(1..=200).contains(&self.limit.unwrap_or(50))
            || self
                .cursor
                .as_ref()
                .is_some_and(|c| c.is_empty() || c.len() > MAX_CURSOR_BYTES)
        {
            return Err(invalid("Invalid list limit or cursor size"));
        }
        Ok(())
    }
    pub fn page_limit(&self) -> usize {
        usize::from(self.limit.unwrap_or(50))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalListRow {
    pub proposal_id: String,
    pub task: ProposalTask,
    pub draft_revision: DecimalU64,
    pub draft_state: DraftState,
    pub source_id: String,
    pub source_revision: DecimalU64,
    pub source_sha256: String,
    pub typed_item_count: u32,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListEvidence {
    NotChecked,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionAuthority {
    None,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalListPage {
    pub schema_version: String,
    pub rows: Vec<ProposalListRow>,
    pub next_cursor: Option<String>,
    pub dataset_witness: String,
    pub snapshot_witness: String,
    pub current_evidence: ListEvidence,
    pub selection_authority: SelectionAuthority,
}
pub fn unavailable() -> ApiError {
    ApiError::new(
        ErrorCode::Unsupported,
        "Proposal listing adapter unavailable",
    )
}
pub(crate) fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
pub(crate) fn exhausted() -> ApiError {
    ApiError::new(
        ErrorCode::ResourceExhausted,
        "Proposal list exceeds its bounded read budget",
    )
}
/// Count serialized bytes without allocating an unbounded intermediate JSON body.
pub(crate) fn encoded_bytes(value: &impl Serialize, maximum: usize) -> Result<usize, ApiError> {
    struct Counter {
        count: usize,
        maximum: usize,
    }
    impl Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.count = self
                .count
                .checked_add(bytes.len())
                .filter(|v| *v <= self.maximum)
                .ok_or(io::ErrorKind::FileTooLarge)?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut writer = Counter { count: 0, maximum };
    serde_json::to_writer(&mut writer, value).map_err(|_| exhausted())?;
    Ok(writer.count)
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Continuation {
    version: String,
    pub(crate) dataset: String,
    snapshot: String,
    task: Option<ProposalTask>,
    limit: u16,
    order: String,
    after_id: String,
}
impl Continuation {
    pub(crate) fn check_dataset(&self, dataset: &str) -> Result<(), ApiError> {
        if self.dataset != dataset {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Proposal list belongs to another dataset",
            ));
        }
        Ok(())
    }
}
pub(crate) fn prepare_cursor(input: &ListProposals) -> Result<Option<Continuation>, ApiError> {
    input.validate()?;
    let Some(encoded) = &input.cursor else {
        return Ok(None);
    };
    let raw = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| invalid("Invalid list cursor encoding"))?;
    let value: Continuation =
        serde_json::from_slice(&raw).map_err(|_| invalid("Invalid list cursor shape"))?;
    if value.version != "proposal-list-cursor/1"
        || value.order != "proposal_id_ascii"
        || value.task != input.task
        || usize::from(value.limit) != input.page_limit()
        || !crate::domain::identifiers::valid_id(&value.after_id)
        || !crate::domain::identifiers::valid_sha256(&value.dataset)
        || !crate::domain::identifiers::valid_sha256(&value.snapshot)
    {
        return Err(invalid("List cursor does not match this query"));
    }
    Ok(Some(value))
}
pub(crate) fn hash_witness(value: &impl Serialize) -> Result<String, ApiError> {
    encoded_bytes(value, MAX_METADATA_BYTES)?;
    let raw = serde_json::to_vec(value).map_err(|_| exhausted())?;
    Ok(format!("{:x}", Sha256::digest(&raw)))
}
pub(crate) fn metadata_row(state: &StoredProposal) -> Result<ProposalListRow, ApiError> {
    let doc = &state.document;
    let count = match &doc.draft {
        None => 0,
        Some(ProposalPayload::Cards { cards }) => cards.len(),
        Some(ProposalPayload::Highlights { highlights }) => highlights.len(),
        Some(ProposalPayload::Correction { edits }) => edits.len(),
        Some(ProposalPayload::DocumentCorrection { edits }) => edits.len(),
    };
    let row = ProposalListRow {
        proposal_id: doc.proposal_id.clone(),
        task: doc.generated.task,
        draft_revision: doc.draft_revision,
        draft_state: doc.state,
        source_id: doc.source.source_id().into(),
        source_revision: doc.source.source_revision(),
        source_sha256: doc.source.source_sha256().into(),
        typed_item_count: u32::try_from(count).map_err(|_| exhausted())?,
    };
    encoded_bytes(&row, MAX_ROW_BYTES)?;
    Ok(row)
}
pub(crate) fn project_page(
    input: &ListProposals,
    continuation: Option<Continuation>,
    dataset: String,
    metadata: Vec<ProposalListRow>,
    witnesses: Vec<(String, DecimalU64, String)>,
) -> Result<ProposalListPage, ApiError> {
    let snapshot = hash_witness(&("proposal-list-snapshot/1", &dataset, &witnesses))?;
    if let Some(c) = &continuation {
        if c.snapshot != snapshot {
            let mut error = ApiError::new(
                ErrorCode::RevisionConflict,
                "Proposal list changed between pages",
            );
            error.remediation = "restart_proposal_list".into();
            return Err(error);
        }
        if !metadata
            .iter()
            .any(|r| r.proposal_id == c.after_id && input.task.is_none_or(|t| r.task == t))
        {
            return Err(invalid(
                "List cursor position is not in this snapshot query",
            ));
        }
    }
    let mut filtered = metadata.into_iter().filter(|r| {
        input.task.is_none_or(|t| r.task == t)
            && continuation
                .as_ref()
                .is_none_or(|c| r.proposal_id > c.after_id)
    });
    let rows: Vec<_> = filtered.by_ref().take(input.page_limit()).collect();
    let next_cursor = if filtered.next().is_some() {
        let last = rows
            .last()
            .ok_or_else(|| invalid("Invalid empty continuation page"))?;
        let c = Continuation {
            version: "proposal-list-cursor/1".into(),
            dataset: dataset.clone(),
            snapshot: snapshot.clone(),
            task: input.task,
            limit: input.limit.unwrap_or(50),
            order: "proposal_id_ascii".into(),
            after_id: last.proposal_id.clone(),
        };
        let encoded = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&c).map_err(|_| exhausted())?);
        if encoded.len() > MAX_CURSOR_BYTES {
            return Err(exhausted());
        }
        Some(encoded)
    } else {
        None
    };
    let result = ProposalListPage {
        schema_version: "proposal-list/1".into(),
        rows,
        next_cursor,
        dataset_witness: dataset,
        snapshot_witness: snapshot,
        current_evidence: ListEvidence::NotChecked,
        selection_authority: SelectionAuthority::None,
    };
    encoded_bytes(&result, MAX_RESPONSE_BYTES.saturating_sub(2048))?;
    Ok(result)
}
pub fn execute(
    store: &dyn ProposalStore,
    input: ListProposals,
    request_id: &str,
) -> Result<ProposalListPage, ApiError> {
    input.validate()?;
    if !store.available() || !store.list_available() {
        return Err(unavailable());
    }
    let page = store.list(&input)?;
    // The bound includes the outer response, not just the rows.
    if page.schema_version != "proposal-list/1"
        || page.rows.len() > input.page_limit()
        || !crate::domain::identifiers::valid_sha256(&page.dataset_witness)
        || !crate::domain::identifiers::valid_sha256(&page.snapshot_witness)
        || page
            .next_cursor
            .as_ref()
            .is_some_and(|c| c.is_empty() || c.len() > MAX_CURSOR_BYTES)
    {
        return Err(ApiError::new(
            ErrorCode::EngineFailed,
            "Invalid proposal list response",
        ));
    }
    for row in &page.rows {
        if !crate::domain::identifiers::valid_id(&row.proposal_id)
            || !crate::domain::identifiers::valid_id(&row.source_id)
            || !crate::domain::identifiers::valid_sha256(&row.source_sha256)
            || row.draft_revision.0 == 0
            || row.typed_item_count > 4096
            || input.task.is_some_and(|task| task != row.task)
        {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Invalid proposal list metadata",
            ));
        }
        encoded_bytes(row, MAX_ROW_BYTES)?;
    }
    if page
        .rows
        .windows(2)
        .any(|rows| rows[0].proposal_id >= rows[1].proposal_id)
        || (page.rows.is_empty() && page.next_cursor.is_some())
    {
        return Err(ApiError::new(
            ErrorCode::EngineFailed,
            "Invalid proposal list ordering",
        ));
    }
    encoded_bytes(&page, MAX_RESPONSE_BYTES.saturating_sub(2048))?;
    let response = Response {
        api_version: crate::application::api::API_VERSION,
        request_id: request_id.to_owned(),
        ok: true,
        data: Some(serde_json::to_value(&page).map_err(|_| exhausted())?),
        error: None,
    };
    encoded_bytes(&response, MAX_RESPONSE_BYTES)?;
    Ok(page)
}
pub fn request_schema() -> Value {
    json!({"type":"object","additionalProperties":false,"properties":{
        "task":{"enum":["correction","cards","highlights"]},
        "limit":{"type":"integer","minimum":1,"maximum":200,"default":50},
        "cursor":{"type":"string","minLength":1,"maxLength":MAX_CURSOR_BYTES}}})
}
pub fn response_schema() -> Value {
    let id = json!({"type":"string","minLength":1,"maxLength":80});
    let decimal = json!({"type":"string","pattern":"^[0-9]+$"});
    let sha = json!({"type":"string","pattern":"^[a-f0-9]{64}$"});
    let row = json!({"type":"object","additionalProperties":false,"required":["proposal_id","task","draft_revision","draft_state","source_id","source_revision","source_sha256","typed_item_count"],"properties":{
        "proposal_id":id,"task":{"enum":["correction","cards","highlights"]},
        "draft_revision":decimal,"draft_state":{"enum":["editable","needs_repair"]},
        "source_id":id,"source_revision":decimal,"source_sha256":sha,
        "typed_item_count":{"type":"integer","minimum":0,"maximum":4096}}});
    json!({"type":"object","additionalProperties":false,"required":["schema_version","rows","next_cursor","dataset_witness","snapshot_witness","current_evidence","selection_authority"],"properties":{
        "schema_version":{"const":"proposal-list/1"},"rows":{"type":"array","maxItems":200,"items":row},
        "next_cursor":{"type":["string","null"],"maxLength":MAX_CURSOR_BYTES},"dataset_witness":sha,"snapshot_witness":sha,
        "current_evidence":{"const":"not_checked"},"selection_authority":{"const":"none"}}})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn response_counter_enforces_exact_serialized_boundary() {
        assert_eq!(encoded_bytes(&"x", 3).unwrap(), 3);
        assert_eq!(
            encoded_bytes(&"x", 2).unwrap_err().code,
            ErrorCode::ResourceExhausted
        );
    }
}
