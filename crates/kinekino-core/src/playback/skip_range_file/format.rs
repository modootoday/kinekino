//! On-disk format v1:
//! `{"version":1,"entries":{"<key>":{"start_ms":N,"end_ms":N,"updated_at_ms":N}}}`.
//! Unknown fields are rejected; seconds in memory are floored to milliseconds on disk.
use crate::playback::skip_range::SkipRange;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const VERSION: u64 = 1;

/// One stored span; all values are integer milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub start_ms: u64,
    pub end_ms: u64,
    pub updated_at_ms: u64,
}

impl Entry {
    pub fn from_range(range: SkipRange, updated_at_ms: u64) -> Self {
        let (start_ms, end_ms) = range.to_ms();
        Self {
            start_ms,
            end_ms,
            updated_at_ms,
        }
    }

    pub fn range(&self) -> Option<SkipRange> {
        SkipRange::from_ms(self.start_ms, self.end_ms)
    }
}

pub type Entries = BTreeMap<String, Entry>;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    version: u64,
    entries: Entries,
}

/// What a file body turned out to be.
#[derive(Debug, PartialEq, Eq)]
pub enum Decoded {
    V1(Entries),
    /// Written by a newer build; must not be read or overwritten.
    Newer,
    /// Not JSON, wrong shape, unknown fields, or no `version`.
    Corrupt,
}

pub fn decode(body: &[u8]) -> Decoded {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return Decoded::Corrupt;
    };
    match value.get("version").and_then(serde_json::Value::as_u64) {
        Some(v) if v > VERSION => Decoded::Newer,
        Some(VERSION) => match serde_json::from_value::<Document>(value) {
            Ok(doc) => Decoded::V1(doc.entries),
            Err(_) => Decoded::Corrupt,
        },
        _ => Decoded::Corrupt,
    }
}

pub fn encode(entries: &Entries) -> serde_json::Result<Vec<u8>> {
    #[derive(Serialize)]
    struct DocumentRef<'a> {
        version: u64,
        entries: &'a Entries,
    }
    serde_json::to_vec(&DocumentRef {
        version: VERSION,
        entries,
    })
}

/// Drops entries not updated for more than `max_age_ms`, then the least recently
/// updated ones until at most `max_entries` remain.
pub fn prune(entries: &mut Entries, now_ms: u64, max_entries: usize, max_age_ms: u64) {
    entries.retain(|_, e| now_ms.saturating_sub(e.updated_at_ms) <= max_age_ms);
    if entries.len() <= max_entries {
        return;
    }
    let mut by_age: Vec<(u64, String)> = entries
        .iter()
        .map(|(k, e)| (e.updated_at_ms, k.clone()))
        .collect();
    by_age.sort_unstable();
    let excess = entries.len() - max_entries;
    for (_, key) in by_age.into_iter().take(excess) {
        entries.remove(&key);
    }
}
