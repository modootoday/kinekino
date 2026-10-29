//! Wire golden fixtures (`fixture` feature). The SDK tests and the host tests read these
//! same files, so either side drifting from the contract fails. Synthetic data only.
pub const MANIFEST: &str = include_str!("../tests/golden/manifest.json");
/// One `source/1` request per method, one JSON object per line.
pub const SOURCE_REQUESTS: &str = include_str!("../tests/golden/source_requests.jsonl");
pub const SOURCE_RESPONSES: &str = include_str!("../tests/golden/source_responses.jsonl");
/// `document/1` request lines as the host writes them (with the protocol field).
pub const DOCUMENT_REQUESTS: &str = include_str!("../tests/golden/document_requests.jsonl");
pub const DOCUMENT_RESPONSES: &str = include_str!("../tests/golden/document_responses.jsonl");

/// Non-empty lines of a `.jsonl` golden.
pub fn lines(golden: &str) -> impl Iterator<Item = &str> {
    golden.lines().filter(|l| !l.trim().is_empty())
}
