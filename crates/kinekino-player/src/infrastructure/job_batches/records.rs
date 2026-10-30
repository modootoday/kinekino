use super::*;
use crate::{
    application::job_batches::BatchProgress,
    domain::{api_values::DecimalU64, identifiers::valid_sha256},
};
use serde::{Deserialize, Serialize};
use std::io::Read;
pub(super) const MAX_EVENT_BYTES: usize = 1024 * 1024;
pub(super) const MAX_EVENTS: usize = 65536;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Record {
    schema_version: String,
    scope_sha256: String,
    batch_id: String,
    pub revision: DecimalU64,
    parent_sha256: Option<String>,
    payload: Payload,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Payload {
    Initial {
        document: Box<BatchDocument>,
    },
    Item {
        index: usize,
        progress: BatchProgress,
    },
}
fn copy<T: Serialize + serde::de::DeserializeOwned>(value: &T) -> Result<T, ApiError> {
    serde_json::from_slice(&serde_json::to_vec(value).map_err(|_| corrupt())?)
        .map_err(|_| corrupt())
}
impl Record {
    pub fn initial(document: &BatchDocument) -> Result<Self, ApiError> {
        Ok(Self {
            schema_version: "1".into(),
            scope_sha256: document.plan.scope_sha256.clone(),
            batch_id: document.plan.batch_id.clone(),
            revision: document.revision,
            parent_sha256: None,
            payload: Payload::Initial {
                document: Box::new(copy(document)?),
            },
        })
    }
    pub fn successor(
        previous: &BatchDocument,
        next: &BatchDocument,
        parent: String,
    ) -> Result<Self, ApiError> {
        let mut changed = None;
        for (index, (old, new)) in previous.entries.iter().zip(&next.entries).enumerate() {
            if serde_json::to_vec(&old.progress).map_err(|_| corrupt())?
                != serde_json::to_vec(&new.progress).map_err(|_| corrupt())?
            {
                if changed.is_some() {
                    return Err(corrupt());
                }
                changed = Some((index, copy(&new.progress)?));
            }
        }
        let (index, progress) = changed.ok_or_else(corrupt)?;
        Ok(Self {
            schema_version: "1".into(),
            scope_sha256: next.plan.scope_sha256.clone(),
            batch_id: next.plan.batch_id.clone(),
            revision: next.revision,
            parent_sha256: Some(parent),
            payload: Payload::Item { index, progress },
        })
    }
}
pub(super) fn scan(
    directory: &Path,
    scope: &str,
    id: &str,
) -> Result<(BatchDocument, String), ApiError> {
    let mut events = Vec::new();
    let mut observed_entries = 0;
    for entry in fs::read_dir(directory).map_err(io)? {
        observed_entries += 1;
        if observed_entries > MAX_EVENTS {
            return Err(corrupt());
        }
        let entry = entry.map_err(io)?;
        let name = entry.file_name().into_string().map_err(|_| corrupt())?;
        if name.starts_with(".pending-") {
            // Retain unpublished attempts, but never read one as an event or resume it.
            let _guard = paths::file(&entry.path())?;
            continue;
        }
        let (number, sha) = name
            .strip_prefix("r-")
            .and_then(|s| s.strip_suffix(".json"))
            .and_then(|s| s.split_once('-'))
            .ok_or_else(corrupt)?;
        let revision = number.parse::<u64>().map_err(|_| corrupt())?;
        if revision == 0 || number != format!("{revision:020}") || !valid_sha256(sha) {
            return Err(corrupt());
        }
        events.push((revision, sha.to_owned(), entry.path()));
        if events.len() > MAX_EVENTS {
            return Err(corrupt());
        }
    }
    events.sort_by_key(|entry| entry.0);
    if events.is_empty() {
        return Err(corrupt());
    }
    let mut document: Option<BatchDocument> = None;
    let mut parent = None;
    for (index, (revision, sha, path)) in events.into_iter().enumerate() {
        if revision != index as u64 + 1 {
            return Err(corrupt());
        }
        let mut file = paths::file(&path)?;
        let mut bytes = Vec::new();
        file.by_ref()
            .take(MAX_EVENT_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(io)?;
        if bytes.len() > MAX_EVENT_BYTES || hash(&bytes) != sha {
            return Err(corrupt());
        }
        let record: Record = serde_json::from_slice(&bytes).map_err(|_| corrupt())?;
        if record.schema_version != "1"
            || record.scope_sha256 != scope
            || record.batch_id != id
            || record.revision.0 != revision
            || record.parent_sha256 != parent
            || serde_json::to_vec(&record).map_err(|_| corrupt())? != bytes
        {
            return Err(corrupt());
        }
        document = Some(match (document, record.payload) {
            (None, Payload::Initial { document }) if revision == 1 => {
                validate_initial(&document, scope).map_err(|_| corrupt())?;
                if document.plan.batch_id != id {
                    return Err(corrupt());
                }
                *document
            }
            (Some(previous), Payload::Item { index, progress }) => {
                let mut next: BatchDocument = copy(&previous)?;
                next.entries.get_mut(index).ok_or_else(corrupt)?.progress = progress;
                next.revision = DecimalU64(revision);
                validate_successor(&previous, &next).map_err(|_| corrupt())?;
                next
            }
            _ => return Err(corrupt()),
        });
        parent = Some(sha);
    }
    Ok((document.ok_or_else(corrupt)?, parent.ok_or_else(corrupt)?))
}
